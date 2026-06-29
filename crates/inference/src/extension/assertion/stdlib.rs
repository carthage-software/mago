use mago_allocator::Arena;
use mago_hir::ir::argument::Argument;
use mago_hir::ir::expression::Call;
use mago_hir::ir::expression::Callee;
use mago_hir::ir::expression::CalleeKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::variable::Variable;
use mago_oracle::assertion::Assertion;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::BOOL;
use mago_oracle::ty::well_known::FLOAT;
use mago_oracle::ty::well_known::INT;
use mago_oracle::ty::well_known::NULL;
use mago_oracle::ty::well_known::STRING;
use mago_oracle::var::Var;

use crate::extension::AssertionSink;
use crate::extension::AssertionTiming;
use crate::extension::ExtensionAssertion;
use crate::extension::ExtensionContext;
use crate::flow::Flow;

/// Yields the assertions PHP's `is_*` type-predicate functions establish about a
/// variable argument: `is_string($x)` narrows `$x` to `string` when true and
/// removes `string` when false. Opt-in, so the core inference carries no
/// built-in stdlib knowledge.
///
/// This activates once call inference lands; until then no `is_*` call reaches
/// it (a call has no inferred form to extract from yet).
#[derive(Debug, Default, Clone, Copy)]
pub struct StdlibExtension;

impl<A: Arena> ExtensionAssertion<A> for StdlibExtension {
    fn assertions<'ctx, 'source, 'arena>(
        &self,
        _context: &mut ExtensionContext<'ctx, 'source, 'arena, A>,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        out: &mut AssertionSink<'ctx, 'source, 'arena, A>,
    ) {
        let ExpressionKind::Call(call) = &expression.kind else {
            return;
        };

        let Some(name) = function_short_name(&call.callee) else {
            return;
        };

        let Some(variable) = first_value_variable(call) else {
            return;
        };

        let Some(atom) = predicate_atom(name) else {
            return;
        };

        out.push(variable, Assertion::IsType(atom), AssertionTiming::WhenTrue);
        out.push(variable, Assertion::IsNotType(atom), AssertionTiming::WhenFalse);
    }
}

/// The atom a type-predicate function asserts, or `None` for an unknown name.
fn predicate_atom(name: &[u8]) -> Option<Atom<'static>> {
    if name.eq_ignore_ascii_case(b"is_string") {
        Some(STRING)
    } else if name.eq_ignore_ascii_case(b"is_int") || name.eq_ignore_ascii_case(b"is_integer") {
        Some(INT)
    } else if name.eq_ignore_ascii_case(b"is_float") || name.eq_ignore_ascii_case(b"is_double") {
        Some(FLOAT)
    } else if name.eq_ignore_ascii_case(b"is_bool") {
        Some(BOOL)
    } else if name.eq_ignore_ascii_case(b"is_null") {
        Some(NULL)
    } else {
        None
    }
}

fn function_short_name<'arena>(callee: &Callee<'arena, SymbolId, Flow, Type<'arena>>) -> Option<&'arena [u8]> {
    let CalleeKind::Function(expression) = &callee.kind else {
        return None;
    };

    match &expression.kind {
        ExpressionKind::Constant(identifier) | ExpressionKind::Identifier(identifier) => {
            Some(identifier.last_segment())
        }
        _ => None,
    }
}

fn first_value_variable<'arena>(call: &Call<'arena, SymbolId, Flow, Type<'arena>>) -> Option<Var<'arena>> {
    let Argument::Value(expression) = call.arguments.items.first()? else {
        return None;
    };

    let ExpressionKind::Variable(Variable::Direct(direct)) = &expression.kind else {
        return None;
    };

    Some(Var::new(direct.name))
}
