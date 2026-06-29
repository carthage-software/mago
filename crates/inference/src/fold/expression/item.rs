use std::marker::PhantomData;

use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_hir::fold::Fold;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::item::expression::ItemExpression;
use mago_hir::ir::item::expression::ItemExpressionKind;
use mago_hir::ir::item::expression::arrow_function::ArrowFunction;
use mago_hir::ir::item::expression::closure::Closure;
use mago_hir::ir::item::member::MemberItemKind;
use mago_hir::ir::item::parameter::Parameter;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::function_like::FunctionLikeSymbol;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameter;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameterFlag;
use mago_oracle::symbol::part::ty::TypeSlot;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::atom::payload::callable::Parameter as CallableParameter;
use mago_oracle::ty::atom::payload::callable::ParameterFlag;
use mago_oracle::ty::atom::payload::callable::Signature;
use mago_oracle::ty::atom::payload::callable::SignatureFlag;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    /// Folds a closure / arrow-function / anonymous-class literal expression. A
    /// closure/arrow's type is the callable signature of its linked symbol, and
    /// its body is inferred for real in a fresh scope with the parameters (and a
    /// closure's `use` captures) bound. Anonymous classes are not yet typed.
    pub fn infer_expression_item(
        &mut self,
        span: Span,
        item: &'source ItemExpression<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let (meta, typed_item) = match item.kind {
            ItemExpressionKind::Closure(closure) => {
                if let Some(FunctionLikeSymbol::Closure(symbol)) = self.symbols.get_function_like(item.meta) {
                    let meta = self.build_callable(symbol.params, symbol.ret);
                    let node = self.infer_closure(closure, symbol.params);
                    let kind = ItemExpressionKind::Closure(self.arena.alloc(node));

                    (meta, ItemExpression { meta: item.meta, span: item.span, kind })
                } else {
                    (TYPE_MIXED, self.fold_item_structurally(item))
                }
            }
            ItemExpressionKind::ArrowFunction(arrow) => {
                if let Some(FunctionLikeSymbol::ArrowFunction(symbol)) = self.symbols.get_function_like(item.meta) {
                    let meta = self.build_callable(symbol.params, symbol.ret);
                    let node = self.infer_arrow_function(arrow, symbol.params);
                    let kind = ItemExpressionKind::ArrowFunction(self.arena.alloc(node));

                    (meta, ItemExpression { meta: item.meta, span: item.span, kind })
                } else {
                    (TYPE_MIXED, self.fold_item_structurally(item))
                }
            }
            ItemExpressionKind::AnonymousClass(_) => (TYPE_MIXED, self.fold_item_structurally(item)),
        };

        Expression { meta, span, kind: ExpressionKind::Item(self.arena.alloc(typed_item)) }
    }

    /// Folds a closure node: the parameters, attributes, and annotation are
    /// folded structurally; the body is inferred in a fresh scope with the `use`
    /// captures and the parameters bound (closures do not see other locals).
    fn infer_closure(
        &mut self,
        closure: &'source Closure<'source, SymbolId, S, E>,
        signature: &'arena [SignatureParameter<'arena>],
    ) -> Closure<'arena, SymbolId, Flow, Type<'arena>> {
        let structural = StructuralFold::<'arena, A, S, E>::new(self.arena).fold_closure(closure);

        let outer = std::mem::replace(&mut self.environment, Environment::new_in(self.source));
        let reachable = self.reachable;
        self.reachable = true;

        if let Some(use_variables) = closure.use_variables {
            for use_variable in use_variables.items {
                let name = Var::new(self.arena.alloc_slice_copy(use_variable.variable.name));
                let captured = outer.get(name);
                self.environment.set(name, captured);
            }
        }
        self.bind_signature_parameters(closure.parameters.items, signature);

        let body = self.infer_statement(closure.body);

        self.environment = outer;
        self.reachable = reachable;

        Closure { body: self.arena.alloc(body), ..structural }
    }

    /// Folds an arrow-function node. Arrow functions capture the enclosing scope
    /// by value automatically, so the body is inferred against the current
    /// environment with the parameters bound on top, then restored.
    fn infer_arrow_function(
        &mut self,
        arrow: &'source ArrowFunction<'source, SymbolId, S, E>,
        signature: &'arena [SignatureParameter<'arena>],
    ) -> ArrowFunction<'arena, SymbolId, Flow, Type<'arena>> {
        let structural = StructuralFold::<'arena, A, S, E>::new(self.arena).fold_arrow_function(arrow);

        let outer = self.environment.clone();
        let reachable = self.reachable;
        self.bind_signature_parameters(arrow.parameters.items, signature);

        let expression = self.infer_expression(arrow.expression);

        self.environment = outer;
        self.reachable = reachable;

        ArrowFunction { expression: self.arena.alloc(expression), ..structural }
    }

    /// Binds each parameter into the environment under its declared signature
    /// type (the native hint or `@param`), defaulting to `mixed`.
    fn bind_signature_parameters(
        &mut self,
        parameters: &[Parameter<'source, SymbolId, S, E>],
        signature: &[SignatureParameter<'arena>],
    ) {
        for (index, parameter) in parameters.iter().enumerate() {
            let ty = signature.get(index).and_then(|parameter| parameter.ty.effective(true)).unwrap_or(TYPE_MIXED);
            let name = Var::new(self.arena.alloc_slice_copy(parameter.variable.name));
            self.environment.set(name, ty);
        }
    }

    /// Builds a `Closure(...)` callable atom from a signature's parameters and
    /// return type. Parameter and return types carry their `@template` atoms
    /// through unchanged, so a call can instantiate them.
    fn build_callable(&mut self, params: &'arena [SignatureParameter<'arena>], ret: TypeSlot<'arena>) -> Type<'arena> {
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
        let return_type = ret.effective(true).unwrap_or(TYPE_MIXED);

        let mut flags = U8Flags::<SignatureFlag>::empty();
        if variadic {
            flags = flags.with(SignatureFlag::IsVariadic);
        }

        let signature = self.ty.signature(Signature { parameters: Some(parameters), return_type, throws: None, flags });

        self.ty.union_of(&[Atom::Callable(CallableAtom::Closure(signature))])
    }

    /// Materializes an item-expression node we do not yet infer (anonymous
    /// classes, or a closure/arrow whose symbol is missing) without inference.
    fn fold_item_structurally(
        &self,
        item: &'source ItemExpression<'source, SymbolId, S, E>,
    ) -> ItemExpression<'arena, SymbolId, Flow, Type<'arena>> {
        StructuralFold::<'arena, A, S, E>::new(self.arena).fold_item_expression(item)
    }
}

/// Re-folds a definition subtree (parameter list, annotations, attributes) from
/// the source metas to inference metas without inferring it: expressions become
/// `mixed`, statements fall through, item/member metas re-derive their id. Used
/// for the structural parts of a closure/arrow node (its body is inferred and
/// spliced in separately) and for not-yet-inferred items.
struct StructuralFold<'arena, A: Arena, S, E> {
    arena: &'arena A,
    _phantom: PhantomData<(S, E)>,
}

impl<'arena, A: Arena, S, E> StructuralFold<'arena, A, S, E> {
    fn new(arena: &'arena A) -> Self {
        Self { arena, _phantom: PhantomData }
    }
}

impl<'arena, A, S, E> Fold<'_, 'arena, A> for StructuralFold<'arena, A, S, E>
where
    A: Arena + 'arena,
{
    type FromItem = SymbolId;
    type FromStatement = S;
    type FromExpression = E;
    type ToItem = SymbolId;
    type ToStatement = Flow;
    type ToExpression = Type<'arena>;

    fn arena(&self) -> &'arena A {
        self.arena
    }

    fn fold_statement_meta(&self, _: Span, _: &StatementKind<'arena, SymbolId, Flow, Type<'arena>>) -> Flow {
        Flow { reachable: true, exit: ControlFlow::Fallthrough }
    }

    fn fold_expression_meta(&self, _: Span, _: &ExpressionKind<'arena, SymbolId, Flow, Type<'arena>>) -> Type<'arena> {
        TYPE_MIXED
    }

    fn fold_item_statement_meta(
        &self,
        _: Span,
        kind: &ItemStatementKind<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> SymbolId {
        match kind {
            ItemStatementKind::Class(node) => SymbolId::class_like(node.name.value),
            ItemStatementKind::Interface(node) => SymbolId::class_like(node.name.value),
            ItemStatementKind::Trait(node) => SymbolId::class_like(node.name.value),
            ItemStatementKind::Enum(node) => SymbolId::class_like(node.name.value),
            ItemStatementKind::Constant(node) => SymbolId::constant(node.name.value),
            ItemStatementKind::Function(node) => SymbolId::function_like(node.name.value),
        }
    }

    fn fold_item_expression_meta(
        &self,
        _: Span,
        kind: &ItemExpressionKind<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> SymbolId {
        match kind {
            ItemExpressionKind::AnonymousClass(node) => SymbolId::class_like(node.name),
            ItemExpressionKind::ArrowFunction(node) => SymbolId::function_like(node.name),
            ItemExpressionKind::Closure(node) => SymbolId::function_like(node.name),
        }
    }

    fn fold_member_item_meta(&self, _: Span, _: &MemberItemKind<'arena, SymbolId, Flow, Type<'arena>>) -> SymbolId {
        // SAFETY: This should never be called because we do not fold member items structurally.
        unsafe { std::hint::unreachable_unchecked() }
    }
}
