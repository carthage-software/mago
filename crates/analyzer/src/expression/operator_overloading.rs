use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
use mago_codex::operator_overloading::OverloadedOperator;
use mago_codex::operator_overloading::get_class_overloading;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::code::IssueCode;
use crate::context::Context;

pub fn report_overloaded_operator_on_operand<A>(
    context: &mut Context<'_, '_, A>,
    operation_span: Span,
    operator: &[u8],
    operand_type: &TUnion,
    overloaded_operator: OverloadedOperator,
) where
    A: Arena,
{
    if context.settings.allow_operator_overloading {
        return;
    }

    let mut class_names: Vec<&'static str> = Vec::new();
    let mut is_always_overloaded = true;
    for atomic in operand_type.types.iter() {
        match get_class_overloading(atomic, overloaded_operator) {
            Some(class) if !class_names.contains(&class.get_name()) => class_names.push(class.get_name()),
            Some(_) => {}
            None => is_always_overloaded = false,
        }
    }

    report_disabled_operator_overloading(context, operation_span, operator, &class_names, is_always_overloaded);
}

pub fn report_disabled_operator_overloading<A>(
    context: &mut Context<'_, '_, A>,
    operation_span: Span,
    operator: &[u8],
    class_names: &[&str],
    is_always_overloaded: bool,
) where
    A: Arena,
{
    if context.settings.allow_operator_overloading || class_names.is_empty() {
        return;
    }

    let operator = BytesDisplay(operator);
    let (issue_code, issue) = if is_always_overloaded {
        (
            IssueCode::InvalidOperand,
            Issue::error(format!("Invalid operand for `{operator}`: operator overloading is disabled.")),
        )
    } else {
        (
            IssueCode::PossiblyInvalidOperand,
            Issue::warning(format!("Possibly invalid operand for `{operator}`: operator overloading is disabled.")),
        )
    };

    context.collector.report_with_code(
        issue_code,
        issue
            .with_annotation(
                Annotation::primary(operation_span)
                    .with_message(format!("`{}` overloads `{operator}` here", class_names.join("|"))),
            )
            .with_note(format!(
                "PHP runs `{operator}` through the class instead of applying it to a number, which the expression does not show."
            ))
            .with_help(
                "Call the class's methods or functions instead, such as `BcMath\\Number::add()` or `gmp_add()`, or enable `allow-operator-overloading` in the analyzer settings.",
            ),
    );
}
