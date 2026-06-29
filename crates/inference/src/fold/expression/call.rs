use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::argument::Argument;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::Call;
use mago_hir::ir::expression::Callee;
use mago_hir::ir::expression::CalleeKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::identifier::Identifier;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::function_like::FunctionLikeSymbol;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::atom::payload::generic_parameter::GenericParameterAtom;
use mago_oracle::ty::template::substitute;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_span::Span;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_call(
        &mut self,
        span: Span,
        call: &'source Call<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut arguments = Vec::new_in(self.arena);
        let mut argument_types = Vec::new_in(self.source);
        for argument in call.arguments.items {
            let typed = match argument {
                Argument::Value(expression) => {
                    Argument::Value(self.infer_call_argument(expression, &mut argument_types)?)
                }
                Argument::Variadic(expression) => {
                    Argument::Variadic(self.infer_call_argument(expression, &mut argument_types)?)
                }
                Argument::Named(name, expression) => Argument::Named(
                    name.copy_into(self.arena),
                    self.infer_call_argument(expression, &mut argument_types)?,
                ),
            };

            arguments.push(typed);
        }

        let arguments = Delimited { span: call.arguments.span, items: arguments.leak() };
        let (kind, meta) = match call.callee.kind {
            CalleeKind::Function(callee) => match &callee.kind {
                ExpressionKind::Identifier(identifier) => {
                    let meta = self.resolve_function_call(identifier, &argument_types);
                    let callee = Expression {
                        meta: TYPE_MIXED,
                        span: callee.span,
                        kind: ExpressionKind::Identifier(identifier.copy_into(self.arena)),
                    };

                    (CalleeKind::Function(self.arena.alloc(callee)), meta)
                }
                _ => {
                    let callee = self.infer_expression(callee)?;
                    let meta = self.resolve_callable_call(callee.meta, &argument_types);

                    (CalleeKind::Function(self.arena.alloc(callee)), meta)
                }
            },
            _ => return Err(InferenceError::Unsupported { span, construct: "method and static-method calls" }),
        };

        let callee = Callee { span: call.callee.span, kind };
        let call = Call { span: call.span, callee, arguments };

        Ok(Expression { meta, span, kind: ExpressionKind::Call(self.arena.alloc(call)) })
    }

    fn infer_call_argument(
        &mut self,
        expression: &'source Expression<'source, SymbolId, S, E>,
        argument_types: &mut Vec<'source, Type<'arena>, A>,
    ) -> InferenceResult<&'arena Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let typed = self.infer_expression(expression)?;
        argument_types.push(typed.meta);

        Ok(self.arena.alloc(typed))
    }

    fn resolve_function_call(
        &mut self,
        identifier: &Identifier<'source>,
        argument_types: &[Type<'arena>],
    ) -> Type<'arena> {
        let Some(FunctionLikeSymbol::Function(function)) = self.resolve_function_symbol(identifier) else {
            return TYPE_MIXED;
        };

        let Some(ret) = function.ret.effective(true) else {
            return TYPE_MIXED;
        };

        if function.generics.is_empty() {
            return ret;
        }

        let mut parameter_types = Vec::new_in(self.source);
        for parameter in function.params {
            parameter_types.push(parameter.ty.effective(true).unwrap_or(TYPE_MIXED));
        }

        self.instantiate_return(&parameter_types, ret, argument_types)
    }

    fn resolve_callable_call(&mut self, callee: Type<'arena>, argument_types: &[Type<'arena>]) -> Type<'arena> {
        let [Atom::Callable(CallableAtom::Closure(signature) | CallableAtom::Signature(signature))] = callee.atoms
        else {
            return TYPE_MIXED;
        };

        let Some(parameters) = signature.parameters else {
            return signature.return_type;
        };

        let mut parameter_types = Vec::new_in(self.source);
        for parameter in parameters {
            parameter_types.push(parameter.r#type);
        }

        self.instantiate_return(&parameter_types, signature.return_type, argument_types)
    }

    fn instantiate_return(
        &mut self,
        parameter_types: &[Type<'arena>],
        ret: Type<'arena>,
        argument_types: &[Type<'arena>],
    ) -> Type<'arena> {
        let mut bindings = Vec::new_in(self.source);
        for (index, parameter_type) in parameter_types.iter().enumerate() {
            if let [Atom::GenericParameter(generic)] = parameter_type.atoms
                && let Some(argument_type) = argument_types.get(index)
            {
                bindings.push((generic.name, *argument_type));
            }
        }

        if bindings.is_empty() {
            return ret;
        }

        let resolver = |parameter: &GenericParameterAtom<'arena>| -> Option<Type<'arena>> {
            bindings.iter().find_map(|(name, ty)| (*name == parameter.name).then_some(*ty))
        };

        substitute(ret, &resolver, &mut self.ty)
    }

    fn resolve_function_symbol(&self, identifier: &Identifier<'source>) -> Option<FunctionLikeSymbol<'arena>> {
        if let Some(symbol) = self.symbols.get_function_like(SymbolId::function_like(identifier.value)) {
            return Some(symbol);
        }

        if identifier.is_local() && !identifier.imported {
            let short_name = identifier.last_segment();
            if short_name != identifier.value {
                return self.symbols.get_function_like(SymbolId::function_like(short_name));
            }
        }

        None
    }
}
