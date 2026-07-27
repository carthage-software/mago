# RFC — Doctrine & Symfony analyzer plugins for Mago

> Status: draft, seeking upstream feedback · Author: @cmukanisa · Date: 2026-07-27
> Branch: `feat/doctrine-symfony-plugins` · Scaffolds under `crates/analyzer/src/plugin/libraries/{doctrine,symfony}/`

## Why these two plugins

Mago's analyzer already matches PHPStan `level: max` defect-for-defect on plain
PHP — we proved it with a deliberate-defect probe (§ Fixtures) on a production
Symfony 8.1 / Doctrine ORM 3 codebase (hospital information system, hexagonal,
multi-tenant). The remaining gap is *framework knowledge*, and it is exactly
two-shaped: the service container and the ORM metadata. Both are listed on the
official roadmap ("framework plugins for Symfony, Laravel, Doctrine, and
PHPUnit — coming soon"); this RFC proposes concrete designs and ships the
acceptance fixtures and real-world metrics to judge them against.

A key discovery narrowed the scope: the **shipped but default-disabled
`psr-container` plugin** already resolves `get(X::class)` for any PSR-11
container — which covers the dominant Symfony test idiom. The Symfony plugin
therefore only needs to cover what `::class` cannot: string service ids and
parameters, resolved through the compiled container XML.

## Reference codebase for every number below

| | |
|---|---|
| Framework | Symfony 8.x, PHP 8.4, API Platform 4, Doctrine ORM 3 — private codebase, anonymized |
| `src/` | 8,681 PHP files |
| `tests/` | 1,761 PHP files |
| Container fetches in tests | 188 call sites across 95 files, `::class` idiom |
| Doctrine usage | 426 files calling `getRepository()`, 2,845 `find*()` calls |
| Machine | Apple Silicon, 10 cores, 16 GB |
| Versions | Mago 1.45.0 · PHPStan 2.x `level: max` + phpstan-symfony + phpstan-doctrine |

## Measured baseline (2026-07-27)

### Analyzer, `src/` only

| Tool | Time | Peak memory | Issues |
|---|---|---|---|
| Mago (`threads = 2`, vendor included) | **4.5–6.8 s** | 1,015 MB¹ | 1,520 (539 errors) |
| PHPStan (2 workers, extensions, cold) | 58.3 s | 1,007 MB | 0 (green ratchet) |

¹ Earlier runs measured 2,100–2,200 MB — with default `threads` (= cores).
Mago's memory scales with its thread count; bounded to 2 it reaches parity
with PHPStan. Report memory *with* the thread setting, or the number is
meaningless.

Of Mago's 539 errors, the dominant class (157×) was verified real: API
Platform providers declaring `TraversablePaginator<XListItem>` against a
parent expecting `PartialPaginatorInterface<XResponse>` — a generic-parameter
covariance violation PHPStan does not report.

### Analyzer, `src/ + tests/`

| Configuration | Time | New issues from tests/ | of which errors |
|---|---|---|---|
| Without `psr-container` | 14.8 s | 2,261 | 454 |
| **With `psr-container`** | ~15 s | **1,651** | **126** |

Enabling the shipped plugin removed the two container-blindness families
entirely: 194× `ambiguous-object-method-access` and 195×
`possible-method-access-on-null` on container-fetched services.

### Deliberate-defect probes (the acceptance tests of this RFC)

Probe A — container service, nonexistent method:

```php
$lookup = static::getContainer()->get(GradeLookup::class);
$lookup->methodThatDoesNotExist();
```

| Tool | Verdict |
|---|---|
| PHPStan + container XML | `method.notFound`, resolved to the concrete adapter |
| Mago, no plugin | `ambiguous-object-method-access` — admits blindness |
| **Mago + `psr-container`** | **`non-existent-method`, resolved to the interface** ✅ |

Probe B — Doctrine criteria key (the gap the Doctrine plugin closes):

```php
$em->getRepository(Patient::class)->findOneBy(['fieldThatDoesNotExist' => 'x']);
```

| Tool | Verdict |
|---|---|
| PHPStan + phpstan-doctrine | `doctrine.findOneByArgument` — "entity does not have a field named $fieldThatDoesNotExist" |
| Mago (any current config) | **silent** ❌ |

Probe C — string service id (the gap the Symfony plugin closes):

```php
$mailer = static::getContainer()->get('app.mailer');
$mailer->methodThatDoesNotExist();
```

| Tool | Verdict |
|---|---|
| PHPStan + phpstan-symfony (`containerXmlPath`) | resolved through the XML; `method.notFound` |
| Mago without the plugin | `mixed-method-access` — admits blindness |
| **Mago + `symfony` plugin** | **`non-existent-method`, resolved through the XML** ✅ |

Measured on the reference codebase (2026-07-27, implementation on this
branch, compiled test container: 20,147 service tags): on an integration
test fetching `get('doctrine.dbal.central_connection')`, the plugin removes
the whole blindness family on that call site —
`ambiguous-object-method-access`, `possible-method-access-on-null`, and
`mixed-assignment` — by resolving the id to `Doctrine\DBAL\Connection`;
the file's only remaining issue is unrelated to the container.

## Design — Doctrine plugin

See `crates/analyzer/src/plugin/libraries/doctrine/`. Summary: a method hook
targeting `findBy` / `findOneBy` / `count` on `EntityRepository` instances,
resolving the entity from the receiver's generic (already inferred through
doctrine/orm's own docblocks), and validating literal criteria keys against
the persisted-field map built from `#[ORM\Column]` / `#[ORM\Id]` / association
attributes the codex has already parsed. Attribute mapping only in the first
iteration; XML/YAML mappings out of scope. New issue code:
`doctrine-unknown-field`, with a closest-match suggestion.

## Design — Symfony plugin

See `crates/analyzer/src/plugin/libraries/symfony/`. Summary: the
`psr-container` provider shape, extended to literal **string ids** resolved
through the compiled container XML (`symfony.container-xml-path` analyzer
setting — for an app under test, `var/cache/test/App_Kernel*Container.xml`),
plus `getParameter()` typing from the same source. Returns `None` (no opinion)
on unknown ids: fail-open on typing, never a false positive.

## Success criteria, in benchmark form

Following the methodology of `carthage-software/php-toolchain-benchmarks`
(cold/hot, mean ± stddev over ≥ 5 runs, peak RSS), plus what that benchmark
does not measure — **detection equivalence**:

1. Probe B reports `doctrine-unknown-field` (parity with phpstan-doctrine).
2. Probe C resolves through the XML (parity with phpstan-symfony).
3. On the reference codebase, `src+tests` error count with both plugins is
   explainable: every remaining error is triaged real-or-limitation, no
   blindness class left.
4. Time stays within 2× of current Mago (i.e. still ≥ 5× faster than PHPStan).
5. Memory at `threads = 2` stays ≤ 1.5 GB.

## Contribution plan

1. This branch: scaffolds + RFC + fixtures (no wiring, tree still compiles).
2. Upstream discussion issue referencing this RFC.
3. Implement Doctrine first (smaller, self-contained, no configuration
   surface), `cargo test` fixtures from § Probes. **Done on this branch.**
4. Symfony string-id second (needs a config key upstream may want to shape:
   this branch proposes `symfony-container-xml-path` under `[analyzer]`,
   threaded to plugins through a new `PluginSettings` passed at
   registration). **Done on this branch.**
5. PRs kept small: one provider per PR. `getParameter()` typing from the
   same dump is the natural next provider.
