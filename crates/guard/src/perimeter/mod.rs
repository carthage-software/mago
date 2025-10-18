mod checker;

use mago_atom::concat_atom;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

use crate::context::GuardContext;
use crate::perimeter::checker::check_usage;
use crate::report::breach::BreachVector;
use crate::settings::PermittedDependencyKind;

#[derive(Debug, Clone, Copy)]
pub struct DependenciesGuardWalker;

impl DependenciesGuardWalker {
    fn check_hint_in_context(hint: &Hint, usage_kind: BreachVector, context: &mut GuardContext<'_, '_>) {
        Self::check_hint_recursive(hint, usage_kind, context);
    }

    fn check_hint_recursive(hint: &Hint, usage_kind: BreachVector, context: &mut GuardContext<'_, '_>) {
        match hint {
            Hint::Identifier(identifier) => {
                let fqn = context.lookup_name(identifier);
                check_usage(context, fqn, PermittedDependencyKind::ClassLike, usage_kind, identifier.span());
            }
            Hint::Parenthesized(parenthesized) => {
                Self::check_hint_recursive(parenthesized.hint, usage_kind, context);
            }
            Hint::Nullable(nullable) => {
                Self::check_hint_recursive(nullable.hint, usage_kind, context);
            }
            Hint::Union(union) => {
                Self::check_hint_recursive(union.left, usage_kind, context);
                Self::check_hint_recursive(union.right, usage_kind, context);
            }
            Hint::Intersection(intersection) => {
                Self::check_hint_recursive(intersection.left, usage_kind, context);
                Self::check_hint_recursive(intersection.right, usage_kind, context);
            }
            _ => {}
        }
    }
}

impl<'ast, 'ctx, 'arena> MutWalker<'ast, 'arena, GuardContext<'ctx, 'arena>> for DependenciesGuardWalker {
    fn walk_in_namespace(&mut self, namespace: &'ast Namespace<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        context.set_current_namespace(namespace.name.as_ref().map(|n| n.value()));
    }

    fn walk_out_namespace(&mut self, _namespace: &'ast Namespace<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        context.set_current_namespace(None);
    }

    // Check use statements
    fn walk_in_use(&mut self, r#use: &'ast Use<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        match &r#use.items {
            UseItems::Sequence(use_item_sequence) => {
                use_item_sequence.items.iter().for_each(|use_item| {
                    check_usage(
                        context,
                        use_item.name.value(),
                        PermittedDependencyKind::ClassLike,
                        BreachVector::Use,
                        use_item.name.span(),
                    );
                });
            }
            UseItems::TypedSequence(typed_use_item_sequence) => {
                let symbol_kind = match typed_use_item_sequence.r#type {
                    UseType::Function(_) => PermittedDependencyKind::Function,
                    UseType::Const(_) => PermittedDependencyKind::Constant,
                };

                typed_use_item_sequence.items.iter().for_each(|typed_use_item| {
                    check_usage(
                        context,
                        typed_use_item.name.value(),
                        symbol_kind,
                        BreachVector::Use,
                        typed_use_item.name.span(),
                    );
                });
            }
            UseItems::TypedList(typed_use_item_list) => {
                let symbol_kind = match typed_use_item_list.r#type {
                    UseType::Function(_) => PermittedDependencyKind::Function,
                    UseType::Const(_) => PermittedDependencyKind::Constant,
                };

                typed_use_item_list.items.iter().for_each(|typed_use_item| {
                    let fqn =
                        concat_atom!(typed_use_item_list.namespace.value(), "\\", typed_use_item.name.value()).as_str();

                    check_usage(context, fqn, symbol_kind, BreachVector::Use, typed_use_item.name.span());
                });
            }
            UseItems::MixedList(mixed_use_item_list) => {
                mixed_use_item_list.items.iter().for_each(|mixed_use_item| {
                    let symbol_kind = match mixed_use_item.r#type {
                        Some(UseType::Function(_)) => PermittedDependencyKind::Function,
                        Some(UseType::Const(_)) => PermittedDependencyKind::Constant,
                        None => PermittedDependencyKind::ClassLike,
                    };

                    let fqn =
                        concat_atom!(mixed_use_item_list.namespace.value(), "\\", mixed_use_item.item.name.value())
                            .as_str();

                    check_usage(context, fqn, symbol_kind, BreachVector::Use, mixed_use_item.item.name.span());
                });
            }
        }
    }

    fn walk_in_attribute(&mut self, attribute: &'ast Attribute<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        let fqn = context.lookup_name(&attribute.name);

        check_usage(context, fqn, PermittedDependencyKind::ClassLike, BreachVector::Attribute, attribute.name.span());
    }

    // Check class-like extends
    fn walk_in_extends(&mut self, extends: &'ast Extends<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        for extended_type in extends.types.iter() {
            let fqn = context.lookup_name(extended_type);

            check_usage(context, fqn, PermittedDependencyKind::ClassLike, BreachVector::Extends, extended_type.span());
        }
    }

    // Check class-like implements
    fn walk_in_implements(&mut self, implements: &'ast Implements<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        for interface in implements.types.iter() {
            let fqn = context.lookup_name(interface);

            check_usage(context, fqn, PermittedDependencyKind::ClassLike, BreachVector::Implements, interface.span());
        }
    }

    // Check trait uses
    fn walk_in_trait_use(&mut self, trait_use: &'ast TraitUse<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        for trait_name in trait_use.trait_names.iter() {
            let fqn = context.lookup_name(trait_name);

            check_usage(context, fqn, PermittedDependencyKind::ClassLike, BreachVector::TraitUse, trait_name.span());
        }
    }

    // Check property type hints
    fn walk_in_property(&mut self, property: &'ast Property<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        if let Some(hint) = property.hint() {
            Self::check_hint_in_context(hint, BreachVector::PropertyType, context);
        }
    }

    // Check function-like parameter type hints
    fn walk_in_function_like_parameter(
        &mut self,
        function_like_parameter: &'ast FunctionLikeParameter<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        if let Some(hint) = &function_like_parameter.hint {
            Self::check_hint_in_context(hint, BreachVector::ParameterType, context);
        }
    }

    // Check function-like return type hints
    fn walk_in_function_like_return_type_hint(
        &mut self,
        function_like_return_type_hint: &'ast FunctionLikeReturnTypeHint<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        Self::check_hint_in_context(&function_like_return_type_hint.hint, BreachVector::ReturnType, context);
    }

    // Check instantiations
    fn walk_in_instantiation(
        &mut self,
        instantiation: &'ast Instantiation<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        if let Expression::Identifier(class_name) = instantiation.class {
            let fqn = context.lookup_name(class_name);

            check_usage(
                context,
                fqn,
                PermittedDependencyKind::ClassLike,
                BreachVector::Instantiation,
                class_name.span(),
            );
        }
    }

    // Check static method calls
    fn walk_in_static_method_call(
        &mut self,
        static_call: &'ast StaticMethodCall<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        if let Expression::Identifier(class_name) = static_call.class {
            let fqn = context.lookup_name(class_name);

            check_usage(
                context,
                fqn,
                PermittedDependencyKind::ClassLike,
                BreachVector::StaticMethodCall,
                class_name.span(),
            );
        }
    }

    // Check static method closure creations
    fn walk_in_static_method_closure_creation(
        &mut self,
        static_method_closure_creation: &'ast StaticMethodClosureCreation<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        if let Expression::Identifier(class_name) = static_method_closure_creation.class {
            let fqn = context.lookup_name(class_name);

            check_usage(
                context,
                fqn,
                PermittedDependencyKind::ClassLike,
                BreachVector::StaticMethodCall,
                class_name.span(),
            );
        }
    }

    // Check static property accesses
    fn walk_in_static_property_access(
        &mut self,
        static_access: &'ast StaticPropertyAccess<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        if let Expression::Identifier(class_name) = static_access.class {
            let fqn = context.lookup_name(class_name);

            check_usage(
                context,
                fqn,
                PermittedDependencyKind::ClassLike,
                BreachVector::StaticPropertyAccess,
                class_name.span(),
            );
        }
    }

    // Check function calls
    fn walk_in_function_call(
        &mut self,
        function_call: &'ast FunctionCall<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        if let Expression::Identifier(function_name) = function_call.function
            && let Some(fqn) = context.try_lookup_name(function_name)
        {
            check_usage(
                context,
                fqn,
                PermittedDependencyKind::Function,
                BreachVector::FunctionCall,
                function_name.span(),
            );
        }
    }

    // Check function closure creations
    fn walk_in_function_closure_creation(
        &mut self,
        function_closure_creation: &'ast FunctionClosureCreation<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        if let Expression::Identifier(function_name) = function_closure_creation.function
            && let Some(fqn) = context.try_lookup_name(function_name)
        {
            check_usage(
                context,
                fqn,
                PermittedDependencyKind::Function,
                BreachVector::FunctionCall,
                function_name.span(),
            );
        }
    }

    // Check class constant accesses
    fn walk_in_class_constant_access(
        &mut self,
        constant_access: &'ast ClassConstantAccess<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        if let Expression::Identifier(class_name) = constant_access.class {
            let fqn = context.lookup_name(class_name);

            check_usage(
                context,
                fqn,
                PermittedDependencyKind::ClassLike,
                BreachVector::ClassConstantAccess,
                class_name.span(),
            );
        }
    }

    // Check constant accesses
    fn walk_in_constant_access(
        &mut self,
        constant_access: &'ast ConstantAccess<'arena>,
        context: &mut GuardContext<'ctx, 'arena>,
    ) {
        if let Some(fqn) = context.try_lookup_name(&constant_access.name) {
            check_usage(
                context,
                fqn,
                PermittedDependencyKind::Constant,
                BreachVector::ConstantAccess,
                constant_access.name.span(),
            );
        }
    }
}
