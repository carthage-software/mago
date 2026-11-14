+++
title = "规则"
description = "Twig linter 每条规则的完整参考,按严重程度排序。点击任意规则可展开其描述、示例和配置。"
nav_order = 45
nav_section = "Tools"
nav_subsection = "Twig Linter"
+++
Mago 的 Twig linter 共提供 25 条规则,分布在 8 个类别中。点击任意规则可展开其描述、要求、默认配置和示例。规则代码以 <code>twig-</code> 开头;<code>[twig.linter.rules]</code> 下的配置键省略该前缀。

<div class="rule-index" role="navigation" aria-label="规则类别"><a class="rule-index__item" href="#clarity"><span class="rule-index__name">清晰度</span><span class="rule-index__count">1 条规则</span><span class="rule-index__blurb">让意图更明确、减少阅读负担的规则。它们指出在技术上合法但会模糊代码意图的写法。</span></a><a class="rule-index__item" href="#bestpractices"><span class="rule-index__name">最佳实践</span><span class="rule-index__count">1 条规则</span><span class="rule-index__blurb">Twig 文档和编码规范推荐的惯用写法。</span></a><a class="rule-index__item" href="#consistency"><span class="rule-index__name">一致性</span><span class="rule-index__count">7 条规则</span><span class="rule-index__blurb">保持整套代码风格一致。选定一种写法,这些规则帮助所有人坚持下去。</span></a><a class="rule-index__item" href="#deprecation"><span class="rule-index__name">弃用</span><span class="rule-index__count">3 条规则</span><span class="rule-index__blurb">上游已弃用、后续 Twig 版本会移除的 Twig 特性。请在它们失效前迁移。</span></a><a class="rule-index__item" href="#maintainability"><span class="rule-index__name">可维护性</span><span class="rule-index__count">2 条规则</span><span class="rule-index__blurb">难以长期维护的代码,过于复杂、纠缠或脆弱。这些规则提前暴露成本。</span></a><a class="rule-index__item" href="#redundancy"><span class="rule-index__name">冗余</span><span class="rule-index__count">1 条规则</span><span class="rule-index__blurb">无用代码、无效值,以及没有可观察效果的写法。清理它们让代码更诚实。</span></a><a class="rule-index__item" href="#security"><span class="rule-index__name">安全</span><span class="rule-index__count">6 条规则</span><span class="rule-index__blurb">绕过转义的输出、暴露模板上下文的调试输出,以及项目禁用的函数、过滤器或标签。</span></a><a class="rule-index__item" href="#correctness"><span class="rule-index__name">正确性</span><span class="rule-index__count">4 条规则</span><span class="rule-index__blurb">缺陷与逻辑错误。本类规则负责发现作者本意之外的行为。</span></a></div>

<h2 id="clarity">清晰度</h2>

让意图更明确、减少阅读负担的规则。它们指出在技术上合法但会模糊代码意图的写法。

<div class="rule-list">

<details class="rule" name="rule" id="twig-no-nested-ternary">
<summary><code class="rule__code">twig-no-nested-ternary</code><a class="rule__anchor" href="#twig-no-nested-ternary" aria-label="twig-no-nested-ternary 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports conditional expressions nested inside other conditional expressions. Split them into
named values or branches.

Parentheses do not hide a nested conditional. A conditional in an arrow function body is not
nested in the conditional around the arrow function.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ flag ? (other ? one : two) : no }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ flag ? yes : no }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="bestpractices">最佳实践</h2>

Twig 文档和编码规范推荐的惯用写法。

<div class="rule-list">

<details class="rule" name="rule" id="twig-prefer-include-function">
<summary><code class="rule__code">twig-prefer-include-function</code><a class="rule__anchor" href="#twig-prefer-include-function" aria-label="twig-prefer-include-function 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Prefers the `include()` function to the `include` tag.

No fix is offered: a rewrite must account for escaping, sandboxing, scope, and call order.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% include 'card.twig' %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ include('card.twig') }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="consistency">一致性</h2>

保持整套代码风格一致。选定一种写法,这些规则帮助所有人坚持下去。

<div class="rule-list">

<details class="rule" name="rule" id="twig-macro-argument-name">
<summary><code class="rule__code">twig-macro-argument-name</code><a class="rule__anchor" href="#twig-macro-argument-name" aria-label="twig-macro-argument-name 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for macro parameter names.

Non-ASCII names are left alone. No fix is offered, because renaming a parameter breaks calls that
pass it by name.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% macro input(className) %}{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% macro input(class_name) %}{% endmacro %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-macro-name">
<summary><code class="rule__code">twig-macro-name</code><a class="rule__anchor" href="#twig-macro-name" aria-label="twig-macro-name 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for macro names.

Non-ASCII names are left alone. No fix is offered, because renaming a macro breaks its callers.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% macro htmlInput() %}{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% macro html_input() %}{% endmacro %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-named-argument-name">
<summary><code class="rule__code">twig-named-argument-name</code><a class="rule__anchor" href="#twig-named-argument-name" aria-label="twig-named-argument-name 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for named arguments.

Enable this only when the functions, filters, and macros you call follow the Twig coding standard.
Non-ASCII names are left alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ input(className: 'input') }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ input(class_name: 'input') }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-variable-name">
<summary><code class="rule__code">twig-variable-name</code><a class="rule__anchor" href="#twig-variable-name" aria-label="twig-variable-name 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Enforces snake_case for the names that `set`, `for`, `import`, `from`, arrow functions,
assignments, and destructuring bind.

Variables from the application, mapping keys, attribute names, and non-ASCII names are left alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% set firstName = user.name %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% set first_name = user.name %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-end-block-name">
<summary><code class="rule__code">twig-end-block-name</code><a class="rule__anchor" href="#twig-end-block-name" aria-label="twig-end-block-name 的永久链接">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Requires the block or macro name on `endblock` and `endmacro` tags.

Short blocks need no closing tag. The fix adds the name and leaves whitespace control intact.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% block title %}Title{% endblock %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% block title %}Title{% endblock title %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="twig-named-argument-separator">
<summary><code class="rule__code">twig-named-argument-separator</code><a class="rule__anchor" href="#twig-named-argument-separator" aria-label="twig-named-argument-separator 的永久链接">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Requires `:` between a named argument and its value, as the Twig coding standard recommends,
instead of `=`.

Macro parameter defaults keep `=`.

<blockquote class="rule-requirement">此规则要求最低 Twig 版本 <code>[twig.linter] version</code> 为 <code>3.12</code> 或更高。</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ range(low = 1, high = 3) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ range(low: 1, high: 3) }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

</div>

</details>

<details class="rule" name="rule" id="twig-single-quotes">
<summary><code class="rule__code">twig-single-quotes</code><a class="rule__anchor" href="#twig-single-quotes" aria-label="twig-single-quotes 的永久链接">¶</a><span class="rule__level rule__level--help">help</span></summary>

<div class="rule__body">

Prefers single quotes for plain strings: strings without escapes, apostrophes, `#`, or line
breaks.

Other strings stay unchanged, because new quotes would change their meaning.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ "title" }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ 'title' }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

</div>

</details>

</div>

<h2 id="deprecation">弃用</h2>

上游已弃用、后续 Twig 版本会移除的 Twig 特性。请在它们失效前迁移。

<div class="rule-list">

<details class="rule" name="rule" id="twig-deprecated-attribute">
<summary><code class="rule__code">twig-deprecated-attribute</code><a class="rule__anchor" href="#twig-deprecated-attribute" aria-label="twig-deprecated-attribute 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports calls to the built-in `attribute()` function, deprecated since Twig 3.15.

Use dynamic attribute access, `object.(name)`, after checking its call and lookup behavior. A
macro imported as `attribute` is left alone.

<blockquote class="rule-requirement">此规则要求最低 Twig 版本 <code>[twig.linter] version</code> 为 <code>3.15</code> 或更高。</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ attribute(object, name) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ object.(name) }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-deprecated-spaceless">
<summary><code class="rule__code">twig-deprecated-spaceless</code><a class="rule__anchor" href="#twig-deprecated-spaceless" aria-label="twig-deprecated-spaceless 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports the `spaceless` filter, deprecated since Twig 3.12, including in `apply` tags.

No fix is offered: check the rendered output before you stop stripping whitespace.

<blockquote class="rule-requirement">此规则要求最低 Twig 版本 <code>[twig.linter] version</code> 为 <code>3.12</code> 或更高。</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ text|spaceless }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ text }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-duplicate-macro">
<summary><code class="rule__code">twig-duplicate-macro</code><a class="rule__anchor" href="#twig-duplicate-macro" aria-label="twig-duplicate-macro 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports macro names repeated in one template. Names are case-sensitive.

Twig 3 uses the last definition; Twig 3.29 deprecates this and Twig 4 rejects it. An `embed` body
has its own macros.

<blockquote class="rule-requirement">此规则要求最低 Twig 版本 <code>[twig.linter] version</code> 为 <code>3.29</code> 或更高。</blockquote>

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% macro title() %}A{% endmacro %}
{% macro title() %}B{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% macro title() %}Title{% endmacro %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="maintainability">可维护性</h2>

难以长期维护的代码,过于复杂、纠缠或脆弱。这些规则提前暴露成本。

<div class="rule-list">

<details class="rule" name="rule" id="twig-explicit-include-context">
<summary><code class="rule__code">twig-explicit-include-context</code><a class="rule__anchor" href="#twig-explicit-include-context" aria-label="twig-explicit-include-context 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Requires `only` on `include` and `embed` tags, and a literal `with_context: false` in calls to
the built-in `include()` function, so templates do not share the whole context by accident.

A macro imported as `include` is left alone. No fix is offered, because it would change the
variables the included template can see.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% include 'card.twig' %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% include 'card.twig' with {title: title} only %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-no-include-ignore-missing">
<summary><code class="rule__code">twig-no-include-ignore-missing</code><a class="rule__anchor" href="#twig-no-include-ignore-missing" aria-label="twig-no-include-ignore-missing 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows `ignore missing` on `include` and `embed` tags, and a literal `ignore_missing: true`
in calls to the built-in `include()` function.

Dynamic values are left alone.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% include 'card.twig' ignore missing %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% include 'card.twig' %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="redundancy">冗余</h2>

无用代码、无效值,以及没有可观察效果的写法。清理它们让代码更诚实。

<div class="rule-list">

<details class="rule" name="rule" id="twig-constant-condition">
<summary><code class="rule__code">twig-constant-condition</code><a class="rule__anchor" href="#twig-constant-condition" aria-label="twig-constant-condition 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports literal boolean conditions in `if` branches and conditional expressions.

The rule does not guess the truthiness of other values. No fix is offered, because removing a
branch removes rendered text.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% if true %}yes{% endif %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% if flag %}yes{% endif %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="security">安全</h2>

绕过转义的输出、暴露模板上下文的调试输出,以及项目禁用的函数、过滤器或标签。

<div class="rule-list">

<details class="rule" name="rule" id="twig-disallowed-filters">
<summary><code class="rule__code">twig-disallowed-filters</code><a class="rule__anchor" href="#twig-disallowed-filters" aria-label="twig-disallowed-filters 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports the filters listed in `names`, including in `apply` tags.

Names match exactly. An empty list reports nothing.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ title|raw }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ title }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `names` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="twig-disallowed-functions">
<summary><code class="rule__code">twig-disallowed-functions</code><a class="rule__anchor" href="#twig-disallowed-functions" aria-label="twig-disallowed-functions 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports calls to the functions listed in `names`.

Names match exactly, including macros imported under those names. An empty list reports nothing.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ source('secret.txt') }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ title }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `names` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="twig-disallowed-tags">
<summary><code class="rule__code">twig-disallowed-tags</code><a class="rule__anchor" href="#twig-disallowed-tags" aria-label="twig-disallowed-tags 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports the tags listed in `names`, including custom tags.

Names match exactly. An empty list reports nothing.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% flush %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ title }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `names` | `array` | `[]` |

</div>

</details>

<details class="rule" name="rule" id="twig-no-autoescape-false">
<summary><code class="rule__code">twig-no-autoescape-false</code><a class="rule__anchor" href="#twig-no-autoescape-false" aria-label="twig-no-autoescape-false 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows `autoescape false` blocks, which turn off escaping for their whole body.

No fix is offered, because Mago cannot tell which output needs escaping.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% autoescape false %}{{ value }}{% endautoescape %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% autoescape 'html' %}{{ value }}{% endautoescape %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-no-dump">
<summary><code class="rule__code">twig-no-dump</code><a class="rule__anchor" href="#twig-no-dump" aria-label="twig-no-dump 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports calls to the built-in `dump()` function. Debug output can expose the template context.

A macro imported as `dump` is left alone. No fix is offered: the calls and their arguments stay
in place.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ dump(user) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ user.name }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-no-raw">
<summary><code class="rule__code">twig-no-raw</code><a class="rule__anchor" href="#twig-no-raw" aria-label="twig-no-raw 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Disallows the `raw` filter, including in `apply` tags. Raw output can be deliberate, so this rule
is off by default.

No fix is offered, because removing the filter can change escaping or token boundaries.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ user.name|raw }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ user.name }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

<h2 id="correctness">正确性</h2>

缺陷与逻辑错误。本类规则负责发现作者本意之外的行为。

<div class="rule-list">

<details class="rule" name="rule" id="twig-duplicate-block">
<summary><code class="rule__code">twig-duplicate-block</code><a class="rule__anchor" href="#twig-duplicate-block" aria-label="twig-duplicate-block 的永久链接">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Reports block names repeated in one template. Twig rejects such templates.

An `embed` body has its own blocks, so it may reuse a name from the enclosing template.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% block title %}A{% endblock %}
{% block title %}B{% endblock %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% block title %}Title{% endblock %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="twig-duplicate-parameter">
<summary><code class="rule__code">twig-duplicate-parameter</code><a class="rule__anchor" href="#twig-duplicate-parameter" aria-label="twig-duplicate-parameter 的永久链接">¶</a><span class="rule__level rule__level--error">error</span></summary>

<div class="rule__body">

Reports macro parameter names that repeat an earlier parameter, including variadic parameters.
Twig rejects such macros.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{% macro input(name, name) %}{% endmacro %}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{% macro input(name, value) %}{% endmacro %}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

</div>

</details>

<details class="rule" name="rule" id="twig-duplicate-argument">
<summary><code class="rule__code">twig-duplicate-argument</code><a class="rule__anchor" href="#twig-duplicate-argument" aria-label="twig-duplicate-argument 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports named arguments that repeat a name in one call, filter, or test.

Twig ignores underscores and case in the argument names of its own functions, filters, and
tests, so `first_name` and `firstName` name the same argument there. Calls to macros and
methods compare names exactly.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ range(low: 1, low: 3) }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ range(low: 1, high: 3) }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

<details class="rule" name="rule" id="twig-duplicate-key">
<summary><code class="rule__code">twig-duplicate-key</code><a class="rule__anchor" href="#twig-duplicate-key" aria-label="twig-duplicate-key 的永久链接">¶</a><span class="rule__level rule__level--warning">warning</span></summary>

<div class="rule__body">

Reports literal mapping keys that repeat an earlier key after PHP's array-key conversion, so
`'1'` and `1` are the same key. The last value replaces the first.

Dynamic keys are left alone. No fix is offered, because both values can have side effects.

<hr class="rule__separator">

<div class="rule-examples">

<div class="rule-example rule-example--bad">
<div class="rule-example__label">避免</div>

```twig
{{ {title: title, title: body} }}
```

</div>

<div class="rule-example rule-example--good">
<div class="rule-example__label">推荐</div>

```twig
{{ {title: title, body: body} }}
```

</div>

</div>

<hr class="rule__separator">

| 选项 | 类型 | 默认值 |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

</div>

</details>

</div>

