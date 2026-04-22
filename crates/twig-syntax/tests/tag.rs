//! Focused tests for every Twig tag the parser recognises.

#[path = "common/mod.rs"]
mod common;

use crate::common::parse;
use crate::common::parse_ok;
use crate::common::parses;
use bumpalo::Bump;
use mago_twig_syntax::ast::Expression;
use mago_twig_syntax::ast::GuardKind;
use mago_twig_syntax::ast::Statement;
use mago_twig_syntax::ast::TriviaKind;

#[test]
fn tag_if_basic() {
    parses("{% if a %}A{% endif %}");
}

#[test]
fn tag_if_elseif_else() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% if a %}A{% elseif b %}B{% else %}C{% endif %}");
    let Statement::If(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.branches.len(), 2, "two branches: if + elseif");
    assert!(n.else_branch.is_some());
}

#[test]
fn tag_if_multiple_elseif() {
    parses("{% if a %}A{% elseif b %}B{% elseif c %}C{% else %}D{% endif %}");
}

#[test]
fn tag_if_with_complex_condition() {
    parses("{% if (a or b) and not c %}X{% endif %}");
}

#[test]
fn tag_for_basic() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% for x in list %}{{ x }}{% endfor %}");
    let Statement::For(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.targets.len(), 1);
    assert_eq!(n.targets.nodes[0].value, "x");
    assert!(n.else_branch.is_none());
    assert!(n.if_clause.is_none());
}

#[test]
fn tag_for_key_value() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% for k, v in items %}{{ k }}={{ v }}{% endfor %}");
    let Statement::For(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.targets.len(), 2);
    assert_eq!(n.targets.nodes[0].value, "k");
    assert_eq!(n.targets.nodes[1].value, "v");
}

#[test]
fn tag_for_with_else() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% for x in list %}{{ x }}{% else %}empty{% endfor %}");
    let Statement::For(n) = &tpl.statements.nodes[0] else { panic!() };
    assert!(n.else_branch.is_some());
}

#[test]
fn tag_for_with_filter_condition() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% for x in list if x > 0 %}{{ x }}{% endfor %}");
    let Statement::For(n) = &tpl.statements.nodes[0] else { panic!() };
    assert!(n.if_clause.is_some());
}

#[test]
fn tag_set_single() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% set a = 1 %}");
    let Statement::Set(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.names.len(), 1);
    match &n.body {
        mago_twig_syntax::ast::SetBody::Inline(i) => assert_eq!(i.values.len(), 1),
        _ => assert_eq!(0usize, 1),
    };
    assert!(matches!(n.body, mago_twig_syntax::ast::SetBody::Inline(_)));
}

#[test]
fn tag_set_multiple() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% set a, b = 1, 2 %}");
    let Statement::Set(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.names.len(), 2);
    match &n.body {
        mago_twig_syntax::ast::SetBody::Inline(i) => assert_eq!(i.values.len(), 2),
        _ => assert_eq!(0usize, 2),
    };
}

#[test]
fn tag_set_capture_body() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% set x %}captured{% endset %}");
    let Statement::Set(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.names.len(), 1);
    assert!(matches!(n.body, mago_twig_syntax::ast::SetBody::Capture(_)));
    match &n.body {
        mago_twig_syntax::ast::SetBody::Inline(i) => assert_eq!(i.values.len(), 0),
        _ => assert_eq!(0usize, 0),
    };
}

#[test]
fn tag_block_short_form() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% block title 'hi' %}");
    let Statement::Block(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.name.value, "title");
    assert!(matches!(n.body, mago_twig_syntax::ast::BlockBody::Short(_)));
}

#[test]
fn tag_block_body_form() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% block main %}body{% endblock %}");
    let Statement::Block(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.name.value, "main");
    assert!(matches!(n.body, mago_twig_syntax::ast::BlockBody::Long(_)));
}

#[test]
fn tag_block_closing_name_must_match() {
    let arena = Bump::new();
    parse_ok(&arena, "{% block main %}body{% endblock main %}");
}

#[test]
fn tag_block_mismatched_closing_name_rejected() {
    crate::common::rejects("{% block main %}body{% endblock other %}");
}

#[test]
fn tag_nested_blocks() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% block outer %}{% block inner %}hi{% endblock %}{% endblock %}");
    let Statement::Block(outer) = &tpl.statements.nodes[0] else { panic!() };
    let mago_twig_syntax::ast::BlockBody::Long(long) = &outer.body else { panic!() };
    assert!(long.body.iter().any(|n| matches!(n, Statement::Block(_))));
}

#[test]
fn tag_extends() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% extends 'base.twig' %}");
    let Statement::Extends(_) = &tpl.statements.nodes[0] else { panic!() };
}

#[test]
fn tag_extends_then_blocks() {
    parses("{% extends 'base.twig' %}{% block main %}hi{% endblock %}");
}

#[test]
fn tag_block_can_call_parent() {
    parses("{% block main %}{{ parent() }}{% endblock %}");
}

#[test]
fn tag_include_basic() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% include 'x.twig' %}");
    let Statement::Include(n) = &tpl.statements.nodes[0] else { panic!() };
    assert!(n.with_clause.is_none());
    assert!(n.only_keyword.is_none());
    assert!(n.ignore_missing.is_none());
}

#[test]
fn tag_include_with_vars() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% include 'x.twig' with vars %}");
    let Statement::Include(n) = &tpl.statements.nodes[0] else { panic!() };
    assert!(n.with_clause.is_some());
}

#[test]
fn tag_include_with_vars_only() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% include 'x.twig' with vars only %}");
    let Statement::Include(n) = &tpl.statements.nodes[0] else { panic!() };
    assert!(n.only_keyword.is_some());
}

#[test]
fn tag_include_ignore_missing() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% include 'x.twig' ignore missing %}");
    let Statement::Include(n) = &tpl.statements.nodes[0] else { panic!() };
    assert!(n.ignore_missing.is_some());
}

#[test]
fn tag_include_ignore_missing_with_vars() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% include 'x.twig' ignore missing with vars %}");
    let Statement::Include(n) = &tpl.statements.nodes[0] else { panic!() };
    assert!(n.ignore_missing.is_some());
    assert!(n.with_clause.is_some());
}

#[test]
fn tag_include_inline_hash_vars() {
    parses("{% include 'x.twig' with { a: 1, b: 2 } only %}");
}

#[test]
fn tag_embed_basic() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% embed 'x.twig' %}{% block b %}y{% endblock %}{% endembed %}");
    let Statement::Embed(_) = &tpl.statements.nodes[0] else { panic!() };
}

#[test]
fn tag_embed_with_vars() {
    parses("{% embed 'x.twig' with {a: 1} %}{% block b %}y{% endblock %}{% endembed %}");
}

#[test]
fn tag_embed_ignore_missing_only() {
    parses("{% embed 'x.twig' ignore missing with {a: 1} only %}{% endembed %}");
}

#[test]
fn tag_macro_no_args() {
    parses("{% macro foo() %}body{% endmacro %}");
}

#[test]
fn tag_macro_with_args() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% macro foo(a, b) %}{{ a }}{{ b }}{% endmacro %}");
    let Statement::Macro(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.name.value, "foo");
    assert_eq!(n.arguments.len(), 2);
}

#[test]
fn tag_macro_default_args() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% macro foo(a, b = 1) %}{{ a + b }}{% endmacro %}");
    let Statement::Macro(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.arguments.len(), 2);
    assert!(n.arguments.nodes[0].default.is_none());
    assert!(n.arguments.nodes[1].default.is_some());
}

#[test]
fn tag_macro_endmacro_with_name() {
    parses("{% macro foo(a) %}x{% endmacro foo %}");
}

#[test]
fn tag_import() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% import 'macros.twig' as m %}");
    let Statement::Import(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.alias.value, "m");
}

#[test]
fn tag_from() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% from 'macros.twig' import foo, bar as baz %}");
    let Statement::From(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.names.len(), 2);
    let e0 = &n.names.nodes[0];
    assert_eq!(e0.from.value, "foo");
    assert_eq!(e0.to.map(|t| t.value).unwrap_or(e0.from.value), "foo");
    let e1 = &n.names.nodes[1];
    assert_eq!(e1.from.value, "bar");
    assert_eq!(e1.to.map(|t| t.value).unwrap_or(e1.from.value), "baz");
}

#[test]
fn tag_use_simple() {
    parses("{% use 'base.twig' %}");
}

#[test]
fn tag_use_with_aliases() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% use 'base.twig' with header as base_header, footer as base_footer %}");
    let Statement::Use(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.aliases.len(), 2);
}

#[test]
fn tag_apply_single_filter() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% apply upper %}hi{% endapply %}");
    let Statement::Apply(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.filters.len(), 1);
    assert_eq!(n.filters.nodes[0].name.value, "upper");
}

#[test]
fn tag_apply_filter_chain() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% apply lower|title %}HELLO{% endapply %}");
    let Statement::Apply(n) = &tpl.statements.nodes[0] else { panic!() };
    let names: Vec<&str> = n.filters.iter().map(|f| f.name.value).collect();
    assert_eq!(names, vec!["lower", "title"]);
    // The `|` separator between the two filters is preserved in the
    // token-separated sequence.
    assert_eq!(n.filters.tokens.len(), 1);
}

#[test]
fn tag_apply_filter_with_args() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, r#"{% apply replace({'a': 'b'})|upper %}x{% endapply %}"#);
    let Statement::Apply(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.filters.len(), 2);
    assert_eq!(n.filters.nodes[0].name.value, "replace");
    assert_eq!(n.filters.nodes[0].argument_list.as_ref().map(|a| a.arguments.len()).unwrap_or(0), 1);
    assert_eq!(n.filters.nodes[1].name.value, "upper");
    assert!(n.filters.nodes[1].argument_list.is_none());
}

#[test]
fn tag_apply_body_keeps_nested_nodes() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% apply upper %}{{ x }}{% endapply %}");
    let Statement::Apply(n) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(n.body.len(), 1);
    let Statement::Print(p) = &n.body.nodes[0] else { panic!() };
    assert!(matches!(p.expression, Expression::Name(_)));
}

#[test]
fn tag_autoescape_with_strategy() {
    parses("{% autoescape 'html' %}{{ x }}{% endautoescape %}");
}

#[test]
fn tag_autoescape_without_strategy() {
    parses("{% autoescape %}{{ x }}{% endautoescape %}");
}

#[test]
fn tag_autoescape_with_boolean() {
    parses("{% autoescape false %}{{ x }}{% endautoescape %}");
}

#[test]
fn tag_sandbox() {
    parses("{% sandbox %}{% include 'child.twig' %}{% endsandbox %}");
}

#[test]
fn tag_deprecated_with_message() {
    parses("{% deprecated 'use the new thing' %}");
}

#[test]
fn tag_deprecated_with_options() {
    parses("{% deprecated 'go away' package='foo' version='1.2' %}");
}

#[test]
fn tag_do_simple() {
    parses("{% do foo() %}");
}

#[test]
fn tag_do_assignment() {
    parses("{% do x = 1 %}");
}

#[test]
fn tag_flush() {
    parses("{% flush %}");
}

#[test]
fn tag_guard_function() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% guard function constant %}ok{% endguard %}");
    let Statement::Guard(g) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(g.kind, GuardKind::Function);
    assert_eq!(g.name.value, "constant");
}

#[test]
fn tag_guard_filter() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% guard filter upper %}ok{% endguard %}");
    let Statement::Guard(g) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(g.kind, GuardKind::Filter);
    assert_eq!(g.name.value, "upper");
}

#[test]
fn tag_guard_test() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% guard test defined %}ok{% endguard %}");
    let Statement::Guard(g) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(g.kind, GuardKind::Test);
    assert_eq!(g.name.value, "defined");
}

#[test]
fn tag_guard_with_else() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% guard filter foobar %}A{% else %}B{% endguard %}");
    let Statement::Guard(g) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(g.kind, GuardKind::Filter);
    assert!(g.else_branch.is_some());
}

#[test]
fn tag_guard_test_compound_divisible_by() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% guard test divisible by %}yes{% endguard %}");
    let Statement::Guard(g) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(g.kind, GuardKind::Test);
    assert_eq!(g.name.value, "divisible by");
}

#[test]
fn tag_with_basic() {
    parses("{% with { a: 1 } %}{{ a }}{% endwith %}");
}

#[test]
fn tag_with_only() {
    parses("{% with { a: 1 } only %}{{ a }}{% endwith %}");
}

#[test]
fn tag_with_empty() {
    parses("{% with %}body{% endwith %}");
}

#[test]
fn tag_cache_basic_key() {
    parses("{% cache 'my-key' %}body{% endcache %}");
}

#[test]
fn tag_cache_with_ttl() {
    parses("{% cache 'k' ttl(60) %}body{% endcache %}");
}

#[test]
fn tag_cache_with_ttl_and_tags() {
    parses("{% cache 'k' ttl(60) tags(['a', 'b']) %}body{% endcache %}");
}

#[test]
fn tag_types() {
    parses("{% types { name: 'string', age: 'int' } %}");
}

#[test]
fn tag_verbatim_preserves_body_raw() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% verbatim %}{{ not_parsed }}{% endverbatim %}");
    let Statement::Verbatim(v) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(v.body, "{{ not_parsed }}");
}

#[test]
fn tag_raw_alias_parses() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% raw %}{{ nope }}{% endraw %}");
    let Statement::Verbatim(v) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(v.body, "{{ nope }}");
}

#[test]
fn tag_verbatim_empty_body() {
    parses("{% verbatim %}{% endverbatim %}");
}

#[test]
fn comment_surfaces_as_trivia() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{# hi #}");
    assert!(tpl.statements.is_empty(), "comments must not appear as statements");
    let c = tpl.trivia.iter().find(|t| t.kind == TriviaKind::Comment).expect("comment trivia");
    assert_eq!(c.value, "{# hi #}");
}

#[test]
fn comment_value_preserves_markers() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{# hello world #}");
    let c = tpl.trivia.iter().find(|t| t.kind == TriviaKind::Comment).expect("comment trivia");
    assert_eq!(c.value, "{# hello world #}");
}

#[test]
fn empty_comment_is_trivia() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{##}");
    assert!(tpl.statements.is_empty());
    let c = tpl.trivia.iter().find(|t| t.kind == TriviaKind::Comment).expect("comment trivia");
    assert_eq!(c.value, "{##}");
}

#[test]
fn tag_unknown_is_opaque_node() {
    let arena = Bump::new();
    let tpl = parse_ok(&arena, "{% nosuchtag foo bar %}");
    let Statement::Unknown(u) = &tpl.statements.nodes[0] else { panic!() };
    assert_eq!(u.name.value, "nosuchtag");
}

#[test]
fn tag_unknown_without_body() {
    parses("{% stopwatch 'start' %}");
}

#[test]
fn tag_if_without_endif_is_rejected() {
    crate::common::rejects("{% if a %}body");
}

#[test]
fn tag_for_without_endfor_is_rejected() {
    crate::common::rejects("{% for x in xs %}body");
}

#[test]
fn tag_macro_without_endmacro_is_rejected() {
    crate::common::rejects("{% macro m() %}body");
}

#[test]
fn tag_embed_without_endembed_is_rejected() {
    crate::common::rejects("{% embed 'x' %}body");
}

#[test]
fn tag_if_endfor_mismatch_is_rejected() {
    crate::common::rejects("{% if a %}body{% endfor %}");
}

#[test]
fn tag_block_endif_mismatch_is_rejected() {
    crate::common::rejects("{% block b %}body{% endif %}");
}

#[test]
fn combined_extends_block_block() {
    let arena = Bump::new();
    let tpl = parse(&arena, "{% extends 'base.twig' %}{% block a %}x{% endblock %}{% block b %}y{% endblock %}");
    assert!(!tpl.has_errors());
    assert!(matches!(&tpl.statements.nodes[0], Statement::Extends(_)));
    assert!(matches!(&tpl.statements.nodes[1], Statement::Block(_)));
    assert!(matches!(&tpl.statements.nodes[2], Statement::Block(_)));
}
