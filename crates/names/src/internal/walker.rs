use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

use crate::ResolvedNames;
use crate::internal::context::NameResolutionContext;
use crate::kind::NameKind;

/// An AST visitor (`MutWalker`) that traverses a PHP Abstract Syntax Tree
/// to resolve names (classes, functions, constants, etc.) according to
/// PHP's scoping and aliasing rules.
#[derive(Debug, Clone, Default)]
pub struct NameWalker<'arena> {
    /// Accumulates the resolved names found during the AST walk.
    pub resolved_names: ResolvedNames<'arena>,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, NameResolutionContext<'arena>> for NameWalker<'arena> {
    fn walk_in_namespace(&mut self, namespace: &'ast Namespace<'arena>, context: &mut NameResolutionContext<'arena>) {
        context.enter_namespace(namespace.name.as_ref().map(|n| n.value()));
    }

    fn walk_in_use(&mut self, r#use: &'ast Use<'arena>, context: &mut NameResolutionContext<'arena>) {
        context.populate_from_use(r#use);
    }

    fn walk_in_constant(&mut self, constant: &'ast Constant<'arena>, context: &mut NameResolutionContext<'arena>) {
        for item in constant.items.iter() {
            let name = context.qualify_name(item.name.value);

            self.resolved_names.insert_at(&item.name.span.start, name, false);
        }
    }

    fn walk_in_function(&mut self, function: &'ast Function<'arena>, context: &mut NameResolutionContext<'arena>) {
        let name = context.qualify_name(function.name.value);

        self.resolved_names.insert_at(&function.name.span.start, name, false);
    }

    fn walk_in_class(&mut self, class: &'ast Class<'arena>, context: &mut NameResolutionContext<'arena>) {
        let classlike = context.qualify_name(class.name.value);

        self.resolved_names.insert_at(&class.name.span.start, classlike, false);
    }

    fn walk_in_interface(&mut self, interface: &'ast Interface<'arena>, context: &mut NameResolutionContext<'arena>) {
        let classlike = context.qualify_name(interface.name.value);

        self.resolved_names.insert_at(&interface.name.span.start, classlike, false);
    }

    fn walk_in_trait(&mut self, r#trait: &'ast Trait<'arena>, context: &mut NameResolutionContext<'arena>) {
        let classlike = context.qualify_name(r#trait.name.value);

        self.resolved_names.insert_at(&r#trait.name.span.start, classlike, false);
    }

    fn walk_in_enum(&mut self, r#enum: &'ast Enum<'arena>, context: &mut NameResolutionContext<'arena>) {
        let classlike = context.qualify_name(r#enum.name.value);

        self.resolved_names.insert_at(&r#enum.name.span.start, classlike, false);
    }

    fn walk_in_trait_use(&mut self, trait_use: &'ast TraitUse<'arena>, context: &mut NameResolutionContext<'arena>) {
        for trait_name in trait_use.trait_names.iter() {
            let (trait_classlike, imported) = context.resolve(NameKind::Default, trait_name.value());

            self.resolved_names.insert_at(&trait_name.span(), trait_classlike, imported);
        }
    }

    fn walk_in_extends(&mut self, extends: &'ast Extends<'arena>, context: &mut NameResolutionContext<'arena>) {
        for parent in extends.types.iter() {
            let (parent_classlike, imported) = context.resolve(NameKind::Default, parent.value());

            self.resolved_names.insert_at(&parent.span().start, parent_classlike, imported);
        }
    }

    fn walk_in_implements(
        &mut self,
        implements: &'ast Implements<'arena>,
        context: &mut NameResolutionContext<'arena>,
    ) {
        for parent in implements.types.iter() {
            let (parent_classlike, imported) = context.resolve(NameKind::Default, parent.value());

            self.resolved_names.insert_at(&parent.span().start, parent_classlike, imported);
        }
    }

    fn walk_in_hint(&mut self, hint: &'ast Hint<'arena>, context: &mut NameResolutionContext<'arena>) {
        if let Hint::Identifier(identifier) = hint {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_in_attribute(&mut self, attribute: &'ast Attribute<'arena>, context: &mut NameResolutionContext<'arena>) {
        let (name, imported) = context.resolve(NameKind::Default, attribute.name.value());

        self.resolved_names.insert_at(&attribute.name.span().start, name, imported);
    }

    fn walk_in_function_call(
        &mut self,
        function_call: &'ast FunctionCall<'arena>,
        context: &mut NameResolutionContext<'arena>,
    ) {
        if let Expression::Identifier(identifier) = function_call.function {
            let (name, imported) = context.resolve(NameKind::Function, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_in_function_closure_creation(
        &mut self,
        function_closure_creation: &'ast FunctionClosureCreation<'arena>,
        context: &mut NameResolutionContext<'arena>,
    ) {
        if let Expression::Identifier(identifier) = function_closure_creation.function {
            let (name, imported) = context.resolve(NameKind::Function, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_in_instantiation(
        &mut self,
        instantiation: &'ast Instantiation<'arena>,
        context: &mut NameResolutionContext<'arena>,
    ) {
        if let Expression::Identifier(identifier) = instantiation.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_in_static_method_call(
        &mut self,
        static_method_call: &'ast StaticMethodCall<'arena>,
        context: &mut NameResolutionContext<'arena>,
    ) {
        if let Expression::Identifier(identifier) = static_method_call.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_in_static_method_closure_creation(
        &mut self,
        static_method_closure_creation: &'ast StaticMethodClosureCreation<'arena>,
        context: &mut NameResolutionContext<'arena>,
    ) {
        if let Expression::Identifier(identifier) = static_method_closure_creation.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_in_static_property_access(
        &mut self,
        static_property_access: &'ast StaticPropertyAccess<'arena>,
        context: &mut NameResolutionContext<'arena>,
    ) {
        if let Expression::Identifier(identifier) = static_property_access.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_in_class_constant_access(
        &mut self,
        class_constant_access: &'ast ClassConstantAccess<'arena>,
        context: &mut NameResolutionContext<'arena>,
    ) {
        if let Expression::Identifier(identifier) = class_constant_access.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_in_binary(&mut self, binary: &'ast Binary<'arena>, context: &mut NameResolutionContext<'arena>) {
        if let (BinaryOperator::Instanceof(_), Expression::Identifier(identifier)) = (binary.operator, binary.rhs) {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_in_constant_access(
        &mut self,
        constant_access: &'ast ConstantAccess<'arena>,
        context: &mut NameResolutionContext<'arena>,
    ) {
        let identifier = &constant_access.name;

        if !self.resolved_names.contains(&identifier.span().start) {
            let (name, imported) = context.resolve(NameKind::Constant, identifier.value());

            self.resolved_names.insert_at(&identifier.span().start, name, imported);
        }
    }

    fn walk_out_namespace(&mut self, _namespace: &Namespace<'arena>, context: &mut NameResolutionContext<'arena>) {
        context.exit_namespace();
    }
}
