//! `intdiv()` divide-by-zero detector.
//!
//! `intdiv($num, $divisor)` throws `DivisionByZeroError` at runtime when
//! `$divisor` is zero. Mago can detect this statically when the divisor's
//! type is a literal `0` or an integer range that includes only zero.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Argument;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;

use crate::code::IssueCode;
use crate::plugin::context::HookContext;
use crate::plugin::hook::FunctionCallHook;
use crate::plugin::hook::HookResult;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;

#[derive(Default)]
pub struct IntdivHook;

impl Provider for IntdivHook {
    fn meta() -> &'static ProviderMeta {
        static META: ProviderMeta =
            ProviderMeta::new("php::math::intdiv", "intdiv", "Detects intdiv() calls with a statically zero divisor.");

        &META
    }
}

impl FunctionCallHook for IntdivHook {
    fn after_function_call(&self, call: &FunctionCall<'_>, context: &mut HookContext<'_, '_>) -> HookResult<()> {
        let Expression::Identifier(identifier) = call.function else {
            return Ok(());
        };

        if !identifier.value().eq_ignore_ascii_case("intdiv") {
            return Ok(());
        }

        let Some(divisor_expr) = lookup_divisor_argument(call) else {
            return Ok(());
        };

        let Some(divisor_type) = context.get_expression_type(divisor_expr) else {
            return Ok(());
        };

        if !is_definitely_zero(divisor_type) {
            return Ok(());
        }

        context.report(
            IssueCode::InvalidOperand,
            Issue::error("Call to `intdiv()` with a zero divisor.")
                .with_annotation(Annotation::primary(divisor_expr.span()).with_message("This divisor is zero"))
                .with_annotation(Annotation::secondary(call.function.span()).with_message("In this `intdiv()` call"))
                .with_note("`intdiv($num, 0)` throws `DivisionByZeroError` at runtime.")
                .with_help("Guard the call with `$divisor !== 0` or restrict the divisor's type to exclude zero."),
        );

        Ok(())
    }
}

fn lookup_divisor_argument<'arena>(call: &FunctionCall<'arena>) -> Option<&'arena Expression<'arena>> {
    let mut seen_positional = 0;
    for argument in call.argument_list.arguments.iter() {
        match argument {
            Argument::Positional(arg) => {
                seen_positional += 1;
                if seen_positional == 2 {
                    return Some(arg.value);
                }
            }
            Argument::Named(arg) if arg.name.value == "divisor" => {
                return Some(arg.value);
            }
            Argument::Named(_) => {}
        }
    }

    None
}

fn is_definitely_zero(ty: &TUnion) -> bool {
    if ty.types.is_empty() {
        return false;
    }

    ty.types.iter().all(|atomic| match atomic {
        TAtomic::Scalar(TScalar::Integer(integer)) => integer.is_zero(),
        _ => false,
    })
}
