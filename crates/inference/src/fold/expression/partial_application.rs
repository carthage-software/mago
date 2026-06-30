use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::argument::PartialArgument;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::Callee;
use mago_hir::ir::expression::CalleeKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::PartialApplication;
use mago_hir::ir::expression::selector::MemberSelector;
use mago_hir::ir::expression::selector::MemberSelectorKind;
use mago_hir::ir::identifier::Identifier;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::Symbol;
use mago_oracle::symbol::class_like::ClassLikeSymbol;
use mago_oracle::symbol::function_like::FunctionLikeSymbol;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::fold::expression::call::object_class;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_partial_application(
        &mut self,
        span: Span,
        partial: &'source PartialApplication<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let arguments = self.infer_partial_arguments(&partial.arguments)?;

        let (kind, meta) = match &partial.callee.kind {
            CalleeKind::Function(callee) => match &callee.kind {
                ExpressionKind::Identifier(identifier) => {
                    let meta = self.function_callable(identifier);
                    let callee = Expression {
                        meta: TYPE_MIXED,
                        span: callee.span,
                        kind: ExpressionKind::Identifier(identifier.copy_into(self.arena)),
                    };

                    (CalleeKind::Function(self.arena.alloc(callee)), meta)
                }
                _ => {
                    let callee = self.infer_expression(callee)?;
                    let meta = if callee.meta.atoms.iter().any(|atom| matches!(atom, Atom::Callable(_))) {
                        callee.meta
                    } else {
                        TYPE_MIXED
                    };

                    (CalleeKind::Function(self.arena.alloc(callee)), meta)
                }
            },
            CalleeKind::Method(object, selector) => {
                let object = self.infer_expression(object)?;
                let meta = self.method_callable(object.meta, selector);
                let selector = self.infer_member_selector(selector)?;

                (CalleeKind::Method(self.arena.alloc(object), selector), meta)
            }
            CalleeKind::NullsafeMethod(object, selector) => {
                let object = self.infer_expression(object)?;
                let meta = self.method_callable(object.meta, selector);
                let selector = self.infer_member_selector(selector)?;

                (CalleeKind::NullsafeMethod(self.arena.alloc(object), selector), meta)
            }
            CalleeKind::StaticMethod(class, selector) => {
                let meta = self.static_method_callable(class, selector);
                let class = self.infer_expression(class)?;
                let selector = self.infer_member_selector(selector)?;

                (CalleeKind::StaticMethod(self.arena.alloc(class), selector), meta)
            }
        };

        let callee = Callee { span: partial.callee.span, kind };
        let node = PartialApplication { span: partial.span, callee, arguments };

        Ok(Expression { meta, span, kind: ExpressionKind::PartialApplication(self.arena.alloc(node)) })
    }

    pub(crate) fn infer_partial_arguments(
        &mut self,
        arguments: &'source Delimited<'source, PartialArgument<'source, SymbolId, S, E>>,
    ) -> InferenceResult<Delimited<'arena, PartialArgument<'arena, SymbolId, Flow, Type<'arena>>>> {
        let mut items = Vec::new_in(self.arena);
        for argument in arguments.items {
            let typed = match argument {
                PartialArgument::Value(value) => {
                    PartialArgument::Value(self.arena.alloc(self.infer_expression(value)?))
                }
                PartialArgument::Variadic(value) => {
                    PartialArgument::Variadic(self.arena.alloc(self.infer_expression(value)?))
                }
                PartialArgument::Named(name, value) => {
                    PartialArgument::Named(name.copy_into(self.arena), self.arena.alloc(self.infer_expression(value)?))
                }
                PartialArgument::Placeholder(span) => PartialArgument::Placeholder(*span),
                PartialArgument::NamedPlaceholder(name) => {
                    PartialArgument::NamedPlaceholder(name.copy_into(self.arena))
                }
                PartialArgument::VariadicPlaceholder(span) => PartialArgument::VariadicPlaceholder(*span),
            };

            items.push(typed);
        }

        Ok(Delimited { span: arguments.span, items: items.leak() })
    }

    fn function_callable(&mut self, identifier: &Identifier<'source>) -> Type<'arena> {
        let Some(FunctionLikeSymbol::Function(function)) = self.resolve_function_symbol(identifier) else {
            return TYPE_MIXED;
        };

        let return_type = function.ret.effective(true).unwrap_or(TYPE_MIXED);
        self.build_callable(function.params, return_type)
    }

    fn method_callable(
        &mut self,
        receiver: Type<'arena>,
        selector: &MemberSelector<'source, SymbolId, S, E>,
    ) -> Type<'arena> {
        let MemberSelectorKind::Name(name) = &selector.kind else {
            return TYPE_MIXED;
        };

        let mut classes = receiver.atoms.iter().filter_map(object_class);
        let Some(class_name) = classes.next() else {
            return TYPE_MIXED;
        };
        if classes.next().is_some() {
            return TYPE_MIXED;
        }

        let Some(symbol) = self.symbols.get_class_like(SymbolId::class_like(class_name)) else {
            return TYPE_MIXED;
        };

        self.method_signature_callable(&symbol, name.value)
    }

    fn static_method_callable(
        &mut self,
        class: &'source Expression<'source, SymbolId, S, E>,
        selector: &MemberSelector<'source, SymbolId, S, E>,
    ) -> Type<'arena> {
        let MemberSelectorKind::Name(name) = &selector.kind else {
            return TYPE_MIXED;
        };
        let Some(symbol) = self.resolve_class(class) else {
            return TYPE_MIXED;
        };

        self.method_signature_callable(&symbol, name.value)
    }

    fn method_signature_callable(&mut self, class: &ClassLikeSymbol<'arena>, method: &[u8]) -> Type<'arena> {
        let target = SymbolId::method(class.path().as_bytes(), method);
        let Some(method) = class.methods().members.iter().find(|member| member.name.id == target) else {
            return TYPE_MIXED;
        };

        let return_type = method.ret.effective(true).unwrap_or(TYPE_MIXED);
        self.build_callable(method.params, return_type)
    }
}
