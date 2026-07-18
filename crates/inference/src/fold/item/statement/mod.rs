mod class;
mod constant;
mod r#enum;
mod function;
mod interface;
mod r#trait;

use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;
use mago_allocator::vec::Vec;
use mago_flags::U8Flags;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::item::member::MemberItem;
use mago_hir::ir::item::member::MemberItemKind;
use mago_hir::ir::item::member::constant::ClassLikeConstant;
use mago_hir::ir::item::member::enum_case::EnumCase;
use mago_hir::ir::item::member::hook::Hook;
use mago_hir::ir::item::member::hook::HookBody;
use mago_hir::ir::item::member::hook::HookBodyKind;
use mago_hir::ir::item::member::method::Method;
use mago_hir::ir::item::member::property::HookedProperty;
use mago_hir::ir::item::member::property::Property;
use mago_hir::ir::item::member::trait_use::TraitUse;
use mago_hir::ir::item::modifier::Modifier;
use mago_hir::ir::item::modifier::ModifierKind;
use mago_hir::ir::item::statement::ItemStatement;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::statement::Block;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Terminator;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::Symbol;
use mago_oracle::symbol::class_like::ClassLikeKind;
use mago_oracle::symbol::class_like::ClassLikeSymbol;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameter;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity;
use mago_oracle::ty::atom::payload::generic_parameter::GenericParameterAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectFlag;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_statement_item(
        &mut self,
        span: Span,
        terminator: Option<Terminator>,
        item: &'source ItemStatement<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let kind = match item.kind {
            ItemStatementKind::Enum(node) => {
                ItemStatementKind::Enum(self.arena.alloc(self.infer_enum(item.meta, node)?))
            }
            ItemStatementKind::Class(node) => {
                ItemStatementKind::Class(self.arena.alloc(self.infer_class(item.meta, node)?))
            }
            ItemStatementKind::Interface(node) => {
                ItemStatementKind::Interface(self.arena.alloc(self.infer_interface(item.meta, node)?))
            }
            ItemStatementKind::Trait(node) => {
                ItemStatementKind::Trait(self.arena.alloc(self.infer_trait(item.meta, node)?))
            }
            ItemStatementKind::Constant(node) => {
                ItemStatementKind::Constant(self.arena.alloc(self.infer_constant_declaration(node)?))
            }
            ItemStatementKind::Function(node) => {
                ItemStatementKind::Function(self.arena.alloc(self.infer_function(node)?))
            }
        };

        let node = ItemStatement { meta: item.meta, span: item.span, kind };

        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::Item(self.arena.alloc(node)),
            terminator,
        })
    }

    /// Folds one class-like member. `class_name` is the enclosing class's
    /// fully-qualified name and `this_type` the type a non-static method's `$this`
    /// takes. Properties are rejected (enums cannot declare them).
    pub(crate) fn infer_member(
        &mut self,
        class_name: &'arena [u8],
        this_type: Type<'arena>,
        member: &'source MemberItem<'source, SymbolId, S, E>,
    ) -> InferenceResult<MemberItem<'arena, SymbolId, Flow, Type<'arena>>> {
        let kind = match member.kind {
            MemberItemKind::Method(method) => {
                MemberItemKind::Method(self.arena.alloc(self.infer_method(class_name, this_type, method)?))
            }
            MemberItemKind::Constant(constant) => {
                MemberItemKind::Constant(self.arena.alloc(self.infer_constant_member(constant)?))
            }
            MemberItemKind::EnumCase(case) => MemberItemKind::EnumCase(self.arena.alloc(self.infer_enum_case(case)?)),
            MemberItemKind::Property(property) => {
                MemberItemKind::Property(self.arena.alloc(self.infer_property_member(property)?))
            }
            MemberItemKind::HookedProperty(property) => {
                MemberItemKind::HookedProperty(self.arena.alloc(self.infer_hooked_property(this_type, property)?))
            }
            MemberItemKind::TraitUse(trait_use) => {
                MemberItemKind::TraitUse(self.arena.alloc(self.infer_trait_use(trait_use)?))
            }
        };

        Ok(MemberItem { meta: member.meta, span: member.span, kind, terminator: member.terminator })
    }

    fn infer_method(
        &mut self,
        class_name: &'arena [u8],
        this_type: Type<'arena>,
        method: &'source Method<'source, SymbolId, S, E>,
    ) -> InferenceResult<Method<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(method.attributes)?;
        let annotation = match method.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let signature = self.method_signature(class_name, method.name.value);

        let outer_environment = std::mem::replace(&mut self.environment, Environment::new_in(self.source));
        let outer_reachable = self.reachable;
        self.reachable = true;

        if !is_static(method.modifiers) {
            self.environment.set(Var::new(self.arena.alloc_slice_copy(b"$this")), this_type);
        }
        self.bind_signature_parameters(method.parameters.items, signature);

        let parameters = self.infer_parameters(&method.parameters, Some(this_type))?;
        let body = match method.body {
            Some(body) => {
                let (statements, _) = self.infer_block(body.statements)?;

                Some(&*self.arena.alloc(Block { span: body.span, statements }))
            }
            None => None,
        };

        self.environment = outer_environment;
        self.reachable = outer_reachable;

        Ok(Method {
            span: method.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(method.version_constraint),
            flags: method.flags,
            modifiers: copy_slice_into(method.modifiers, self.arena),
            name: method.name.copy_into(self.arena),
            parameters,
            return_type: method.return_type.map(|return_type| copy_ref_into(return_type, self.arena)),
            direct_accessed_globals: copy_slice_into(method.direct_accessed_globals, self.arena),
            body,
        })
    }

    fn infer_constant_member(
        &mut self,
        constant: &'source ClassLikeConstant<'source, SymbolId, S, E>,
    ) -> InferenceResult<ClassLikeConstant<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(constant.attributes)?;
        let annotation = match constant.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let value = self.infer_expression(constant.value)?;

        Ok(ClassLikeConstant {
            span: constant.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(constant.version_constraint),
            modifiers: copy_slice_into(constant.modifiers, self.arena),
            r#type: constant.r#type.map(|r#type| copy_ref_into(r#type, self.arena)),
            name: constant.name.copy_into(self.arena),
            value: self.arena.alloc(value),
            flattened: constant.flattened,
        })
    }

    fn infer_property_member(
        &mut self,
        property: &'source Property<'source, SymbolId, S, E>,
    ) -> InferenceResult<Property<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(property.attributes)?;
        let annotation = match property.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let default_value = match property.default_value {
            Some(default_value) => Some(&*self.arena.alloc(self.infer_expression(default_value)?)),
            None => None,
        };

        Ok(Property {
            span: property.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(property.version_constraint),
            modifiers: copy_slice_into(property.modifiers, self.arena),
            r#type: property.r#type.map(|r#type| copy_ref_into(r#type, self.arena)),
            variable: property.variable.copy_into(self.arena),
            default_value,
            flattened: property.flattened,
        })
    }

    fn infer_hooked_property(
        &mut self,
        this_type: Type<'arena>,
        property: &'source HookedProperty<'source, SymbolId, S, E>,
    ) -> InferenceResult<HookedProperty<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(property.attributes)?;
        let annotation = match property.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let default_value = match property.default_value {
            Some(default_value) => Some(&*self.arena.alloc(self.infer_expression(default_value)?)),
            None => None,
        };

        let mut hooks = Vec::new_in(self.arena);
        for hook in property.hooks.items {
            hooks.push(self.infer_hook(this_type, hook)?);
        }

        Ok(HookedProperty {
            span: property.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(property.version_constraint),
            modifiers: copy_slice_into(property.modifiers, self.arena),
            r#type: property.r#type.map(|r#type| copy_ref_into(r#type, self.arena)),
            variable: property.variable.copy_into(self.arena),
            default_value,
            hooks: Delimited { span: property.hooks.span, items: hooks.leak() },
        })
    }

    pub(crate) fn infer_hook(
        &mut self,
        this_type: Type<'arena>,
        hook: &'source Hook<'source, SymbolId, S, E>,
    ) -> InferenceResult<Hook<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(hook.attributes)?;
        let annotation = match hook.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };

        let outer_environment = std::mem::replace(&mut self.environment, Environment::new_in(self.source));
        let outer_reachable = self.reachable;
        self.reachable = true;
        self.environment.set(Var::new(self.arena.alloc_slice_copy(b"$this")), this_type);

        let parameters = match &hook.parameters {
            Some(parameters) => {
                self.bind_signature_parameters(parameters.items, &[]);

                Some(self.infer_parameters(parameters, None)?)
            }
            None => None,
        };
        let body = match &hook.body {
            Some(body) => Some(self.infer_hook_body(body)?),
            None => None,
        };

        self.environment = outer_environment;
        self.reachable = outer_reachable;

        Ok(Hook {
            span: hook.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(hook.version_constraint),
            flags: hook.flags,
            modifiers: copy_slice_into(hook.modifiers, self.arena),
            name: hook.name.copy_into(self.arena),
            parameters,
            body,
        })
    }

    fn infer_hook_body(
        &mut self,
        body: &'source HookBody<'source, SymbolId, S, E>,
    ) -> InferenceResult<HookBody<'arena, SymbolId, Flow, Type<'arena>>> {
        let kind = match &body.kind {
            HookBodyKind::Expression(expression) => {
                HookBodyKind::Expression(self.arena.alloc(self.infer_expression(expression)?))
            }
            HookBodyKind::Block(block) => {
                let (statements, _exit) = self.infer_block(block.statements)?;

                HookBodyKind::Block(self.arena.alloc(Block { span: block.span, statements }))
            }
        };

        Ok(HookBody { span: body.span, kind })
    }

    fn infer_trait_use(
        &mut self,
        trait_use: &'source TraitUse<'source, SymbolId, S, E>,
    ) -> InferenceResult<TraitUse<'arena, SymbolId, Flow, Type<'arena>>> {
        let annotation = match trait_use.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };

        Ok(TraitUse {
            span: trait_use.span,
            annotation,
            traits: copy_slice_into(trait_use.traits, self.arena),
            adaptations: trait_use.adaptations.map(|adaptations| adaptations.copy_into(self.arena)),
        })
    }

    fn infer_enum_case(
        &mut self,
        case: &'source EnumCase<'source, SymbolId, S, E>,
    ) -> InferenceResult<EnumCase<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(case.attributes)?;
        let annotation = match case.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let value = match case.value {
            Some(value) => Some(&*self.arena.alloc(self.infer_expression(value)?)),
            None => None,
        };

        Ok(EnumCase {
            span: case.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(case.version_constraint),
            name: case.name.copy_into(self.arena),
            value,
        })
    }

    /// The signature parameters of `class_name::method_name`, or an empty slice
    /// when the class or method is unknown.
    fn method_signature(&self, class_name: &'arena [u8], method_name: &[u8]) -> &'arena [SignatureParameter<'arena>] {
        let target = SymbolId::method(class_name, method_name);
        self.symbols
            .get_class_like(SymbolId::class_like(class_name))
            .and_then(|symbol| symbol.methods().members.iter().find(|member| member.name.id == target))
            .map_or(&[], |member| member.params)
    }

    /// The class context for folding a class-like body: the canonical name its
    /// symbol is keyed by (for `self::`/`static::`) and the type its `$this` takes.
    /// Falls back to the written name and a bare named object when the symbol is
    /// somehow absent.
    pub(crate) fn class_context(&mut self, symbol: SymbolId, fallback_name: &[u8]) -> (&'arena [u8], Type<'arena>) {
        match self.symbols.get_class_like(symbol) {
            Some(symbol) => (symbol.path().as_bytes(), self.this_type(&symbol)),
            None => {
                let class_name = self.arena.alloc_slice_copy(fallback_name);
                let atom = self.ty.named_object_atom(class_name);

                (class_name, self.ty.union_of(&[atom]))
            }
        }
    }

    /// The type a non-static method's `$this` takes: an enum atom for an enum, or
    /// a `$this`/`static` named object for a class, interface, or trait that
    /// carries the class's own template parameters so a generic self resolves.
    fn this_type(&mut self, symbol: &ClassLikeSymbol<'arena>) -> Type<'arena> {
        let class_name = symbol.path().as_bytes();
        if matches!(symbol.kind(), ClassLikeKind::Enum) {
            let atom = self.ty.enum_atom(class_name);

            return self.ty.union_of(&[atom]);
        }

        let name = self.ty.intern_class_like_path(class_name);
        let generics = symbol.generics();
        let type_arguments = if generics.is_empty() {
            None
        } else {
            let mut arguments = self.ty.scratch_vec::<Type<'arena>>();
            for generic in generics {
                let atom = self.ty.generic_parameter(GenericParameterAtom {
                    name: generic.name,
                    defining_entity: DefiningEntity::ClassLike(name),
                    constraint: generic.constraint,
                });
                arguments.push(self.ty.union_of(&[atom]));
            }

            Some(self.ty.types(&arguments))
        };

        let flags = U8Flags::<ObjectFlag>::empty().with(ObjectFlag::IsThis).with(ObjectFlag::IsStatic);
        let atom = self.ty.object(ObjectAtom { name, type_arguments, flags });

        self.ty.union_of(&[atom])
    }
}

fn is_static(modifiers: &[Modifier]) -> bool {
    modifiers.iter().any(|modifier| modifier.kind == ModifierKind::Static)
}
