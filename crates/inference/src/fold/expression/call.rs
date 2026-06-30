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
use mago_hir::ir::expression::selector::MemberSelector;
use mago_hir::ir::expression::selector::MemberSelectorKind;
use mago_hir::ir::identifier::Identifier;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::Symbol;
use mago_oracle::symbol::class_like::ClassLikeSymbol;
use mago_oracle::symbol::function_like::FunctionLikeSymbol;
use mago_oracle::symbol::function_like::function::FunctionSymbol;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameter;
use mago_oracle::symbol::part::ty::TypeSlot;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::atom::payload::generic_parameter::GenericParameterAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::template::substitute;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::ty::well_known::TYPE_NULL;
use mago_span::Span;

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
        let (kind, meta) = match &call.callee.kind {
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
            CalleeKind::Method(object, selector) => {
                let object = self.infer_expression(object)?;
                let meta = self.method_return(object.meta, selector, &argument_types);
                let selector = self.infer_member_selector(selector)?;

                (CalleeKind::Method(self.arena.alloc(object), selector), meta)
            }
            CalleeKind::NullsafeMethod(object, selector) => {
                let object = self.infer_expression(object)?;
                let mut meta = self.method_return(object.meta, selector, &argument_types);
                if object.meta.atoms.iter().any(|atom| matches!(atom, Atom::Null)) {
                    meta = self.union(meta, TYPE_NULL);
                }
                let selector = self.infer_member_selector(selector)?;

                (CalleeKind::NullsafeMethod(self.arena.alloc(object), selector), meta)
            }
            CalleeKind::StaticMethod(class, selector) => {
                let meta = self.static_method_return(class, selector, &argument_types);
                let class = self.infer_expression(class)?;
                let selector = self.infer_member_selector(selector)?;

                (CalleeKind::StaticMethod(self.arena.alloc(class), selector), meta)
            }
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

        self.function_return(function, argument_types)
    }

    fn function_return(&mut self, function: &FunctionSymbol<'arena>, argument_types: &[Type<'arena>]) -> Type<'arena> {
        self.signature_return(function.ret, function.generics.is_empty(), function.params, argument_types)
    }

    fn signature_return(
        &mut self,
        ret: TypeSlot<'arena>,
        monomorphic: bool,
        params: &[SignatureParameter<'arena>],
        argument_types: &[Type<'arena>],
    ) -> Type<'arena> {
        let Some(ret) = ret.effective(true) else {
            return TYPE_MIXED;
        };

        if monomorphic {
            return ret;
        }

        let mut parameter_types = Vec::new_in(self.source);
        for parameter in params {
            parameter_types.push(parameter.ty.effective(true).unwrap_or(TYPE_MIXED));
        }

        self.instantiate_return(&parameter_types, ret, argument_types)
    }

    fn method_return(
        &mut self,
        receiver: Type<'arena>,
        selector: &MemberSelector<'source, SymbolId, S, E>,
        argument_types: &[Type<'arena>],
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

        self.method_return_type(&symbol, name.value, argument_types)
    }

    fn static_method_return(
        &mut self,
        class: &'source Expression<'source, SymbolId, S, E>,
        selector: &MemberSelector<'source, SymbolId, S, E>,
        argument_types: &[Type<'arena>],
    ) -> Type<'arena> {
        let MemberSelectorKind::Name(name) = &selector.kind else {
            return TYPE_MIXED;
        };

        let Some(symbol) = self.resolve_class(class) else {
            return TYPE_MIXED;
        };

        self.method_return_type(&symbol, name.value, argument_types)
    }

    fn method_return_type(
        &mut self,
        class: &ClassLikeSymbol<'arena>,
        method: &[u8],
        argument_types: &[Type<'arena>],
    ) -> Type<'arena> {
        let target = SymbolId::method(class.path().as_bytes(), method);
        let Some(method) = class.methods().members.iter().find(|member| member.name.id == target) else {
            return TYPE_MIXED;
        };

        self.signature_return(method.ret, method.generics.is_empty(), method.params, argument_types)
    }

    pub(crate) fn infer_member_selector(
        &mut self,
        selector: &MemberSelector<'source, SymbolId, S, E>,
    ) -> InferenceResult<MemberSelector<'arena, SymbolId, Flow, Type<'arena>>> {
        let kind = match &selector.kind {
            MemberSelectorKind::Missing => MemberSelectorKind::Missing,
            MemberSelectorKind::Name(name) => MemberSelectorKind::Name(name.copy_into(self.arena)),
            MemberSelectorKind::Variable(variable) => MemberSelectorKind::Variable(variable.copy_into(self.arena)),
            MemberSelectorKind::Expression(expression) => {
                let expression = self.infer_expression(expression)?;

                MemberSelectorKind::Expression(self.arena.alloc(expression))
            }
        };

        Ok(MemberSelector { span: selector.span, kind })
    }

    pub(crate) fn resolve_callable_call(
        &mut self,
        callee: Type<'arena>,
        argument_types: &[Type<'arena>],
    ) -> Type<'arena> {
        if let [Atom::String(string)] = callee.atoms
            && let StringLiteral::Value(name) = string.literal
            && !name.contains(&b':')
            && let Some(FunctionLikeSymbol::Function(function)) =
                self.symbols.get_function_like(SymbolId::function_like(name))
        {
            return self.function_return(function, argument_types);
        }

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

fn object_class<'arena>(atom: &Atom<'arena>) -> Option<&'arena [u8]> {
    match atom {
        Atom::Object(object) => Some(object.name.as_bytes()),
        Atom::Enum(enumeration) => Some(enumeration.name.as_bytes()),
        _ => None,
    }
}
