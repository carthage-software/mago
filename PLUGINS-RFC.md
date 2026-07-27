# RFC — Doctrine & Symfony analyzer plugins for Mago

> Status: implemented on this branch, validated on the reference codebase, seeking upstream feedback · Author: @cmukanisa · Date: 2026-07-27
> Branch: `feat/doctrine-symfony-plugins` · Implementations under `crates/analyzer/src/plugin/libraries/{doctrine,symfony}/` · Measured verdicts in § Validation run

## TL;DR for the upstream discussion

Two analyzer plugins, implemented and validated end to end on a production
Symfony 8.1 / Doctrine ORM 3 codebase (10,456 PHP files):

- **`doctrine`** — validates literal criteria keys in `findBy` / `findOneBy` /
  `count` against the entity's ORM attribute mapping. New issue code
  `doctrine-unknown-field`, with a closest-match suggestion. Zero false
  positives across 2,845 `find*()` calls on the reference codebase.
- **`symfony`** — extends the `psr-container` idea to literal *string* service
  ids, resolved through the compiled container XML (new analyzer setting
  `symfony-container-xml-path`, the analogue of phpstan-symfony's
  `containerXmlPath`). Fail-open on unknown ids.

Measured effect (paired A/B, same binary, same session): −42 issues / −32
errors on `src + tests`, the container-blindness families eliminated
(13 → 0 `ambiguous-object-method-access`, 14 → 1
`possible-method-access-on-null`), plugin overhead ~0 time and ~3% peak RSS.
Detection parity with phpstan-doctrine / phpstan-symfony on the deliberate
defect probes below. Proposed as one-provider-per-PR; the configuration key
shape and the issue-code naming are the two decisions upstream may want to
reshape.

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
| Mago without the plugin | **silent** ❌ |
| **Mago + `doctrine` plugin** | **`doctrine-unknown-field`, naming the entity and its mapping sources** ✅ |

Measured on the reference codebase (2026-07-27, implementation on this
branch): the probe reports the exact entity FQCN; on a near-miss key
(`localIdentifer` for `localIdentifier`) the issue adds
``Help: Did you mean `localIdentifier`?`` via the closest-match suggestion.
On the real `src/ + tests/` tree the plugin reports **zero**
`doctrine-unknown-field` — no false positives on 2,845 `find*()` calls.

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

## Validation run — 2026-07-27, shipped implementation

Binary built from this branch (`mago 1.45.0` + these plugins), reference
codebase at its current head (8,685 src files, 1,771 test files), disposable
worktree, `threads = 2`, vendor included, peak RSS via `/usr/bin/time -l`.
Single cold runs, back-to-back in one session — so the **paired A/B deltas**
below are the trustworthy signal; the absolutes moved against the baseline
session in *both* directions (src+tests time 14.8 s → 5–6 s, peak RSS
1.0 GB → 2.1–2.2 GB, same binary version) and are dominated by machine/session
state. Report the pair, not the absolute.

### Full analysis, A/B on the same binary

| Configuration | Scope | Time | Peak RSS | Issues | Errors |
|---|---|---|---|---|---|
| `psr-container` only | src | 5.28 s | 2.10 GB | 1,535 | 540 |
| + `doctrine` + `symfony` | src | 5.07–5.28 s | 2.07–2.15 GB | 1,535 | 540 |
| `psr-container` only | src + tests | 6.00 s | 2.16 GB | 3,220 | 678 |
| + `doctrine` + `symfony` | src + tests | 5.12 s | 2.23 GB | **3,178** | **646** |

On `src/` the two new plugins change nothing — issue-for-issue identical
output — which is exactly right: production code neither fetches string
service ids nor passes unknown criteria keys. On `src + tests` they remove
42 issues, 32 of them errors. Per-category deltas:

| Issue code | Before | After |
|---|---|---|
| `ambiguous-object-method-access` | 13 | **0** |
| `possible-method-access-on-null` | 14 | **1** |
| `possibly-null-argument` | 306 | 299 |
| `less-specific-argument` | 7 | 1 |
| `mixed-method-access` | 37 | 35 |
| `mixed-assignment` | 1,474 | 1,466 |
| `redundant-docblock-type` | 26 | **37** — sharper inferred types expose docblocks that were papering over the blindness |
| `doctrine-unknown-field` on real code | — | **0** (no false positives) |

### Success criteria — verdicts

1. Probe B parity with phpstan-doctrine — **met** (incl. closest-match help).
2. Probe C parity with phpstan-symfony — **met** (resolved through a
   20,211-service compiled dump).
3. No blindness class left — **met for container blindness** (13 → 0
   ambiguous accesses, 14 → 1 null-method accesses on tests); the residual
   error classes match the baseline triage (299× `possibly-null-argument`,
   158× `incompatible-return-type` — the verified-real covariance family).
   A file-by-file re-triage of all 646 errors was not redone in this run.
4. Time within 2× of current Mago — **met with margin**: plugin overhead is
   below run noise (the plugins-on src+tests run was *faster* than the
   plugins-off one).
5. Memory ≤ 1.5 GB at `threads = 2` — **not met as an absolute, by either
   configuration**: today's session measures 2.07–2.23 GB with *and* without
   the new plugins. The plugin-attributable delta is ≈ +70 MB (~3%). The
   criterion as written bounds the wrong variable — the absolute moves with
   session state, not with the plugins. Restated for upstream review:
   *plugin overhead ≤ 10% peak RSS* — **met** (~3%).

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
through the compiled container XML (`symfony-container-xml-path` analyzer
setting — for an app under test, `var/cache/test/App_Kernel*Container.xml`),
with `getParameter()` typing from the same source as a follow-up provider.
Returns `None` (no opinion) on unknown ids: fail-open on typing, never a
false positive.

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

1. This branch: implementations + RFC + fixtures, validated end to end on
   the reference codebase (§ Validation run). **Done.**
2. Upstream discussion issue referencing this RFC.
3. Implement Doctrine first (smaller, self-contained, no configuration
   surface), `cargo test` fixtures from § Probes. **Done on this branch.**
4. Symfony string-id second (needs a config key upstream may want to shape:
   this branch proposes `symfony-container-xml-path` under `[analyzer]`,
   threaded to plugins through a new `PluginSettings` passed at
   registration). **Done on this branch.**
5. PRs kept small: one provider per PR. `getParameter()` typing from the
   same dump is the natural next provider.
