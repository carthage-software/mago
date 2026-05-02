+++
title = "Règles"
description = "Référence complète de chaque règle du linter, triée par sévérité. Cliquez sur une règle pour afficher sa description, ses exemples et sa configuration."
nav_order = 70
nav_section = "Tools"
nav_subsection = "Linter"
+++
Le linter de Mago propose 175 règles réparties sur 9 catégories. Cliquez sur une règle pour afficher sa description, ses prérequis, sa configuration par défaut et ses exemples.

<div class="rule-index" role="navigation" aria-label="Catégories de règles"><a class="rule-index__item" href="#clarity"><span class="rule-index__name">Clarté</span><span class="rule-index__count">22 règles</span><span class="rule-index__blurb">Règles qui rendent l'intention explicite et réduisent l'effort de lecture. Elles signalent des constructions techniquement valides mais qui obscurcissent ce que fait le code.</span></a><a class="rule-index__item" href="#bestpractices"><span class="rule-index__name">Bonnes pratiques</span><span class="rule-index__count">39 règles</span><span class="rule-index__blurb">Patterns idiomatiques et conventions largement adoptées. Ces règles orientent le code vers la manière dont le PHP moderne est écrit par celles et ceux qui en livrent beaucoup.</span></a><a class="rule-index__item" href="#consistency"><span class="rule-index__name">Cohérence</span><span class="rule-index__count">27 règles</span><span class="rule-index__blurb">Uniformité stylistique sur l'ensemble du code. Choisissez une façon de faire, ces règles aident tout le monde à s'y tenir.</span></a><a class="rule-index__item" href="#deprecation"><span class="rule-index__name">Obsolescence</span><span class="rule-index__count">7 règles</span><span class="rule-index__blurb">Fonctionnalités et API PHP marquées comme obsolètes en amont, et qui finiront par être supprimées. Migrez avant qu'elles ne cassent.</span></a><a class="rule-index__item" href="#maintainability"><span class="rule-index__name">Maintenabilité</span><span class="rule-index__count">11 règles</span><span class="rule-index__blurb">Du code difficile à maintenir dans la durée, trop complexe, trop emmêlé, trop fragile. Ces règles font remonter le coût tôt.</span></a><a class="rule-index__item" href="#redundancy"><span class="rule-index__name">Redondance</span><span class="rule-index__count">29 règles</span><span class="rule-index__blurb">Code mort, valeurs inutilisées, constructions sans effet observable. Les retirer garde le code honnête.</span></a><a class="rule-index__item" href="#security"><span class="rule-index__name">Sécurité</span><span class="rule-index__count">12 règles</span><span class="rule-index__blurb">Règles qui signalent des vulnérabilités, vecteurs d'injection, désérialisation non sûre, données non fiables atteignant des points dangereux.</span></a><a class="rule-index__item" href="#safety"><span class="rule-index__name">Sûreté</span><span class="rule-index__count">9 règles</span><span class="rule-index__blurb">Constructions qui compilent mais explosent à l'exécution. Ces règles attrapent les pièges avant les utilisateurs.</span></a><a class="rule-index__item" href="#correctness"><span class="rule-index__name">Correction</span><span class="rule-index__count">19 règles</span><span class="rule-index__blurb">Bugs et erreurs de logique. Les règles de cette catégorie attrapent du code qui fait probablement autre chose que ce que l'auteur voulait.</span></a></div>

<h2 id="integration-specific-rules">Règles spécifiques aux intégrations</h2>

Certaines règles ne se déclenchent que lorsque Mago détecte une bibliothèque ou un framework particulier. Chaque règle renvoie à sa description complète dans la section ci-dessus.

<h3 id="integration-cakephp">CakePHP</h3>

- [`final-controller`](#final-controller)

<h3 id="integration-laravel">Laravel</h3>

- [`final-controller`](#final-controller)
- [`middleware-in-routes`](#middleware-in-routes)
- [`no-request-all`](#no-request-all)
- [`prefer-anonymous-migration`](#prefer-anonymous-migration)
- [`prefer-view-array`](#prefer-view-array)

<h3 id="integration-phpunit">PHPUnit</h3>

- [`assertion-style`](#assertion-style)
- [`prefer-test-attribute`](#prefer-test-attribute)
- [`strict-assertions`](#strict-assertions)
- [`use-specific-assertions`](#use-specific-assertions)

<h3 id="integration-pest">Pest</h3>

- [`no-only`](#no-only)
- [`use-dedicated-expectation`](#use-dedicated-expectation)
- [`use-simpler-expectation`](#use-simpler-expectation)
- [`use-specific-expectations`](#use-specific-expectations)

<h3 id="integration-psl">Psl</h3>

- [`psl-array-functions`](#psl-array-functions)
- [`psl-data-structures`](#psl-data-structures)
- [`psl-datetime`](#psl-datetime)
- [`psl-math-functions`](#psl-math-functions)
- [`psl-output`](#psl-output)
- [`psl-randomness-functions`](#psl-randomness-functions)
- [`psl-regex-functions`](#psl-regex-functions)
- [`psl-sleep-functions`](#psl-sleep-functions)
- [`psl-string-functions`](#psl-string-functions)

<h3 id="integration-spiral">Spiral</h3>

- [`final-controller`](#final-controller)

<h3 id="integration-symfony">Symfony</h3>

- [`final-controller`](#final-controller)
- [`no-service-state-mutation`](#no-service-state-mutation)
- [`prefer-interface`](#prefer-interface)
- [`prefer-pre-increment`](#prefer-pre-increment)

<h3 id="integration-tempest">Tempest</h3>

- [`final-controller`](#final-controller)

<h3 id="integration-wordpress">WordPress</h3>

- [`no-db-schema-change`](#no-db-schema-change)
- [`no-direct-db-query`](#no-direct-db-query)
- [`no-roles-as-capabilities`](#no-roles-as-capabilities)
- [`no-unescaped-output`](#no-unescaped-output)
- [`use-wp-functions`](#use-wp-functions)

<h3 id="integration-yii">Yii</h3>

- [`final-controller`](#final-controller)

<h2 id="clarity">Clarté</h2>

Règles qui rendent l'intention explicite et réduisent l'effort de lecture. Elles signalent des constructions techniquement valides mais qui obscurcissent ce que fait le code.

<div class="rule-list">

<details class="rule" name="rule" id="no-empty">
<summary><code class="rule__code">no-empty</code><a class="rule__anchor" href="#no-empty" aria-label="Lien permanent vers no-empty">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects the use of the `empty()` construct.

The `empty()` language construct can lead to ambiguous and potentially buggy code due to
loose and counterintuitive definition of emptiness. It fails to clearly convey
developer's intent or expectation, making it preferable to use explicit checks.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if (!empty($myArray)) {
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if ($myArray === []) {
    // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="explicit-octal">
<summary><code class="rule__code">explicit-octal</code><a class="rule__anchor" href="#explicit-octal" aria-label="Lien permanent vers explicit-octal">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects implicit octal numeral notation and suggests replacing it with explicit octal numeral notation.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.1.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$a = 0123;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$a = 0o123;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="instanceof-stringable">
<summary><code class="rule__code">instanceof-stringable</code><a class="rule__anchor" href="#instanceof-stringable" aria-label="Lien permanent vers instanceof-stringable">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects the legacy pattern `is_object($x) && method_exists($x, '__toString')` and suggests
replacing it with `$x instanceof Stringable` for improved readability and performance.

Since PHP 8.0, all classes with `__toString()` automatically implement the `Stringable` interface.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.0.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function stringify(mixed $value): string {
    if (is_object($value) && method_exists($value, '__toString')) {
        return (string) $value;
    }

    return '';
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function stringify(mixed $value): string {
    if ($value instanceof Stringable) {
        return (string) $value;
    }

    return '';
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="literal-named-argument">
<summary><code class="rule__code">literal-named-argument</code><a class="rule__anchor" href="#literal-named-argument" aria-label="Lien permanent vers literal-named-argument">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces that literal values used as arguments in function or method calls
are passed as **named arguments**.

This improves readability by clarifying the purpose of the literal value at the call site.
It is particularly helpful for boolean flags, numeric constants, and `null` values
where the intent is often ambiguous without the parameter name.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.0.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function set_option(string $key, bool $enable_feature) {}

set_option('feature_x', true); // ❌ intent unclear
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function set_option(string $key, bool $enable_feature) {}

set_option(key: 'feature_x', enable_feature: true); // ✅ clear intent
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `check-first-argument` | `boolean` | `false` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `threshold` | `number` | `1` |

</div>

</details>

<details class="rule" name="rule" id="no-hash-emoji">
<summary><code class="rule__code">no-hash-emoji</code><a class="rule__anchor" href="#no-hash-emoji" aria-label="Lien permanent vers no-hash-emoji">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Discourages usage of the `#️⃣` emoji in place of the ASCII `#`.

While PHP allows the use of emojis in comments, it is generally discouraged to use them in place
of the normal ASCII `#` symbol. This is because it can confuse readers and may break external
tools that expect the normal ASCII `#` symbol.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

#️⃣ This is a comment

#️⃣[MyAttribute] <- not a valid attribute
class Foo {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

# This is a comment

#[MyAttribute]
class Foo {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-isset">
<summary><code class="rule__code">no-isset</code><a class="rule__anchor" href="#no-isset" aria-label="Lien permanent vers no-isset">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects the use of the `isset()` construct.

The `isset()` language construct checks whether a variable is set and is not null.
However, it can lead to ambiguous code because it conflates two distinct checks:
variable existence and null comparison. Using explicit null checks or the null
coalescing operator (`??`) is often clearer and more maintainable.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if (isset($value)) {
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if ($value !== null) {
    // ...
}

$result = $value ?? 'default';
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `allow-array-checks` | `boolean` | `false` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-multi-assignments">
<summary><code class="rule__code">no-multi-assignments</code><a class="rule__anchor" href="#no-multi-assignments" aria-label="Lien permanent vers no-multi-assignments">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Flags any instances of multiple assignments in a single statement. This can lead to
confusion and unexpected behavior, and is generally considered poor practice.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$a = $b = 0;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$b = 0;
$a = $b;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-nested-ternary">
<summary><code class="rule__code">no-nested-ternary</code><a class="rule__anchor" href="#no-nested-ternary" aria-label="Lien permanent vers no-nested-ternary">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Nested ternary expressions are disallowed to improve code clarity and prevent potential bugs arising from confusion over operator associativity.

In PHP 8.0 and later, the ternary operator (`? :`) is non-associative. Before PHP 8.0, it was left-associative, which is now deprecated. Most other programming languages treat it as right-associative. This inconsistency across versions and languages can make nested ternaries hard to reason about, even when using parentheses.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$allowed = $user->isAdmin() ? true : ($user->isEditor() ? true : false);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if ($user->isAdmin()) {
    $allowed = true;
} else {
    $allowed = $user->isEditor();
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-shorthand-ternary">
<summary><code class="rule__code">no-shorthand-ternary</code><a class="rule__anchor" href="#no-shorthand-ternary" aria-label="Lien permanent vers no-shorthand-ternary">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects the use of the shorthand ternary and elvis operators.

Both shorthand ternary operator (`$a ? : $b`) and elvis operator (`$a ?: $b`) relies on loose comparison.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php
$value = $foo ?: $default;
$value = $foo ? : $default;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$value = $foo ?? $default;
$value = $foo ? $foo : $default;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-variable-variable">
<summary><code class="rule__code">no-variable-variable</code><a class="rule__anchor" href="#no-variable-variable" aria-label="Lien permanent vers no-variable-variable">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Discourages usage of PHP's variable variables feature.

Variable variables can make code harder to read and maintain, as they introduce a level of indirection that can confuse readers and complicate static analysis.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$foo = 'bar';
$varName = 'foo';

echo $$varName; // Outputs 'bar'
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$foo = 'bar';

echo $foo; // Outputs 'bar'
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="readable-literal">
<summary><code class="rule__code">readable-literal</code><a class="rule__anchor" href="#readable-literal" aria-label="Lien permanent vers readable-literal">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces using underscore separators in numeric literals for improved readability.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>7.4.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$a = 1000000;
$b = 0xCAFEF00D;
$c = 0b01011111;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$a = 1_000_000;
$b = 0xCAFE_F00D;
$c = 0b0101_1111;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `min-digits` | `number` | `5` |

</div>

</details>

<details class="rule" name="rule" id="str-contains">
<summary><code class="rule__code">str-contains</code><a class="rule__anchor" href="#str-contains" aria-label="Lien permanent vers str-contains">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects `strpos($a, $b) !== false` and `strpos($a, $b) === false` comparisons and suggests
replacing them with `str_contains($a, $b)` or `!str_contains($a, $b)` for improved readability
and intent clarity.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.0.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$a = 'hello world';
$b = 'world';

if (strpos($a, $b) !== false) {
    echo 'Found';
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$a = 'hello world';
$b = 'world';

if (str_contains($a, $b)) {
    echo 'Found';
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="str-starts-with">
<summary><code class="rule__code">str-starts-with</code><a class="rule__anchor" href="#str-starts-with" aria-label="Lien permanent vers str-starts-with">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects `strpos($a, $b) === 0` comparisons and suggests replacing them with `str_starts_with($a, $b)`
for improved readability and intent clarity.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.0.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$a = 'hello world';
$b = 'hello';
if (strpos($a, $b) === 0) {
    echo 'Found';
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$a = 'hello world';
$b = 'hello';
if (str_starts_with($a, $b)) {
    echo 'Found';
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="tagged-fixme">
<summary><code class="rule__code">tagged-fixme</code><a class="rule__anchor" href="#tagged-fixme" aria-label="Lien permanent vers tagged-fixme">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects FIXME comments that are not tagged with a user or issue reference. Untagged FIXME comments
are not actionable and can be easily missed by the team. Tagging the FIXME comment with a user or
issue reference ensures that the issue is tracked and resolved.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// FIXME: This is an invalid FIXME comment.
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// FIXME(@azjezz) This is a valid FIXME comment.
// FIXME(azjezz) This is a valid FIXME comment.
// FIXME(#123) This is a valid FIXME comment.
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="tagged-todo">
<summary><code class="rule__code">tagged-todo</code><a class="rule__anchor" href="#tagged-todo" aria-label="Lien permanent vers tagged-todo">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects TODO comments that are not tagged with a user or issue reference. Untagged TODOs
can be difficult to track and may be forgotten. Tagging TODOs with a user or issue reference
makes it easier to track progress and ensures that tasks are not forgotten.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// TODO: This is an invalid TODO comment.
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// TODO(@azjezz) This is a valid TODO comment.
// TODO(azjezz) This is a valid TODO comment.
// TODO(#123) This is a valid TODO comment.
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="use-dedicated-expectation">
<summary><code class="rule__code">use-dedicated-expectation</code><a class="rule__anchor" href="#use-dedicated-expectation" aria-label="Lien permanent vers use-dedicated-expectation">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Use dedicated matchers instead of function calls in Pest tests.

Instead of `expect(is_array($x))->toBeTrue()`, use `expect($x)->toBeArray()`.
This provides clearer intent and better error messages.

Supported patterns:
- Type checks: is_array, is_string, is_int, is_float, is_bool, is_numeric, is_callable, is_iterable, is_object, is_resource, is_scalar, is_null
- String: str_starts_with, str_ends_with, ctype_alpha, ctype_alnum
- Array: in_array, array_key_exists
- File: is_file, is_dir, is_readable, is_writable, file_exists
- Object: property_exists

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-pest"><code>Pest</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

test('function calls', function () {
    expect(is_array($value))->toBeTrue();
    expect(is_string($value))->toBeTrue();
    expect(str_starts_with($string, 'prefix'))->toBeTrue();
    expect(in_array($item, $array))->toBeTrue();
    expect(is_file($path))->toBeTrue();
    expect(property_exists($obj, 'name'))->toBeTrue();
});
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

test('dedicated matchers', function () {
    expect($value)->toBeArray();
    expect($value)->toBeString();
    expect($string)->toStartWith('prefix');
    expect($array)->toContain($item);
    expect($path)->toBeFile();
    expect($obj)->toHaveProperty('name');
});
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="use-simpler-expectation">
<summary><code class="rule__code">use-simpler-expectation</code><a class="rule__anchor" href="#use-simpler-expectation" aria-label="Lien permanent vers use-simpler-expectation">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Simplify expect() expressions in Pest tests by using dedicated matchers.

This rule detects patterns where the expect() argument contains an expression that can be simplified:
- `expect(!$x)->toBeTrue()` -> `expect($x)->toBeFalse()`
- `expect(!$x)->toBeFalse()` -> `expect($x)->toBeTrue()`
- `expect($a > $b)->toBeTrue()` -> `expect($a)->toBeGreaterThan($b)`
- `expect($a >= $b)->toBeTrue()` -> `expect($a)->toBeGreaterThanOrEqual($b)`
- `expect($a < $b)->toBeTrue()` -> `expect($a)->toBeLessThan($b)`
- `expect($a <= $b)->toBeTrue()` -> `expect($a)->toBeLessThanOrEqual($b)`
- `expect($a === $b)->toBeTrue()` -> `expect($a)->toBe($b)`
- `expect($a !== $b)->toBeTrue()` -> `expect($a)->not->toBe($b)`
- `expect($x instanceof Y)->toBeTrue()` -> `expect($x)->toBeInstanceOf(Y::class)`
- `expect($x >= min && $x <= max)->toBeTrue()` -> `expect($x)->toBeBetween(min, max)`

Using dedicated matchers provides clearer intent and better error messages.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-pest"><code>Pest</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

test('complex expectations', function () {
    expect(!$condition)->toBeTrue();
    expect($a > $b)->toBeTrue();
    expect($a === $b)->toBeTrue();
    expect($obj instanceof ClassName)->toBeTrue();
    expect($x >= 1 && $x <= 10)->toBeTrue();
});
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

test('simplified expectations', function () {
    expect($condition)->toBeFalse();
    expect($a)->toBeGreaterThan($b);
    expect($a)->toBe($b);
    expect($obj)->toBeInstanceOf(ClassName::class);
    expect($x)->toBeBetween(1, 10);
});
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="use-specific-expectations">
<summary><code class="rule__code">use-specific-expectations</code><a class="rule__anchor" href="#use-specific-expectations" aria-label="Lien permanent vers use-specific-expectations">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Use dedicated matchers instead of generic comparisons in Pest tests.

This rule suggests more specific matchers for common patterns:
- `toBe(true)` / `toEqual(true)` -> `toBeTrue()`
- `toBe(false)` / `toEqual(false)` -> `toBeFalse()`
- `toBe(null)` / `toEqual(null)` -> `toBeNull()`
- `toBe([])` / `toBe('')` -> `toBeEmpty()`
- `not->toBeFalse()` -> `toBeTrue()`
- `not->toBeTrue()` -> `toBeFalse()`

Using dedicated matchers provides clearer intent and better error messages.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-pest"><code>Pest</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

test('generic comparisons', function () {
    expect($value)->toBe(true);
    expect($value)->toBe(false);
    expect($value)->toBe(null);
    expect($array)->toBe([]);
    expect($value)->not->toBeFalse();
});
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

test('specific matchers', function () {
    expect($value)->toBeTrue();
    expect($value)->toBeFalse();
    expect($value)->toBeNull();
    expect($array)->toBeEmpty();
});
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="valid-docblock">
<summary><code class="rule__code">valid-docblock</code><a class="rule__anchor" href="#valid-docblock" aria-label="Lien permanent vers valid-docblock">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Checks for syntax errors in docblock comments, such as malformed `{@see}` or
`{@link}` annotations. It does not enforce the presence of docblocks or verify
that declared types match the native declaration.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

/**
 * For more information, {@see https://example.com
 *
 * @param int $a
 *
 * @return int
 */
function foo($a) {
    return $a;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

/**
 * For more information, {@see https://example.com}.
 *
 * @param int $a
 *
 * @return int
 */
function foo($a) {
    return $a;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="missing-docs">
<summary><code class="rule__code">missing-docs</code><a class="rule__anchor" href="#missing-docs" aria-label="Lien permanent vers missing-docs">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects declarations that are missing a docblock.

This rule can be configured to require documentation for functions,
classes, interfaces, traits, enums, enum cases, constants, statics,
methods, and properties.

Documentation is useful when it explains intent, behaviour, usage,
invariants, or other details that are not obvious from the code alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function foo() {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

/**
 * A helpful piece of documentation.
 */
function foo() {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `classes` | `boolean` | `false` |
| `constants` | `boolean` | `true` |
| `enabled` | `boolean` | `false` |
| `enum-cases` | `boolean` | `true` |
| `enums` | `boolean` | `false` |
| `functions` | `boolean` | `true` |
| `interfaces` | `boolean` | `false` |
| `level` | `string` | `"help"` |
| `methods` | `boolean` | `true` |
| `properties` | `boolean` | `true` |
| `statics` | `boolean` | `true` |
| `traits` | `boolean` | `false` |

</div>

</details>

<details class="rule" name="rule" id="no-negated-ternary">
<summary><code class="rule__code">no-negated-ternary</code><a class="rule__anchor" href="#no-negated-ternary" aria-label="Lien permanent vers no-negated-ternary">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Flags ternary expressions whose condition is a logical negation
(`!$foo ? a : b`).

A negated condition adds a layer of indirection the reader has to
undo to follow the branches. Removing the negation and swapping
the `then` and `else` branches produces an equivalent expression
that reads more directly.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$x = !$foo ? 1 : 0;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$x = $foo ? 0 : 1;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-short-bool-cast">
<summary><code class="rule__code">no-short-bool-cast</code><a class="rule__anchor" href="#no-short-bool-cast" aria-label="Lien permanent vers no-short-bool-cast">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects the use of double negation `!!$expr` as a shorthand for `(bool) $expr`.

The explicit `(bool)` cast is clearer about the intent to convert a value
to a boolean.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$active = !!$value;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$active = (bool) $value;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

</div>

<h2 id="bestpractices">Bonnes pratiques</h2>

Patterns idiomatiques et conventions largement adoptées. Ces règles orientent le code vers la manière dont le PHP moderne est écrit par celles et ceux qui en livrent beaucoup.

<div class="rule-list">

<details class="rule" name="rule" id="final-controller">
<summary><code class="rule__code">final-controller</code><a class="rule__anchor" href="#final-controller" aria-label="Lien permanent vers final-controller">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Enforces that controller classes are declared as `final`.

In modern MVC frameworks, controllers should be treated as entry points that orchestrate the application's response to a request. They are not designed to be extension points.

Extending controllers can lead to deep inheritance chains, making the codebase rigid and difficult to maintain. It's a best practice to favor composition (injecting services for shared logic) over inheritance.

If a controller is intended as a base for others, it should be explicitly marked as `abstract`. All other concrete controllers should be `final` to prevent extension.

<blockquote class="rule-requirement">Cette règle requiert l'activation de l'un des ensembles d'intégrations suivants : <a href="#integration-symfony"><code>Symfony</code></a> ; ou <a href="#integration-laravel"><code>Laravel</code></a> ; ou <a href="#integration-tempest"><code>Tempest</code></a> ; ou <a href="#integration-spiral"><code>Spiral</code></a> ; ou <a href="#integration-cakephp"><code>CakePHP</code></a> ; ou <a href="#integration-yii"><code>Yii</code></a>.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App\Http\Controllers;

class UserController
{
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App\Http\Controllers;

final class UserController
{
    // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-inline">
<summary><code class="rule__code">no-inline</code><a class="rule__anchor" href="#no-inline" aria-label="Lien permanent vers no-inline">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Disallows inline content (text outside of PHP tags) in source files.

Most modern PHP applications are source-code only and do not use PHP as a templating
language. Inline content before `<?php`, after `?>`, or between PHP tags is typically
unintentional and can cause issues such as unexpected output or "headers already sent"
errors.

This rule is disabled by default and is intended for codebases that do not use PHP
templates.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
Hello
<?php

echo "Hello, world!";

?>
Goodbye
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App;

echo "Hello, world!";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="psl-output">
<summary><code class="rule__code">psl-output</code><a class="rule__anchor" href="#psl-output" aria-label="Lien permanent vers psl-output">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

This rule enforces the usage of Psl output functions over their PHP counterparts.
Psl output functions are preferred because they are type-safe and provide more consistent behavior.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-psl"><code>Psl</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

echo "Hello, world!";
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

Psl\IO\write_line("Hello, world!");
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="combine-consecutive-issets">
<summary><code class="rule__code">combine-consecutive-issets</code><a class="rule__anchor" href="#combine-consecutive-issets" aria-label="Lien permanent vers combine-consecutive-issets">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Suggests combining consecutive calls to `isset()` when they are joined by a logical AND.

For example, `isset($a) && isset($b)` can be turned into `isset($a, $b)`, which is more concise
and avoids repeated function calls. If one or both `isset()` calls are wrapped in parentheses,
the rule will still warn, but it will not attempt an automated fix.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if (isset($a) && isset($b)) {
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if (isset($a, $b)) {
    // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="loop-does-not-iterate">
<summary><code class="rule__code">loop-does-not-iterate</code><a class="rule__anchor" href="#loop-does-not-iterate" aria-label="Lien permanent vers loop-does-not-iterate">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects loops (for, foreach, while, do-while) that unconditionally break or return
before executing even a single iteration. Such loops are misleading or redundant
since they give the impression of iteration but never actually do so.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

for ($i = 0; $i < 3; $i++) {
    break; // The loop never truly iterates, as this break is unconditional.
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

for ($i = 0; $i < 3; $i++) {
    echo $i;
    if ($some_condition) {
        break; // This break is conditional.
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="middleware-in-routes">
<summary><code class="rule__code">middleware-in-routes</code><a class="rule__anchor" href="#middleware-in-routes" aria-label="Lien permanent vers middleware-in-routes">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule warns against applying middlewares in controllers.

Middlewares should be applied in the routes file, not in the controller.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-laravel"><code>Laravel</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App\Http\Controllers;

class UserController extends Controller
{
    public function __construct()
    {
        $this->middleware('auth');
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// routes/web.php
Route::get('/user', 'UserController@index')->middleware('auth');
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-array-accumulation-in-loop">
<summary><code class="rule__code">no-array-accumulation-in-loop</code><a class="rule__anchor" href="#no-array-accumulation-in-loop" aria-label="Lien permanent vers no-array-accumulation-in-loop">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects O(n²) array accumulation patterns inside loops.

Calling `array_merge()`, `array_merge_recursive()`, `array_unique()`, or
`array_values()` on an accumulator inside a loop copies the entire array on
every iteration. Similarly, using spread syntax (`[...$result, ...$item]`)
in a reassignment has the same cost.

Collect items first and transform once after the loop instead.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$result = [];
foreach ($items as $item) {
    $result = array_merge($result, $item);
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$chunks = [];
foreach ($items as $item) {
    $chunks[] = $item;
}
$result = array_merge(...$chunks);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-direct-db-query">
<summary><code class="rule__code">no-direct-db-query</code><a class="rule__anchor" href="#no-direct-db-query" aria-label="Lien permanent vers no-direct-db-query">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule flags all direct method calls on the global `$wpdb` object. Direct database queries
bypass the WordPress object cache, which can lead to poor performance. Using high-level functions
like `get_posts()` is safer and more efficient.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-wordpress"><code>WordPress</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

global $wpdb;
$posts = $wpdb->get_results("SELECT * FROM {$wpdb->posts} WHERE post_author = 1");
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$posts = get_posts(['author' => $author_id]);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-ini-set">
<summary><code class="rule__code">no-ini-set</code><a class="rule__anchor" href="#no-ini-set" aria-label="Lien permanent vers no-ini-set">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces that ini_set is not used.

Runtime configuration changes via ini_set make application behavior unpredictable and environment-dependent. They can mask misconfigured servers, introduce subtle bugs, and lead to inconsistent behavior between development, testing, and production environments.

Modern applications should rely on well-defined configuration through php.ini or framework specific configuration. This ensures that configuration is explicit, consistent, and controlled across all environments.

If a setting truly needs to vary between contexts, it should be handled at the infrastructure or framework configuration level, never by calling ini_set within the application code.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// This can override server settings in an unpredictable way.
ini_set( 'display_errors', 1 );
ini_set( 'memory_limit', '256M' );
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// In framework config files (e.g., wp-config.php), use constants.
define( 'WP_DEBUG', true );

// Use framework-provided functions where available.
wp_raise_memory_limit( 'admin' );
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-literal-namespace-string">
<summary><code class="rule__code">no-literal-namespace-string</code><a class="rule__anchor" href="#no-literal-namespace-string" aria-label="Lien permanent vers no-literal-namespace-string">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Flags hardcoded fully qualified class name strings. Use `::class` notation
instead for better IDE support, refactoring safety, and static analysis.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$className = 'App\Models\User';
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$className = \App\Models\User::class;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-parameter-shadowing">
<summary><code class="rule__code">no-parameter-shadowing</code><a class="rule__anchor" href="#no-parameter-shadowing" aria-label="Lien permanent vers no-parameter-shadowing">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects when a function or method parameter is shadowed by a loop variable
or catch variable, making the original parameter value inaccessible.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function read(array $domains, array $locales): void
{
    $translations = getTranslations($domains, $locales);

    foreach ($translations as $namespace => $locales) {
        // $locales now refers to the loop value, original argument is lost
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function read(array $domains, array $locales): void
{
    $translations = getTranslations($domains, $locales);

    foreach ($translations as $namespace => $namespaceLocales) {
        // $locales is still accessible
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-side-effects-with-declarations">
<summary><code class="rule__code">no-side-effects-with-declarations</code><a class="rule__anchor" href="#no-side-effects-with-declarations" aria-label="Lien permanent vers no-side-effects-with-declarations">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces that a PHP file either declares symbols (classes, functions,
constants, interfaces, traits, enums) or causes side-effects, but not
both.

Side-effects include `echo`, `print`, top-level function calls,
assignments, `include`/`require` statements, and any other executable
code outside of a symbol declaration.

This follows the PSR-1 basic coding standard: files SHOULD either
declare symbols or execute logic, but SHOULD NOT do both.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

echo 'Loading utility file...';

class StringHelper
{
    public static function slugify(string $input): string
    {
        return '';
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App;

class UserManager
{
    public function find(int $id): ?User
    {
        return null;
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `allow-class-alias` | `boolean` | `true` |
| `allow-class-exists` | `boolean` | `true` |
| `allow-conditional-declarations` | `boolean` | `true` |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-sprintf-concat">
<summary><code class="rule__code">no-sprintf-concat</code><a class="rule__anchor" href="#no-sprintf-concat" aria-label="Lien permanent vers no-sprintf-concat">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows string concatenation with the result of an `sprintf` call.

Concatenating with `sprintf` is less efficient and can be less readable than
incorporating the string directly into the format template. This pattern
creates an unnecessary intermediate string and can make the final output
harder to see at a glance.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$name = 'World';
$greeting = 'Hello, ' . sprintf('%s!', $name);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$name = 'World';
$greeting = sprintf('Hello, %s!', $name);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-anonymous-migration">
<summary><code class="rule__code">prefer-anonymous-migration</code><a class="rule__anchor" href="#prefer-anonymous-migration" aria-label="Lien permanent vers prefer-anonymous-migration">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Prefer using anonymous classes for Laravel migrations instead of named classes.
Anonymous classes are more concise and reduce namespace pollution,
making them the recommended approach for migrations.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-laravel"><code>Laravel</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class MyMigration extends Migration {
    public function up(): void {
        Schema::create('flights', function (Blueprint $table) {
            $table->id();
            $table->string('name');
            $table->string('airline');
            $table->timestamps();
        });
    }

    public function down(): void {
        Schema::drop('flights');
    }
}

return new MyMigration();
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    public function up(): void {
        Schema::create('flights', function (Blueprint $table) {
            $table->id();
            $table->string('name');
            $table->string('airline');
            $table->timestamps();
        });
    }

    public function down(): void {
        Schema::drop('flights');
    }
};
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-explode-over-preg-split">
<summary><code class="rule__code">prefer-explode-over-preg-split</code><a class="rule__anchor" href="#prefer-explode-over-preg-split" aria-label="Lien permanent vers prefer-explode-over-preg-split">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects calls to `preg_split()` whose pattern has no regex meta-characters and no
modifiers, which means the split could be done with `explode()` and no regex engine
at all.

`explode()` is faster (no compilation step), easier to read, and expresses the
intent more directly when the separator is a plain string.

The rule only fires when:

- the pattern argument is a string literal,
- the pattern has no flags after the closing delimiter,
- the content between the delimiters contains no regex meta-characters,
- and the `flags` argument (if present) is literal `0`.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$parts = preg_split('/, /', $csv);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$parts = explode(', ', $csv);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-first-class-callable">
<summary><code class="rule__code">prefer-first-class-callable</code><a class="rule__anchor" href="#prefer-first-class-callable" aria-label="Lien permanent vers prefer-first-class-callable">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Promotes the use of first-class callable syntax (`...`) for creating closures.

This rule identifies closures and arrow functions that do nothing but forward their arguments to another function or method.
In such cases, the more concise and modern first-class callable syntax, introduced in PHP 8.1, can be used instead.
This improves readability by reducing boilerplate code.

By default, this rule only checks method and static method calls. Optionally, function calls can also
be checked by enabling `check-functions`, but this may produce false positives with internal PHP
functions that enforce strict argument counts.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.1.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$names = ['Alice', 'Bob', 'Charlie'];
$uppercased_names = array_map(fn($name) => $formatter->format($name), $names);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$names = ['Alice', 'Bob', 'Charlie'];
$uppercased_names = array_map($formatter->format(...), $names);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `check-functions` | `boolean` | `false` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-test-attribute">
<summary><code class="rule__code">prefer-test-attribute</code><a class="rule__anchor" href="#prefer-test-attribute" aria-label="Lien permanent vers prefer-test-attribute">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Suggests using PHPUnit's `#[Test]` attribute instead of the `test` method name prefix.

When a method name starts with `test`, it can be replaced with a `#[Test]` attribute
and a shorter method name. This is the modern PHPUnit style (PHPUnit 10+).

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-phpunit"><code>PHPUnit</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

use PHPUnit\Framework\TestCase;

class UserTest extends TestCase
{
    public function testItReturnsFullName(): void {}
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

use PHPUnit\Framework\TestCase;
use PHPUnit\Framework\Attributes\Test;

class UserTest extends TestCase
{
    #[Test]
    public function itReturnsFullName(): void {}
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="psl-array-functions">
<summary><code class="rule__code">psl-array-functions</code><a class="rule__anchor" href="#psl-array-functions" aria-label="Lien permanent vers psl-array-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule enforces the usage of Psl array functions over their PHP counterparts.
Psl array functions are preferred because they are type-safe and provide more consistent behavior.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-psl"><code>Psl</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$filtered = array_filter($xs, fn($x) => $x > 2);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$filtered = Psl\Vec\filter($xs, fn($x) => $x > 2);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="psl-data-structures">
<summary><code class="rule__code">psl-data-structures</code><a class="rule__anchor" href="#psl-data-structures" aria-label="Lien permanent vers psl-data-structures">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule enforces the usage of Psl data structures over their SPL counterparts.

Psl data structures are preferred because they are type-safe and provide more consistent behavior.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-psl"><code>Psl</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

declare(strict_types=1);

$stack = new SplStack();
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

declare(strict_types=1);

use Psl\DataStructure\Stack;

$stack = new Stack();
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="psl-datetime">
<summary><code class="rule__code">psl-datetime</code><a class="rule__anchor" href="#psl-datetime" aria-label="Lien permanent vers psl-datetime">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule enforces the usage of Psl DateTime classes and functions over their PHP counterparts.

Psl DateTime classes and functions are preferred because they are type-safe and provide more consistent behavior.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-psl"><code>Psl</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$dateTime = new DateTime();
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$dateTime = new Psl\DateTime\DateTime();
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="psl-math-functions">
<summary><code class="rule__code">psl-math-functions</code><a class="rule__anchor" href="#psl-math-functions" aria-label="Lien permanent vers psl-math-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule enforces the usage of Psl math functions over their PHP counterparts.
Psl math functions are preferred because they are type-safe and provide more consistent behavior.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-psl"><code>Psl</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$abs = abs($number);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$abs = Psl\Math\abs($number);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="psl-randomness-functions">
<summary><code class="rule__code">psl-randomness-functions</code><a class="rule__anchor" href="#psl-randomness-functions" aria-label="Lien permanent vers psl-randomness-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule enforces the usage of Psl randomness functions over their PHP counterparts.

Psl randomness functions are preferred because they are type-safe and provide more consistent behavior.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-psl"><code>Psl</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$randomInt = random_int(0, 10);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$randomInt = Psl\SecureRandom\int(0, 10);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="psl-regex-functions">
<summary><code class="rule__code">psl-regex-functions</code><a class="rule__anchor" href="#psl-regex-functions" aria-label="Lien permanent vers psl-regex-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule enforces the usage of Psl regex functions over their PHP counterparts.

Psl regex functions are preferred because they are type-safe and provide more consistent behavior.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-psl"><code>Psl</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$result = preg_match('/\w+/', 'Hello, World!');
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$result = Psl\Regex\matches('Hello, World!', '/\w+/');
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="psl-sleep-functions">
<summary><code class="rule__code">psl-sleep-functions</code><a class="rule__anchor" href="#psl-sleep-functions" aria-label="Lien permanent vers psl-sleep-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule enforces the usage of Psl sleep functions over their PHP counterparts.

Psl sleep functions are preferred because they are type-safe, provide more consistent behavior,
and allow other tasks within the event loop to continue executing while the current Fiber pauses.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-psl"><code>Psl</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

sleep(1);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

use Psl\Async;
use Psl\DateTime;

Async\sleep(DateTime\Duration::seconds(1));
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="psl-string-functions">
<summary><code class="rule__code">psl-string-functions</code><a class="rule__anchor" href="#psl-string-functions" aria-label="Lien permanent vers psl-string-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule enforces the usage of Psl string functions over their PHP counterparts.

Psl string functions are preferred because they are type-safe and provide more consistent behavior.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-psl"><code>Psl</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$capitalized = ucfirst($string);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$capitalized = Psl\Str\capitalize($string);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="require-namespace">
<summary><code class="rule__code">require-namespace</code><a class="rule__anchor" href="#require-namespace" aria-label="Lien permanent vers require-namespace">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects files that contain definitions (classes, interfaces, enums, traits, functions, or constants)
but do not declare a namespace. Using namespaces helps avoid naming conflicts and improves code organization.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

class Foo {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App;

class Foo {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="single-class-per-file">
<summary><code class="rule__code">single-class-per-file</code><a class="rule__anchor" href="#single-class-per-file" aria-label="Lien permanent vers single-class-per-file">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Ensures that each file contains at most one class-like definition (class, interface, enum, or trait).

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App;

class Foo
{
}

class Bar
{
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App;

class Foo
{
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="use-wp-functions">
<summary><code class="rule__code">use-wp-functions</code><a class="rule__anchor" href="#use-wp-functions" aria-label="Lien permanent vers use-wp-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule encourages using WordPress's wrapper functions instead of native PHP functions for
common tasks like HTTP requests, filesystem operations, and data handling. The WordPress APIs
provide a consistent, secure, and reliable abstraction that works across different hosting
environments.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-wordpress"><code>WordPress</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// For remote requests:
$ch = curl_init();
curl_setopt($ch, CURLOPT_URL, 'https://example.com/api/data');
// ...

// For filesystem operations:
file_put_contents('/path/to/my-file.txt', 'data');
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// For remote requests:
$response = wp_remote_get('https://example.com/api/data');

// For filesystem operations:
global $wp_filesystem;
require_once ABSPATH . 'wp-admin/includes/file.php';
WP_Filesystem();
$wp_filesystem->put_contents( '/path/to/my-file.txt', 'data' );
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-interface">
<summary><code class="rule__code">prefer-interface</code><a class="rule__anchor" href="#prefer-interface" aria-label="Lien permanent vers prefer-interface">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Detects when an implementation class is used instead of the interface.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-symfony"><code>Symfony</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

use Symfony\Component\Serializer\Serializer;

class UserController
{
    public function __construct(Serializer $serializer)
    {
        $this->serializer = $serializer;
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

use Symfony\Component\Serializer\SerializerInterface;

class UserController
{
    public function __construct(SerializerInterface $serializer)
    {
        $this->serializer = $serializer;
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-while-loop">
<summary><code class="rule__code">prefer-while-loop</code><a class="rule__anchor" href="#prefer-while-loop" aria-label="Lien permanent vers prefer-while-loop">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Suggests using a `while` loop instead of a `for` loop when the `for` loop does not have any
initializations or increments. This can make the code more readable and concise.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

for (; $i < 10;) {
    echo $i;

    $i++;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

while ($i < 10) {
    echo $i;

    $i++;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-arrow-function">
<summary><code class="rule__code">prefer-arrow-function</code><a class="rule__anchor" href="#prefer-arrow-function" aria-label="Lien permanent vers prefer-arrow-function">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Promotes the use of arrow functions (`fn() => ...`) over traditional closures (`function() { ... }`).

This rule identifies closures that consist solely of a single return statement
and suggests converting them to arrow functions.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>7.4.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$a = function($x) {
    return $x + 1;
};
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$a = fn($x) => $x + 1;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-early-continue">
<summary><code class="rule__code">prefer-early-continue</code><a class="rule__anchor" href="#prefer-early-continue" aria-label="Lien permanent vers prefer-early-continue">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Suggests using early continue pattern when a loop body contains only a single if statement.

This improves code readability by reducing nesting and making the control flow more explicit.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

for ($i = 0; $i < 10; $i++) {
    if ($condition) {
        doSomething();
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

for ($i = 0; $i < 10; $i++) {
    if (!$condition) {
        continue;
    }
    doSomething();
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |
| `max_allowed_statements` | `number` | `0` |

</div>

</details>

<details class="rule" name="rule" id="prefer-pre-increment">
<summary><code class="rule__code">prefer-pre-increment</code><a class="rule__anchor" href="#prefer-pre-increment" aria-label="Lien permanent vers prefer-pre-increment">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Enforces the use of pre-increment (`++$i`) and pre-decrement (`--$i`) over
post-increment (`$i++`) and post-decrement (`$i--`).

Pre-increment is marginally more efficient and is the convention used by
the Symfony coding standards.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-symfony"><code>Symfony</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$i++;
$count--;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

++$i;
--$count;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-self-return-type">
<summary><code class="rule__code">prefer-self-return-type</code><a class="rule__anchor" href="#prefer-self-return-type" aria-label="Lien permanent vers prefer-self-return-type">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Suggests using `self` when a method's return type refers to its own enclosing
class by name.

Using `self` decouples the signature from the class name, so renaming the class
doesn't require updating return types. It also communicates intent more clearly:
'this returns an instance of the same class'.

Note: this rule does not apply to traits, because `self` inside a trait resolves
to the using class, not the trait itself. If you want to return a subclass in
inheritance-aware factory patterns, use `static` instead of `self`.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

final class Box
{
    public static function create(): Box
    {
        return new Box();
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

final class Box
{
    public static function create(): self
    {
        return new self();
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-static-closure">
<summary><code class="rule__code">prefer-static-closure</code><a class="rule__anchor" href="#prefer-static-closure" aria-label="Lien permanent vers prefer-static-closure">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Suggests adding the `static` keyword to closures and arrow functions that don't use `$this`.

Static closures don't bind `$this`, making them more memory-efficient and their intent clearer.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

class Foo {
    public function bar() {
        // Missing static - doesn't use $this
        $fn = fn($x) => $x * 2;

        // Missing static - doesn't use $this
        $closure = function($x) {
            return $x * 2;
        };
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

class Foo {
    public function bar() {
        // Static closure - doesn't use $this
        $fn = static fn($x) => $x * 2;

        // Non-static - uses $this
        $fn2 = fn() => $this->doSomething();

        // Static function - doesn't use $this
        $closure = static function($x) {
            return $x * 2;
        };
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="prefer-view-array">
<summary><code class="rule__code">prefer-view-array</code><a class="rule__anchor" href="#prefer-view-array" aria-label="Lien permanent vers prefer-view-array">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Prefer passing data to views using the array parameter in the `view()` function,
rather than chaining the `with()` method.`

Using the array parameter directly is more concise and readable.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-laravel"><code>Laravel</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

return view('user.profile')->with([
    'user' => $user,
    'profile' => $profile,
]);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

return view('user.profile', [
    'user' => $user,
    'profile' => $profile,
]);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="sorted-integer-keys">
<summary><code class="rule__code">sorted-integer-keys</code><a class="rule__anchor" href="#sorted-integer-keys" aria-label="Lien permanent vers sorted-integer-keys">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects array literals with integer keys that are not in ascending order.

PHP internally uses a "packed array" optimization for arrays with integer
keys in natural ascending order, which consumes significantly less memory
and is faster. When integer keys are out of order, PHP falls back to a
regular hash table.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$weights = [
    3  => 0.506,
    5  => 0.21246,
    10 => 0.10823,
    20 => 0.06206,
    2  => 0.06011,
    4  => 0.01233,
];
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$weights = [
    2  => 0.06011,
    3  => 0.506,
    4  => 0.01233,
    5  => 0.21246,
    10 => 0.10823,
    20 => 0.06206,
];
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="use-compound-assignment">
<summary><code class="rule__code">use-compound-assignment</code><a class="rule__anchor" href="#use-compound-assignment" aria-label="Lien permanent vers use-compound-assignment">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Enforces the use of compound assignment operators (e.g., `+=`, `.=`)
over their more verbose equivalents (`$var = $var + ...`).

Using compound assignments is more concise and idiomatic. For string
concatenation (`.=`), it can also be more performant as it avoids
creating an intermediate copy of the string.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$count = $count + 1;
$message = $message . ' Hello';
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$count += 1;
$message .= ' Hello';
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="yoda-conditions">
<summary><code class="rule__code">yoda-conditions</code><a class="rule__anchor" href="#yoda-conditions" aria-label="Lien permanent vers yoda-conditions">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

This rule enforces the use of "Yoda" conditions for comparisons. The variable should always be
on the right side of the comparison, while the constant, literal, or function call is on the left.
This prevents the common bug of accidentally using an assignment (`=`) instead of a comparison (`==`),
which would cause a fatal error in a Yoda condition instead of a silent logical bug.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// Vulnerable to the accidental assignment bug, e.g., if ($is_active = true).
if ( $is_active === true ) { /* ... */ }
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if ( true === $is_active ) { /* ... */ }
if ( 5 === $count ) { /* ... */ }
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

</div>

<h2 id="consistency">Cohérence</h2>

Uniformité stylistique sur l'ensemble du code. Choisissez une façon de faire, ces règles aident tout le monde à s'y tenir.

<div class="rule-list">

<details class="rule" name="rule" id="assertion-style">
<summary><code class="rule__code">assertion-style</code><a class="rule__anchor" href="#assertion-style" aria-label="Lien permanent vers assertion-style">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces a consistent style for PHPUnit assertion calls within test methods.

Maintaining a consistent style (e.g., always using `static::` or `$this->`)
improves code readability and helps enforce team-wide coding standards in test suites.
This rule can be configured to enforce the preferred style.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-phpunit"><code>PHPUnit</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php
// configured style: "static"
final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertTrue(true); // Incorrect style
        self::assertFalse(false); // Incorrect style
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php
// configured style: "static"
final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        static::assertTrue(true);
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `style` | `string` | `"static"` |

</div>

</details>

<details class="rule" name="rule" id="file-name">
<summary><code class="rule__code">file-name</code><a class="rule__anchor" href="#file-name" aria-label="Lien permanent vers file-name">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Ensures that a file containing a single class-like definition is named after that definition.

For example, a file containing `class Foo` must be named `Foo.php`.
Optionally, this rule can also check functions: a file containing a single function `foo`
must be named `foo.php`.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php
// File: test.php

namespace App;

class Foo
{
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php
// File: test.php

namespace App;

class test
{
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `check-functions` | `boolean` | `false` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-alternative-syntax">
<summary><code class="rule__code">no-alternative-syntax</code><a class="rule__anchor" href="#no-alternative-syntax" aria-label="Lien permanent vers no-alternative-syntax">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects the use of alternative syntax for control structures
(`endif`, `endwhile`, `endfor`, `endforeach`, `endswitch`).

The brace-style syntax is preferred for consistency with the rest
of the codebase and is the convention used by the Symfony coding standards.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if ($condition):
    echo 'yes';
endif;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if ($condition) {
    echo 'yes';
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-hash-comment">
<summary><code class="rule__code">no-hash-comment</code><a class="rule__anchor" href="#no-hash-comment" aria-label="Lien permanent vers no-hash-comment">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects shell-style comments ('#') in PHP code. Double slash comments ('//') are preferred
in PHP, as they are more consistent with the language's syntax and are easier to read.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

# This is a shell-style comment.
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// This is a good comment.
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="array-style">
<summary><code class="rule__code">array-style</code><a class="rule__anchor" href="#array-style" aria-label="Lien permanent vers array-style">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Suggests using the short array style `[..]` instead of the long array style `array(..)`,
or vice versa, depending on the configuration. The short array style is more concise and
is the preferred way to define arrays in PHP.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// By default, 'short' is enforced, so array(...) triggers a warning:
$arr = array(1, 2, 3);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// By default, `style` is 'short', so this snippet is valid:
$arr = [1, 2, 3];
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |
| `style` | `string` | `"short"` |

</div>

</details>

<details class="rule" name="rule" id="block-statement">
<summary><code class="rule__code">block-statement</code><a class="rule__anchor" href="#block-statement" aria-label="Lien permanent vers block-statement">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Enforces that `if`, `else`, `for`, `foreach`, `while`, `do-while` statements always use a block
statement body (`{ ... }`) even if they contain only a single statement.

This improves readability and prevents potential errors when adding new statements.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if (true)
    echo "Hello";

for ($i = 0; $i < 10; $i++)
    echo $i;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if (true) {
    echo "Hello";
}

for ($i = 0; $i < 10; $i++) {
    echo $i;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="braced-string-interpolation">
<summary><code class="rule__code">braced-string-interpolation</code><a class="rule__anchor" href="#braced-string-interpolation" aria-label="Lien permanent vers braced-string-interpolation">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Enforces the use of curly braces around variables within string interpolation.

Using curly braces (`{$variable}`) within interpolated strings ensures clarity and avoids potential ambiguity,
especially when variables are followed by alphanumeric characters. This rule promotes consistent and predictable code.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$a = "Hello, $name!";
$b = "Hello, ${name}!";
$c = "Hello, ${$name}!";
$d = "Hello, ${$object->getMethod()}!";
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$a = "Hello, {$name}!";
$b = "Hello, {$name}!";
$c = "Hello, {$$name}!";
$d = "Hello, {${$object->getMethod()}}!";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="no-alias-function">
<summary><code class="rule__code">no-alias-function</code><a class="rule__anchor" href="#no-alias-function" aria-label="Lien permanent vers no-alias-function">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Detects usage of function aliases (e.g., `diskfreespace` instead of `disk_free_space`)
and suggests calling the canonical (original) function name instead.
This is primarily for consistency and clarity.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// 'diskfreespace' is an alias for 'disk_free_space'
$freeSpace = diskfreespace("/");
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// 'disk_free_space' is the proper name instead of 'diskfreespace'
$freeSpace = disk_free_space("/");
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="no-php-tag-terminator">
<summary><code class="rule__code">no-php-tag-terminator</code><a class="rule__anchor" href="#no-php-tag-terminator" aria-label="Lien permanent vers no-php-tag-terminator">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Discourages the use of `?><?php` as a statement terminator. Recommends using a semicolon
(`;`) instead for clarity and consistency.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

echo "Hello World" ?><?php
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

echo "Hello World";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="no-trailing-space">
<summary><code class="rule__code">no-trailing-space</code><a class="rule__anchor" href="#no-trailing-space" aria-label="Lien permanent vers no-trailing-space">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Detects trailing whitespace at the end of comments. Trailing whitespace can cause unnecessary
diffs and formatting issues, so it is recommended to remove it.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// This is a comment with trailing whitespace.
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// This is a good comment.
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="string-style">
<summary><code class="rule__code">string-style</code><a class="rule__anchor" href="#string-style" aria-label="Lien permanent vers string-style">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Enforces a consistent string style: either prefer string interpolation
over concatenation, or prefer concatenation over interpolation.

With `style: interpolation` (default), flags concatenation like
`"foo" . $a . "bar"` and suggests `"foo{$a}bar"` instead.

With `style: concatenation`, flags interpolation like `"foo{$a}bar"`
and suggests `"foo" . $a . "bar"` instead.

Only simple, interpolable expressions are considered: variables,
property accesses, array accesses, and method calls. Concatenation
involving function calls, static access, or complex expressions is
never flagged.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// With the default `style: interpolation`:
$a = "Hello, " . $name . "!";
$b = "value: " . $obj->name;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// With the default `style: interpolation`:
$a = "Hello, {$name}!";
$b = "value: {$obj->name}";

// Complex expressions stay as concatenation (never flagged):
$c = "result: " . strtolower($input);
$d = "class: " . Foo::class;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"note"` |
| `style` | `string` | `"interpolation"` |

</div>

</details>

<details class="rule" name="rule" id="ambiguous-constant-access">
<summary><code class="rule__code">ambiguous-constant-access</code><a class="rule__anchor" href="#ambiguous-constant-access" aria-label="Lien permanent vers ambiguous-constant-access">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Enforces that all constant references made from within a namespace are explicit.

When an unqualified constant like `PHP_VERSION` is referenced from within a namespace,
PHP performs a runtime fallback check (current namespace -> global namespace). This
ambiguity can lead to unexpected behavior if a constant with the same name is later
defined in the namespace.

Making references explicit improves readability and prevents bugs.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App;

// Ambiguous: could be App\PHP_VERSION or \PHP_VERSION
$version = PHP_VERSION;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App;

use const PHP_VERSION;

// OK: Explicitly imported
$version1 = PHP_VERSION;

// OK: Explicitly global
$version2 = \PHP_VERSION;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="ambiguous-function-call">
<summary><code class="rule__code">ambiguous-function-call</code><a class="rule__anchor" href="#ambiguous-function-call" aria-label="Lien permanent vers ambiguous-function-call">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Enforces that all function calls made from within a namespace are explicit.

When an unqualified function like `strlen()` is called from within a namespace, PHP
performs a runtime fallback check (current namespace -> global namespace). This
ambiguity prevents PHP from performing powerful compile-time optimizations,
such as replacing a call to `strlen()` with the highly efficient `STRLEN` opcode.

Making calls explicit improves readability, prevents bugs, and allows for significant
performance gains in some cases.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App;

// Ambiguous: could be App\strlen or \strlen
$length = strlen("hello");
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App;

use function strlen;

// OK: Explicitly imported
$length1 = strlen("hello");

// OK: Explicitly global
$length2 = \strlen("hello");

// OK: Explicitly namespaced
$value = namespace\my_function();
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="class-name">
<summary><code class="rule__code">class-name</code><a class="rule__anchor" href="#class-name" aria-label="Lien permanent vers class-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects class declarations that do not follow class naming convention.

Class names should be in class case, also known as PascalCase.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

class my_class {}

class myClass {}

class MY_CLASS {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

class MyClass {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |
| `psr` | `boolean` | `false` |

</div>

</details>

<details class="rule" name="rule" id="constant-name">
<summary><code class="rule__code">constant-name</code><a class="rule__anchor" href="#constant-name" aria-label="Lien permanent vers constant-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects constant declarations that do not follow constant naming convention.

Constant names should be in constant case, also known as UPPER_SNAKE_CASE.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

const myConstant = 42;
const my_constant = 42;
const My_Constant = 42;

class MyClass {
    public const int myConstant = 42;
    public const int my_constant = 42;
    public const int My_Constant = 42;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

const MY_CONSTANT = 42;

class MyClass {
    public const int MY_CONSTANT = 42;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="enum-name">
<summary><code class="rule__code">enum-name</code><a class="rule__anchor" href="#enum-name" aria-label="Lien permanent vers enum-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects enum declarations that do not follow class naming convention.

Enum names should be in class case, also known as PascalCase.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

enum my_enum {}
enum myEnum {}
enum MY_ENUM {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

enum MyEnum {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="function-name">
<summary><code class="rule__code">function-name</code><a class="rule__anchor" href="#function-name" aria-label="Lien permanent vers function-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects function declarations that do not follow camel or snake naming convention.

Function names should be in camel case or snake case, depending on the configuration.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function MyFunction() {}

function My_Function() {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function my_function() {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `camel` | `boolean` | `false` |
| `either` | `boolean` | `false` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="interface-name">
<summary><code class="rule__code">interface-name</code><a class="rule__anchor" href="#interface-name" aria-label="Lien permanent vers interface-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects interface declarations that do not follow class naming convention.

Interface names should be in class case and suffixed with `Interface`, depending on the configuration.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

interface myInterface {}
interface my_interface {}
interface MY_INTERFACE {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

interface MyInterface {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |
| `psr` | `boolean` | `false` |

</div>

</details>

<details class="rule" name="rule" id="lowercase-keyword">
<summary><code class="rule__code">lowercase-keyword</code><a class="rule__anchor" href="#lowercase-keyword" aria-label="Lien permanent vers lowercase-keyword">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Enforces that PHP keywords (like `if`, `else`, `return`, `function`, etc.) be written
in lowercase. Using uppercase or mixed case is discouraged for consistency and readability.

When the `drupal` integration is enabled, `TRUE`, `FALSE`, and `NULL` are exempted to
match Drupal's coding standards (and the `drupal` formatter preset).

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?PHP

IF (TRUE) {
    ECHO "Keywords not in lowercase";
} ELSE {
    RETURN;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if (true) {
    echo "All keywords in lowercase";
} else {
    return;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="lowercase-type-hint">
<summary><code class="rule__code">lowercase-type-hint</code><a class="rule__anchor" href="#lowercase-type-hint" aria-label="Lien permanent vers lowercase-type-hint">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Enforces that PHP type hints (like `void`, `bool`, `int`, `float`, etc.) be written
in lowercase. Using uppercase or mixed case is discouraged for consistency
and readability.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function example(Int $param): VOID {
    return;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function example(int $param): void {
    return;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="method-name">
<summary><code class="rule__code">method-name</code><a class="rule__anchor" href="#method-name" aria-label="Lien permanent vers method-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects method declarations that do not follow the configured naming convention.

By default, method names should be in camelCase. Magic methods (prefixed with `__`)
are always excluded.

The `use-snake-case-for-tests` option enforces snake_case for test methods
(names starting with `test`), which is a common convention in PHPUnit.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

class Foo
{
    public function GetName(): string {}
    public function set_name(string $name): void {}
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

class Foo
{
    public function getName(): string {}
    public function setName(string $name): void {}
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `camel` | `boolean` | `true` |
| `either` | `boolean` | `false` |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |
| `use-snake-case-for-tests` | `boolean` | `false` |

</div>

</details>

<details class="rule" name="rule" id="no-fully-qualified-global-class-like">
<summary><code class="rule__code">no-fully-qualified-global-class-like</code><a class="rule__anchor" href="#no-fully-qualified-global-class-like" aria-label="Lien permanent vers no-fully-qualified-global-class-like">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Disallows fully-qualified class-like references within a namespace.

Instead of using the backslash prefix (e.g., `new \DateTime()` or `\Exception`
in a type hint), prefer an explicit `use` import statement. This improves
readability and keeps imports centralized at the top of the file.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App;

$dt = new \DateTime();

function foo(\DateTime $dt): \Exception {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App;

use DateTime;
use Exception;

$dt = new DateTime();

function foo(DateTime $dt): Exception {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-fully-qualified-global-constant">
<summary><code class="rule__code">no-fully-qualified-global-constant</code><a class="rule__anchor" href="#no-fully-qualified-global-constant" aria-label="Lien permanent vers no-fully-qualified-global-constant">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Disallows fully-qualified references to global constants within a namespace.

Instead of using the backslash prefix (e.g., `\PHP_VERSION`),
prefer an explicit `use const` import statement. This improves
readability and keeps imports centralized at the top of the file.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App;

$version = \PHP_VERSION;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App;

use const PHP_VERSION;

$version = PHP_VERSION;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-fully-qualified-global-function">
<summary><code class="rule__code">no-fully-qualified-global-function</code><a class="rule__anchor" href="#no-fully-qualified-global-function" aria-label="Lien permanent vers no-fully-qualified-global-function">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Disallows fully-qualified references to global functions within a namespace.

Instead of using the backslash prefix (e.g., `\strlen()`),
prefer an explicit `use function` import statement. This improves
readability and keeps imports centralized at the top of the file.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App;

$length = \strlen("hello");
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App;

use function strlen;

$length = strlen("hello");
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="property-name">
<summary><code class="rule__code">property-name</code><a class="rule__anchor" href="#property-name" aria-label="Lien permanent vers property-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects class property declarations that do not follow camel or snake naming convention.

Property names should be in camel case or snake case, depending on the configuration.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

final class Foo {
    public string $My_Property;

    public function __construct(
        public int $My_Promoted_Property,
    ) {}
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

final class Foo {
    public string $myProperty;

    public function __construct(
        public int $myPromotedProperty,
    ) {}
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `camel` | `boolean` | `true` |
| `either` | `boolean` | `false` |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="trait-name">
<summary><code class="rule__code">trait-name</code><a class="rule__anchor" href="#trait-name" aria-label="Lien permanent vers trait-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects trait declarations that do not follow class naming convention.
Trait names should be in class case and suffixed with `Trait`, depending on the configuration.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

trait myTrait {}
trait my_trait {}
trait MY_TRAIT {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

trait MyTrait {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |
| `psr` | `boolean` | `false` |

</div>

</details>

<details class="rule" name="rule" id="variable-name">
<summary><code class="rule__code">variable-name</code><a class="rule__anchor" href="#variable-name" aria-label="Lien permanent vers variable-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects variable declarations that do not follow camel or snake naming convention.

Variable names should be in camel case or snake case, depending on the configuration.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$MyVariable = 1;
$My_Variable = 2;

function foo($MyParam) {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$my_variable = 1;

function foo($my_param) {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `camel` | `boolean` | `false` |
| `check-parameters` | `boolean` | `true` |
| `either` | `boolean` | `true` |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

</div>

<h2 id="deprecation">Obsolescence</h2>

Fonctionnalités et API PHP marquées comme obsolètes en amont, et qui finiront par être supprimées. Migrez avant qu'elles ne cassent.

<div class="rule-list">

<details class="rule" name="rule" id="deprecated-cast">
<summary><code class="rule__code">deprecated-cast</code><a class="rule__anchor" href="#deprecated-cast" aria-label="Lien permanent vers deprecated-cast">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detect the usage of deprecated type casts in PHP code.

In PHP 8.5, the following type casts have been deprecated:

- `(integer)`: The integer cast has been deprecated in favor of `(int)`.
- `(boolean)`: The boolean cast has been deprecated in favor of `(bool)`.
- `(double)`: The double cast has been deprecated in favor of `(float)`.
- `(binary)`: The binary cast has been deprecated in favor of `(string)`.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.5.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

(integer) $value;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

(int) $value;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="deprecated-shell-execute-string">
<summary><code class="rule__code">deprecated-shell-execute-string</code><a class="rule__anchor" href="#deprecated-shell-execute-string" aria-label="Lien permanent vers deprecated-shell-execute-string">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detect the usage of deprecated shell execute strings in PHP code.

In PHP 8.5, the shell execute string syntax (enclosed in backticks, e.g., `` `ls -l` ``) has been deprecated.

This rule identifies instances of shell execute strings and provides guidance on how to replace them with safer alternatives,
such as using the `shell_exec()` function or other appropriate methods for executing shell commands.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.5.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

`ls -l`;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

shell_exec('ls -l');
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="deprecated-switch-semicolon">
<summary><code class="rule__code">deprecated-switch-semicolon</code><a class="rule__anchor" href="#deprecated-switch-semicolon" aria-label="Lien permanent vers deprecated-switch-semicolon">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detect the usage of semicolon as a switch case separator.

In PHP 8.5, the use of a semicolon (`;`) as a case separator in switch statements has been deprecated.

Instead, the colon (`:`) should be used to separate case statements.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.5.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

switch ($value) {
    case 1;
        // code for case 1
        break;
    case 2;
        // code for case 2
        break;
    default;
        // default case
        break;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

switch ($value) {
    case 1:
        // code for case 1
        break;
    case 2:
        // code for case 2
        break;
    default:
        // default case
        break;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="explicit-nullable-param">
<summary><code class="rule__code">explicit-nullable-param</code><a class="rule__anchor" href="#explicit-nullable-param" aria-label="Lien permanent vers explicit-nullable-param">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects parameters that are implicitly nullable and rely on a deprecated feature.

Such parameters are considered deprecated; an explicit nullable type hint is recommended.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.4.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function foo(string $param = null) {}

function bar(string $param = NULL) {}

function baz(object $param = null) {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function foo(?string $param) {}

function bar(null|string $param) {}

function baz(null|object $param = null) {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-underscore-class">
<summary><code class="rule__code">no-underscore-class</code><a class="rule__anchor" href="#no-underscore-class" aria-label="Lien permanent vers no-underscore-class">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects class, interface, trait, or enum declarations named `_`.

Such names are considered deprecated; a more descriptive identifier is recommended.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.4.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

class _ {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

class MyService {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-void-reference-return">
<summary><code class="rule__code">no-void-reference-return</code><a class="rule__anchor" href="#no-void-reference-return" aria-label="Lien permanent vers no-void-reference-return">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects functions, methods, closures, arrow functions, and set property hooks that return by reference from a void function.
Such functions are considered deprecated; returning by reference from a void function is deprecated since PHP 8.0.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.2.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function &foo(): void {
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function &foo(): string {
    // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="optional-param-order">
<summary><code class="rule__code">optional-param-order</code><a class="rule__anchor" href="#optional-param-order" aria-label="Lien permanent vers optional-param-order">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects optional parameters defined before required parameters in function-like declarations.
Such parameter order is considered deprecated; required parameters should precede optional parameters.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.0.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function foo(?string $optional = null, string $required): void {}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function foo(string $required, ?string $optional = null): void {}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="maintainability">Maintenabilité</h2>

Du code difficile à maintenir dans la durée, trop complexe, trop emmêlé, trop fragile. Ces règles font remonter le coût tôt.

<div class="rule-list">

<details class="rule" name="rule" id="cyclomatic-complexity">
<summary><code class="rule__code">cyclomatic-complexity</code><a class="rule__anchor" href="#cyclomatic-complexity" aria-label="Lien permanent vers cyclomatic-complexity">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Checks the cyclomatic complexity of classes, traits, enums, interfaces, functions, and closures.

Cyclomatic complexity is a measure of the number of linearly independent paths through a program's source code.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function validateUser($user) {
    if (!isset($user['email'])) {
        return false;
    }

    if (!filter_var($user['email'], FILTER_VALIDATE_EMAIL)) {
        return false;
    }

    if (!isset($user['age'])) {
        return false;
    }

    if ($user['age'] < 18) {
        return false;
    }

    if ($user['age'] > 120) {
        return false;
    }

    if (!isset($user['name'])) {
        return false;
    }

    if (strlen($user['name']) < 2) {
        return false;
    }

    if (!isset($user['country'])) {
        return false;
    }

    if (!in_array($user['country'], ['US', 'UK', 'CA'])) {
        return false;
    }

    if (isset($user['phone'])) {
        if (!preg_match('/^\d{10}$/', $user['phone'])) {
            return false;
        }
    }

    if (isset($user['preferences'])) {
        if (is_array($user['preferences'])) {
            foreach ($user['preferences'] as $key => $value) {
                if ($key === 'newsletter') {
                    if ($value !== true && $value !== false) {
                        return false;
                    }
                }
            }
        }
    }

    if (isset($user['address'])) {
        if (!isset($user['address']['street'])) {
            return false;
        }
        if (!isset($user['address']['city'])) {
            return false;
        }
    }

    return true;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function validateUser($user) {
    if (!isValidEmail($user['email'])) {
        return false;
    }

    if (!isValidAge($user['age'])) {
        return false;
    }

    if (!hasRequiredFields($user)) {
        return false;
    }

    return true;
}

function isValidEmail($email) {
    return filter_var($email, FILTER_VALIDATE_EMAIL) !== false;
}

function isValidAge($age) {
    return $age >= 18 && $age <= 120;
}

function hasRequiredFields($user) {
    return isset($user['name']) && isset($user['email']);
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `method-threshold` | `null` | `null` |
| `threshold` | `number` | `15` |

</div>

</details>

<details class="rule" name="rule" id="excessive-parameter-list">
<summary><code class="rule__code">excessive-parameter-list</code><a class="rule__anchor" href="#excessive-parameter-list" aria-label="Lien permanent vers excessive-parameter-list">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects functions, closures, and methods with too many parameters.

If the number of parameters exceeds a configurable threshold, an issue is reported.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function createUser($name, $email, $password, $age, $country, $city, $zipCode) {
    return true;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function processOrder($orderId, $userId, $total, $status, $date) {
    return true;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `constructor-threshold` | `null` | `null` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `number` | `5` |

</div>

</details>

<details class="rule" name="rule" id="kan-defect">
<summary><code class="rule__code">kan-defect</code><a class="rule__anchor" href="#kan-defect" aria-label="Lien permanent vers kan-defect">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects classes, traits, interfaces, functions, and closures with high kan defect.

The "Kan Defect" metric is a heuristic for estimating defect proneness in a class or similar structure.
It counts control-flow statements (`while`, `do`, `foreach`, `if`, and `switch`) and sums them using a
formula loosely based on the work of Stephen H. Kan.

References:
  - https://github.com/phpmetrics/PhpMetrics/blob/c43217cd7783bbd54d0b8c1dd43f697bc36ef79d/src/Hal/Metric/Class_/Complexity/KanDefectVisitor.php
  - https://phpmetrics.org/

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function handleRequest($request) {
    if (empty($request)) {
        return null;
    }

    if (!isset($request['type'])) {
        return null;
    }

    switch ($request['type']) {
        case 'create':
            if (!isset($request['data'])) {
                return null;
            }
            break;
        case 'update':
            if (!isset($request['id'])) {
                return null;
            }
            break;
        case 'delete':
            if (!isset($request['id'])) {
                return null;
            }
            break;
    }

    if (isset($request['filters'])) {
        foreach ($request['filters'] as $key => $value) {
            switch ($key) {
                case 'status':
                    if ($value === 'active') {
                        // filter
                    }
                    break;
                case 'category':
                    if (!empty($value)) {
                        // filter
                    }
                    break;
            }
        }
    }

    while (!empty($request['items'])) {
        $item = array_shift($request['items']);
        if ($item['valid']) {
            foreach ($item['tags'] as $tag) {
                if ($tag === 'important') {
                    // process
                }
            }
        }
    }

    foreach ($request['metadata'] as $meta) {
        switch ($meta['type']) {
            case 'timestamp':
                break;
            case 'user':
                break;
        }
    }

    return ['status' => 'success'];
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function handleRequest($request) {
    $validated = validateRequest($request);
    $processed = processRequest($validated);
    return formatResponse($processed);
}

function validateRequest($request) {
    if (empty($request['type'])) {
        return null;
    }
    return $request;
}

function processRequest($request) {
    return match($request['type']) {
        'create' => createResource($request),
        'update' => updateResource($request),
        'delete' => deleteResource($request),
        default => null
    };
}

function formatResponse($data) {
    return ['status' => 'success', 'data' => $data];
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `number` | `1.6` |

</div>

</details>

<details class="rule" name="rule" id="too-many-enum-cases">
<summary><code class="rule__code">too-many-enum-cases</code><a class="rule__anchor" href="#too-many-enum-cases" aria-label="Lien permanent vers too-many-enum-cases">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects enums with too many cases.

This rule checks the number of cases in enums. If the number of cases exceeds a configurable threshold, an issue is reported.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

enum LargeEnum {
    case A;
    case B;
    case C;
    case D;
    case E;
    case F;
    case G;
    case H;
    case I;
    case J;
    case K;
    case L;
    case M;
    case N;
    case O;
    case P;
    case Q;
    case R;
    case S;
    case T;
    case U;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

enum SimpleEnum {
    case A;
    case B;
    case C;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `number` | `20` |

</div>

</details>

<details class="rule" name="rule" id="too-many-methods">
<summary><code class="rule__code">too-many-methods</code><a class="rule__anchor" href="#too-many-methods" aria-label="Lien permanent vers too-many-methods">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects class-like structures with too many methods.

This rule checks the number of methods in classes, traits, enums, and interfaces.
If the number of methods exceeds a configurable threshold, an issue is reported.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

class ComplexClass {
    public function a() {}
    public function b() {}
    public function c() {}
    public function d() {}
    public function e() {}
    public function f() {}
    public function g() {}
    public function h() {}
    public function i() {}
    public function j() {}
    public function k() {}
    public function l() {}
    public function m() {}
    public function n() {}
    public function o() {}
    public function p() {}
    public function q() {}
    public function r() {}
    public function s() {}
    public function t() {}
    public function u() {}
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

class SimpleClass {
    public function a() {}
    public function b() {}
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `count-hooks` | `boolean` | `false` |
| `count-setters-and-getters` | `boolean` | `false` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `number` | `10` |

</div>

</details>

<details class="rule" name="rule" id="too-many-properties">
<summary><code class="rule__code">too-many-properties</code><a class="rule__anchor" href="#too-many-properties" aria-label="Lien permanent vers too-many-properties">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects class-like structures with too many properties.

This rule checks the number of properties in classes, traits, and interfaces.
If the number of properties exceeds a configurable threshold, an issue is reported.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

class ComplexClass {
    public $a; public $b; public $c; public $d; public $e;
    public $f; public $g; public $h; public $i; public $j; public $k;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

class SimpleClass {
    public $a;
    public $b;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `number` | `10` |

</div>

</details>

<details class="rule" name="rule" id="excessive-nesting">
<summary><code class="rule__code">excessive-nesting</code><a class="rule__anchor" href="#excessive-nesting" aria-label="Lien permanent vers excessive-nesting">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Checks if the nesting level in any block exceeds a configurable threshold.

Deeply nested code is harder to read, understand, and maintain.
Consider refactoring to use early returns, helper methods, or clearer control flow.

The `function-like-threshold` option allows setting a separate, typically lower,
threshold for individual functions, methods, closures, and property hooks.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if ($a) {
    if ($b) {
        if ($c) {
            if ($d) {
                if ($e) {
                    if ($f) {
                        if ($g) {
                            if ($h) {
                                echo "Too deeply nested!";
                            }
                        }
                    }
                }
            }
        }
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if ($condition) {
    while ($otherCondition) {
        echo "Hello"; // nesting depth = 2
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `function-like-threshold` | `null` | `null` |
| `level` | `string` | `"warning"` |
| `threshold` | `number` | `7` |

</div>

</details>

<details class="rule" name="rule" id="halstead">
<summary><code class="rule__code">halstead</code><a class="rule__anchor" href="#halstead" aria-label="Lien permanent vers halstead">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Computes Halstead metrics (volume, difficulty, effort) and reports if they exceed configured thresholds.

Halstead metrics are calculated by counting operators and operands in the analyzed code.
For more info: https://en.wikipedia.org/wiki/Halstead_complexity_measures

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function processOrderData($orders) {
    $result = [];
    $total1 = 0;
    $total2 = 0;
    $total3 = 0;
    $discount1 = 0;
    $discount2 = 0;
    $discount3 = 0;
    $count1 = 0;
    $count2 = 0;
    $count3 = 0;
    $sum1 = 0;
    $sum2 = 0;
    $sum3 = 0;

    for ($i = 0; $i < count($orders); $i++) {
        $order = $orders[$i];
        if ($order['status'] === 'pending') {
            $price = $order['price'];
            $quantity = $order['quantity'];
            $subtotal = $price * $quantity;
            $total1 = $total1 + $subtotal;

            if ($subtotal > 100) {
                $discount1 = $subtotal * 0.1;
                $total1 = $total1 - $discount1;
                $count1 = $count1 + 1;
            }

            if ($subtotal > 200) {
                $discount2 = $subtotal * 0.15;
                $total2 = $total2 + $subtotal - $discount2;
                $count2 = $count2 + 1;
            }

            if ($subtotal > 300) {
                $discount3 = $subtotal * 0.2;
                $total3 = $total3 + $subtotal - $discount3;
                $count3 = $count3 + 1;
            }

            $sum1 = $sum1 + $price;
            $sum2 = $sum2 + $quantity;
            $sum3 = $sum3 + $subtotal;

            for ($j = 0; $j < $quantity; $j++) {
                $itemCost = $price / $quantity;
                $taxRate = 0.08;
                $tax = $itemCost * $taxRate;
                $finalCost = $itemCost + $tax;
                $sum1 = $sum1 + $finalCost;

                if ($finalCost > 50) {
                    $extraDiscount = $finalCost * 0.05;
                    $sum2 = $sum2 + $extraDiscount;
                }
            }
        }
    }

    $result['total1'] = $total1;
    $result['total2'] = $total2;
    $result['total3'] = $total3;
    $result['count1'] = $count1;
    $result['count2'] = $count2;
    $result['count3'] = $count3;
    $result['sum1'] = $sum1;
    $result['sum2'] = $sum2;
    $result['sum3'] = $sum3;

    return $result;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function processOrderData($orders) {
    $filtered = filterValidOrders($orders);
    $totals = calculateTotals($filtered);
    return applyDiscounts($totals);
}

function filterValidOrders($orders) {
    return array_filter($orders, fn($order) => $order['status'] === 'valid');
}

function calculateTotals($orders) {
    return array_map(fn($order) => $order['price'] * $order['quantity'], $orders);
}

function applyDiscounts($totals) {
    return array_map(fn($total) => $total > 100 ? $total * 0.9 : $total, $totals);
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `difficulty-threshold` | `number` | `12.0` |
| `effort-threshold` | `number` | `5000.0` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `volume-threshold` | `number` | `1000.0` |

</div>

</details>

<details class="rule" name="rule" id="no-goto">
<summary><code class="rule__code">no-goto</code><a class="rule__anchor" href="#no-goto" aria-label="Lien permanent vers no-goto">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Detects the use of `goto` statements and labels. The `goto` statement can make
code harder to read, understand, and maintain. It can lead to "spaghetti code"
and make it difficult to follow the flow of execution.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$i = 0;
loop:
if ($i >= 10) {
    goto end;
}

$i++;
goto loop;
end:
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$i = 0;
while ($i < 10) {
    if ($i === 5) {
        break; // Structured control flow.
    }
    $i++;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="no-boolean-flag-parameter">
<summary><code class="rule__code">no-boolean-flag-parameter</code><a class="rule__anchor" href="#no-boolean-flag-parameter" aria-label="Lien permanent vers no-boolean-flag-parameter">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Flags function-like parameters that use a boolean type.

Boolean flag parameters can indicate a violation of the Single Responsibility Principle (SRP).
Refactor by extracting the flag logic into its own class or method.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function get_difference(string $a, string $b, bool $ignore_case): string {
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function get_difference(string $a, string $b): string {
    // ...
}

function get_difference_case_insensitive(string $a, string $b): string {
    // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `exclude-constructors` | `boolean` | `true` |
| `exclude-setters` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-else-clause">
<summary><code class="rule__code">no-else-clause</code><a class="rule__anchor" href="#no-else-clause" aria-label="Lien permanent vers no-else-clause">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Flags `if` statements that include an `else` or `elseif` branch.

Using `else` or `elseif` can lead to deeply nested code and complex control flow.
This can often be simplified by using early returns (guard clauses), which makes
the code easier to read and maintain by reducing its cyclomatic complexity.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function process($user) {
    if ($user->isVerified()) {
        // "Happy path" is nested
        $user->login();
    } else {
        // Logic is split across branches
        return;
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function process($user) {
    if (!$user->isVerified()) {
        return; // Early return
    }

    // "Happy path" continues here
    $user->login();
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

</div>

<h2 id="redundancy">Redondance</h2>

Code mort, valeurs inutilisées, constructions sans effet observable. Les retirer garde le code honnête.

<div class="rule-list">

<details class="rule" name="rule" id="inline-variable-return">
<summary><code class="rule__code">inline-variable-return</code><a class="rule__anchor" href="#inline-variable-return" aria-label="Lien permanent vers inline-variable-return">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects unnecessary variable assignments immediately before returning the variable.

When a variable is only used once right after being assigned, the assignment
can be inlined into the return statement.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function getValue() {
    $result = computeResult();
    return $result;
}

function getArray() {
    $arr = [1, 2, 3];
    return $arr;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function getValue() {
    return computeResult();
}

function process() {
    $result = computeResult();
    log($result);
    return $result;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-iterator-to-array-in-foreach">
<summary><code class="rule__code">no-iterator-to-array-in-foreach</code><a class="rule__anchor" href="#no-iterator-to-array-in-foreach" aria-label="Lien permanent vers no-iterator-to-array-in-foreach">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects `iterator_to_array()` calls used directly as a `foreach` expression.

Since `foreach` natively supports any `Traversable`, wrapping an iterator in
`iterator_to_array()` is redundant and causes unnecessary memory allocation.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

foreach (iterator_to_array($iterator) as $value) {
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

foreach ($iterator as $value) {
    // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-literal-return">
<summary><code class="rule__code">no-redundant-literal-return</code><a class="rule__anchor" href="#no-redundant-literal-return" aria-label="Lien permanent vers no-redundant-literal-return">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects redundant literal guard patterns where an if statement checks if a variable
equals a literal and returns that same literal, followed by returning the variable.

This pattern is redundant because if the variable equals the literal, returning the
variable would return the same value anyway.

This includes patterns with else clauses and elseif chains where all branches
follow the same redundant pattern.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function getValue($x) {
    if ($x === null) {
        return null;
    }
    return $x;
}

function getWithElse($x) {
    if ($x === null) {
        return null;
    } else {
        return $x;
    }
}

function getWithElseIf($x) {
    if ($x === null) {
        return null;
    } elseif ($x === '') {
        return '';
    }
    return $x;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function getValue($x) {
    return $x;
}

function getValueOrDefault($x, $default) {
    if ($x === null) {
        return $default;
    }
    return $x;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-use">
<summary><code class="rule__code">no-redundant-use</code><a class="rule__anchor" href="#no-redundant-use" aria-label="Lien permanent vers no-redundant-use">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects `use` statements that import items that are never used or are redundant
because they import from the same namespace.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php
namespace App;

use App\Helpers\ArrayHelper;
use App\Helpers\StringHelper; // StringHelper is not used.

$result = ArrayHelper::combine([]);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php
namespace App;

use App\Helpers\ArrayHelper;

$result = ArrayHelper::combine([]);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-self-assignment">
<summary><code class="rule__code">no-self-assignment</code><a class="rule__anchor" href="#no-self-assignment" aria-label="Lien permanent vers no-self-assignment">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects and removes self-assignments where a variable or property is assigned to itself.

Self-assignments have no effect and are typically mistakes or leftover from refactoring.
For object properties, the fix is marked as potentially unsafe because reading or writing
properties may have side effects through magic methods (__get, __set) or property hooks (PHP 8.4+).

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$a = $a;
$this->x = $this->x;
$foo->bar = $foo->bar;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$a = $b;
$this->x = $other->x;
$foo->bar = $baz->bar;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-empty-comment">
<summary><code class="rule__code">no-empty-comment</code><a class="rule__anchor" href="#no-empty-comment" aria-label="Lien permanent vers no-empty-comment">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Detects empty comments in the codebase. Empty comments are not useful and should be removed
to keep the codebase clean and maintainable.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

//
#
/**/
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// This is a useful comment.
//
// And so is this whole single line comment block, including the enclosed empty line.
# This is also a useful comment.
/**
 * This is a docblock.
 */
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |
| `preserve-single-line-comments` | `boolean` | `false` |

</div>

</details>

<details class="rule" name="rule" id="no-empty-loop">
<summary><code class="rule__code">no-empty-loop</code><a class="rule__anchor" href="#no-empty-loop" aria-label="Lien permanent vers no-empty-loop">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Detects loops (`for`, `foreach`, `while`, `do-while`) that have an empty body. An empty
loop body does not perform any actions and is likely a mistake or redundant code.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

while (should_wait()) {
    // Empty loop body
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

foreach ($items as $item) {
    process($item);
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="no-is-null">
<summary><code class="rule__code">no-is-null</code><a class="rule__anchor" href="#no-is-null" aria-label="Lien permanent vers no-is-null">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Detects usage of the `is_null()` function and suggests using a strict `=== null` comparison instead.

The `is_null()` function is redundant because `=== null` achieves the same result with clearer intent
and without the overhead of a function call.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if (is_null($value)) {
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if ($value === null) {
    // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"note"` |

</div>

</details>

<details class="rule" name="rule" id="constant-condition">
<summary><code class="rule__code">constant-condition</code><a class="rule__anchor" href="#constant-condition" aria-label="Lien permanent vers constant-condition">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects `if` statements where the condition is a constant that always
evaluates to `true` or `false`.

Such statements are redundant. If the condition is always `true`, the `if`
wrapper is unnecessary. If it's always `false`, the enclosed code is dead
and can be removed or refactored.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php
if (true) {
    echo "This will always run";
}

if (false) {
    echo "This is dead code";
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php
if ($variable > 10) {
    echo "Greater than 10";
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-closing-tag">
<summary><code class="rule__code">no-closing-tag</code><a class="rule__anchor" href="#no-closing-tag" aria-label="Lien permanent vers no-closing-tag">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant closing tags ( `?>` ) at the end of a file.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

echo "Hello, world!";

?>
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

echo "Hello, world!";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-noop">
<summary><code class="rule__code">no-noop</code><a class="rule__anchor" href="#no-noop" aria-label="Lien permanent vers no-noop">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant `noop` statements.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

echo "Hello, world!";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-null-property-init">
<summary><code class="rule__code">no-null-property-init</code><a class="rule__anchor" href="#no-null-property-init" aria-label="Lien permanent vers no-null-property-init">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant `= null` initialization on untyped properties.

Untyped properties already default to `null`, making an explicit
`= null` initializer unnecessary.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

class Foo {
    public $name = null;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

class Foo {
    public $name;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-protected-in-final">
<summary><code class="rule__code">no-protected-in-final</code><a class="rule__anchor" href="#no-protected-in-final" aria-label="Lien permanent vers no-protected-in-final">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects `protected` items in final classes or enums.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

final class Foo {
    protected string $foo;
    protected(set) string $bar;
    protected private(set) string $baz;

    protected function fun(): void {
        // ...
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

final class Foo {
    private string $foo;
    private(set) string $bar;
    private string $baz;

    private function fun(): void {
        // ...
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-binary-string-prefix">
<summary><code class="rule__code">no-redundant-binary-string-prefix</code><a class="rule__anchor" href="#no-redundant-binary-string-prefix" aria-label="Lien permanent vers no-redundant-binary-string-prefix">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects the redundant `b`/`B` prefix on string literals. The binary string prefix
has no effect in PHP and can be safely removed.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$foo = b'hello';
$bar = b"world";
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$foo = 'hello';
$bar = "world";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-block">
<summary><code class="rule__code">no-redundant-block</code><a class="rule__anchor" href="#no-redundant-block" aria-label="Lien permanent vers no-redundant-block">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant blocks around statements.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

{
    echo "Hello, world!";
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

echo "Hello, world!";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-continue">
<summary><code class="rule__code">no-redundant-continue</code><a class="rule__anchor" href="#no-redundant-continue" aria-label="Lien permanent vers no-redundant-continue">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant `continue` statements in loops.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

while (true) {
    echo "Hello, world!";
    continue; // Redundant `continue` statement
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

while (true) {
    echo "Hello, world!";
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-else">
<summary><code class="rule__code">no-redundant-else</code><a class="rule__anchor" href="#no-redundant-else" aria-label="Lien permanent vers no-redundant-else">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Flags `if`/`else` statements where the `if` branch always terminates
control flow (via `return`, `throw`, `exit`, `die`, `continue`, or `break`).

When the `if` branch unconditionally terminates, the `else` branch becomes
unnecessary nesting. Extracting the `else` body to follow the `if` flattens
the control flow without changing semantics.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function process($user) {
    if (!$user->isVerified()) {
        return;
    } else {
        $user->login();
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function process($user) {
    if (!$user->isVerified()) {
        return;
    }

    $user->login();
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-file">
<summary><code class="rule__code">no-redundant-file</code><a class="rule__anchor" href="#no-redundant-file" aria-label="Lien permanent vers no-redundant-file">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant files that contain no executable code or declarations.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

declare(strict_types=1);
// This file is redundant.
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

declare(strict_types=1);

function foo(): void {
    return 42;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-final">
<summary><code class="rule__code">no-redundant-final</code><a class="rule__anchor" href="#no-redundant-final" aria-label="Lien permanent vers no-redundant-final">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant `final` modifiers on methods in final classes or enum methods.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

final class Foo {
    final public function bar(): void {
        // ...
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

final class Foo {
    public function bar(): void {
        // ...
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-isset">
<summary><code class="rule__code">no-redundant-isset</code><a class="rule__anchor" href="#no-redundant-isset" aria-label="Lien permanent vers no-redundant-isset">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant arguments in `isset()` calls where a nested access already implies the parent checks.

For example, `isset($d, $d['first'], $d['first']['second'])` can be simplified to
`isset($d['first']['second'])` because checking a nested array access or property access
implicitly verifies that all parent levels exist.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if (isset($d, $d['first'], $d['first']['second'])) {
    echo 'all present';
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if (isset($d['first']['second'])) {
    echo 'all present';
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-label">
<summary><code class="rule__code">no-redundant-label</code><a class="rule__anchor" href="#no-redundant-label" aria-label="Lien permanent vers no-redundant-label">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant `goto` labels that are declared but not used.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

label:
echo "Hello, world!";
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

goto end;
echo "Hello, world!";
end:
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-math">
<summary><code class="rule__code">no-redundant-math</code><a class="rule__anchor" href="#no-redundant-math" aria-label="Lien permanent vers no-redundant-math">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant mathematical operations that can be simplified or removed.
Includes operations like multiplying by 1/-1, adding 0, modulo 1/-1, etc.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$result = $value * 1;
$sum = 0 + $total;
$difference = $value - 0;
$remainder = $x % 1;
$negative = $value * -1;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$result = $value * 2;
$sum = 1 + $total;
$difference = $value - 1;
$remainder = $x % 2;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-method-override">
<summary><code class="rule__code">no-redundant-method-override</code><a class="rule__anchor" href="#no-redundant-method-override" aria-label="Lien permanent vers no-redundant-method-override">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects methods that override a parent method but only call the parent method with the same arguments.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

class Parent
{
    public function foo(): void
    {
        // ...
    }
}

class Child extends Parent
{
    public function foo(): void
    {
        parent::foo();
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

class Parent
{
    public function foo(): void
    {
        // ...
    }
}

class Child extends Parent
{
    public function foo(): void
    {
        parent::foo();

        echo 'Additional logic here';
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-nullsafe">
<summary><code class="rule__code">no-redundant-nullsafe</code><a class="rule__anchor" href="#no-redundant-nullsafe" aria-label="Lien permanent vers no-redundant-nullsafe">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Flags the use of the nullsafe operator (`?->`) in contexts where its null-checking behavior is redundant.

This occurs in two common situations:
1. When an expression using `?->` is immediately followed by the null coalescing operator (`??`).
2. When an expression using `?->` is checked with `isset()`.

In both scenarios, the surrounding language construct (`??` or `isset()`) already handles `null` values safely,
making the `?->` operator superfluous and the code unnecessarily verbose.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$name = $user?->name ?? 'Guest';

if (isset($user?->profile)) {
    // Do something with $user->profile
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$name = $user->name ?? 'Guest';

if (isset($user->profile)) {
    // Do something with $user->profile
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-parentheses">
<summary><code class="rule__code">no-redundant-parentheses</code><a class="rule__anchor" href="#no-redundant-parentheses" aria-label="Lien permanent vers no-redundant-parentheses">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant parentheses around expressions.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$foo = (42);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$foo = 42;
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-readonly">
<summary><code class="rule__code">no-redundant-readonly</code><a class="rule__anchor" href="#no-redundant-readonly" aria-label="Lien permanent vers no-redundant-readonly">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant readonly modifiers on properties.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

readonly class User
{
    public readonly $name;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

readonly class User
{
    public $name;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-string-concat">
<summary><code class="rule__code">no-redundant-string-concat</code><a class="rule__anchor" href="#no-redundant-string-concat" aria-label="Lien permanent vers no-redundant-string-concat">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant string concatenation expressions.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$foo = "Hello" . " World";
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$foo = "Hello World";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-write-visibility">
<summary><code class="rule__code">no-redundant-write-visibility</code><a class="rule__anchor" href="#no-redundant-write-visibility" aria-label="Lien permanent vers no-redundant-write-visibility">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant write visibility modifiers on properties.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

final class User
{
    public public(set) $name;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

final class User
{
    public $name;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-yield-from">
<summary><code class="rule__code">no-redundant-yield-from</code><a class="rule__anchor" href="#no-redundant-yield-from" aria-label="Lien permanent vers no-redundant-yield-from">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Detects redundant use of `yield from` with single-element array literals.

Using `yield from` with a single-element array literal creates unnecessary
overhead in the generated opcodes. Direct `yield` is simpler and more efficient.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function gen(): Generator {
    yield from [1];
    yield from ['foo' => new stdClass()];
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function gen(): Generator {
    yield 1;
    yield 'foo' => new stdClass();
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

</div>

<h2 id="security">Sécurité</h2>

Règles qui signalent des vulnérabilités, vecteurs d'injection, désérialisation non sûre, données non fiables atteignant des points dangereux.

<div class="rule-list">

<details class="rule" name="rule" id="no-db-schema-change">
<summary><code class="rule__code">no-db-schema-change</code><a class="rule__anchor" href="#no-db-schema-change" aria-label="Lien permanent vers no-db-schema-change">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

This rule flags any attempt to alter the database schema (using `CREATE`, `ALTER`, or `DROP`)
within a `$wpdb` call. Schema modifications must only occur within a plugin activation hook
to prevent catastrophic performance issues and data corruption.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-wordpress"><code>WordPress</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// This schema change runs on every page load, which is very dangerous.
global $wpdb;
$wpdb->query("ALTER TABLE {$wpdb->posts} ADD my_column VARCHAR(255)");
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function my_plugin_activate() {
    global $wpdb;

    // Running schema changes inside an activation hook is safe.
    $wpdb->query("ALTER TABLE {$wpdb->posts} ADD my_column VARCHAR(255)");
}

register_activation_hook(__FILE__, 'my_plugin_activate');
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-insecure-comparison">
<summary><code class="rule__code">no-insecure-comparison</code><a class="rule__anchor" href="#no-insecure-comparison" aria-label="Lien permanent vers no-insecure-comparison">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects insecure comparison of passwords or tokens using `==`, `!=`, `===`, or `!==`.

These operators are vulnerable to timing attacks, which can expose sensitive information.
Instead, use `hash_equals` for comparing strings or `password_verify` for validating hashes.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if ($storedToken == $userToken) {
    // Vulnerable to timing attacks
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if (hash_equals($storedToken, $userToken)) {
    // Valid token
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-literal-password">
<summary><code class="rule__code">no-literal-password</code><a class="rule__anchor" href="#no-literal-password" aria-label="Lien permanent vers no-literal-password">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects the use of literal values for passwords or sensitive data.
Storing passwords or sensitive information as literals in code is a security risk
and should be avoided. Use environment variables or secure configuration management instead.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$password = "supersecret";
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$password = getenv('DB_PASSWORD');
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-unescaped-output">
<summary><code class="rule__code">no-unescaped-output</code><a class="rule__anchor" href="#no-unescaped-output" aria-label="Lien permanent vers no-unescaped-output">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

This rule ensures that any variable or function call that is output directly to the page is
properly escaped. All data must be escaped before printing to prevent Cross-Site Scripting (XSS)
vulnerabilities.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-wordpress"><code>WordPress</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// This is a major XSS vulnerability.
echo $_GET['user_comment'];
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

echo esc_html( $user_comment );
?>
<a href="<?php echo esc_url( $user_provided_url ); ?>">Link</a>
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="sensitive-parameter">
<summary><code class="rule__code">sensitive-parameter</code><a class="rule__anchor" href="#sensitive-parameter" aria-label="Lien permanent vers sensitive-parameter">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Requires that parameters that are likely to contain sensitive information (e.g., passwords)
are marked with the `#[SensitiveParameter]` attribute to prevent accidental logging or exposure.

This rule only applies to PHP 8.2 and later, as the `SensitiveParameter` attribute was introduced in PHP 8.2.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>8.2.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function login(string $username, string $password): void {
   // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function login(string $username, #[SensitiveParameter] string $password): void {
   // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="tainted-data-to-sink">
<summary><code class="rule__code">tainted-data-to-sink</code><a class="rule__anchor" href="#tainted-data-to-sink" aria-label="Lien permanent vers tainted-data-to-sink">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects user (tainted) data being passed directly to sink functions or constructs
(such as `echo`, `print`, or user-defined "log" functions). If these functions emit
or store data without sanitization, it could lead to Cross-Site Scripting (XSS)
or other injection attacks.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// This is considered unsafe:
echo $_GET['name'] ?? '';
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// Properly escape data before using a sink like `echo`
echo htmlspecialchars($_GET['name'] ?? '', ENT_QUOTES, 'UTF-8');
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `known-sink-functions` | `array` | `["printf"]` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="disallowed-functions">
<summary><code class="rule__code">disallowed-functions</code><a class="rule__anchor" href="#disallowed-functions" aria-label="Lien permanent vers disallowed-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Flags calls to functions that are disallowed via rule configuration.

You can specify which functions or extensions should be disallowed through the
`functions` or `extensions` options. This helps enforce coding standards,
security restrictions, or the usage of preferred alternatives.

Each entry can be a simple string or an object with `name` and optional `help`:

```toml
functions = [
    'eval',
    { name = 'error_log', help = 'Use MyLogger instead.' },
]
```

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

curl_init(); // Error: part of a disallowed extension
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function allowed_function(): void {
    // ...
}

allowed_function(); // Not flagged
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `extensions` | `array` | `[]` |
| `functions` | `array` | `[]` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="disallowed-type-instantiation">
<summary><code class="rule__code">disallowed-type-instantiation</code><a class="rule__anchor" href="#disallowed-type-instantiation" aria-label="Lien permanent vers disallowed-type-instantiation">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Flags direct instantiation of specific types that are disallowed via rule configuration.

This rule helps enforce architectural patterns such as factory methods or provider patterns
by preventing direct instantiation of specific classes. This is useful for ensuring consistent
configuration, centralizing object creation, and maintaining architectural boundaries.

Each entry can be a simple string or an object with `name` and optional `help`:

```toml
[linter.rules]
disallowed-type-instantiation = {
    enabled = true,
    types = [
        'HttpService\\Client',
        { name = 'DatabaseConnection', help = 'Use DatabaseFactory::create() instead' },
    ]
}
```

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// Direct instantiation of disallowed type
$client = new HttpService\Client();

// Another disallowed instantiation
$db = new DatabaseConnection('localhost', 'user', 'pass');
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// Using factory pattern instead of direct instantiation
$client = ClientProvider::getClient();
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `types` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="no-roles-as-capabilities">
<summary><code class="rule__code">no-roles-as-capabilities</code><a class="rule__anchor" href="#no-roles-as-capabilities" aria-label="Lien permanent vers no-roles-as-capabilities">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule flags the use of user roles (e.g., `'administrator'`) in functions that expect a
granular capability (e.g., `'edit_posts'`). Checking against specific capabilities is a
core security principle in WordPress.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-wordpress"><code>WordPress</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// This check is brittle and will fail if roles are customized.
if ( current_user_can( 'editor' ) ) { /* ... */ }
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if ( current_user_can( 'edit_posts' ) ) { /* ... */ }
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-short-opening-tag">
<summary><code class="rule__code">no-short-opening-tag</code><a class="rule__anchor" href="#no-short-opening-tag" aria-label="Lien permanent vers no-short-opening-tag">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows the use of short opening tags (`<?`).

The availability of `<?` depends on the `short_open_tag` directive in `php.ini`. If
this setting is disabled on a server, any code within the short tags will be
exposed as plain text, which is a significant security risk. Using the full `<?php`
opening tag is the only guaranteed portable way to ensure your code is always
interpreted correctly.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?

echo "Hello, World!";
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

echo "Hello, World!";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="require-preg-quote-delimiter">
<summary><code class="rule__code">require-preg-quote-delimiter</code><a class="rule__anchor" href="#require-preg-quote-delimiter" aria-label="Lien permanent vers require-preg-quote-delimiter">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

This rule requires that when using `preg_quote()`, the second `$delimiter` argument is always provided.
If the string being quoted contains the same character used for your regex delimiter (e.g., `/`),
failing to provide the second argument will prevent that character from being escaped,
which can break the regular expression.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// If $user_input contains '/', the regex will be invalid.
$pattern = '/' . preg_quote( $user_input ) . '/';
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// The delimiter is provided, ensuring it gets escaped if necessary.
$pattern = '/' . preg_quote( $user_input, '/' ) . '/';
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-debug-symbols">
<summary><code class="rule__code">no-debug-symbols</code><a class="rule__anchor" href="#no-debug-symbols" aria-label="Lien permanent vers no-debug-symbols">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Flags calls to debug functions like `var_dump`, `print_r`, `dd`, etc.

These functions are useful for debugging, but they should not be committed to
version control as they can expose sensitive information and are generally not
intended for production environments.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function process_request(array $data) {
    var_dump($data); // Debug call that should be removed
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// Production-safe code
error_log('Processing user request.');
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

</div>

<h2 id="safety">Sûreté</h2>

Constructions qui compilent mais explosent à l'exécution. Ces règles attrapent les pièges avant les utilisateurs.

<div class="rule-list">

<details class="rule" name="rule" id="no-error-control-operator">
<summary><code class="rule__code">no-error-control-operator</code><a class="rule__anchor" href="#no-error-control-operator" aria-label="Lien permanent vers no-error-control-operator">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects the use of the error control operator `@`.

The error control operator suppresses errors and makes debugging more difficult.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$result = @file_get_contents('example.txt');
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

try {
    $result = file_get_contents('example.txt');
} catch (Throwable $e) {
    // Handle error
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-eval">
<summary><code class="rule__code">no-eval</code><a class="rule__anchor" href="#no-eval" aria-label="Lien permanent vers no-eval">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects unsafe uses of the `eval` construct.
The `eval` construct executes arbitrary code, which can be a major security risk if not used carefully.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

eval('echo "Hello, world!";');
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// Safe alternative to eval
$result = json_decode($jsonString);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-ffi">
<summary><code class="rule__code">no-ffi</code><a class="rule__anchor" href="#no-ffi" aria-label="Lien permanent vers no-ffi">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects unsafe use of the PHP FFI (Foreign Function Interface) extension.

The FFI extension allows interaction with code written in other languages, such as C, C++, and Rust.
This can introduce potential security risks and stability issues if not handled carefully.

If you are confident in your use of FFI and understand the risks, you can disable this rule in your Mago configuration.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

use FFI;

$ffi = FFI::cdef("void* malloc(size_t size);");
$ffi->malloc(1024); // Allocate memory but never free it
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// Using a safe alternative to FFI
$data = 'some data';
$hash = hash('sha256', $data);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-global">
<summary><code class="rule__code">no-global</code><a class="rule__anchor" href="#no-global" aria-label="Lien permanent vers no-global">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects the use of the `global` keyword and the `$GLOBALS` variable.

The `global` keyword introduces global state into your function, making it harder to reason about and test.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function foo(): void {
    global $bar;
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function foo(string $bar): void {
    // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-request-variable">
<summary><code class="rule__code">no-request-variable</code><a class="rule__anchor" href="#no-request-variable" aria-label="Lien permanent vers no-request-variable">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects the use of the `$_REQUEST` variable, which is considered unsafe.

Use `$_GET`, `$_POST`, or `$_COOKIE` instead for better clarity.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$identifier = $_REQUEST['id'];
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$identifier = $_GET['id'];
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-shell-execute-string">
<summary><code class="rule__code">no-shell-execute-string</code><a class="rule__anchor" href="#no-shell-execute-string" aria-label="Lien permanent vers no-shell-execute-string">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects the use of shell execute strings (`...`) in PHP code.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$output = `ls -l`;
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$output = shell_exec('ls -l');
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-unsafe-finally">
<summary><code class="rule__code">no-unsafe-finally</code><a class="rule__anchor" href="#no-unsafe-finally" aria-label="Lien permanent vers no-unsafe-finally">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects control flow statements in `finally` blocks.

Control flow statements in `finally` blocks override control flows from `try` and `catch` blocks,
leading to unexpected behavior.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function example(): int {
    try {
        return get_value();
    } finally {
        return 42; // Unsafe control flow statement in finally block
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function example(): int {
    try {
        return get_value();
    } finally {
        // no control flow statements
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="no-request-all">
<summary><code class="rule__code">no-request-all</code><a class="rule__anchor" href="#no-request-all" aria-label="Lien permanent vers no-request-all">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects the use of `$request->all()` or `Request::all()` in Laravel applications.

Such calls retrieve all input values, including ones you might not expect or intend to handle.
It is recommended to use `$request->only([...])` to specify the inputs you need explicitly, ensuring better security and validation.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-laravel"><code>Laravel</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App\Http\Controllers;

use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;

class UserController extends Controller
{
    /**
     * Store a new user.
     */
    public function store(Request $request): RedirectResponse
    {
        $data = $request->all();

        // ...
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App\Http\Controllers;

use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;

class UserController extends Controller
{
    /**
     * Store a new user.
     */
    public function store(Request $request): RedirectResponse
    {
        $data = $request->only(['name', 'email', 'password']);

        // ...
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-service-state-mutation">
<summary><code class="rule__code">no-service-state-mutation</code><a class="rule__anchor" href="#no-service-state-mutation" aria-label="Lien permanent vers no-service-state-mutation">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects mutations to `$this->property` inside service methods.

In worker-mode PHP runtimes (FrankenPHP, RoadRunner, Swoole), services persist across
requests. Mutating `$this->property` in a service method introduces shared mutable state
that leaks between requests, leading to subtle and hard-to-reproduce bugs.

Mutations include direct assignment (`$this->count = 0`), compound assignment
(`$this->count += 1`), increment/decrement (`$this->count++`, `++$this->count`),
array append (`$this->items[] = $item`), and `unset($this->cache)`.

The `__construct` and `reset` methods are allowed by default.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-symfony"><code>Symfony</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

namespace App\Service;

final class InvoiceService
{
    private int $processedCount = 0;

    public function process(Invoice $invoice): void
    {
        $this->processedCount++;
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

namespace App\Service;

final class InvoiceService
{
    public function __construct(
        private readonly InvoiceRepository $repository,
    ) {}

    public function process(Invoice $invoice): void
    {
        $total = $invoice->getTotal();
        $this->repository->save($invoice);
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `allowed-methods` | `array` | `["__construct","reset"]` |
| `enabled` | `boolean` | `false` |
| `exclude-namespaces` | `array` | `["App\\Entity\\","App\\DTO\\","App\\ValueObject\\"]` |
| `include-namespaces` | `array` | `["App\\"]` |
| `level` | `string` | `"warning"` |
| `reset-interfaces` | `array` | `["Symfony\\Contracts\\Service\\ResetInterface"]` |

</div>

</details>

</div>

<h2 id="correctness">Correction</h2>

Bugs et erreurs de logique. Les règles de cette catégorie attrapent du code qui fait probablement autre chose que ce que l'auteur voulait.

<div class="rule-list">

<details class="rule" name="rule" id="no-only">
<summary><code class="rule__code">no-only</code><a class="rule__anchor" href="#no-only" aria-label="Lien permanent vers no-only">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Detects usage of `->only()` in Pest tests which should not be committed.

The `->only()` modifier causes only that specific test to run, which can lead to
incomplete test coverage if accidentally committed to the repository.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-pest"><code>Pest</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

test('example test', function () {
    expect(true)->toBeTrue();
})->only();

it('does something', function () {
    expect(1)->toBe(1);
})->only();
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

test('example test', function () {
    expect(true)->toBeTrue();
});

it('does something', function () {
    expect(1)->toBe(1);
});
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="assert-description">
<summary><code class="rule__code">assert-description</code><a class="rule__anchor" href="#assert-description" aria-label="Lien permanent vers assert-description">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects assert functions that do not have a description.

Assert functions should have a description to make it easier to understand the purpose of the assertion.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

assert($user->isActivated());
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

assert($user->isActivated(), 'User MUST be activated at this point.');
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="identity-comparison">
<summary><code class="rule__code">identity-comparison</code><a class="rule__anchor" href="#identity-comparison" aria-label="Lien permanent vers identity-comparison">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects equality and inequality comparisons that should use identity comparison operators.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if ($a == $b) {
    echo '$a is same as $b';
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

if ($a === $b) {
    echo '$a is same as $b';
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="ineffective-format-ignore-next">
<summary><code class="rule__code">ineffective-format-ignore-next</code><a class="rule__anchor" href="#ineffective-format-ignore-next" aria-label="Lien permanent vers ineffective-format-ignore-next">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects `@mago-format-ignore-next` markers that will have no effect.

The formatter's ignore-next marker works at the statement level. When a
marker is placed inside an expression (like function call arguments,
array elements, or other non-statement contexts), it will not affect
the formatter's output.

To effectively ignore the next statement, place the marker immediately
before a complete statement at the top level of a block or file.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// This doesn't work - marker is inside an array literal
$arr = [ // @mago-format-ignore-next
    1,
    2,
];
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// This works - marker is before a statement
// @mago-format-ignore-next
const GRID = [
  [1, 2, 3], [1, 2, ], [0,    0],
];

foo();
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="ineffective-format-ignore-region">
<summary><code class="rule__code">ineffective-format-ignore-region</code><a class="rule__anchor" href="#ineffective-format-ignore-region" aria-label="Lien permanent vers ineffective-format-ignore-region">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects `@mago-format-ignore-start` markers that will have no effect.

The formatter's ignore regions work at the statement level. When an
ignore marker is placed inside an expression (like function call arguments,
array elements, or other non-statement contexts), it will not affect
the formatter's output.

To effectively ignore a region, place the ignore markers between complete
statements at the top level of a block or file.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

// This doesn't work - markers are inside a function call
foo( // @mago-format-ignore-start
    $x,
    $y
// @mago-format-ignore-end
);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

// This works - markers are between statements
// @mago-format-ignore-start
$x = 1;  $y = 2;  // preserved as-is
// @mago-format-ignore-end

foo();
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-assign-in-argument">
<summary><code class="rule__code">no-assign-in-argument</code><a class="rule__anchor" href="#no-assign-in-argument" aria-label="Lien permanent vers no-assign-in-argument">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects assignments in function call arguments which can lead to unexpected behavior and make
the code harder to read and understand.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

foo($x = 5);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$x = 5;
foo($x);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-assign-in-condition">
<summary><code class="rule__code">no-assign-in-condition</code><a class="rule__anchor" href="#no-assign-in-condition" aria-label="Lien permanent vers no-assign-in-condition">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects assignments in conditions which can lead to unexpected behavior and make the code harder
to read and understand.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

if ($x = 1) {
    // ...
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$x = 1;
if ($x == 1) {
    // ...
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-dead-store">
<summary><code class="rule__code">no-dead-store</code><a class="rule__anchor" href="#no-dead-store" aria-label="Lien permanent vers no-dead-store">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Flags assignments to a variable whose value is overwritten by a later
assignment without being read in between. The earlier assignment is dead;
its value never reaches anything observable.

Detection is limited to linear (non-branching) flow. Writes inside conditional
branches (if/else, loops, match arms, try paths, switch cases) don't pair up
with writes in sibling branches, so this rule produces no false positives for
code like `if ($cond) { $x = 1; } else { $x = 2; } return $x;`.

Variables whose name starts with an underscore (`$_`, `$_foo`) are ignored.
Variables declared via `global` or `static` are also ignored.

The rule analyses one function-like scope at a time. It bails out of any scope
that uses variable variables (`$$x`, `${expr}`) or calls `extract()`.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function f() {
    $x = 1; // dead - overwritten before being read
    $x = compute();
    return $x;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function f() {
    $x = compute();
    return $x;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-empty-catch-clause">
<summary><code class="rule__code">no-empty-catch-clause</code><a class="rule__anchor" href="#no-empty-catch-clause" aria-label="Lien permanent vers no-empty-catch-clause">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Warns when a `catch` clause is empty.

An empty `catch` clause suppresses exceptions without handling or logging them,
potentially hiding errors that should be addressed. This practice, known as
"exception swallowing," can make debugging significantly more difficult.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

try {
    // some code
} catch(Exception $e) {
    // This block is empty and swallows the exception.
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

try {
    // some code that might throw an exception
} catch(Exception $e) {
    // Handle the error, log it, or re-throw it.
    error_log($e->getMessage());
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-redundant-variable">
<summary><code class="rule__code">no-redundant-variable</code><a class="rule__anchor" href="#no-redundant-variable" aria-label="Lien permanent vers no-redundant-variable">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Flags variables that are written or declared but whose value is never read.

Detects fully-unused variables (assigned and never referenced) as well as
variables whose only mention is on the write side, for example, an
undefined name passed to a function as a potential by-reference output where
the result is never observed by the caller.

Variables whose name starts with an underscore (`$_`, `$_foo`) are treated as
intentionally-discarded and are ignored. Variables declared via `global` or
`static` are also ignored, since they are bindings to external scope.

The rule analyses one function-like scope at a time. It bails out of any scope
that uses variable variables (`$$x`, `${expr}`) or calls `extract()`, since
those introduce names the linter cannot resolve.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function greet(string $name): string
{
    $unused = compute_something();

    return "Hello, $name!";
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function greet(string $name): string
{
    $greeting = "Hello, $name!";

    return $greeting;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-unused-closure-capture">
<summary><code class="rule__code">no-unused-closure-capture</code><a class="rule__anchor" href="#no-unused-closure-capture" aria-label="Lien permanent vers no-unused-closure-capture">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Flags variables in a closure's `use (...)` clause that are never read
or written inside the closure body.

Captures only earn their keep when the body refers to them. A capture
that nothing observes is usually a leftover from a refactor or a typo
in the captured name.

Names beginning with an underscore (`$_`, `$_foo`) are treated as
intentionally-discarded and are ignored. By-reference captures
(`use (&$x)`) are also ignored, they are commonly used for their
side-effect on the outer scope, even when the inner body doesn't
otherwise touch the binding. The rule bails out of any closure body
that uses variable variables (`$$x`, `${expr}`) or calls `extract()`.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

$base = 10;
$add = function (int $x) use ($base): int {
    return $x;
};
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

$base = 10;
$add = function (int $x) use ($base): int {
    return $x + $base;
};
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-unused-global">
<summary><code class="rule__code">no-unused-global</code><a class="rule__anchor" href="#no-unused-global" aria-label="Lien permanent vers no-unused-global">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Flags `global $x;` declarations whose name is never read or written
inside the surrounding function-like scope.

A `global` statement only earns its keep when later code refers to the
imported binding. If the name is never used, the statement is dead ,
usually a leftover from a refactor or a typo in the imported name.

Names beginning with an underscore (`$_`, `$_foo`) are treated as
intentionally-discarded and are ignored. The rule bails out of any
scope that uses variable variables (`$$x`, `${expr}`) or calls
`extract()`, since those introduce names the linter cannot resolve.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function f(): void {
    global $forgotten;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function bump(): void {
    global $counter;
    $counter++;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="no-unused-static">
<summary><code class="rule__code">no-unused-static</code><a class="rule__anchor" href="#no-unused-static" aria-label="Lien permanent vers no-unused-static">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Flags `static $x;` declarations whose name is never read or written
inside the surrounding function-like scope.

A `static` declaration only earns its keep when later code refers to
the binding. If the name is never used after the declaration, the
statement is dead, usually a leftover from a refactor.

Names beginning with an underscore (`$_`, `$_foo`) are treated as
intentionally-discarded and are ignored. The rule bails out of any
scope that uses variable variables (`$$x`, `${expr}`) or calls
`extract()`, since those introduce names the linter cannot resolve.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

function f(): void {
    static $forgotten = 0;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

function next_id(): int {
    static $counter = 0;
    return ++$counter;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="strict-assertions">
<summary><code class="rule__code">strict-assertions</code><a class="rule__anchor" href="#strict-assertions" aria-label="Lien permanent vers strict-assertions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects non-strict assertions in test methods.
Assertions should use strict comparison methods, such as `assertSame` or `assertNotSame`
instead of `assertEquals` or `assertNotEquals`.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-phpunit"><code>PHPUnit</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertEquals(42, 42);
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertSame(42, 42);
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="strict-behavior">
<summary><code class="rule__code">strict-behavior</code><a class="rule__anchor" href="#strict-behavior" aria-label="Lien permanent vers strict-behavior">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects functions relying on loose comparison unless the `$strict` parameter is specified.
The use of loose comparison for these functions may lead to hard-to-debug, unexpected behaviors.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>7.0.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

in_array(1, ['foo', 'bar', 'baz']);
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

in_array(1, ['foo', 'bar', 'baz'], strict: true);
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `allow-loose-behavior` | `boolean` | `false` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="strict-types">
<summary><code class="rule__code">strict-types</code><a class="rule__anchor" href="#strict-types" aria-label="Lien permanent vers strict-types">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects missing `declare(strict_types=1);` statement at the beginning of the file.

<blockquote class="rule-requirement">Cette règle requiert PHP <code>7.0.0</code> ou supérieur.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

echo "Hello, World!";
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

declare(strict_types=1);

echo "Hello, World!";
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `allow-disabling` | `boolean` | `false` |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="switch-continue-to-break">
<summary><code class="rule__code">switch-continue-to-break</code><a class="rule__anchor" href="#switch-continue-to-break" aria-label="Lien permanent vers switch-continue-to-break">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Detects the use of `continue` inside a `switch` statement, which should
be `break` instead.

In PHP, `continue` inside a `switch` behaves the same as `break`, but
using `continue` is misleading because it suggests continuing a loop.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

switch ($value) {
    case 1:
        echo 'one';
        continue;
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

switch ($value) {
    case 1:
        echo 'one';
        break;
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="use-specific-assertions">
<summary><code class="rule__code">use-specific-assertions</code><a class="rule__anchor" href="#use-specific-assertions" aria-label="Lien permanent vers use-specific-assertions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Suggests using specific PHPUnit assertions instead of generic equality assertions
when comparing with `null`, `true`, or `false`.

Using specific assertions like `assertNull`, `assertTrue`, and `assertFalse`
provides clearer error messages and makes test intent more explicit.

<blockquote class="rule-requirement">Cette règle requiert que l'intégration <a href="#integration-phpunit"><code>PHPUnit</code></a> soit activée.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertEquals(null, $value);
        $this->assertSame(true, $flag);
        $this->assertEquals(false, $condition);
    }
}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertNull($value);
        $this->assertTrue($flag);
        $this->assertFalse($condition);
    }
}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="invalid-open-tag">
<summary><code class="rule__code">invalid-open-tag</code><a class="rule__anchor" href="#invalid-open-tag" aria-label="Lien permanent vers invalid-open-tag">¶</a><span class="rule__level rule__level--note">note</span></summary>

<div class="rule__body">

Detects misspelled PHP opening tags like `<php?` instead of `<?php`.

A misspelled opening tag will cause the PHP interpreter to treat the
following code as plain text, leading to the code being output directly
to the browser instead of being executed. This can cause unexpected
behavior and potential security vulnerabilities.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```php
<php?

echo 'Hello, world!';
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```php
<?php

echo 'Hello, world!';
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

</div>

</details>

</div>

