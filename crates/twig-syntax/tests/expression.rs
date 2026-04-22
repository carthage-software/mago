//! Focused tests for the expression parser.

#[path = "common/mod.rs"]
mod common;

use bumpalo::Bump;
use mago_twig_syntax::ast::BinaryOperator;
use mago_twig_syntax::ast::Expression;
use mago_twig_syntax::ast::Statement;
use mago_twig_syntax::ast::UnaryOperator;

use crate::common::parse_and_roundtrip;
use crate::common::parse_ok;
use crate::common::parses;

fn print_expr<'a>(arena: &'a Bump, src: &'a str) -> &'a Expression<'a> {
    let tpl = parse_ok(arena, src);
    let first = tpl.statements.first().expect("at least one statement");
    match first {
        Statement::Print(p) => arena.alloc(p.expression.clone()),
        _ => panic!("expected Print statement, got {first:?}"),
    }
}

fn binary_operator_name<'a>(arena: &'a Bump, src: &'a str) -> String {
    let e = print_expr(arena, src);
    match e {
        Expression::Binary(b) => b.operator.to_string(),
        other => panic!("expected Binary, got {other:?}"),
    }
}

fn unary_operator_name<'a>(arena: &'a Bump, src: &'a str) -> String {
    let e = print_expr(arena, src);
    match e {
        Expression::Unary(u) => u.operator.to_string(),
        other => panic!("expected Unary, got {other:?}"),
    }
}

fn filter_argument_count(f: &mago_twig_syntax::ast::Filter<'_>) -> usize {
    f.argument_list.as_ref().map_or(0, |list| list.arguments.len())
}

#[test]
fn literal_integer() {
    parses("{{ 42 }}");
}

#[test]
fn literal_float() {
    parses("{{ 3.14 }}");
}

#[test]
fn literal_number_with_exponent() {
    parses("{{ 1.5e10 }}");
}

#[test]
fn literal_number_with_underscores() {
    parses("{{ 1_000_000 }}");
}

#[test]
fn literal_single_quoted_string() {
    parses(r#"{{ 'hello' }}"#);
}

#[test]
fn literal_double_quoted_string() {
    parses(r#"{{ "hello" }}"#);
}

#[test]
fn literal_true_false_null() {
    parses("{{ true }}");
    parses("{{ false }}");
    parses("{{ TRUE }}");
    parses("{{ FALSE }}");
    parses("{{ null }}");
    parses("{{ NULL }}");
    parses("{{ none }}");
    parses("{{ NONE }}");
}

#[test]
fn literal_name() {
    parses("{{ foo }}");
    parses("{{ _underscore }}");
    parses("{{ with_digits_0 }}");
}

#[test]
fn adjacent_strings_are_concatenated() {
    let arena = Bump::new();
    let e = print_expr(&arena, r#"{{ 'a' 'b' }}"#);
    match e {
        Expression::Binary(b) => assert!(matches!(b.operator, BinaryOperator::StringConcat(_))),
        _ => panic!("expected implicit concat, got {e:?}"),
    }
}

#[test]
fn interpolated_string_plain() {
    parse_and_roundtrip(r#"{{ "hello #{name} world" }}"#);
}

#[test]
fn interpolated_string_with_expression() {
    parse_and_roundtrip(r#"{{ "n = #{ a + b }" }}"#);
}

#[test]
fn interpolated_string_empty_parts() {
    parse_and_roundtrip(r##"{{ "#{a}" }}"##);
}

#[test]
fn interpolated_string_nested() {
    parse_and_roundtrip(r#"{{ "foo #{"bar #{baz} qux"} quux" }}"#);
}

#[test]
fn unary_neg() {
    let arena = Bump::new();
    assert_eq!(unary_operator_name(&arena, "{{ -a }}"), "MinusSign");
}

#[test]
fn unary_pos() {
    let arena = Bump::new();
    assert_eq!(unary_operator_name(&arena, "{{ +a }}"), "PlusSign");
}

#[test]
fn unary_not() {
    let arena = Bump::new();
    let e = print_expr(&arena, "{{ not a }}");
    match e {
        Expression::Unary(u) => assert!(matches!(u.operator, UnaryOperator::Not(_))),
        other => panic!("{other:?}"),
    }
}

#[test]
fn unary_neg_of_number_literal() {
    parses("{{ -1 }}");
    parses("{{ -3.14 }}");
}

#[test]
fn unary_double_not() {
    parses("{{ not not a }}");
}

#[test]
fn binary_add() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 1 + 2 }}"), "Addition");
}

#[test]
fn binary_sub() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 1 - 2 }}"), "Subtraction");
}

#[test]
fn binary_mul() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 1 * 2 }}"), "Multiplication");
}

#[test]
fn binary_div() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 1 / 2 }}"), "Division");
}

#[test]
fn binary_floor_div() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 1 // 2 }}"), "FloorDivision");
}

#[test]
fn binary_mod() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 1 % 2 }}"), "Modulo");
}

#[test]
fn binary_pow() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 1 ** 2 }}"), "Exponentiation");
}

#[test]
fn binary_concat() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 'a' ~ 'b' }}"), "StringConcat");
}

#[test]
fn binary_range() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 1..10 }}"), "Range");
}

#[test]
fn binary_eq_neq() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ a == b }}"), "Equal");
    assert_eq!(binary_operator_name(&a, "{{ a != b }}"), "NotEqual");
}

#[test]
fn binary_comparisons() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ a < b }}"), "LessThan");
    assert_eq!(binary_operator_name(&a, "{{ a > b }}"), "GreaterThan");
    assert_eq!(binary_operator_name(&a, "{{ a <= b }}"), "LessThanOrEqual");
    assert_eq!(binary_operator_name(&a, "{{ a >= b }}"), "GreaterThanOrEqual");
    assert_eq!(binary_operator_name(&a, "{{ a <=> b }}"), "Spaceship");
    assert_eq!(binary_operator_name(&a, "{{ a === b }}"), "Identical");
    assert_eq!(binary_operator_name(&a, "{{ a !== b }}"), "NotIdentical");
}

#[test]
fn binary_logical_and_or_xor() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ a and b }}"), "And");
    assert_eq!(binary_operator_name(&a, "{{ a or b }}"), "Or");
    assert_eq!(binary_operator_name(&a, "{{ a xor b }}"), "Xor");
}

#[test]
fn binary_bitwise() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ a b-and b }}"), "BitwiseAnd");
    assert_eq!(binary_operator_name(&a, "{{ a b-or b }}"), "BitwiseOr");
    assert_eq!(binary_operator_name(&a, "{{ a b-xor b }}"), "BitwiseXor");
}

#[test]
fn binary_null_coalesce() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ a ?? b }}"), "NullCoalesce");
}

#[test]
fn binary_in_not_in() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ a in xs }}"), "In");
    assert_eq!(binary_operator_name(&a, "{{ a not in xs }}"), "NotIn");
}

#[test]
fn binary_matches() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, r#"{{ 'x' matches '/x/' }}"#), "Matches");
}

#[test]
fn binary_starts_ends_with() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ 'foo' starts with 'f' }}"), "StartsWith");
    assert_eq!(binary_operator_name(&a, "{{ 'foo' ends with 'o' }}"), "EndsWith");
}

#[test]
fn binary_has_some_every() {
    let a = Bump::new();
    assert_eq!(binary_operator_name(&a, "{{ xs has some [1, 2] }}"), "HasSome");
    assert_eq!(binary_operator_name(&a, "{{ xs has every [1, 2] }}"), "HasEvery");
}

#[test]
fn precedence_mul_binds_tighter_than_add() {
    let arena = Bump::new();
    let e = print_expr(&arena, "{{ 1 + 2 * 3 }}");
    match e {
        Expression::Binary(b) => {
            assert!(matches!(b.operator, BinaryOperator::Addition(_)));
            match b.rhs {
                Expression::Binary(inner) => assert!(matches!(inner.operator, BinaryOperator::Multiplication(_))),
                other => panic!("{other:?}"),
            }
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn precedence_parens_override() {
    let arena = Bump::new();
    let e = print_expr(&arena, "{{ (1 + 2) * 3 }}");
    match e {
        Expression::Binary(b) => {
            assert!(matches!(b.operator, BinaryOperator::Multiplication(_)));
            assert!(matches!(b.lhs, Expression::Parenthesized(_)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn precedence_pow_is_right_associative() {
    let arena = Bump::new();
    let e = print_expr(&arena, "{{ a ** b ** c }}");
    match e {
        Expression::Binary(b) => {
            assert!(matches!(b.operator, BinaryOperator::Exponentiation(_)));
            assert!(matches!(b.lhs, Expression::Name(_)));
            match b.rhs {
                Expression::Binary(inner) => {
                    assert!(matches!(inner.operator, BinaryOperator::Exponentiation(_)))
                }
                other => panic!("{other:?}"),
            }
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn precedence_subtraction_is_left_associative() {
    let arena = Bump::new();
    let e = print_expr(&arena, "{{ a - b - c }}");
    match e {
        Expression::Binary(b) => {
            assert!(matches!(b.operator, BinaryOperator::Subtraction(_)));
            match b.lhs {
                Expression::Binary(inner) => {
                    assert!(matches!(inner.operator, BinaryOperator::Subtraction(_)))
                }
                other => panic!("{other:?}"),
            }
            assert!(matches!(b.rhs, Expression::Name(_)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn precedence_and_binds_tighter_than_or() {
    let arena = Bump::new();
    let e = print_expr(&arena, "{{ a or b and c }}");
    match e {
        Expression::Binary(b) => {
            assert!(matches!(b.operator, BinaryOperator::Or(_)));
            match b.rhs {
                Expression::Binary(inner) => assert!(matches!(inner.operator, BinaryOperator::And(_))),
                other => panic!("{other:?}"),
            }
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn precedence_not_binds_tighter_than_eq() {
    let arena = Bump::new();
    let e = print_expr(&arena, "{{ not a == b }}");
    match e {
        Expression::Binary(b) => {
            assert!(matches!(b.operator, BinaryOperator::Equal(_)));
            assert!(matches!(b.lhs, Expression::Unary(_)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn precedence_null_coalesce_is_right_associative() {
    let arena = Bump::new();
    let e = print_expr(&arena, "{{ a ?? b ?? c }}");
    match e {
        Expression::Binary(b) => {
            assert!(matches!(b.operator, BinaryOperator::NullCoalesce(_)));
            match b.rhs {
                Expression::Binary(inner) => assert!(matches!(inner.operator, BinaryOperator::NullCoalesce(_))),
                other => panic!("{other:?}"),
            }
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn precedence_nested_conditional() {
    parses("{{ a ? b : c ? d : e }}");
}

#[test]
fn precedence_is_not_null() {
    let arena = Bump::new();
    let e = print_expr(&arena, "{{ a is not null }}");
    match e {
        Expression::Test(t) => {
            assert!(t.not_keyword.is_some());
            assert_eq!(t.name.value, "null");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn precedence_range_mixes_with_arithmetic() {
    parses("{{ 1 + 2..10 - 1 }}");
}

#[test]
fn precedence_concat_with_arithmetic() {
    parses("{{ 'a' ~ 1 + 2 }}");
}

#[test]
fn conditional_full() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ a ? b : c }}") {
        Expression::Conditional(c) => {
            assert!(c.then.is_some());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn conditional_with_elvis() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ a ?: b }}") {
        Expression::Conditional(c) => {
            assert!(c.then.is_none());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn group_expression() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ (a) }}") {
        Expression::Parenthesized(_) => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn rejects_empty_parens() {
    crate::common::rejects("{{ () }}");
}

#[test]
fn attribute_single() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ a.b }}") {
        Expression::GetAttribute(_) => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn attribute_nested_chain() {
    parses("{{ a.b.c.d.e }}");
}

#[test]
fn attribute_null_safe() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ a?.b }}") {
        Expression::GetAttribute(g) => assert!(g.null_safe),
        other => panic!("{other:?}"),
    }
}

#[test]
fn method_call_no_args() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ a.b() }}") {
        Expression::MethodCall(m) => assert_eq!(m.method.value, "b"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn method_call_with_args() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ a.b(1, 2) }}") {
        Expression::MethodCall(m) => {
            assert_eq!(m.method.value, "b");
            assert_eq!(m.argument_list.arguments.len(), 2);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn chained_method_calls() {
    parses("{{ a.b().c().d() }}");
}

#[test]
fn get_item_string_key() {
    let arena = Bump::new();
    match print_expr(&arena, r#"{{ a["key"] }}"#) {
        Expression::GetItem(_) => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn get_item_integer_index() {
    parses("{{ a[0] }}");
}

#[test]
fn get_item_name_index() {
    parses("{{ a[name] }}");
}

#[test]
fn get_item_expression_index() {
    parses("{{ a[i + 1] }}");
}

#[test]
fn slice_start_stop() {
    parses("{{ a[1:3] }}");
}

#[test]
fn slice_from_start() {
    parses("{{ a[:3] }}");
}

#[test]
fn slice_to_end() {
    parses("{{ a[1:] }}");
}

#[test]
fn array_empty() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ [] }}") {
        Expression::Array(a) => assert_eq!(a.elements.len(), 0),
        other => panic!("{other:?}"),
    }
}

#[test]
fn array_single_element() {
    parses("{{ [1] }}");
}

#[test]
fn array_multiple() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ [1, 2, 3] }}") {
        Expression::Array(a) => assert_eq!(a.elements.len(), 3),
        other => panic!("{other:?}"),
    }
}

#[test]
fn array_trailing_comma() {
    parses("{{ [1, 2, 3,] }}");
}

#[test]
fn array_with_spread() {
    parses("{{ [1, ...others, 3] }}");
}

#[test]
fn hash_empty() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ {} }}") {
        Expression::HashMap(h) => assert_eq!(h.entries.len(), 0),
        other => panic!("{other:?}"),
    }
}

#[test]
fn hash_single_identifier_key() {
    parses("{{ { a: 1 } }}");
}

#[test]
fn hash_string_key() {
    parses(r#"{{ { "a": 1 } }}"#);
}

#[test]
fn hash_computed_key_in_parens() {
    parses("{{ { (k): v } }}");
}

#[test]
fn hash_shorthand_entry() {
    parses("{{ { a, b } }}");
}

#[test]
fn hash_with_spread() {
    parses("{{ { a: 1, ...other } }}");
}

#[test]
fn hash_number_key() {
    parses("{{ { 1: 'one', 2: 'two' } }}");
}

#[test]
fn filter_without_args() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ x|upper }}") {
        Expression::Filter(f) => {
            assert_eq!(f.name.value, "upper");
            assert_eq!(filter_argument_count(f), 0);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn filter_with_positional_arg() {
    let arena = Bump::new();
    match print_expr(&arena, r#"{{ x|default("fallback") }}"#) {
        Expression::Filter(f) => {
            assert_eq!(f.name.value, "default");
            assert_eq!(filter_argument_count(f), 1);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn filter_with_hash_arg() {
    parses(r#"{{ x|replace({'a': 'b'}) }}"#);
}

#[test]
fn filter_chained() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ x|upper|title }}") {
        Expression::Filter(outer) => {
            assert_eq!(outer.name.value, "title");
            match outer.operand {
                Expression::Filter(inner) => assert_eq!(inner.name.value, "upper"),
                other => panic!("{other:?}"),
            }
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn filter_with_named_arg() {
    parses(r#"{{ x|slice(start=0, length=3) }}"#);
}

#[test]
fn test_is_null() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ x is null }}") {
        Expression::Test(t) => {
            assert!(t.not_keyword.is_none());
            assert_eq!(t.name.value, "null");
        }
        _ => panic!(),
    }
}

#[test]
fn test_is_not_null() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ x is not null }}") {
        Expression::Test(t) => {
            assert!(t.not_keyword.is_some());
            assert_eq!(t.name.value, "null");
        }
        _ => panic!(),
    }
}

#[test]
fn test_is_divisible_by() {
    parses("{{ x is divisible by(3) }}");
}

#[test]
fn test_is_odd() {
    parses("{{ x is odd }}");
}

#[test]
fn test_is_even() {
    parses("{{ x is even }}");
}

#[test]
fn test_is_defined() {
    parses("{{ x is defined }}");
}

#[test]
fn test_is_iterable() {
    parses("{{ x is iterable }}");
}

#[test]
fn test_is_same_as() {
    parses("{{ x is same as(y) }}");
}

#[test]
fn test_is_not_odd() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ x is not odd }}") {
        Expression::Test(t) => {
            assert!(t.not_keyword.is_some());
            assert_eq!(t.name.value, "odd");
        }
        _ => panic!(),
    }
}

#[test]
fn arrow_single_param_no_parens() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ v => v * 2 }}") {
        Expression::ArrowFunction(a) => {
            assert_eq!(a.parameters.len(), 1);
            assert_eq!(a.parameters.as_slice()[0].value, "v");
        }
        _ => panic!(),
    }
}

#[test]
fn arrow_multi_param_in_parens() {
    let arena = Bump::new();
    match print_expr(&arena, "{{ (a, b) => a + b }}") {
        Expression::ArrowFunction(a) => {
            assert_eq!(a.parameters.len(), 2);
            assert_eq!(a.parameters.as_slice()[0].value, "a");
            assert_eq!(a.parameters.as_slice()[1].value, "b");
        }
        _ => panic!(),
    }
}

#[test]
fn arrow_no_params() {
    parses("{{ () => 42 }}");
}

#[test]
fn arrow_used_in_filter() {
    parses("{{ list|map(x => x * 2) }}");
}

#[test]
fn arrow_nested_body() {
    parses("{{ list|filter(x => x.enabled) }}");
}

#[test]
fn call_no_args() {
    parses("{{ f() }}");
}

#[test]
fn call_positional_args() {
    parses("{{ f(1, 2, 3) }}");
}

#[test]
fn call_named_arg_equal_form() {
    parses("{{ f(a=1, b=2) }}");
}

#[test]
fn call_named_arg_colon_form() {
    parses("{{ f(a: 1, b: 2) }}");
}

#[test]
fn call_mixed_positional_and_named() {
    parses("{{ f(1, b=2) }}");
}

#[test]
fn call_with_spread() {
    parses("{{ f(...xs) }}");
}

#[test]
fn spread_in_array_literal() {
    parses("{{ [1, ...xs, 3] }}");
}

#[test]
fn spread_in_hash_literal() {
    parses("{{ { a: 1, ...other } }}");
}

#[test]
fn spread_in_call() {
    parses("{{ f(...xs) }}");
}

#[test]
fn expression_chain_filter_after_call() {
    parses("{{ f(1, 2)|upper }}");
}

#[test]
fn expression_method_then_filter() {
    parses("{{ a.b()|default('x') }}");
}

#[test]
fn expression_get_item_then_filter() {
    parses("{{ a[0]|first }}");
}

#[test]
fn expression_conditional_inside_filter_args() {
    parses(r#"{{ x|default(a ? 'y' : 'n') }}"#);
}

#[test]
fn expression_not_in_chain() {
    parses("{{ not (a in xs) }}");
}

#[test]
fn expression_complex_boolean() {
    parses("{{ (a or b) and not (c or d) }}");
}

#[test]
fn expression_chained_null_coalesce() {
    parses("{{ a.b ?? c.d ?? 'fallback' }}");
}

#[test]
fn expression_deep_method_chain() {
    parses("{{ obj.one().two().three()[0].four() }}");
}

#[test]
fn expression_filter_inside_hash_value() {
    parses(r#"{{ { a: x|upper } }}"#);
}

#[test]
fn expression_array_with_complex_elements() {
    parses("{{ [a.b, c|d, e ?? f] }}");
}

#[test]
fn expression_string_concat_chain() {
    parses(r#"{{ 'a' ~ 'b' ~ 'c' ~ 'd' }}"#);
}
