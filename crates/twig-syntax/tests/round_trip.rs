//! Round-trip coverage over a broad corpus of Twig snippets.

#[path = "common/mod.rs"]
mod common;

use indoc::indoc;

use crate::common::parse_and_roundtrip;

fn each(corpus: &[&str]) {
    for src in corpus {
        parse_and_roundtrip(src);
    }
}

#[test]
fn corpus_raw_and_print() {
    each(&[
        "",
        "hello",
        "{{ x }}",
        "{{ 1 }}",
        "{{ 'a' }}",
        "{{ true }}",
        "{{ null }}",
        "{{ -1 }}",
        "{{ a + b }}",
        "{{ a|upper }}",
        "{{ a.b.c }}",
        "{{ a[0] }}",
        "{{ [1, 2, 3] }}",
        "{{ {a: 1} }}",
        "{{ a ? b : c }}",
        "{{ a ?? b }}",
        "{{ a is defined }}",
        "Plain text with {{ a }} and more text.",
    ]);
}

#[test]
fn corpus_tags() {
    each(&[
        "{% if a %}A{% endif %}",
        "{% if a %}A{% else %}B{% endif %}",
        "{% if a %}A{% elseif b %}B{% else %}C{% endif %}",
        "{% for x in xs %}{{ x }}{% endfor %}",
        "{% for k, v in h %}{{ k }}={{ v }}{% endfor %}",
        "{% for x in xs if x > 0 %}{{ x }}{% endfor %}",
        "{% for x in xs %}{{ x }}{% else %}none{% endfor %}",
        "{% set x = 1 %}",
        "{% set x, y = 1, 2 %}",
        "{% set body %}captured{% endset %}",
        "{% block main %}body{% endblock %}",
        "{% block main %}body{% endblock main %}",
        "{% extends 'base.twig' %}",
        "{% include 'x.twig' %}",
        "{% include 'x.twig' with vars only %}",
        "{% include 'x.twig' ignore missing %}",
        "{% embed 'x.twig' %}{% block y %}z{% endblock %}{% endembed %}",
        "{% macro foo(a, b = 1) %}{{ a }}{{ b }}{% endmacro %}",
        "{% import 'm.twig' as m %}",
        "{% from 'm.twig' import foo, bar as baz %}",
        "{% use 'base.twig' %}",
        "{% use 'base.twig' with header as base_header %}",
        "{% apply upper %}hi{% endapply %}",
        "{% apply lower|title %}HELLO{% endapply %}",
        "{% autoescape 'html' %}{{ x }}{% endautoescape %}",
        "{% sandbox %}{% include 'c.twig' %}{% endsandbox %}",
        "{% do foo() %}",
        "{% flush %}",
        "{% deprecated 'old' %}",
        "{% cache 'k' %}body{% endcache %}",
        "{% verbatim %}{{ x }}{% endverbatim %}",
        "{% raw %}{{ y }}{% endraw %}",
        "{% with { a: 1 } only %}{{ a }}{% endwith %}",
        "{% guard function constant %}ok{% endguard %}",
        "{% guard filter upper %}ok{% endguard %}",
        "{% guard test defined %}ok{% else %}no{% endguard %}",
    ]);
}

#[test]
fn corpus_whitespace_control() {
    each(&[
        "x {{- y -}} z",
        "a {%- if c -%} b {%- endif -%} d",
        "{#- c -#}",
        "{{~ x ~}}",
        "{%~ if c ~%}b{%~ endif ~%}",
        "{#~ c ~#}",
    ]);
}

#[test]
fn corpus_strings_and_interp() {
    each(&[
        r#"{{ 'hello' }}"#,
        r#"{{ "hello" }}"#,
        r#"{{ "a\"b" }}"#,
        r#"{{ 'it\'s' }}"#,
        r#"{{ "hi #{name}" }}"#,
        r#"{{ "n=#{a + b}" }}"#,
        r#"{{ "foo #{"bar #{baz}"} qux" }}"#,
    ]);
}

#[test]
fn corpus_operators() {
    each(&[
        "{{ a + b - c }}",
        "{{ a * b / c % d }}",
        "{{ a // b }}",
        "{{ a ** b }}",
        "{{ a == b }}",
        "{{ a != b }}",
        "{{ a <= b }}",
        "{{ a >= b }}",
        "{{ a <=> b }}",
        "{{ a === b }}",
        "{{ a !== b }}",
        "{{ a and b }}",
        "{{ a or b }}",
        "{{ a xor b }}",
        "{{ a b-and b }}",
        "{{ a b-or b }}",
        "{{ a b-xor b }}",
        "{{ a in xs }}",
        "{{ a not in xs }}",
        "{{ 'a' starts with 's' }}",
        "{{ 'a' ends with 's' }}",
        "{{ xs has some [1] }}",
        "{{ xs has every [1] }}",
        "{{ x is same as(y) }}",
        "{{ x is divisible by(3) }}",
        "{{ 'x' matches '/x/' }}",
        "{{ 1..10 }}",
    ]);
}

#[test]
fn corpus_expressions_misc() {
    each(&[
        "{{ list|map(x => x * 2) }}",
        "{{ list|filter(x => x > 0) }}",
        "{{ f(1, 2, 3) }}",
        "{{ f(a=1, b=2) }}",
        "{{ f(a: 1, b: 2) }}",
        "{{ f(...args) }}",
        "{{ a[0:3] }}",
        "{{ a[:3] }}",
        "{{ a[1:] }}",
        "{{ a?.b }}",
        "{{ a.b(1, 2).c.d() }}",
        "{{ [1, ...xs, 3] }}",
        "{{ { a: 1, ...rest } }}",
        "{{ (a, b) => a + b }}",
    ]);
}

#[test]
fn corpus_multiline_templates() {
    parse_and_roundtrip(indoc! {r#"
        {% extends 'base.twig' %}

        {% block head %}
            <title>{{ title|escape }}</title>
        {% endblock %}

        {% block body %}
            {% for item in items %}
                <li>{{ item.name }}</li>
            {% else %}
                <li>none</li>
            {% endfor %}
        {% endblock %}
    "#});

    parse_and_roundtrip(indoc! {"
        {% macro input(name, value = '', type = 'text') %}
            <input type=\"{{ type }}\" name=\"{{ name }}\" value=\"{{ value }}\">
        {% endmacro %}

        {% import _self as forms %}
        {{ forms.input('username') }}
    "});

    parse_and_roundtrip(indoc! {r#"
        {# A comment block #}
        {% if user is defined and user.is_admin %}
            Hi {{ user.name|title }}!
        {% elseif user is defined %}
            Hi {{ user.name }}.
        {% else %}
            Welcome, guest.
        {% endif %}
    "#});
}

#[test]
fn corpus_nested_if_for() {
    parse_and_roundtrip(indoc! {r#"
        {% for category in categories %}
            <h1>{{ category.name }}</h1>
            {% if category.items|length > 0 %}
                <ul>
                    {% for item in category.items %}
                        <li>{{ item.title }} — {{ item.price }}</li>
                    {% endfor %}
                </ul>
            {% else %}
                <p>No items.</p>
            {% endif %}
        {% endfor %}
    "#});
}

#[test]
fn corpus_comments_and_verbatim_mixed() {
    parse_and_roundtrip(indoc! {r#"
        {# top comment #}
        {% verbatim %}
            {{ literal_output }}
        {% endverbatim %}
        {% raw %}{{ x }}{% endraw %}
        {# tail comment #}
    "#});
}

#[test]
fn corpus_deep_filter_chains() {
    each(&["{{ x|a|b|c|d|e|f|g|h }}", "{{ x|a(1)|b(2)|c(3) }}", "{{ x|default('a')|upper|replace({' ': '_'}) }}"]);
}

#[test]
fn corpus_random_selection_from_fixtures() {
    each(&[
        "{% set items = [{name: 'x'}, {name: 'y'}] %}{% for i in items %}{{ i.name }}{% endfor %}",
        "{{ users|filter(u => u.active)|map(u => u.name)|join(', ') }}",
        "{{ a ?? b ?? c ?? 'fallback' }}",
        "{{ (a or b) and (c or d) }}",
        "{% if user.age >= 18 and user.consent %}OK{% endif %}",
        r#"{{ "foo" ~ bar ~ 'baz' }}"#,
        "{% set x %}<p>Captured markup: {{ now }}</p>{% endset %}",
        "{% block title %}Default{% endblock title %}",
        "{{ {(k1): v1, 'plain': v2, 0: v3, ...rest} }}",
        "{% guard test divisible by %}yes{% endguard %}",
        "{{ -3 ** 2 }}",
        "{{ not not true }}",
        "{{ (x + 1) * (y - 2) }}",
        "{{ 'a' in 'abc' }}",
        "{{ 'a' not in ['b', 'c'] }}",
        r#"{{ attribute(obj, 'prop') }}"#,
        "{{ cycle(['a', 'b', 'c'], loop.index0) }}",
        "{{ range(1, 10, 2)|length }}",
        "{{ obj.name ?? obj['name'] ?? 'default' }}",
        "{% for k, v in {a: 1, b: 2} %}{{ k }}={{ v }}{% endfor %}",
        "{% apply upper|trim(' ') %}  hi  {% endapply %}",
        "{% include 'child.twig' with {val: 1, fallback: 'x'} only %}",
        "{% set name = first ~ ' ' ~ last %}",
        "{{ 1_000 + 2_000 }}",
        "{{ 1.5e10 * 2 }}",
        "{% cache 'k' ttl(60) tags(['t1', 't2']) %}body{% endcache %}",
        "{% autoescape 'html' %}{{ x|raw }}{% endautoescape %}",
        "{% with {depth: depth + 1} %}{{ render() }}{% endwith %}",
        "{{ a matches '#^foo#' }}",
        "{{ a ?: 'fallback' }}",
    ]);
}
