mod anonymous_class;
mod arrow_function;
mod closure;

use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_flags::U8Flags;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::item::expression::ItemExpression;
use mago_hir::ir::item::expression::ItemExpressionKind;
use mago_hir::ir::statement::Block;
use mago_hir::ir::statement::NamespaceBody;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::function_like::FunctionLikeSymbol;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameter;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameterFlag;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::atom::payload::callable::Parameter as CallableParameter;
use mago_oracle::ty::atom::payload::callable::ParameterFlag;
use mago_oracle::ty::atom::payload::callable::Signature;
use mago_oracle::ty::atom::payload::callable::SignatureFlag;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_span::Span;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_expression_item(
        &mut self,
        span: Span,
        item: &'source ItemExpression<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let (meta, typed_item) = match item.kind {
            ItemExpressionKind::Closure(closure) => {
                let Some(FunctionLikeSymbol::Closure(symbol)) = self.symbols.get_function_like(item.meta) else {
                    return Err(InferenceError::UnresolvedItemSymbol { span: item.span, kind: "closure" });
                };

                let (meta, node) = self.infer_closure(closure, symbol.params, symbol.ret.effective(true))?;
                let kind = ItemExpressionKind::Closure(self.arena.alloc(node));

                (meta, ItemExpression { meta: item.meta, span: item.span, kind })
            }
            ItemExpressionKind::ArrowFunction(arrow) => {
                let Some(FunctionLikeSymbol::ArrowFunction(symbol)) = self.symbols.get_function_like(item.meta) else {
                    return Err(InferenceError::UnresolvedItemSymbol { span: item.span, kind: "arrow function" });
                };

                let (meta, node) = self.infer_arrow_function(arrow, symbol.params, symbol.ret.effective(true))?;
                let kind = ItemExpressionKind::ArrowFunction(self.arena.alloc(node));

                (meta, ItemExpression { meta: item.meta, span: item.span, kind })
            }
            ItemExpressionKind::AnonymousClass(node) => {
                let (meta, node) = self.infer_anonymous_class(item.meta, node)?;
                let kind = ItemExpressionKind::AnonymousClass(self.arena.alloc(node));

                (meta, ItemExpression { meta: item.meta, span: item.span, kind })
            }
        };

        Ok(Expression { meta, span, kind: ExpressionKind::Item(self.arena.alloc(typed_item)) })
    }

    pub(crate) fn infer_returned_type(
        &mut self,
        body: &Block<'arena, SymbolId, Flow, Type<'arena>>,
        exit: ControlFlow,
    ) -> Type<'arena> {
        let mut atoms = Vec::new_in(self.source);
        let mut returns_null = collect_returned_atoms_slice(body.statements, &mut atoms);

        if matches!(exit, ControlFlow::Fallthrough) {
            returns_null = true;
        }
        if returns_null {
            atoms.push(Atom::Null);
        }

        if atoms.is_empty() { TYPE_NEVER } else { self.ty.union_of(&atoms) }
    }

    /// Builds a `Closure(...)` callable atom from a signature's parameters and an
    /// already-resolved return type. Parameter types carry their `@template`
    /// atoms through unchanged, so a call can instantiate them.
    pub(crate) fn build_callable(
        &mut self,
        params: &'arena [SignatureParameter<'arena>],
        return_type: Type<'arena>,
    ) -> Type<'arena> {
        let mut parameters = self.ty.scratch_vec::<CallableParameter<'arena>>();
        let mut variadic = false;
        for parameter in params {
            let r#type = parameter.ty.effective(true).unwrap_or(TYPE_MIXED);

            let mut flags = U8Flags::<ParameterFlag>::empty();
            if parameter.flags.contains(SignatureParameterFlag::HasDefault) {
                flags = flags.with(ParameterFlag::HasDefault);
            }
            if parameter.flags.contains(SignatureParameterFlag::ByReference) {
                flags = flags.with(ParameterFlag::ByReference);
            }
            if parameter.flags.contains(SignatureParameterFlag::Variadic) {
                flags = flags.with(ParameterFlag::Variadic);
                variadic = true;
            }

            parameters.push(CallableParameter { name: &[], r#type, flags });
        }

        let parameters = self.ty.parameters(&parameters);

        let mut flags = U8Flags::<SignatureFlag>::empty();
        if variadic {
            flags = flags.with(SignatureFlag::IsVariadic);
        }

        let signature = self.ty.signature(Signature { parameters: Some(parameters), return_type, throws: None, flags });

        self.ty.union_of(&[Atom::Callable(CallableAtom::Closure(signature))])
    }
}

fn collect_returned_atoms_slice<'arena, A>(
    statements: &[Statement<'arena, SymbolId, Flow, Type<'arena>>],
    atoms: &mut Vec<'_, Atom<'arena>, A>,
) -> bool
where
    A: Arena,
{
    let mut returns_null = false;
    for statement in statements {
        returns_null |= collect_returned_atoms(statement, atoms);
    }

    returns_null
}

/// Accumulates the value-atoms of every `return` reachable in `statement`,
/// without descending into nested function-likes (their bodies live in the
/// surrounding expression's meta). Returns `true` when a `return;` with no value
/// is found, which contributes an implicit `null`.
fn collect_returned_atoms<'arena, A>(
    statement: &Statement<'arena, SymbolId, Flow, Type<'arena>>,
    atoms: &mut Vec<'_, Atom<'arena>, A>,
) -> bool
where
    A: Arena,
{
    match statement.kind {
        StatementKind::Return(Some(value)) => {
            atoms.extend_from_slice(value.meta.atoms);
            false
        }
        StatementKind::Return(None) => true,
        StatementKind::Sequence(statements) => collect_returned_atoms_slice(statements, atoms),
        StatementKind::Block(block) => collect_returned_atoms_slice(block.statements, atoms),
        StatementKind::Namespace(namespace) => match namespace.body {
            NamespaceBody::BraceDelimited(block) => collect_returned_atoms_slice(block.statements, atoms),
            NamespaceBody::Implicit { statements, .. } => collect_returned_atoms_slice(statements, atoms),
        },
        StatementKind::If(conditional) => {
            let mut returns_null = collect_returned_atoms(conditional.then, atoms);
            if let Some(else_clause) = conditional.else_clause {
                returns_null |= collect_returned_atoms(else_clause.statement, atoms);
            }
            returns_null
        }
        _ => false,
    }
}
