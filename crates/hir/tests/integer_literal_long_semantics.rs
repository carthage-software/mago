use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::expression::AccessKind;
use mago_hir::ir::expression::CompositeStringPart;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::literal::LiteralKind;
use mago_hir::ir::statement::StatementKind;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_syntax::parser::parse_file;

fn with_ir(code: &str, check: impl FnOnce(&IR<'_, (), (), ()>)) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"t.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();

    assert!(ir.errors.is_empty(), "`{code}` lowered with errors: {:?}", ir.errors);

    check(&ir);
}

fn interpolated_offset_kind<'ir, 'arena>(ir: &'ir IR<'arena, (), (), ()>) -> &'ir ExpressionKind<'arena, (), (), ()> {
    for statement in ir.statements {
        let StatementKind::Expression(expression) = &statement.kind else {
            continue;
        };
        let ExpressionKind::CompositeString(parts) = &expression.kind else {
            continue;
        };

        for part in *parts {
            if let CompositeStringPart::Expression(part) = part
                && let ExpressionKind::Access(access) = &part.kind
                && let AccessKind::Array(_, index) = &access.kind
            {
                return &index.kind;
            }
        }
    }

    panic!("no interpolated array access found");
}

#[test]
fn simple_interpolation_int_min_offset_lowers_to_an_integer() {
    with_ir(r#"<?php "$a[-9223372036854775808]";"#, |ir| match interpolated_offset_kind(ir) {
        ExpressionKind::Literal(literal) => {
            assert!(
                matches!(literal.kind, LiteralKind::Integer(integer) if integer.value.map(|value| value as i64) == Some(i64::MIN)),
                "expected int(i64::MIN), got {:?}",
                literal.kind
            );
        }
        other => panic!("expected a single integer literal, got {other:?}"),
    });
}

#[test]
fn braced_interpolation_int_min_offset_lowers_to_a_negated_float() {
    with_ir(r#"<?php "{$a[-9223372036854775808]}";"#, |ir| match interpolated_offset_kind(ir) {
        ExpressionKind::UnaryPrefix(unary) => {
            assert!(
                matches!(unary.operand.kind, ExpressionKind::Literal(literal) if matches!(literal.kind, LiteralKind::Float(_))),
                "expected negation of a float, got {:?}",
                unary.operand.kind
            );
        }
        other => panic!("expected a negated float, got {other:?}"),
    });
}

fn assignment_rhs_is_float(code: &str) -> bool {
    let mut result = false;
    with_ir(code, |ir| {
        for statement in ir.statements {
            if let StatementKind::Expression(expression) = &statement.kind
                && let ExpressionKind::Assignment(assignment) = &expression.kind
                && let ExpressionKind::Literal(literal) = &assignment.right.kind
            {
                result = matches!(literal.kind, LiteralKind::Float(_));
                return;
            }
        }

        panic!("no assignment with a literal right-hand side found");
    });

    result
}

#[test]
fn integer_literal_overflowing_i64_lowers_to_a_float() {
    assert!(assignment_rhs_is_float("<?php $x = 9223372036854775808;"), "2^63 overflows i64, so it is a float");
}

#[test]
fn integer_literal_at_i64_max_stays_an_integer() {
    assert!(!assignment_rhs_is_float("<?php $x = 9223372036854775807;"), "i64::MAX fits, so it stays an integer");
    assert!(!assignment_rhs_is_float("<?php $x = 5;"), "a small integer stays an integer");
}

fn assignment_rhs_float_value(code: &str) -> f64 {
    let mut value = None;
    with_ir(code, |ir| {
        for statement in ir.statements {
            if let StatementKind::Expression(expression) = &statement.kind
                && let ExpressionKind::Assignment(assignment) = &expression.kind
                && let ExpressionKind::Literal(literal) = &assignment.right.kind
                && let LiteralKind::Float(float) = literal.kind
            {
                value = Some(float.value.into_inner());
                return;
            }
        }

        panic!("no assignment with a float literal right-hand side found");
    });

    value.unwrap_or(f64::NAN)
}

#[test]
fn integer_literal_past_u64_uses_the_real_magnitude_not_the_saturated_value() {
    let decimal = assignment_rhs_float_value("<?php $x = 111111111111111111111;");
    assert!((1.0e20..1.2e20).contains(&decimal), "expected ~1.11e20, got {decimal}");

    let hex = assignment_rhs_float_value("<?php $x = 0xFFFFFFFFFFFFFFFF0;");
    assert!((2.9e20..3.0e20).contains(&hex), "expected ~2.95e20, got {hex}");
}

#[test]
fn enormous_integer_literal_lowers_to_infinity() {
    let value = assignment_rhs_float_value(&format!("<?php $x = {};", "9".repeat(400)));
    assert!(value.is_infinite(), "an enormous literal overflows to infinity, got {value}");
}
