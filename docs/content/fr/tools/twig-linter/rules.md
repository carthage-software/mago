+++
title = "Règles"
description = "La référence complète de chaque règle du linter Twig, triée par gravité. Cliquez sur une règle pour afficher sa description, ses exemples et sa configuration."
nav_order = 45
nav_section = "Tools"
nav_subsection = "Twig Linter"
+++
Le linter Twig de Mago propose 25 règles réparties sur 8 catégories. Cliquez sur une règle pour afficher sa description, ses prérequis, sa configuration par défaut et ses exemples. Les codes des règles commencent par <code>twig-</code> ; les clés de configuration sous <code>[twig.linter.rules]</code> omettent ce préfixe.

<div class="rule-index" role="navigation" aria-label="Catégories de règles"><a class="rule-index__item" href="#clarity"><span class="rule-index__name">Clarté</span><span class="rule-index__count">1 règle</span><span class="rule-index__blurb">Règles qui rendent l'intention explicite et réduisent l'effort de lecture. Elles signalent des constructions techniquement valides mais qui obscurcissent ce que fait le code.</span></a><a class="rule-index__item" href="#bestpractices"><span class="rule-index__name">Bonnes pratiques</span><span class="rule-index__count">1 règle</span><span class="rule-index__blurb">Patterns Twig idiomatiques que recommandent la documentation et les conventions de codage de Twig.</span></a><a class="rule-index__item" href="#consistency"><span class="rule-index__name">Cohérence</span><span class="rule-index__count">7 règles</span><span class="rule-index__blurb">Uniformité stylistique sur l'ensemble du code. Choisissez une façon de faire, ces règles aident tout le monde à s'y tenir.</span></a><a class="rule-index__item" href="#deprecation"><span class="rule-index__name">Obsolescence</span><span class="rule-index__count">3 règles</span><span class="rule-index__blurb">Fonctionnalités Twig obsolètes en amont, qu'une version ultérieure de Twig supprime. Migrez avant qu'elles ne cassent.</span></a><a class="rule-index__item" href="#maintainability"><span class="rule-index__name">Maintenabilité</span><span class="rule-index__count">2 règles</span><span class="rule-index__blurb">Du code difficile à maintenir dans la durée, trop complexe, trop emmêlé, trop fragile. Ces règles font remonter le coût tôt.</span></a><a class="rule-index__item" href="#redundancy"><span class="rule-index__name">Redondance</span><span class="rule-index__count">1 règle</span><span class="rule-index__blurb">Code mort, valeurs inutilisées, constructions sans effet observable. Les retirer garde le code honnête.</span></a><a class="rule-index__item" href="#security"><span class="rule-index__name">Sécurité</span><span class="rule-index__count">6 règles</span><span class="rule-index__blurb">Sorties qui contournent l'échappement, sorties de débogage qui exposent le contexte du template, et fonctions, filtres ou tags interdits par votre projet.</span></a><a class="rule-index__item" href="#correctness"><span class="rule-index__name">Correction</span><span class="rule-index__count">4 règles</span><span class="rule-index__blurb">Bugs et erreurs de logique. Les règles de cette catégorie attrapent du code qui fait probablement autre chose que ce que l'auteur voulait.</span></a></div>

<h2 id="clarity">Clarté</h2>

Règles qui rendent l'intention explicite et réduisent l'effort de lecture. Elles signalent des constructions techniquement valides mais qui obscurcissent ce que fait le code.

<div class="rule-list">

<details class="rule" name="rule" id="twig-no-nested-ternary">
<summary><code class="rule__code">twig-no-nested-ternary</code><a class="rule__anchor" href="#twig-no-nested-ternary" aria-label="Lien permanent vers twig-no-nested-ternary">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports conditional expressions nested inside other conditional expressions. Split them into
named values or branches.

Parentheses do not hide a nested conditional. A conditional in an arrow function body is not
nested in the conditional around the arrow function.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ flag ? (other ? one : two) : no }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ flag ? yes : no }}
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

</div>

<h2 id="bestpractices">Bonnes pratiques</h2>

Patterns Twig idiomatiques que recommandent la documentation et les conventions de codage de Twig.

<div class="rule-list">

<details class="rule" name="rule" id="twig-prefer-include-function">
<summary><code class="rule__code">twig-prefer-include-function</code><a class="rule__anchor" href="#twig-prefer-include-function" aria-label="Lien permanent vers twig-prefer-include-function">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Prefers the `include()` function to the `include` tag.

No fix is offered: a rewrite must account for escaping, sandboxing, scope, and call order.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% include 'card.twig' %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ include('card.twig') }}
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

</div>

<h2 id="consistency">Cohérence</h2>

Uniformité stylistique sur l'ensemble du code. Choisissez une façon de faire, ces règles aident tout le monde à s'y tenir.

<div class="rule-list">

<details class="rule" name="rule" id="twig-macro-argument-name">
<summary><code class="rule__code">twig-macro-argument-name</code><a class="rule__anchor" href="#twig-macro-argument-name" aria-label="Lien permanent vers twig-macro-argument-name">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for macro parameter names.

Non-ASCII names are left alone. No fix is offered, because renaming a parameter breaks calls that
pass it by name.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% macro input(className) %}{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% macro input(class_name) %}{% endmacro %}
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

<details class="rule" name="rule" id="twig-macro-name">
<summary><code class="rule__code">twig-macro-name</code><a class="rule__anchor" href="#twig-macro-name" aria-label="Lien permanent vers twig-macro-name">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for macro names.

Non-ASCII names are left alone. No fix is offered, because renaming a macro breaks its callers.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% macro htmlInput() %}{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% macro html_input() %}{% endmacro %}
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

<details class="rule" name="rule" id="twig-named-argument-name">
<summary><code class="rule__code">twig-named-argument-name</code><a class="rule__anchor" href="#twig-named-argument-name" aria-label="Lien permanent vers twig-named-argument-name">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for named arguments.

Enable this only when the functions, filters, and macros you call follow the Twig coding standard.
Non-ASCII names are left alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ input(className: 'input') }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ input(class_name: 'input') }}
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

<details class="rule" name="rule" id="twig-variable-name">
<summary><code class="rule__code">twig-variable-name</code><a class="rule__anchor" href="#twig-variable-name" aria-label="Lien permanent vers twig-variable-name">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for the names that `set`, `for`, `import`, `from`, arrow functions,
assignments, and destructuring bind.

Variables from the application, mapping keys, attribute names, and non-ASCII names are left alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% set firstName = user.name %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% set first_name = user.name %}
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

<details class="rule" name="rule" id="twig-end-block-name">
<summary><code class="rule__code">twig-end-block-name</code><a class="rule__anchor" href="#twig-end-block-name" aria-label="Lien permanent vers twig-end-block-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Requires the block or macro name on `endblock` and `endmacro` tags.

Short blocks need no closing tag. The fix adds the name and leaves whitespace control intact.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% block title %}Title{% endblock %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% block title %}Title{% endblock title %}
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

<details class="rule" name="rule" id="twig-named-argument-separator">
<summary><code class="rule__code">twig-named-argument-separator</code><a class="rule__anchor" href="#twig-named-argument-separator" aria-label="Lien permanent vers twig-named-argument-separator">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Requires `:` between a named argument and its value, as the Twig coding standard recommends,
instead of `=`.

Macro parameter defaults keep `=`.

<blockquote class="rule-requirement">Cette règle requiert une version minimale de Twig, <code>[twig.linter] version</code>, égale ou supérieure à <code>3.12</code>.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ range(low = 1, high = 3) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ range(low: 1, high: 3) }}
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

<details class="rule" name="rule" id="twig-single-quotes">
<summary><code class="rule__code">twig-single-quotes</code><a class="rule__anchor" href="#twig-single-quotes" aria-label="Lien permanent vers twig-single-quotes">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Prefers single quotes for plain strings: strings without escapes, apostrophes, `#`, or line
breaks.

Other strings stay unchanged, because new quotes would change their meaning.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ "title" }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ 'title' }}
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

<h2 id="deprecation">Obsolescence</h2>

Fonctionnalités Twig obsolètes en amont, qu'une version ultérieure de Twig supprime. Migrez avant qu'elles ne cassent.

<div class="rule-list">

<details class="rule" name="rule" id="twig-deprecated-attribute">
<summary><code class="rule__code">twig-deprecated-attribute</code><a class="rule__anchor" href="#twig-deprecated-attribute" aria-label="Lien permanent vers twig-deprecated-attribute">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports calls to the built-in `attribute()` function, deprecated since Twig 3.15.

Use dynamic attribute access, `object.(name)`, after checking its call and lookup behavior. A
macro imported as `attribute` is left alone.

<blockquote class="rule-requirement">Cette règle requiert une version minimale de Twig, <code>[twig.linter] version</code>, égale ou supérieure à <code>3.15</code>.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ attribute(object, name) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ object.(name) }}
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

<details class="rule" name="rule" id="twig-deprecated-spaceless">
<summary><code class="rule__code">twig-deprecated-spaceless</code><a class="rule__anchor" href="#twig-deprecated-spaceless" aria-label="Lien permanent vers twig-deprecated-spaceless">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports the `spaceless` filter, deprecated since Twig 3.12, including in `apply` tags.

No fix is offered: check the rendered output before you stop stripping whitespace.

<blockquote class="rule-requirement">Cette règle requiert une version minimale de Twig, <code>[twig.linter] version</code>, égale ou supérieure à <code>3.12</code>.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ text|spaceless }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ text }}
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

<details class="rule" name="rule" id="twig-duplicate-macro">
<summary><code class="rule__code">twig-duplicate-macro</code><a class="rule__anchor" href="#twig-duplicate-macro" aria-label="Lien permanent vers twig-duplicate-macro">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports macro names repeated in one template. Names are case-sensitive.

Twig 3 uses the last definition; Twig 3.29 deprecates this and Twig 4 rejects it. An `embed` body
has its own macros.

<blockquote class="rule-requirement">Cette règle requiert une version minimale de Twig, <code>[twig.linter] version</code>, égale ou supérieure à <code>3.29</code>.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% macro title() %}A{% endmacro %}
{% macro title() %}B{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% macro title() %}Title{% endmacro %}
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

<details class="rule" name="rule" id="twig-explicit-include-context">
<summary><code class="rule__code">twig-explicit-include-context</code><a class="rule__anchor" href="#twig-explicit-include-context" aria-label="Lien permanent vers twig-explicit-include-context">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Requires `only` on `include` and `embed` tags, and a literal `with_context: false` in calls to
the built-in `include()` function, so templates do not share the whole context by accident.

A macro imported as `include` is left alone. No fix is offered, because it would change the
variables the included template can see.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% include 'card.twig' %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% include 'card.twig' with {title: title} only %}
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

<details class="rule" name="rule" id="twig-no-include-ignore-missing">
<summary><code class="rule__code">twig-no-include-ignore-missing</code><a class="rule__anchor" href="#twig-no-include-ignore-missing" aria-label="Lien permanent vers twig-no-include-ignore-missing">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows `ignore missing` on `include` and `embed` tags, and a literal `ignore_missing: true`
in calls to the built-in `include()` function.

Dynamic values are left alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% include 'card.twig' ignore missing %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% include 'card.twig' %}
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

</div>

<h2 id="redundancy">Redondance</h2>

Code mort, valeurs inutilisées, constructions sans effet observable. Les retirer garde le code honnête.

<div class="rule-list">

<details class="rule" name="rule" id="twig-constant-condition">
<summary><code class="rule__code">twig-constant-condition</code><a class="rule__anchor" href="#twig-constant-condition" aria-label="Lien permanent vers twig-constant-condition">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports literal boolean conditions in `if` branches and conditional expressions.

The rule does not guess the truthiness of other values. No fix is offered, because removing a
branch removes rendered text.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% if true %}yes{% endif %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% if flag %}yes{% endif %}
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

</div>

<h2 id="security">Sécurité</h2>

Sorties qui contournent l'échappement, sorties de débogage qui exposent le contexte du template, et fonctions, filtres ou tags interdits par votre projet.

<div class="rule-list">

<details class="rule" name="rule" id="twig-disallowed-filters">
<summary><code class="rule__code">twig-disallowed-filters</code><a class="rule__anchor" href="#twig-disallowed-filters" aria-label="Lien permanent vers twig-disallowed-filters">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports the filters listed in `names`, including in `apply` tags.

Names match exactly. An empty list reports nothing.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ title|raw }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ title }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `names` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="twig-disallowed-functions">
<summary><code class="rule__code">twig-disallowed-functions</code><a class="rule__anchor" href="#twig-disallowed-functions" aria-label="Lien permanent vers twig-disallowed-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports calls to the functions listed in `names`.

Names match exactly, including macros imported under those names. An empty list reports nothing.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ source('secret.txt') }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ title }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `names` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="twig-disallowed-tags">
<summary><code class="rule__code">twig-disallowed-tags</code><a class="rule__anchor" href="#twig-disallowed-tags" aria-label="Lien permanent vers twig-disallowed-tags">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports the tags listed in `names`, including custom tags.

Names match exactly. An empty list reports nothing.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% flush %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ title }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Défaut |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `names` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="twig-no-autoescape-false">
<summary><code class="rule__code">twig-no-autoescape-false</code><a class="rule__anchor" href="#twig-no-autoescape-false" aria-label="Lien permanent vers twig-no-autoescape-false">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows `autoescape false` blocks, which turn off escaping for their whole body.

No fix is offered, because Mago cannot tell which output needs escaping.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% autoescape false %}{{ value }}{% endautoescape %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% autoescape 'html' %}{{ value }}{% endautoescape %}
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

<details class="rule" name="rule" id="twig-no-dump">
<summary><code class="rule__code">twig-no-dump</code><a class="rule__anchor" href="#twig-no-dump" aria-label="Lien permanent vers twig-no-dump">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports calls to the built-in `dump()` function. Debug output can expose the template context.

A macro imported as `dump` is left alone. No fix is offered: the calls and their arguments stay
in place.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ dump(user) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ user.name }}
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

<details class="rule" name="rule" id="twig-no-raw">
<summary><code class="rule__code">twig-no-raw</code><a class="rule__anchor" href="#twig-no-raw" aria-label="Lien permanent vers twig-no-raw">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows the `raw` filter, including in `apply` tags. Raw output can be deliberate, so this rule
is off by default.

No fix is offered, because removing the filter can change escaping or token boundaries.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ user.name|raw }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ user.name }}
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

</div>

<h2 id="correctness">Correction</h2>

Bugs et erreurs de logique. Les règles de cette catégorie attrapent du code qui fait probablement autre chose que ce que l'auteur voulait.

<div class="rule-list">

<details class="rule" name="rule" id="twig-duplicate-block">
<summary><code class="rule__code">twig-duplicate-block</code><a class="rule__anchor" href="#twig-duplicate-block" aria-label="Lien permanent vers twig-duplicate-block">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Reports block names repeated in one template. Twig rejects such templates.

An `embed` body has its own blocks, so it may reuse a name from the enclosing template.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% block title %}A{% endblock %}
{% block title %}B{% endblock %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% block title %}Title{% endblock %}
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

<details class="rule" name="rule" id="twig-duplicate-parameter">
<summary><code class="rule__code">twig-duplicate-parameter</code><a class="rule__anchor" href="#twig-duplicate-parameter" aria-label="Lien permanent vers twig-duplicate-parameter">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Reports macro parameter names that repeat an earlier parameter, including variadic parameters.
Twig rejects such macros.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{% macro input(name, name) %}{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{% macro input(name, value) %}{% endmacro %}
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

<details class="rule" name="rule" id="twig-duplicate-argument">
<summary><code class="rule__code">twig-duplicate-argument</code><a class="rule__anchor" href="#twig-duplicate-argument" aria-label="Lien permanent vers twig-duplicate-argument">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports named arguments that repeat a name in one call, filter, or test.

Twig ignores underscores and case in the argument names of its own functions, filters, and
tests, so `first_name` and `firstName` name the same argument there. Calls to macros and
methods compare names exactly.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ range(low: 1, low: 3) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ range(low: 1, high: 3) }}
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

<details class="rule" name="rule" id="twig-duplicate-key">
<summary><code class="rule__code">twig-duplicate-key</code><a class="rule__anchor" href="#twig-duplicate-key" aria-label="Lien permanent vers twig-duplicate-key">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports literal mapping keys that repeat an earlier key after PHP's array-key conversion, so
`'1'` and `1` are the same key. The last value replaces the first.

Dynamic keys are left alone. No fix is offered, because both values can have side effects.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">À éviter</div>

```twig
{{ {title: title, title: body} }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">À privilégier</div>

```twig
{{ {title: title, body: body} }}
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

