use ahash::HashMap;
use fennec_ast::ast::*;
use fennec_interner::StringIdentifier;
use fennec_span::HasSpan;
use fennec_walker::MutWalker;

use crate::internal::context::NameContext;
use crate::internal::context::NameKind;
use crate::Names;

#[derive(Debug, Clone)]
pub struct NameResolver {
    pub resolved_names: Names,
}

impl<'a> NameResolver {
    pub fn new() -> Self {
        NameResolver { resolved_names: Names { names: HashMap::default() } }
    }
}

impl<'a> MutWalker<NameContext<'a>> for NameResolver {
    fn walk_in_namespace<'ast>(&mut self, namespace: &'ast Namespace, context: &mut NameContext<'a>) {
        let name = match &namespace.name {
            Some(name) => name.value(),
            None => StringIdentifier::empty(),
        };

        context.enter_namespace(name);
    }

    fn walk_in_use<'ast>(&mut self, r#use: &'ast Use, context: &mut NameContext<'a>) {
        match &r#use.items {
            UseItems::Sequence(use_item_sequence) => {
                for use_item in use_item_sequence.items.iter() {
                    let name = use_item.name.value();
                    let alias = use_item.alias.as_ref().map(|alias| alias.identifier.value);

                    context.add_name(NameKind::Default, name, alias);
                }
            }
            UseItems::TypedSequence(typed_use_item_sequence) => {
                let name_kind = match &typed_use_item_sequence.r#type {
                    UseType::Function(_) => NameKind::Function,
                    UseType::Const(_) => NameKind::Constant,
                };

                for use_item in typed_use_item_sequence.items.iter() {
                    let name = use_item.name.value();
                    let alias = use_item.alias.as_ref().map(|alias| alias.identifier.value);

                    context.add_name(name_kind, name, alias);
                }
            }
            UseItems::TypedList(typed_use_item_list) => {
                let name_kind = match &typed_use_item_list.r#type {
                    UseType::Function(_) => NameKind::Function,
                    UseType::Const(_) => NameKind::Constant,
                };

                let prefix = context.interner.lookup(&typed_use_item_list.namespace.value()).to_string();

                for use_item in typed_use_item_list.items.iter() {
                    let name = use_item.name.value();
                    let alias = use_item.alias.as_ref().map(|alias| alias.identifier.value);

                    let mut namespaced = prefix.clone();
                    namespaced.push('\\');
                    namespaced.extend(context.interner.lookup(&name).chars());

                    let namespaced_id = context.interner.intern(&namespaced);

                    context.add_name(name_kind, namespaced_id, alias);
                }
            }
            UseItems::MixedList(mixed_use_item_list) => {
                let prefix = context.interner.lookup(&mixed_use_item_list.namespace.value()).to_string();

                for use_item in mixed_use_item_list.items.iter() {
                    let kind = match &use_item.r#type {
                        None => NameKind::Default,
                        Some(UseType::Function(_)) => NameKind::Function,
                        Some(UseType::Const(_)) => NameKind::Constant,
                    };

                    let name = use_item.item.name.value();
                    let alias = use_item.item.alias.as_ref().map(|alias| alias.identifier.value);

                    let mut namespaced = prefix.clone();
                    namespaced.push('\\');
                    namespaced.extend(context.interner.lookup(&name).chars());

                    let namespaced_id = context.interner.intern(&namespaced);

                    context.add_name(kind, namespaced_id, alias);
                }
            }
        };
    }

    fn walk_in_constant<'ast>(&mut self, constant: &'ast Constant, context: &mut NameContext<'a>) {
        for item in constant.items.iter() {
            let name = context.get_namespaced_identifier(&item.name);

            self.resolved_names.insert_at(item.name.span().start, name, false);
        }
    }

    fn walk_in_function<'ast>(&mut self, function: &'ast Function, context: &mut NameContext<'a>) {
        let name = context.get_namespaced_identifier(&function.name);

        self.resolved_names.insert_at(function.name.span().start, name, false);
    }

    fn walk_in_class<'ast>(&mut self, class: &'ast Class, context: &mut NameContext<'a>) {
        let classlike = context.get_namespaced_identifier(&class.name);

        self.resolved_names.insert_at(class.name.span().start, classlike, false);
    }

    fn walk_in_interface<'ast>(&mut self, interface: &'ast Interface, context: &mut NameContext<'a>) {
        let classlike = context.get_namespaced_identifier(&interface.name);

        self.resolved_names.insert_at(interface.name.span().start, classlike, false);
    }

    fn walk_in_trait<'ast>(&mut self, r#trait: &'ast Trait, context: &mut NameContext<'a>) {
        let classlike = context.get_namespaced_identifier(&r#trait.name);

        self.resolved_names.insert_at(r#trait.name.span().start, classlike, false);
    }

    fn walk_in_enum<'ast>(&mut self, r#enum: &'ast Enum, context: &mut NameContext<'a>) {
        let classlike = context.get_namespaced_identifier(&r#enum.name);

        self.resolved_names.insert_at(r#enum.name.span().start, classlike, false);
    }

    fn walk_in_trait_use<'ast>(&mut self, trait_use: &'ast TraitUse, context: &mut NameContext<'a>) {
        for trait_name in trait_use.trait_names.iter() {
            let (trait_classlike, imported) = context.resolve_name(NameKind::Default, trait_name.value());

            self.resolved_names.insert_at(trait_name.span().start, trait_classlike, imported);
        }
    }

    fn walk_in_extends<'ast>(&mut self, extends: &'ast Extends, context: &mut NameContext<'a>) {
        for parent in extends.types.iter() {
            let (parent_classlike, imported) = context.resolve_name(NameKind::Default, parent.value());

            self.resolved_names.insert_at(parent.span().start, parent_classlike, imported);
        }
    }

    fn walk_in_implements<'ast>(&mut self, implements: &'ast Implements, context: &mut NameContext<'a>) {
        for parent in implements.types.iter() {
            let (parent_classlike, imported) = context.resolve_name(NameKind::Default, parent.value());

            self.resolved_names.insert_at(parent.span().start, parent_classlike, imported);
        }
    }

    fn walk_in_hint<'ast>(&mut self, hint: &'ast Hint, context: &mut NameContext<'a>) {
        if let Hint::Identifier(identifier) = hint {
            let (name, imported) = context.resolve_name(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(identifier.span().start, name, imported);
        }
    }

    fn walk_in_attribute<'ast>(&mut self, attribute: &'ast Attribute, context: &mut NameContext<'a>) {
        let (name, imported) = context.resolve_name(NameKind::Default, attribute.name.value());

        self.resolved_names.insert_at(attribute.name.span().start, name, imported);
    }

    fn walk_in_function_call<'ast>(&mut self, function_call: &'ast FunctionCall, context: &mut NameContext<'a>) {
        if let Expression::Identifier(identifier) = function_call.function.as_ref() {
            let (name, imported) = context.resolve_name(NameKind::Function, identifier.value());

            self.resolved_names.insert_at(identifier.span().start, name, imported);
        }
    }

    fn walk_in_function_closure_creation<'ast>(
        &mut self,
        function_closure_creation: &'ast FunctionClosureCreation,

        context: &mut NameContext<'a>,
    ) {
        if let Expression::Identifier(identifier) = &function_closure_creation.function {
            let (name, imported) = context.resolve_name(NameKind::Function, identifier.value());

            self.resolved_names.insert_at(identifier.span().start, name, imported);
        }
    }

    fn walk_in_instantiation<'ast>(&mut self, instantiation: &'ast Instantiation, context: &mut NameContext<'a>) {
        if let Expression::Identifier(identifier) = &instantiation.class {
            let (name, imported) = context.resolve_name(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(identifier.span().start, name, imported);
        }
    }

    fn walk_in_static_method_call<'ast>(
        &mut self,
        static_method_call: &'ast StaticMethodCall,

        context: &mut NameContext<'a>,
    ) {
        if let Expression::Identifier(identifier) = static_method_call.class.as_ref() {
            let (name, imported) = context.resolve_name(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(identifier.span().start, name, imported);
        }
    }

    fn walk_in_static_method_closure_creation<'ast>(
        &mut self,
        static_method_closure_creation: &'ast StaticMethodClosureCreation,

        context: &mut NameContext<'a>,
    ) {
        if let Expression::Identifier(identifier) = &static_method_closure_creation.class {
            let (name, imported) = context.resolve_name(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(identifier.span().start, name, imported);
        }
    }

    fn walk_in_static_property_access<'ast>(
        &mut self,
        static_property_access: &'ast StaticPropertyAccess,

        context: &mut NameContext<'a>,
    ) {
        if let Expression::Identifier(identifier) = &static_property_access.class {
            let (name, imported) = context.resolve_name(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(identifier.span().start, name, imported);
        }
    }

    fn walk_in_class_constant_access<'ast>(
        &mut self,
        class_constant_access: &'ast ClassConstantAccess,

        context: &mut NameContext<'a>,
    ) {
        if let Expression::Identifier(identifier) = &class_constant_access.class {
            let (name, imported) = context.resolve_name(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(identifier.span().start, name, imported);
        }
    }

    fn walk_in_binary<'ast>(&mut self, binary: &'ast Binary, context: &mut NameContext<'a>) {
        if let (BinaryOperator::Instanceof(_), Expression::Identifier(identifier)) =
            (binary.operator, binary.rhs.as_ref())
        {
            let (name, imported) = context.resolve_name(NameKind::Default, identifier.value());

            self.resolved_names.insert_at(identifier.span().start, name, imported);
        }
    }

    fn walk_in_expression<'ast>(&mut self, expression: &'ast Expression, context: &mut NameContext<'a>) {
        if let Expression::Identifier(identifier) = expression {
            if !self.resolved_names.contains(&identifier.span().start) {
                let (name, imported) = context.resolve_name(NameKind::Constant, identifier.value());

                self.resolved_names.insert_at(identifier.span().start, name, imported);
            }
        }
    }

    fn walk_out_namespace<'ast>(&mut self, _namespace: &'ast Namespace, context: &mut NameContext<'a>) {
        context.exit_namespace();
    }
}
