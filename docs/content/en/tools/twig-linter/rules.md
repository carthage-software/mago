+++
title = "Rules"
description = "The full reference of every Twig linter rule, sorted by severity. Click any rule to expand its description, examples, and configuration."
nav_order = 45
nav_section = "Tools"
nav_subsection = "Twig Linter"
+++
Mago's Twig linter ships 25 rules across 8 categories. Click any rule to expand its description, requirements, default configuration, and examples. Rule codes start with <code>twig-</code>; the configuration keys under <code>[twig.linter.rules]</code> leave that prefix out.

<div class="rule-index" role="navigation" aria-label="Rule categories"><a class="rule-index__item" href="#clarity"><span class="rule-index__name">Clarity</span><span class="rule-index__count">1 rule</span><span class="rule-index__blurb">Rules that make intent explicit and reduce reader effort. They flag patterns that are technically valid but obscure what the code is doing.</span></a><a class="rule-index__item" href="#bestpractices"><span class="rule-index__name">Best practices</span><span class="rule-index__count">1 rule</span><span class="rule-index__blurb">Idiomatic Twig patterns that the Twig documentation and coding standard recommend.</span></a><a class="rule-index__item" href="#consistency"><span class="rule-index__name">Consistency</span><span class="rule-index__count">7 rules</span><span class="rule-index__blurb">Stylistic uniformity across a codebase. Pick one way to do a thing, these rules help everyone stick to it.</span></a><a class="rule-index__item" href="#deprecation"><span class="rule-index__name">Deprecation</span><span class="rule-index__count">3 rules</span><span class="rule-index__blurb">Twig features deprecated upstream that a later Twig release removes. Migrate before they break.</span></a><a class="rule-index__item" href="#maintainability"><span class="rule-index__name">Maintainability</span><span class="rule-index__count">2 rules</span><span class="rule-index__blurb">Code that is hard to keep alive over time, too complex, too tangled, too fragile. These rules surface the cost early.</span></a><a class="rule-index__item" href="#redundancy"><span class="rule-index__name">Redundancy</span><span class="rule-index__count">1 rule</span><span class="rule-index__blurb">Dead code, unused values, and constructs that have no observable effect. Trimming them keeps the codebase honest.</span></a><a class="rule-index__item" href="#security"><span class="rule-index__name">Security</span><span class="rule-index__count">6 rules</span><span class="rule-index__blurb">Output that bypasses escaping, debug output that exposes the template context, and functions, filters, or tags your project bans.</span></a><a class="rule-index__item" href="#correctness"><span class="rule-index__name">Correctness</span><span class="rule-index__count">4 rules</span><span class="rule-index__blurb">Bugs and logic errors. Rules in this category catch code that does something the author probably did not intend.</span></a></div>

<h2 id="clarity">Clarity</h2>

Rules that make intent explicit and reduce reader effort. They flag patterns that are technically valid but obscure what the code is doing.

<div class="rule-list">

<details class="rule" name="rule" id="twig-no-nested-ternary">
<summary><code class="rule__code">twig-no-nested-ternary</code><a class="rule__anchor" href="#twig-no-nested-ternary" aria-label="Permalink to twig-no-nested-ternary">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports conditional expressions nested inside other conditional expressions. Split them into
named values or branches.

Parentheses do not hide a nested conditional. A conditional in an arrow function body is not
nested in the conditional around the arrow function.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ flag ? (other ? one : two) : no }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ flag ? yes : no }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="bestpractices">Best practices</h2>

Idiomatic Twig patterns that the Twig documentation and coding standard recommend.

<div class="rule-list">

<details class="rule" name="rule" id="twig-prefer-include-function">
<summary><code class="rule__code">twig-prefer-include-function</code><a class="rule__anchor" href="#twig-prefer-include-function" aria-label="Permalink to twig-prefer-include-function">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Prefers the `include()` function to the `include` tag.

No fix is offered: a rewrite must account for escaping, sandboxing, scope, and call order.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% include 'card.twig' %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ include('card.twig') }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="consistency">Consistency</h2>

Stylistic uniformity across a codebase. Pick one way to do a thing, these rules help everyone stick to it.

<div class="rule-list">

<details class="rule" name="rule" id="twig-macro-argument-name">
<summary><code class="rule__code">twig-macro-argument-name</code><a class="rule__anchor" href="#twig-macro-argument-name" aria-label="Permalink to twig-macro-argument-name">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for macro parameter names.

Non-ASCII names are left alone. No fix is offered, because renaming a parameter breaks calls that
pass it by name.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% macro input(className) %}{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% macro input(class_name) %}{% endmacro %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-macro-name">
<summary><code class="rule__code">twig-macro-name</code><a class="rule__anchor" href="#twig-macro-name" aria-label="Permalink to twig-macro-name">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for macro names.

Non-ASCII names are left alone. No fix is offered, because renaming a macro breaks its callers.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% macro htmlInput() %}{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% macro html_input() %}{% endmacro %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-named-argument-name">
<summary><code class="rule__code">twig-named-argument-name</code><a class="rule__anchor" href="#twig-named-argument-name" aria-label="Permalink to twig-named-argument-name">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for named arguments.

Enable this only when the functions, filters, and macros you call follow the Twig coding standard.
Non-ASCII names are left alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ input(className: 'input') }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ input(class_name: 'input') }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-variable-name">
<summary><code class="rule__code">twig-variable-name</code><a class="rule__anchor" href="#twig-variable-name" aria-label="Permalink to twig-variable-name">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for the names that `set`, `for`, `import`, `from`, arrow functions,
assignments, and destructuring bind.

Variables from the application, mapping keys, attribute names, and non-ASCII names are left alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% set firstName = user.name %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% set first_name = user.name %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-end-block-name">
<summary><code class="rule__code">twig-end-block-name</code><a class="rule__anchor" href="#twig-end-block-name" aria-label="Permalink to twig-end-block-name">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Requires the block or macro name on `endblock` and `endmacro` tags.

Short blocks need no closing tag. The fix adds the name and leaves whitespace control intact.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% block title %}Title{% endblock %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% block title %}Title{% endblock title %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="twig-named-argument-separator">
<summary><code class="rule__code">twig-named-argument-separator</code><a class="rule__anchor" href="#twig-named-argument-separator" aria-label="Permalink to twig-named-argument-separator">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Requires `:` between a named argument and its value, as the Twig coding standard recommends,
instead of `=`.

Macro parameter defaults keep `=`.

<blockquote class="rule-requirement">This rule needs the minimum Twig version, <code>[twig.linter] version</code>, to be <code>3.12</code> or newer.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ range(low = 1, high = 3) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ range(low: 1, high: 3) }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="twig-single-quotes">
<summary><code class="rule__code">twig-single-quotes</code><a class="rule__anchor" href="#twig-single-quotes" aria-label="Permalink to twig-single-quotes">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Prefers single quotes for plain strings: strings without escapes, apostrophes, `#`, or line
breaks.

Other strings stay unchanged, because new quotes would change their meaning.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ "title" }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ 'title' }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

</div>

<h2 id="deprecation">Deprecation</h2>

Twig features deprecated upstream that a later Twig release removes. Migrate before they break.

<div class="rule-list">

<details class="rule" name="rule" id="twig-deprecated-attribute">
<summary><code class="rule__code">twig-deprecated-attribute</code><a class="rule__anchor" href="#twig-deprecated-attribute" aria-label="Permalink to twig-deprecated-attribute">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports calls to the built-in `attribute()` function, deprecated since Twig 3.15.

Use dynamic attribute access, `object.(name)`, after checking its call and lookup behavior. A
macro imported as `attribute` is left alone.

<blockquote class="rule-requirement">This rule needs the minimum Twig version, <code>[twig.linter] version</code>, to be <code>3.15</code> or newer.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ attribute(object, name) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ object.(name) }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-deprecated-spaceless">
<summary><code class="rule__code">twig-deprecated-spaceless</code><a class="rule__anchor" href="#twig-deprecated-spaceless" aria-label="Permalink to twig-deprecated-spaceless">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports the `spaceless` filter, deprecated since Twig 3.12, including in `apply` tags.

No fix is offered: check the rendered output before you stop stripping whitespace.

<blockquote class="rule-requirement">This rule needs the minimum Twig version, <code>[twig.linter] version</code>, to be <code>3.12</code> or newer.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ text|spaceless }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ text }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-duplicate-macro">
<summary><code class="rule__code">twig-duplicate-macro</code><a class="rule__anchor" href="#twig-duplicate-macro" aria-label="Permalink to twig-duplicate-macro">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports macro names repeated in one template. Names are case-sensitive.

Twig 3 uses the last definition; Twig 3.29 deprecates this and Twig 4 rejects it. An `embed` body
has its own macros.

<blockquote class="rule-requirement">This rule needs the minimum Twig version, <code>[twig.linter] version</code>, to be <code>3.29</code> or newer.</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% macro title() %}A{% endmacro %}
{% macro title() %}B{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% macro title() %}Title{% endmacro %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="maintainability">Maintainability</h2>

Code that is hard to keep alive over time, too complex, too tangled, too fragile. These rules surface the cost early.

<div class="rule-list">

<details class="rule" name="rule" id="twig-explicit-include-context">
<summary><code class="rule__code">twig-explicit-include-context</code><a class="rule__anchor" href="#twig-explicit-include-context" aria-label="Permalink to twig-explicit-include-context">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Requires `only` on `include` and `embed` tags, and a literal `with_context: false` in calls to
the built-in `include()` function, so templates do not share the whole context by accident.

A macro imported as `include` is left alone. No fix is offered, because it would change the
variables the included template can see.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% include 'card.twig' %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% include 'card.twig' with {title: title} only %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-no-include-ignore-missing">
<summary><code class="rule__code">twig-no-include-ignore-missing</code><a class="rule__anchor" href="#twig-no-include-ignore-missing" aria-label="Permalink to twig-no-include-ignore-missing">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows `ignore missing` on `include` and `embed` tags, and a literal `ignore_missing: true`
in calls to the built-in `include()` function.

Dynamic values are left alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% include 'card.twig' ignore missing %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% include 'card.twig' %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="redundancy">Redundancy</h2>

Dead code, unused values, and constructs that have no observable effect. Trimming them keeps the codebase honest.

<div class="rule-list">

<details class="rule" name="rule" id="twig-constant-condition">
<summary><code class="rule__code">twig-constant-condition</code><a class="rule__anchor" href="#twig-constant-condition" aria-label="Permalink to twig-constant-condition">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports literal boolean conditions in `if` branches and conditional expressions.

The rule does not guess the truthiness of other values. No fix is offered, because removing a
branch removes rendered text.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% if true %}yes{% endif %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% if flag %}yes{% endif %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="security">Security</h2>

Output that bypasses escaping, debug output that exposes the template context, and functions, filters, or tags your project bans.

<div class="rule-list">

<details class="rule" name="rule" id="twig-disallowed-filters">
<summary><code class="rule__code">twig-disallowed-filters</code><a class="rule__anchor" href="#twig-disallowed-filters" aria-label="Permalink to twig-disallowed-filters">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports the filters listed in `names`, including in `apply` tags.

Names match exactly. An empty list reports nothing.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ title|raw }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ title }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `names` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="twig-disallowed-functions">
<summary><code class="rule__code">twig-disallowed-functions</code><a class="rule__anchor" href="#twig-disallowed-functions" aria-label="Permalink to twig-disallowed-functions">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports calls to the functions listed in `names`.

Names match exactly, including macros imported under those names. An empty list reports nothing.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ source('secret.txt') }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ title }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `names` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="twig-disallowed-tags">
<summary><code class="rule__code">twig-disallowed-tags</code><a class="rule__anchor" href="#twig-disallowed-tags" aria-label="Permalink to twig-disallowed-tags">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports the tags listed in `names`, including custom tags.

Names match exactly. An empty list reports nothing.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% flush %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ title }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `names` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="twig-no-autoescape-false">
<summary><code class="rule__code">twig-no-autoescape-false</code><a class="rule__anchor" href="#twig-no-autoescape-false" aria-label="Permalink to twig-no-autoescape-false">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows `autoescape false` blocks, which turn off escaping for their whole body.

No fix is offered, because Mago cannot tell which output needs escaping.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% autoescape false %}{{ value }}{% endautoescape %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% autoescape 'html' %}{{ value }}{% endautoescape %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-no-dump">
<summary><code class="rule__code">twig-no-dump</code><a class="rule__anchor" href="#twig-no-dump" aria-label="Permalink to twig-no-dump">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports calls to the built-in `dump()` function. Debug output can expose the template context.

A macro imported as `dump` is left alone. No fix is offered: the calls and their arguments stay
in place.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ dump(user) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ user.name }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-no-raw">
<summary><code class="rule__code">twig-no-raw</code><a class="rule__anchor" href="#twig-no-raw" aria-label="Permalink to twig-no-raw">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows the `raw` filter, including in `apply` tags. Raw output can be deliberate, so this rule
is off by default.

No fix is offered, because removing the filter can change escaping or token boundaries.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ user.name|raw }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ user.name }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="correctness">Correctness</h2>

Bugs and logic errors. Rules in this category catch code that does something the author probably did not intend.

<div class="rule-list">

<details class="rule" name="rule" id="twig-duplicate-block">
<summary><code class="rule__code">twig-duplicate-block</code><a class="rule__anchor" href="#twig-duplicate-block" aria-label="Permalink to twig-duplicate-block">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Reports block names repeated in one template. Twig rejects such templates.

An `embed` body has its own blocks, so it may reuse a name from the enclosing template.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% block title %}A{% endblock %}
{% block title %}B{% endblock %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% block title %}Title{% endblock %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="twig-duplicate-parameter">
<summary><code class="rule__code">twig-duplicate-parameter</code><a class="rule__anchor" href="#twig-duplicate-parameter" aria-label="Permalink to twig-duplicate-parameter">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Reports macro parameter names that repeat an earlier parameter, including variadic parameters.
Twig rejects such macros.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{% macro input(name, name) %}{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{% macro input(name, value) %}{% endmacro %}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="twig-duplicate-argument">
<summary><code class="rule__code">twig-duplicate-argument</code><a class="rule__anchor" href="#twig-duplicate-argument" aria-label="Permalink to twig-duplicate-argument">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports named arguments that repeat a name in one call, filter, or test.

Twig ignores underscores and case in the argument names of its own functions, filters, and
tests, so `first_name` and `firstName` name the same argument there. Calls to macros and
methods compare names exactly.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ range(low: 1, low: 3) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ range(low: 1, high: 3) }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-duplicate-key">
<summary><code class="rule__code">twig-duplicate-key</code><a class="rule__anchor" href="#twig-duplicate-key" aria-label="Permalink to twig-duplicate-key">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports literal mapping keys that repeat an earlier key after PHP's array-key conversion, so
`'1'` and `1` are the same key. The last value replaces the first.

Dynamic keys are left alone. No fix is offered, because both values can have side effects.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">Avoid</div>

```twig
{{ {title: title, title: body} }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">Prefer</div>

```twig
{{ {title: title, body: body} }}
```

</div>

</div>

<hr class="rule__separator">

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

