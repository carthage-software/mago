use mago_ast::*;
use mago_span::*;
use mago_walker::Walker;

use crate::internal::context::Context;
use crate::Reference;
use crate::ReferenceKind;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ReferenceFindingWalker;

impl<'a> Walker<Context<'a>> for ReferenceFindingWalker {
    #[inline]
    fn walk_use(&self, r#use: &Use, context: &mut Context<'a>) {
        match &r#use.items {
            UseItems::Sequence(use_item_sequence) => {
                for item in &use_item_sequence.items.nodes {
                    let item_name_id = item.name.value();
                    let item_name = context.interner.lookup(&item_name_id);

                    if context.query.matches(item_name) {
                        context.references.push(Reference {
                            value: item_name_id,
                            kind: ReferenceKind::Import,
                            span: item.name.span(),
                        });
                    }
                }
            }
            UseItems::TypedSequence(typed_use_item_sequence) => {
                for item in &typed_use_item_sequence.items.nodes {
                    let item_name_id = item.name.value();
                    let item_name = context.interner.lookup(&item_name_id);

                    if context.query.matches(item_name) {
                        context.references.push(Reference {
                            value: item_name_id,
                            kind: ReferenceKind::Import,
                            span: item.name.span(),
                        });
                    }
                }
            }
            UseItems::TypedList(typed_use_item_list) => {
                let prefix_id = typed_use_item_list.namespace.value();
                let prefix = context.interner.lookup(&prefix_id);

                for item in &typed_use_item_list.items.nodes {
                    let item_name_id = item.name.value();
                    let item_name = context.interner.lookup(&item_name_id);
                    let full_name = format!("{}\\{}", prefix, item_name);

                    if context.query.matches(&full_name) {
                        let full_name_id = context.interner.intern(&full_name);

                        context.references.push(Reference {
                            value: full_name_id,
                            kind: ReferenceKind::Import,
                            span: item.name.span(),
                        });
                    }
                }
            }
            UseItems::MixedList(mixed_use_item_list) => {
                let prefix_id = mixed_use_item_list.namespace.value();
                let prefix = context.interner.lookup(&prefix_id);

                for maybe_typed_item in &mixed_use_item_list.items.nodes {
                    let item_name_id = maybe_typed_item.item.name.value();
                    let item_name = context.interner.lookup(&item_name_id);
                    let full_name = format!("{}\\{}", prefix, item_name);

                    if context.query.matches(&full_name) {
                        let full_name_id = context.interner.intern(&full_name);

                        context.references.push(Reference {
                            value: full_name_id,
                            kind: ReferenceKind::Import,
                            span: maybe_typed_item.item.name.span(),
                        });
                    }
                }
            }
        }
    }

    #[inline]
    fn walk_in_class(&self, class: &Class, context: &mut Context<'a>) {
        let class_name_id = context.module.names.get(&class.name);
        let class_name = context.interner.lookup(class_name_id);

        if context.query.matches(class_name) {
            context.references.push(Reference {
                value: *class_name_id,
                kind: ReferenceKind::Definition,
                span: class.name.span(),
            });
        }

        if let Some(extends) = class.extends.as_ref() {
            for extended in &extends.types.nodes {
                let extended_name_id = context.module.names.get(&extended);
                let extended_name = context.interner.lookup(extended_name_id);

                if context.query.matches(extended_name) {
                    context.references.push(Reference {
                        value: *extended_name_id,
                        kind: ReferenceKind::Extension,
                        span: extended.span(),
                    });
                }
            }
        }

        if let Some(implements) = class.implements.as_ref() {
            for implemented in &implements.types.nodes {
                let implemented_name_id = context.module.names.get(&implemented);
                let implemented_name = context.interner.lookup(implemented_name_id);

                if context.query.matches(implemented_name) {
                    context.references.push(Reference {
                        value: *implemented_name_id,
                        kind: ReferenceKind::Implementation,
                        span: implemented.span(),
                    });
                }
            }
        }
    }

    #[inline]
    fn walk_in_interface(&self, interface: &Interface, context: &mut Context<'a>) {
        let interface_name_id = context.module.names.get(&interface.name);
        let interface_name = context.interner.lookup(interface_name_id);

        if context.query.matches(interface_name) {
            context.references.push(Reference {
                value: *interface_name_id,
                kind: ReferenceKind::Definition,
                span: interface.name.span(),
            });
        }

        if let Some(extends) = interface.extends.as_ref() {
            for extended in &extends.types.nodes {
                let extended_name_id = context.module.names.get(&extended);
                let extended_name = context.interner.lookup(extended_name_id);

                if context.query.matches(extended_name) {
                    context.references.push(Reference {
                        value: *extended_name_id,
                        kind: ReferenceKind::Extension,
                        span: extended.span(),
                    });
                }
            }
        }
    }

    #[inline]
    fn walk_in_trait(&self, r#trait: &Trait, context: &mut Context<'a>) {
        let trait_name_id = context.module.names.get(&r#trait.name);
        let trait_name = context.interner.lookup(trait_name_id);

        if context.query.matches(trait_name) {
            context.references.push(Reference {
                value: *trait_name_id,
                kind: ReferenceKind::Definition,
                span: r#trait.name.span(),
            });
        }
    }

    #[inline]
    fn walk_in_enum(&self, r#enum: &Enum, context: &mut Context<'a>) {
        let enum_name_id = context.module.names.get(&r#enum.name);
        let enum_name = context.interner.lookup(enum_name_id);

        if context.query.matches(enum_name) {
            context.references.push(Reference {
                value: *enum_name_id,
                kind: ReferenceKind::Definition,
                span: r#enum.name.span(),
            });
        }

        if let Some(implements) = r#enum.implements.as_ref() {
            for implemented in &implements.types.nodes {
                let implemented_name_id = context.module.names.get(&implemented);
                let implemented_name = context.interner.lookup(implemented_name_id);

                if context.query.matches(implemented_name) {
                    context.references.push(Reference {
                        value: *implemented_name_id,
                        kind: ReferenceKind::Implementation,
                        span: implemented.span(),
                    });
                }
            }
        }
    }

    #[inline]
    fn walk_in_function(&self, function: &Function, context: &mut Context<'a>) {
        let function_name_id = context.module.names.get(&function.name);
        let function_name = context.interner.lookup(function_name_id);

        if context.query.matches(function_name) {
            context.references.push(Reference {
                value: *function_name_id,
                kind: ReferenceKind::Definition,
                span: function.name.span(),
            });
        }
    }

    #[inline]
    fn walk_in_constant(&self, constant: &Constant, context: &mut Context<'a>) {
        for item in &constant.items.nodes {
            let item_name_id = context.module.names.get(&item.name);
            let item_name = context.interner.lookup(item_name_id);

            if context.query.matches(item_name) {
                context.references.push(Reference {
                    value: *item_name_id,
                    kind: ReferenceKind::Definition,
                    span: item.name.span(),
                });
            }
        }
    }

    #[inline]
    fn walk_in_trait_use(&self, trait_use: &TraitUse, context: &mut Context<'a>) {
        for r#trait in &trait_use.trait_names.nodes {
            let trait_name_id = context.module.names.get(&r#trait);
            let trait_name = context.interner.lookup(trait_name_id);

            if context.query.matches(trait_name) {
                context.references.push(Reference {
                    value: *trait_name_id,
                    kind: ReferenceKind::Implementation,
                    span: r#trait.span(),
                });
            }
        }
    }

    #[inline]
    fn walk_in_binary(&self, binary: &Binary, context: &mut Context<'a>) {
        let Binary { operator: BinaryOperator::Instanceof(_), rhs, .. } = binary else {
            return;
        };

        check_expression(rhs, context);
    }

    #[inline]
    fn walk_in_hint(&self, hint: &Hint, context: &mut Context<'a>) {
        let Hint::Identifier(identifier) = hint else {
            return;
        };

        check_identifier(identifier, context);
    }

    #[inline]
    fn walk_in_instantiation(&self, instantiation: &Instantiation, context: &mut Context<'a>) {
        let Instantiation { class, .. } = instantiation;

        check_expression(class, context);
    }

    #[inline]
    fn walk_in_call(&self, call: &Call, context: &mut Context<'a>) {
        let expression = match call {
            Call::Function(function_call) => function_call.function.as_ref(),
            Call::Method(method_call) => method_call.object.as_ref(),
            Call::NullSafeMethod(null_safe_method_call) => null_safe_method_call.object.as_ref(),
            Call::StaticMethod(static_method_call) => static_method_call.class.as_ref(),
        };

        check_expression(expression, context);
    }

    #[inline]
    fn walk_in_closure_creation(&self, closure_creation: &ClosureCreation, context: &mut Context<'a>) {
        let expression = match &closure_creation {
            ClosureCreation::Function(function_closure_creation) => function_closure_creation.function.as_ref(),
            ClosureCreation::Method(method_closure_creation) => method_closure_creation.object.as_ref(),
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                static_method_closure_creation.class.as_ref()
            }
        };

        check_expression(expression, context);
    }

    #[inline]
    fn walk_in_access(&self, access: &Access, context: &mut Context<'a>) {
        let expression = match &access {
            Access::Property(property_access) => property_access.object.as_ref(),
            Access::NullSafeProperty(null_safe_property_access) => null_safe_property_access.object.as_ref(),
            Access::StaticProperty(static_property_access) => static_property_access.class.as_ref(),
            Access::ClassConstant(class_constant_access) => class_constant_access.class.as_ref(),
        };

        check_expression(expression, context);
    }

    #[inline]
    fn walk_in_constant_access(&self, constant_access: &ConstantAccess, context: &mut Context<'a>) {
        check_identifier(&constant_access.name, context);
    }

    #[inline]
    fn walk_in_magic_constant(&self, magic_constant: &MagicConstant, context: &mut Context<'a>) {
        let identifier_name_id = magic_constant.value().value;
        let identifier_name = context.interner.lookup(&identifier_name_id);

        if context.query.matches(identifier_name) {
            context.references.push(Reference {
                value: identifier_name_id,
                kind: ReferenceKind::Usage,
                span: magic_constant.span(),
            });
        }
    }

    fn walk_in_attribute(&self, attribute: &Attribute, context: &mut Context<'a>) {
        let attribute_name_id = context.module.names.get(&attribute.name);
        let attribute_name = context.interner.lookup(attribute_name_id);

        if context.query.matches(attribute_name) {
            context.references.push(Reference {
                value: *attribute_name_id,
                kind: ReferenceKind::Definition,
                span: attribute.name.span(),
            });
        }
    }
}

#[inline]
fn check_expression(expression: &Expression, context: &mut Context<'_>) {
    let Expression::Identifier(identifier) = expression else {
        return;
    };

    check_identifier(identifier, context);
}

#[inline]
fn check_identifier(identifier: &Identifier, context: &mut Context<'_>) {
    let identifier_name_id = context.module.names.get(identifier);
    let identifier_name = context.interner.lookup(identifier_name_id);

    if context.query.matches(identifier_name) {
        context.references.push(Reference {
            value: *identifier_name_id,
            kind: ReferenceKind::Usage,
            span: identifier.span(),
        });
    }
}
