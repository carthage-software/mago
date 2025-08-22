#![allow(unused_variables)]

use crate::ast::Program;
use crate::ast::ast::*;

/// Helper macro to generate the core walk logic.
macro_rules! define_walk_body {
    ($walker:ident, $context:ident, $var_name:ident, $code:block) => {
        paste::paste! {
            $walker.[<walk_in_ $var_name>]($var_name, $context);
            $code
            $walker.[<walk_out_ $var_name>]($var_name, $context);
        }
    };
}

/// Helper macro to generate trait methods for the mutable walker.
macro_rules! gen_mut_trait_methods {
    // This arm matches nodes that have an arena lifetime.
    ('arena, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $var_name>](&mut self, $var_name: & $ast $node_type<$arena>, context: &mut C) {}
            #[inline]
            fn [<walk_ $var_name>](&mut self, $var_name: & $ast $node_type<$arena>, $context: &mut C) {
                let $walker = self;
                define_walk_body!($walker, $context, $var_name, $code);
            }
            #[inline]
            fn [<walk_out_ $var_name>](&mut self, $var_name: & $ast $node_type<$arena>, context: &mut C) {}
        }
    };
    // This arm matches simple/copy nodes that DO NOT have an arena lifetime.
    (_, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $var_name>](&mut self, $var_name: & $ast $node_type, context: &mut C) {}
            #[inline]
            fn [<walk_ $var_name>](&mut self, $var_name: & $ast $node_type, $context: &mut C) {
                let $walker = self;
                define_walk_body!($walker, $context, $var_name, $code);
            }
            #[inline]
            fn [<walk_out_ $var_name>](&mut self, $var_name: & $ast $node_type, context: &mut C) {}
        }
    };
}

/// Helper macro to generate trait methods for the immutable walker.
macro_rules! gen_const_trait_methods {
    // This arm matches nodes that have an arena lifetime.
    ('arena, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $var_name>](&self, $var_name: & $ast $node_type<$arena>, context: &mut C) {}
            #[inline]
            fn [<walk_ $var_name>](&self, $var_name: & $ast $node_type<$arena>, $context: &mut C) {
                let $walker = self;
                define_walk_body!($walker, $context, $var_name, $code);
            }
            #[inline]
            fn [<walk_out_ $var_name>](&self, $var_name: & $ast $node_type<$arena>, context: &mut C) {}
        }
    };
    // This arm matches simple/copy nodes that DO NOT have an arena lifetime.
    (_, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $var_name>](&self, $var_name: & $ast $node_type, context: &mut C) {}
            #[inline]
            fn [<walk_ $var_name>](&self, $var_name: & $ast $node_type, $context: &mut C) {
                let $walker = self;
                define_walk_body!($walker, $context, $var_name, $code);
            }
            #[inline]
            fn [<walk_out_ $var_name>](&self, $var_name: & $ast $node_type, context: &mut C) {}
        }
    };
}

/// Helper macro to generate standalone walk functions.
macro_rules! gen_standalone_funcs {
    // This arm matches nodes that have an arena lifetime.
    ('arena, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            pub fn [<walk_ $var_name _mut>]<$ast, $arena, W, C>($walker: &mut W, $var_name: & $ast $node_type<$arena>, $context: &mut C)
                where W: ?Sized + MutWalker<$ast, $arena, C>
            {
                define_walk_body!($walker, $context, $var_name, $code);
            }

            #[inline]
            pub fn [<walk_ $var_name>]<$ast, $arena, W, C>($walker: &W, $var_name: & $ast $node_type<$arena>, $context: &mut C)
                where W: ?Sized + Walker<$ast, $arena, C>
            {
                define_walk_body!($walker, $context, $var_name, $code);
            }
        }
    };
    // This arm matches simple/copy nodes that DO NOT have an arena lifetime.
    (_, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            pub fn [<walk_ $var_name _mut>]<$ast, $arena, W, C>($walker: &mut W, $var_name: & $ast $node_type, $context: &mut C)
                where W: ?Sized + MutWalker<$ast, $arena, C>
            {
                define_walk_body!($walker, $context, $var_name, $code);
            }

            #[inline]
            pub fn [<walk_ $var_name>]<$ast, $arena, W, C>($walker: &W, $var_name: & $ast $node_type, $context: &mut C)
                where W: ?Sized + Walker<$ast, $arena, C>
            {
                define_walk_body!($walker, $context, $var_name, $code);
            }
        }
    };
}

/// Macro for generating a walker trait and associated functions for traversing an AST.
macro_rules! generate_ast_walker {
    (
        using($walker:ident, $context:ident, $ast:lifetime, $arena:lifetime):
        $(
            $prefix:tt $node_type:ty as $var_name:ident => $code:block
        )*
    ) => {
        /// A trait that defines a mutable walker to traverse AST nodes.
        pub trait MutWalker<$ast, $arena, C>: Sync + Send {
            $(
                gen_mut_trait_methods!($prefix, $node_type, $var_name, $walker, $context, $ast, $arena, $code);
            )*
        }

        /// A trait that defines an immutable walker to traverse AST nodes.
        pub trait Walker<$ast, $arena, C>: Sync + Send {
            $(
                gen_const_trait_methods!($prefix, $node_type, $var_name, $walker, $context, $ast, $arena, $code);
            )*
        }

        $(
            gen_standalone_funcs!($prefix, $node_type, $var_name, $walker, $context, $ast, $arena, $code);
        )*
    }
}

generate_ast_walker! {
    using(walker, context, 'ast, 'arena):

    'arena Program as program => {
        for statement in program.statements.iter() {
            walker.walk_statement(statement, context);
        }
    }

    'arena Statement as statement => {
        match &statement {
            Statement::OpeningTag(opening_tag) => walker.walk_opening_tag(opening_tag, context),
            Statement::ClosingTag(closing_tag) => walker.walk_closing_tag(closing_tag, context),
            Statement::Inline(inline) => walker.walk_inline(inline, context),
            Statement::Namespace(namespace) => walker.walk_namespace(namespace, context),
            Statement::Use(r#use) => walker.walk_use(r#use, context),
            Statement::Class(class) => walker.walk_class(class, context),
            Statement::Interface(interface) => walker.walk_interface(interface, context),
            Statement::Trait(r#trait) => walker.walk_trait(r#trait, context),
            Statement::Enum(r#enum) => walker.walk_enum(r#enum, context),
            Statement::Block(block) => walker.walk_block(block, context),
            Statement::Constant(constant) => walker.walk_constant(constant, context),
            Statement::Function(function) => walker.walk_function(function, context),
            Statement::Declare(declare) => walker.walk_declare(declare, context),
            Statement::Goto(goto) => walker.walk_goto(goto, context),
            Statement::Label(label) => walker.walk_label(label, context),
            Statement::Try(r#try) => walker.walk_try(r#try, context),
            Statement::Foreach(foreach) => walker.walk_foreach(foreach, context),
            Statement::For(r#for) => walker.walk_for(r#for, context),
            Statement::While(r#while) => walker.walk_while(r#while, context),
            Statement::DoWhile(do_while) => walker.walk_do_while(do_while, context),
            Statement::Continue(r#continue) => walker.walk_continue(r#continue, context),
            Statement::Break(r#break) => walker.walk_break(r#break, context),
            Statement::Switch(switch) => walker.walk_switch(switch, context),
            Statement::If(r#if) => walker.walk_if(r#if, context),
            Statement::Return(r#return) => walker.walk_return(r#return, context),
            Statement::Expression(expression) => walker.walk_statement_expression(expression, context),
            Statement::Echo(echo) => walker.walk_echo(echo, context),
            Statement::Global(global) => walker.walk_global(global, context),
            Statement::Static(r#static) => walker.walk_static(r#static, context),
            Statement::HaltCompiler(halt_compiler) => walker.walk_halt_compiler(halt_compiler, context),
            Statement::Unset(unset) => walker.walk_unset(unset, context),
            Statement::Noop(_) => {
                // Do nothing by default
            },
        }
    }

    'arena OpeningTag as opening_tag => {
        match opening_tag {
            OpeningTag::Full(full_opening_tag) => walker.walk_full_opening_tag(full_opening_tag, context),
            OpeningTag::Short(short_opening_tag) => walker.walk_short_opening_tag(short_opening_tag, context),
            OpeningTag::Echo(echo_opening_tag) => walker.walk_echo_opening_tag(echo_opening_tag, context),
        }
    }

    'arena FullOpeningTag as full_opening_tag => {
        // Do nothing by default
    }

    _ ShortOpeningTag as short_opening_tag => {
        // Do nothing by default
    }

    _ EchoOpeningTag as echo_opening_tag => {
        // Do nothing by default
    }

    _ ClosingTag as closing_tag => {
        // Do nothing by default
    }

    'arena Inline as inline => {
        // Do nothing by default
    }

    'arena Namespace as namespace => {
        walker.walk_keyword(&namespace.namespace, context);
        if let Some(name) = &namespace.name {
            walker.walk_identifier(name, context);
        }

        walker.walk_namespace_body(&namespace.body, context);
    }

    'arena NamespaceBody as namespace_body => {
        match namespace_body {
            NamespaceBody::Implicit(namespace_implicit_body) => walker.walk_namespace_implicit_body(namespace_implicit_body, context),
            NamespaceBody::BraceDelimited(block) => walker.walk_block(block, context),
        }
    }

    'arena NamespaceImplicitBody as namespace_implicit_body => {
        walker.walk_terminator(&namespace_implicit_body.terminator, context);

        for statement in namespace_implicit_body.statements.iter() {
            walker.walk_statement(statement, context);
        }
    }

    'arena Terminator as terminator => {
        match terminator {
            Terminator::Semicolon(_) => {
                // Do nothing by default
            }
            Terminator::ClosingTag(closing_tag) => {
                walker.walk_closing_tag(closing_tag, context);
            }
            Terminator::TagPair(closing_tag, opening_tag) => {
                walker.walk_closing_tag(closing_tag, context);
                walker.walk_opening_tag(opening_tag, context);
            }
        }
    }

    'arena Use as r#use => {
        walker.walk_keyword(&r#use.r#use, context);

        walker.walk_use_items(&r#use.items, context);

        walker.walk_terminator(&r#use.terminator, context);
    }

    'arena UseItems as use_items => {
        match use_items {
            UseItems::Sequence(use_item_sequence) => {
                walker.walk_use_item_sequence(use_item_sequence, context);
            }
            UseItems::TypedSequence(typed_use_item_sequence) => {
                walker.walk_typed_use_item_sequence(typed_use_item_sequence, context);
            }
            UseItems::TypedList(typed_use_item_list) => {
                walker.walk_typed_use_item_list(typed_use_item_list, context);
            }
            UseItems::MixedList(mixed_use_item_list) => {
                walker.walk_mixed_use_item_list(mixed_use_item_list, context);
            }
        }
    }

    'arena UseItemSequence as use_item_sequence => {
        for use_item in use_item_sequence.items.iter() {
            walker.walk_use_item(use_item, context);
        }
    }

    'arena UseItem as use_item => {
        walker.walk_identifier(&use_item.name, context);

        if let Some(alias) = &use_item.alias {
            walker.walk_use_item_alias(alias, context);
        }
    }

    'arena UseItemAlias as use_item_alias => {
        walker.walk_keyword(&use_item_alias.r#as, context);
        walker.walk_local_identifier(&use_item_alias.identifier, context);
    }

    'arena TypedUseItemSequence as typed_use_item_sequence => {
        walker.walk_use_type(&typed_use_item_sequence.r#type, context);

        for use_item in typed_use_item_sequence.items.iter() {
            walker.walk_use_item(use_item, context);
        }
    }

    'arena UseType as use_type => {
        match &use_type {
            UseType::Function(keyword) => walker.walk_keyword(keyword, context),
            UseType::Const(keyword) => walker.walk_keyword(keyword, context),
        }
    }

    'arena TypedUseItemList as typed_use_item_list => {
        walker.walk_use_type(&typed_use_item_list.r#type, context);
        walker.walk_identifier(&typed_use_item_list.namespace, context);

        for use_item in typed_use_item_list.items.iter() {
            walker.walk_use_item(use_item, context);
        }
    }

    'arena MixedUseItemList as mixed_use_item_list => {
        walker.walk_identifier(&mixed_use_item_list.namespace, context);

        for maybe_typed_use_item in mixed_use_item_list.items.iter() {
            walker.walk_maybe_typed_use_item(maybe_typed_use_item, context);
        }
    }

    'arena MaybeTypedUseItem as maybe_typed_use_item => {
        if let Some(use_type) = &maybe_typed_use_item.r#type {
            walker.walk_use_type(use_type, context);
        }

        walker.walk_use_item(&maybe_typed_use_item.item, context);
    }

    'arena AttributeList as attribute_list => {
        for attribute in attribute_list.attributes.iter() {
            walker.walk_attribute(attribute, context);
        }
    }

    'arena Attribute as attribute => {
        walker.walk_identifier(&attribute.name, context);

        if let Some(argument_list) = &attribute.argument_list {
            walker.walk_argument_list(argument_list, context);
        }
    }

    'arena ArgumentList as argument_list => {
        for argument in argument_list.arguments.iter() {
            walker.walk_argument(argument, context);
        }
    }

    'arena Argument as argument => {
        match &argument {
            Argument::Positional(positional_argument) => {
                walker.walk_positional_argument(positional_argument, context);
            }
            Argument::Named(named_argument) => {
                walker.walk_named_argument(named_argument, context);
            }
        }
    }

    'arena PositionalArgument as positional_argument => {
        walker.walk_expression(&positional_argument.value, context);
    }

    'arena NamedArgument as named_argument => {
        walker.walk_local_identifier(&named_argument.name, context);
        walker.walk_expression(&named_argument.value, context);
    }

    'arena Modifier as modifier => {
        walker.walk_keyword(modifier.get_keyword(), context);
    }

    'arena Extends as extends => {
        walker.walk_keyword(&extends.extends, context);

        for ty in extends.types.iter() {
            walker.walk_identifier(ty, context);
        }
    }

    'arena Implements as implements => {
        walker.walk_keyword(&implements.implements, context);

        for ty in implements.types.iter() {
            walker.walk_identifier(ty, context);
        }
    }

    'arena Class as class => {
        for attribute_list in class.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        for modifier in class.modifiers.iter() {
            walker.walk_modifier(modifier, context);
        }

        walker.walk_keyword(&class.class, context);
        walker.walk_local_identifier(&class.name, context);
        if let Some(extends) = &class.extends {
            walker.walk_extends(extends, context);
        }

        if let Some(implements) = &class.implements {
            walker.walk_implements(implements, context);
        }

        for class_member in class.members.iter() {
            walker.walk_class_like_member(class_member, context);
        }
    }

    'arena Interface as interface => {
        for attribute_list in interface.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        walker.walk_keyword(&interface.interface, context);
        walker.walk_local_identifier(&interface.name, context);

        if let Some(extends) = &interface.extends {
            walker.walk_extends(extends, context);
        }

        for class_member in interface.members.iter() {
            walker.walk_class_like_member(class_member, context);
        }
    }

    'arena Trait as r#trait => {
        for attribute_list in r#trait.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        walker.walk_keyword(&r#trait.r#trait, context);
        walker.walk_local_identifier(&r#trait.name, context);

        for class_member in r#trait.members.iter() {
            walker.walk_class_like_member(class_member, context);
        }
    }

    'arena Enum as r#enum => {
        for attribute_list in r#enum.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        walker.walk_keyword(&r#enum.r#enum, context);
        walker.walk_local_identifier(&r#enum.name, context);

        if let Some(backing_type_hint) = &r#enum.backing_type_hint {
            walker.walk_enum_backing_type_hint(backing_type_hint, context);
        }

        if let Some(implements) = &r#enum.implements {
            walker.walk_implements(implements, context);
        }

        for class_member in r#enum.members.iter() {
            walker.walk_class_like_member(class_member, context);
        }
    }

    'arena EnumBackingTypeHint as enum_backing_type_hint => {
        walker.walk_hint(&enum_backing_type_hint.hint, context);
    }

    'arena ClassLikeMember as class_like_member => {
        match class_like_member {
            ClassLikeMember::TraitUse(trait_use) => {
                walker.walk_trait_use(trait_use, context);
            }
            ClassLikeMember::Constant(class_like_constant) => {
                walker.walk_class_like_constant(class_like_constant, context);
            }
            ClassLikeMember::Property(property) => {
                walker.walk_property(property, context);
            }
            ClassLikeMember::EnumCase(enum_case) => {
                walker.walk_enum_case(enum_case, context);
            }
            ClassLikeMember::Method(method) => {
                walker.walk_method(method, context);
            }
        }
    }

    'arena TraitUse as trait_use => {
        walker.walk_keyword(&trait_use.r#use, context);

        for trait_name in trait_use.trait_names.iter() {
            walker.walk_identifier(trait_name, context);
        }

        walker.walk_trait_use_specification(&trait_use.specification, context);
    }

    'arena TraitUseSpecification as trait_use_specification => {
        match trait_use_specification {
            TraitUseSpecification::Abstract(trait_use_abstract_specification) => {
                walker.walk_trait_use_abstract_specification(trait_use_abstract_specification, context);
            }
            TraitUseSpecification::Concrete(trait_use_concrete_specification) => {
                walker.walk_trait_use_concrete_specification(trait_use_concrete_specification, context);
            }
        }
    }

    'arena TraitUseAbstractSpecification as trait_use_abstract_specification => {
        walker.walk_terminator(&trait_use_abstract_specification.0, context);
    }

    'arena TraitUseConcreteSpecification as trait_use_concrete_specification => {
        for adaptation in trait_use_concrete_specification.adaptations.iter() {
            walker.walk_trait_use_adaptation(
                adaptation,

                context,
            );
        }
    }

    'arena TraitUseAdaptation as trait_use_adaptation => {
        match trait_use_adaptation {
            TraitUseAdaptation::Precedence(trait_use_precedence_adaptation) => {
                walker.walk_trait_use_precedence_adaptation(trait_use_precedence_adaptation, context);
            },
            TraitUseAdaptation::Alias(trait_use_alias_adaptation) => {
                walker.walk_trait_use_alias_adaptation(trait_use_alias_adaptation, context);
            },
        }
    }

    'arena TraitUsePrecedenceAdaptation as trait_use_precedence_adaptation => {
        walker.walk_trait_use_absolute_method_reference(
            &trait_use_precedence_adaptation.method_reference,

            context,
        );

        walker.walk_keyword(&trait_use_precedence_adaptation.insteadof, context);

        for trait_name in trait_use_precedence_adaptation.trait_names.iter() {
            walker.walk_identifier(trait_name, context);
        }

        walker.walk_terminator(&trait_use_precedence_adaptation.terminator, context);
    }

    'arena TraitUseAbsoluteMethodReference as trait_use_absolute_method_reference => {
        walker.walk_identifier(&trait_use_absolute_method_reference.trait_name, context);
        walker.walk_local_identifier(&trait_use_absolute_method_reference.method_name, context);
    }

    'arena TraitUseAliasAdaptation as trait_use_alias_adaptation => {
        walker.walk_trait_use_method_reference(
            &trait_use_alias_adaptation.method_reference,

            context,
        );

        walker.walk_keyword(&trait_use_alias_adaptation.r#as, context);

        if let Some(modifier) = &trait_use_alias_adaptation.visibility {
            walker.walk_modifier(modifier, context);
        }

        if let Some(alias) = &trait_use_alias_adaptation.alias {
            walker.walk_local_identifier(alias, context);
        }

        walker.walk_terminator(&trait_use_alias_adaptation.terminator, context);
    }

    'arena TraitUseMethodReference as trait_use_method_reference => {
        match trait_use_method_reference {
            TraitUseMethodReference::Identifier(local_identifier) => {
                walker.walk_local_identifier(local_identifier, context);
            },
            TraitUseMethodReference::Absolute(absolute) => {
                walker.walk_trait_use_absolute_method_reference(absolute, context);
            },
        }
    }

    'arena ClassLikeConstant as class_like_constant => {
        for attribute_list in class_like_constant.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        for modifier in class_like_constant.modifiers.iter() {
            walker.walk_modifier(modifier, context);
        }

        walker.walk_keyword(&class_like_constant.r#const, context);

        if let Some(hint) = &class_like_constant.hint {
            walker.walk_hint(hint, context);
        }

        for item in class_like_constant.items.iter() {
            walker.walk_class_like_constant_item(item, context);
        }

        walker.walk_terminator(&class_like_constant.terminator, context);
    }

    'arena ClassLikeConstantItem as class_like_constant_item => {
        walker.walk_local_identifier(&class_like_constant_item.name, context);
        walker.walk_expression(&class_like_constant_item.value, context);
    }

    'arena Property as property => {
        match property {
            Property::Plain(plain_property) => {
                walker.walk_plain_property(plain_property, context);
            }
            Property::Hooked(hooked_property) => {
                walker.walk_hooked_property(hooked_property, context);
            }
        }
    }

    'arena PlainProperty as plain_property => {
        for attribute_list in plain_property.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        for modifier in plain_property.modifiers.iter() {
            walker.walk_modifier(modifier, context);
        }

        if let Some(var) = &plain_property.var {
            walker.walk_keyword(var, context);
        }

        if let Some(hint) = &plain_property.hint {
            walker.walk_hint(hint, context);
        }

        for item in plain_property.items.iter() {
            walker.walk_property_item(item, context);
        }

        walker.walk_terminator(&plain_property.terminator, context);
    }

    'arena PropertyItem as property_item => {
        match property_item {
            PropertyItem::Abstract(property_abstract_item) => {
                walker.walk_property_abstract_item(property_abstract_item, context);
            }
            PropertyItem::Concrete(property_concrete_item) => {
                walker.walk_property_concrete_item(property_concrete_item, context);
            }
        }
    }

    'arena PropertyAbstractItem as property_abstract_item => {
        walker.walk_direct_variable(&property_abstract_item.variable, context);
    }

    'arena PropertyConcreteItem as property_concrete_item => {
        walker.walk_direct_variable(&property_concrete_item.variable, context);
        walker.walk_expression(&property_concrete_item.value, context);
    }

    'arena HookedProperty as hooked_property => {
        for attribute_list in hooked_property.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        for modifier in hooked_property.modifiers.iter() {
            walker.walk_modifier(modifier, context);
        }

        if let Some(var) = &hooked_property.var {
            walker.walk_keyword(var, context);
        }

        if let Some(hint) = &hooked_property.hint {
            walker.walk_hint(hint, context);
        }

        walker.walk_property_item(&hooked_property.item, context);
        walker.walk_property_hook_list(&hooked_property.hook_list, context);
    }

    'arena PropertyHookList as property_hook_list => {
        for hook in property_hook_list.hooks.iter() {
            walker.walk_property_hook(hook, context);
        }
    }

    'arena PropertyHook as property_hook => {
        for attribute_list in property_hook.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        for modifier in property_hook.modifiers.iter() {
            walker.walk_modifier(modifier, context);
        }

        walker.walk_local_identifier(&property_hook.name, context);
        if let Some(parameters) = &property_hook.parameters {
            walker.walk_function_like_parameter_list(parameters, context);
        }

        walker.walk_property_hook_body(&property_hook.body, context);
    }

    'arena PropertyHookBody as property_hook_body => {
        match property_hook_body {
            PropertyHookBody::Abstract(property_hook_abstract_body) => {
                walker.walk_property_hook_abstract_body(property_hook_abstract_body, context);
            }
            PropertyHookBody::Concrete(property_hook_concrete_body) => {
                walker.walk_property_hook_concrete_body(property_hook_concrete_body, context);
            }
        }
    }

    _ PropertyHookAbstractBody as property_hook_abstract_body => {
        // Do nothing by default
    }

    'arena PropertyHookConcreteBody as property_hook_concrete_body => {
        match property_hook_concrete_body {
            PropertyHookConcreteBody::Block(block) => {
                walker.walk_block(block, context);
            }
            PropertyHookConcreteBody::Expression(property_hook_concrete_expression_body) => {
                walker.walk_property_hook_concrete_expression_body(property_hook_concrete_expression_body, context);
            }
        }
    }

    'arena PropertyHookConcreteExpressionBody as property_hook_concrete_expression_body => {
        walker.walk_expression(&property_hook_concrete_expression_body.expression, context);
    }

    'arena FunctionLikeParameterList as function_like_parameter_list => {
        for parameter in function_like_parameter_list.parameters.iter() {
            walker.walk_function_like_parameter(parameter, context);
        }
    }

    'arena FunctionLikeParameter as function_like_parameter => {
        for attribute_list in function_like_parameter.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        for modifier in function_like_parameter.modifiers.iter() {
            walker.walk_modifier(modifier, context);
        }

        if let Some(hint) = &function_like_parameter.hint {
            walker.walk_hint(hint, context);
        }

        walker.walk_direct_variable(&function_like_parameter.variable, context);
        if let Some(default_value) = &function_like_parameter.default_value {
            walker.walk_function_like_parameter_default_value(default_value, context);
        }

        if let Some(hooks) = &function_like_parameter.hooks {
            walker.walk_property_hook_list(hooks, context);
        }
    }

    'arena FunctionLikeParameterDefaultValue as function_like_parameter_default_value => {
        walker.walk_expression(&function_like_parameter_default_value.value, context);
    }

    'arena EnumCase as enum_case => {
        for attribute_list in enum_case.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        walker.walk_keyword(&enum_case.case, context);
        walker.walk_enum_case_item(&enum_case.item, context);
        walker.walk_terminator(&enum_case.terminator, context);
    }

    'arena EnumCaseItem as enum_case_item => {
        match enum_case_item {
            EnumCaseItem::Unit(enum_case_unit_item) => {
                walker.walk_enum_case_unit_item(enum_case_unit_item, context);
            }
            EnumCaseItem::Backed(enum_case_backed_item) => {
                walker.walk_enum_case_backed_item(enum_case_backed_item, context);
            }
        }
    }

    'arena EnumCaseUnitItem as enum_case_unit_item => {
        walker.walk_local_identifier(&enum_case_unit_item.name, context);
    }

    'arena EnumCaseBackedItem as enum_case_backed_item => {
        walker.walk_local_identifier(&enum_case_backed_item.name, context);
        walker.walk_expression(&enum_case_backed_item.value, context);
    }

    'arena Method as method => {
        for attribute_list in method.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        for modifier in method.modifiers.iter() {
            walker.walk_modifier(modifier, context);
        }

        walker.walk_keyword(&method.function, context);
        walker.walk_local_identifier(&method.name, context);
        walker.walk_function_like_parameter_list(&method.parameter_list, context);
        if let Some(hint) = &method.return_type_hint {
            walker.walk_function_like_return_type_hint(hint, context);
        }

        walker.walk_method_body(&method.body, context);
    }

    'arena MethodBody as method_body => {
        match method_body {
            MethodBody::Abstract(method_abstract_body) => {
                walker.walk_method_abstract_body(method_abstract_body, context);
            }
            MethodBody::Concrete(method_concrete_body) => {
                walker.walk_block(method_concrete_body, context);
            }
        }
    }

    _ MethodAbstractBody as method_abstract_body => {
        // Do nothing by default
    }

    'arena FunctionLikeReturnTypeHint as function_like_return_type_hint => {
        walker.walk_hint(&function_like_return_type_hint.hint, context);
    }

    'arena Block as block => {
        for statement in block.statements.iter() {
            walker.walk_statement(statement, context);
        }
    }

    'arena Constant as constant => {
        for attribute_list in constant.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        walker.walk_keyword(&constant.r#const, context);
        for item in constant.items.iter() {
            walker.walk_constant_item(item, context);
        }

        walker.walk_terminator(&constant.terminator, context);
    }

    'arena ConstantItem as constant_item => {
        walker.walk_local_identifier(&constant_item.name, context);
        walker.walk_expression(&constant_item.value, context);
    }

    'arena Function as function => {
        for attribute_list in function.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        walker.walk_keyword(&function.function, context);
        walker.walk_local_identifier(&function.name, context);
        walker.walk_function_like_parameter_list(&function.parameter_list, context);
        if let Some(hint) = &function.return_type_hint {
            walker.walk_function_like_return_type_hint(hint, context);
        }

        walker.walk_block(&function.body, context);
    }

    'arena Declare as declare => {
        walker.walk_keyword(&declare.declare, context);
        for item in declare.items.iter() {
            walker.walk_declare_item(item, context);
        }

        walker.walk_declare_body(&declare.body, context);
    }

    'arena DeclareItem as declare_item => {
        walker.walk_local_identifier(&declare_item.name, context);
        walker.walk_expression(&declare_item.value, context);
    }

    'arena DeclareBody as declare_body => {
        match declare_body {
            DeclareBody::Statement(statement) => {
                walker.walk_statement(statement, context);
            }
            DeclareBody::ColonDelimited(declare_colon_delimited_body) => {
                walker.walk_declare_colon_delimited_body(declare_colon_delimited_body, context);
            }
        }
    }

    'arena DeclareColonDelimitedBody as declare_colon_delimited_body => {
        for statement in declare_colon_delimited_body.statements.iter() {
            walker.walk_statement(statement, context);
        }

        walker.walk_terminator(&declare_colon_delimited_body.terminator, context);
    }

    'arena Goto as goto => {
        walker.walk_keyword(&goto.goto, context);
        walker.walk_local_identifier(&goto.label, context);
        walker.walk_terminator(&goto.terminator, context);
    }

    'arena Label as label => {
        walker.walk_local_identifier(&label.name, context);
    }

    'arena Try as r#try => {
        walker.walk_keyword(&r#try.r#try, context);
        walker.walk_block(&r#try.block, context);
        for catch in r#try.catch_clauses.iter() {
            walker.walk_try_catch_clause(catch, context);
        }

        if let Some(finally) = &r#try.finally_clause {
            walker.walk_try_finally_clause(finally, context);
        }
    }

    'arena TryCatchClause as try_catch_clause => {
        walker.walk_keyword(&try_catch_clause.catch, context);
        walker.walk_hint(&try_catch_clause.hint, context);
        if let Some(variable) = &try_catch_clause.variable {
            walker.walk_direct_variable(variable, context);
        }

        walker.walk_block(&try_catch_clause.block, context);
    }

    'arena TryFinallyClause as try_finally_clause => {
        walker.walk_keyword(&try_finally_clause.finally, context);
        walker.walk_block(&try_finally_clause.block, context);
    }

    'arena Foreach as foreach => {
        walker.walk_keyword(&foreach.foreach, context);
        walker.walk_expression(foreach.expression, context);
        walker.walk_keyword(&foreach.r#as, context);
        walker.walk_foreach_target(&foreach.target, context);
        walker.walk_foreach_body(&foreach.body, context);
    }

    'arena ForeachTarget as foreach_target => {
        match foreach_target {
            ForeachTarget::Value(foreach_value_target) => {
                walker.walk_foreach_value_target(foreach_value_target, context);
            }
            ForeachTarget::KeyValue(foreach_key_value_target) => {
                walker.walk_foreach_key_value_target(foreach_key_value_target, context);
            }
        }
    }

    'arena ForeachValueTarget as foreach_value_target => {
        walker.walk_expression(foreach_value_target.value, context);
    }

    'arena ForeachKeyValueTarget as foreach_key_value_target => {
        walker.walk_expression(foreach_key_value_target.key, context);
        walker.walk_expression(foreach_key_value_target.value, context);
    }

    'arena ForeachBody as foreach_body => {
        match foreach_body {
            ForeachBody::Statement(statement) => {
                walker.walk_statement(statement, context);
            }
            ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                walker.walk_foreach_colon_delimited_body(foreach_colon_delimited_body, context);
            }
        }
    }

    'arena ForeachColonDelimitedBody as foreach_colon_delimited_body => {
        for statement in foreach_colon_delimited_body.statements.iter() {
            walker.walk_statement(statement, context);
        }

        walker.walk_keyword(&foreach_colon_delimited_body.end_foreach, context);
        walker.walk_terminator(&foreach_colon_delimited_body.terminator, context);
    }

    'arena For as r#for => {
        walker.walk_keyword(&r#for.r#for, context);

        for initialization in r#for.initializations.iter() {
            walker.walk_expression(initialization, context);
        }

        for condition in r#for.conditions.iter() {
            walker.walk_expression(condition, context);
        }

        for increment in r#for.increments.iter() {
            walker.walk_expression(increment, context);
        }

        walker.walk_for_body(&r#for.body, context);
    }

    'arena ForBody as for_body => {
        match for_body {
            ForBody::Statement(statement) => {
                walker.walk_statement(statement, context);
            }
            ForBody::ColonDelimited(for_colon_delimited_body) => {
                walker.walk_for_colon_delimited_body(for_colon_delimited_body, context);
            }
        }
    }

    'arena ForColonDelimitedBody as for_colon_delimited_body => {
        for statement in for_colon_delimited_body.statements.iter() {
            walker.walk_statement(statement, context);
        }

        walker.walk_keyword(&for_colon_delimited_body.end_for, context);
        walker.walk_terminator(&for_colon_delimited_body.terminator, context);
    }

    'arena While as r#while => {
        walker.walk_keyword(&r#while.r#while, context);
        walker.walk_expression(r#while.condition, context);
        walker.walk_while_body(&r#while.body, context);
    }

    'arena WhileBody as while_body => {
        match while_body {
            WhileBody::Statement(statement) => {
                walker.walk_statement(statement, context);
            }
            WhileBody::ColonDelimited(while_colon_delimited_body) => {
                walker.walk_while_colon_delimited_body(while_colon_delimited_body, context);
            }
        }
    }

    'arena WhileColonDelimitedBody as while_colon_delimited_body => {
        for statement in while_colon_delimited_body.statements.iter() {
            walker.walk_statement(statement, context);
        }

        walker.walk_keyword(&while_colon_delimited_body.end_while, context);
        walker.walk_terminator(&while_colon_delimited_body.terminator, context);
    }

    'arena DoWhile as do_while => {
        walker.walk_keyword(&do_while.r#do, context);
        walker.walk_statement(do_while.statement, context);
        walker.walk_keyword(&do_while.r#while, context);
        walker.walk_expression(do_while.condition, context);
        walker.walk_terminator(&do_while.terminator, context);
    }

    'arena Continue as r#continue => {
        walker.walk_keyword(&r#continue.r#continue, context);
        if let Some(level) = &r#continue.level {
            walker.walk_expression(level, context);
        }

        walker.walk_terminator(&r#continue.terminator, context);
    }

    'arena Break as r#break => {
        walker.walk_keyword(&r#break.r#break, context);
        if let Some(level) = &r#break.level {
            walker.walk_expression(level, context);
        }

        walker.walk_terminator(&r#break.terminator, context);
    }

    'arena Switch as switch => {
        walker.walk_keyword(&switch.r#switch, context);
        walker.walk_expression(switch.expression, context);
        walker.walk_switch_body(&switch.body, context);
    }

    'arena SwitchBody as switch_body => {
        match switch_body {
            SwitchBody::BraceDelimited(switch_brace_delimited_body) => {
                walker.walk_switch_brace_delimited_body(switch_brace_delimited_body, context);
            }
            SwitchBody::ColonDelimited(switch_colon_delimited_body) => {
                walker.walk_switch_colon_delimited_body(switch_colon_delimited_body, context);
            }
        }
    }

    'arena SwitchBraceDelimitedBody as switch_brace_delimited_body => {
        if let Some(terminator) = &switch_brace_delimited_body.optional_terminator {
            walker.walk_terminator(terminator, context);
        }

        for case in switch_brace_delimited_body.cases.iter() {
            walker.walk_switch_case(case, context);
        }
    }

    'arena SwitchColonDelimitedBody as switch_colon_delimited_body => {
        if let Some(terminator) = &switch_colon_delimited_body.optional_terminator {
            walker.walk_terminator(terminator, context);
        }

        for case in switch_colon_delimited_body.cases.iter() {
            walker.walk_switch_case(case, context);
        }

        walker.walk_keyword(&switch_colon_delimited_body.end_switch, context);
        walker.walk_terminator(&switch_colon_delimited_body.terminator, context);
    }

    'arena SwitchCase as switch_case => {
        match switch_case {
            SwitchCase::Expression(switch_expression_case) => {
                walker.walk_switch_expression_case(switch_expression_case, context);
            }
            SwitchCase::Default(switch_default_case) => {
                walker.walk_switch_default_case(switch_default_case, context);
            }
        }
    }

    'arena SwitchExpressionCase as switch_expression_case => {
        walker.walk_keyword(&switch_expression_case.r#case, context);
        walker.walk_expression(switch_expression_case.expression, context);
        walker.walk_switch_case_separator(&switch_expression_case.separator, context);
        for statement in switch_expression_case.statements.iter() {
            walker.walk_statement(statement, context);
        }
    }

    'arena SwitchDefaultCase as switch_default_case => {
        walker.walk_keyword(&switch_default_case.r#default, context);
        walker.walk_switch_case_separator(&switch_default_case.separator, context);
        for statement in switch_default_case.statements.iter() {
            walker.walk_statement(statement, context);
        }
    }

    _ SwitchCaseSeparator as switch_case_separator => {
        // Do nothing by default
    }

    'arena If as r#if => {
        walker.walk_keyword(&r#if.r#if, context);
        walker.walk_expression(r#if.condition, context);
        walker.walk_if_body(&r#if.body, context);
    }

    'arena IfBody as if_body => {
        match if_body {
            IfBody::Statement(statement) => {
                walker.walk_if_statement_body(statement, context);
            }
            IfBody::ColonDelimited(if_colon_delimited_body) => {
                walker.walk_if_colon_delimited_body(if_colon_delimited_body, context);
            }
        }
    }

    'arena IfStatementBody as if_statement_body => {
        walker.walk_statement(if_statement_body.statement, context);

        for else_if_clause in if_statement_body.else_if_clauses.iter() {
            walker.walk_if_statement_body_else_if_clause(else_if_clause, context);
        }

        if let Some(else_clause) = &if_statement_body.else_clause {
            walker.walk_if_statement_body_else_clause(else_clause, context);
        }
    }

    'arena IfStatementBodyElseIfClause as if_statement_body_else_if_clause => {
        walker.walk_keyword(&if_statement_body_else_if_clause.r#elseif, context);
        walker.walk_expression(if_statement_body_else_if_clause.condition, context);
        walker.walk_statement(if_statement_body_else_if_clause.statement, context);
    }

    'arena IfStatementBodyElseClause as if_statement_body_else_clause => {
        walker.walk_keyword(&if_statement_body_else_clause.r#else, context);
        walker.walk_statement(if_statement_body_else_clause.statement, context);
    }

    'arena IfColonDelimitedBody as if_colon_delimited_body => {
        for statement in if_colon_delimited_body.statements.iter() {
            walker.walk_statement(statement, context);
        }

        for else_if_clause in if_colon_delimited_body.else_if_clauses.iter() {
            walker.walk_if_colon_delimited_body_else_if_clause(else_if_clause, context);
        }

        if let Some(else_clause) = &if_colon_delimited_body.else_clause {
            walker.walk_if_colon_delimited_body_else_clause(else_clause, context);
        }

        walker.walk_keyword(&if_colon_delimited_body.endif, context);
        walker.walk_terminator(&if_colon_delimited_body.terminator, context);
    }

    'arena IfColonDelimitedBodyElseIfClause as if_colon_delimited_body_else_if_clause => {
        walker.walk_keyword(&if_colon_delimited_body_else_if_clause.r#elseif, context);
        walker.walk_expression(if_colon_delimited_body_else_if_clause.condition, context);
        for statement in if_colon_delimited_body_else_if_clause.statements.iter() {
            walker.walk_statement(statement, context);
        }
    }

    'arena IfColonDelimitedBodyElseClause as if_colon_delimited_body_else_clause => {
        walker.walk_keyword(&if_colon_delimited_body_else_clause.r#else, context);
        for statement in if_colon_delimited_body_else_clause.statements.iter() {
            walker.walk_statement(statement, context);
        }
    }

    'arena Return as r#return => {
        walker.walk_keyword(&r#return.r#return, context);
        if let Some(expression) = &r#return.value {
            walker.walk_expression(expression, context);
        }

        walker.walk_terminator(&r#return.terminator, context);
    }

    'arena ExpressionStatement as statement_expression => {
        walker.walk_expression(statement_expression.expression, context);
        walker.walk_terminator(&statement_expression.terminator, context);
    }

    'arena Echo as echo => {
        walker.walk_keyword(&echo.echo, context);
        for expression in echo.values.iter() {
            walker.walk_expression(expression, context);
        }

        walker.walk_terminator(&echo.terminator, context);
    }

    'arena Global as global => {
        walker.walk_keyword(&global.global, context);
        for variable in global.variables.iter() {
            walker.walk_variable(variable, context);
        }

        walker.walk_terminator(&global.terminator, context);
    }

    'arena Static as r#static => {
        walker.walk_keyword(&r#static.r#static, context);
        for item in r#static.items.iter() {
            walker.walk_static_item(item, context);
        }

        walker.walk_terminator(&r#static.terminator, context);
    }

    'arena StaticItem as static_item => {
        match static_item {
            StaticItem::Abstract(static_abstract_item) => {
                walker.walk_static_abstract_item(static_abstract_item, context);
            }
            StaticItem::Concrete(static_concrete_item) => {
                walker.walk_static_concrete_item(static_concrete_item, context);
            }
        }
    }

    'arena StaticAbstractItem as static_abstract_item => {
        walker.walk_direct_variable(&static_abstract_item.variable, context);
    }

    'arena StaticConcreteItem as static_concrete_item => {
        walker.walk_direct_variable(&static_concrete_item.variable, context);
        walker.walk_expression(&static_concrete_item.value, context);
    }

    'arena HaltCompiler as halt_compiler => {
        walker.walk_keyword(&halt_compiler.halt_compiler, context);
    }

    'arena Unset as unset => {
        walker.walk_keyword(&unset.unset, context);
        for value in unset.values.iter() {
            walker.walk_expression(value, context);
        }

        walker.walk_terminator(&unset.terminator, context);
    }

    'arena Expression as expression => {
        match &expression {
            Expression::Parenthesized(parenthesized) => walker.walk_parenthesized(parenthesized, context),
            Expression::Binary(expr) => walker.walk_binary(expr, context),
            Expression::UnaryPrefix(operation) => walker.walk_unary_prefix(operation, context),
            Expression::UnaryPostfix(operation) => walker.walk_unary_postfix(operation, context),
            Expression::Literal(literal) => walker.walk_literal_expression(literal, context),
            Expression::CompositeString(string) => walker.walk_composite_string(string, context),
            Expression::Assignment(assignment) => {
                walker.walk_assignment(assignment, context)
            }
            Expression::Conditional(conditional) => {
                walker.walk_conditional(conditional, context)
            }
            Expression::Array(array) => walker.walk_array(array, context),
            Expression::LegacyArray(legacy_array) => walker.walk_legacy_array(legacy_array, context),
            Expression::List(list) => walker.walk_list(list, context),
            Expression::ArrayAccess(array_access) => walker.walk_array_access(array_access, context),
            Expression::ArrayAppend(array_append) => walker.walk_array_append(array_append, context),
            Expression::AnonymousClass(anonymous_class) => {
                walker.walk_anonymous_class(anonymous_class, context)
            }
            Expression::Closure(closure) => walker.walk_closure(closure, context),
            Expression::ArrowFunction(arrow_function) => walker.walk_arrow_function(arrow_function, context),
            Expression::Variable(variable) => walker.walk_variable(variable, context),
            Expression::Identifier(identifier) => walker.walk_identifier(identifier, context),
            Expression::Match(r#match) => walker.walk_match(r#match, context),
            Expression::Yield(r#yield) => walker.walk_yield(r#yield, context),
            Expression::Construct(construct) => walker.walk_construct(construct, context),
            Expression::Throw(throw) => walker.walk_throw(throw, context),
            Expression::Clone(clone) => walker.walk_clone(clone, context),
            Expression::Call(call) => walker.walk_call(call, context),
            Expression::Access(access) => walker.walk_access(access, context),
            Expression::ConstantAccess(expr) => walker.walk_constant_access(expr, context),
            Expression::ClosureCreation(closure_creation) => {
                walker.walk_closure_creation(closure_creation, context)
            }
            Expression::Parent(keyword) => walker.walk_parent_keyword(keyword, context),
            Expression::Static(keyword) => walker.walk_static_keyword(keyword, context),
            Expression::Self_(keyword) => walker.walk_self_keyword(keyword, context),
            Expression::Instantiation(instantiation) => walker.walk_instantiation(instantiation, context),
            Expression::MagicConstant(magic_constant) => walker.walk_magic_constant(magic_constant, context),
            Expression::Pipe(pipe) => walker.walk_pipe(pipe, context),
        }
    }

    'arena Binary as binary => {
        walker.walk_expression(binary.lhs, context);
        walker.walk_binary_operator(&binary.operator, context);
        walker.walk_expression(binary.rhs, context);
    }

    'arena BinaryOperator as binary_operator => {
        match binary_operator {
            BinaryOperator::Instanceof(keyword)
            | BinaryOperator::LowAnd(keyword)
            | BinaryOperator::LowOr(keyword)
            | BinaryOperator::LowXor(keyword) => {
                walker.walk_keyword(keyword, context);
            }
            _ => {}
        }
    }

    'arena UnaryPrefix as unary_prefix => {
        walker.walk_unary_prefix_operator(&unary_prefix.operator, context);
        walker.walk_expression(unary_prefix.operand, context);
    }

    'arena UnaryPrefixOperator as unary_prefix_operator => {
        // Do nothing
    }

    'arena UnaryPostfix as unary_postfix => {
        walker.walk_expression(unary_postfix.operand, context);
        walker.walk_unary_postfix_operator(&unary_postfix.operator, context);
    }

    _ UnaryPostfixOperator as unary_postfix_operator => {
        // Do nothing
    }

    'arena Parenthesized as parenthesized => {
        walker.walk_expression(parenthesized.expression, context)
    }

    'arena Literal as literal_expression => {
        match literal_expression {
            Literal::String(string) => walker.walk_literal_string(string, context),
            Literal::Integer(integer) => walker.walk_literal_integer(integer, context),
            Literal::Float(float) => walker.walk_literal_float(float, context),
            Literal::True(keyword) => walker.walk_true_keyword(keyword, context),
            Literal::False(keyword) => walker.walk_false_keyword(keyword, context),
            Literal::Null(keyword) => walker.walk_null_keyword(keyword, context),
        }
    }

    'arena LiteralString as literal_string => {
        // Do nothing by default
    }

    'arena LiteralInteger as literal_integer => {
        // Do nothing by default
    }

    'arena LiteralFloat as literal_float => {
        // Do nothing by default
    }

    'arena Keyword as true_keyword => {
        // Do nothing by default
    }

    'arena Keyword as false_keyword => {
        // Do nothing by default
    }

    'arena Keyword as null_keyword => {
        // Do nothing by default
    }

    'arena CompositeString as composite_string => {
        match composite_string {
            CompositeString::ShellExecute(str) => walker.walk_shell_execute_string(str, context),
            CompositeString::Interpolated(str) => walker.walk_interpolated_string(str, context),
            CompositeString::Document(str) => walker.walk_document_string(str, context),
        }
    }

    'arena ShellExecuteString as shell_execute_string => {
        for part in shell_execute_string.parts.iter() {
            walker.walk_string_part(part, context);
        }
    }

    'arena InterpolatedString as interpolated_string => {
        for part in interpolated_string.parts.iter() {
            walker.walk_string_part(part, context);
        }
    }

    'arena DocumentString as document_string => {
        for part in document_string.parts.iter() {
            walker.walk_string_part(part, context);
        }
    }

    'arena StringPart as string_part => {
        match string_part {
            StringPart::Literal(literal) => walker.walk_literal_string_part(literal, context),
            StringPart::Expression(expression) => walker.walk_expression(expression, context),
            StringPart::BracedExpression(braced_expression_string_part) => {
                walker.walk_braced_expression_string_part(braced_expression_string_part, context)
            }
        };
    }

    'arena LiteralStringPart as literal_string_part => {
        // Do nothing
    }

    'arena BracedExpressionStringPart as braced_expression_string_part => {
        walker.walk_expression(braced_expression_string_part.expression, context);
    }

    'arena Assignment as assignment => {
        walker.walk_expression(assignment.lhs, context);
        walker.walk_assignment_operator(&assignment.operator, context);
        walker.walk_expression(assignment.rhs, context);
    }

    _ AssignmentOperator as assignment_operator => {
        // Do nothing
    }

    'arena Conditional as conditional => {
        walker.walk_expression(conditional.condition, context);
        if let Some(then) = &conditional.then {
            walker.walk_expression(then, context);
        }

        walker.walk_expression(conditional.r#else, context);
    }

    'arena Array as array => {
        for element in array.elements.iter() {
            walker.walk_array_element(element, context);
        }
    }

    'arena ArrayElement as array_element => {
        match array_element {
            ArrayElement::KeyValue(key_value_array_element) => {
                walker.walk_key_value_array_element(key_value_array_element, context);
            }
            ArrayElement::Value(value_array_element) => {
                walker.walk_value_array_element(value_array_element, context);
            }
            ArrayElement::Variadic(variadic_array_element) => {
                walker.walk_variadic_array_element(variadic_array_element, context);
            }
            ArrayElement::Missing(missing_array_element) => {
                walker.walk_missing_array_element(missing_array_element, context);
            }
        }
    }

    'arena KeyValueArrayElement as key_value_array_element => {
        walker.walk_expression(key_value_array_element.key, context);
        walker.walk_expression(key_value_array_element.value, context);
    }

    'arena ValueArrayElement as value_array_element => {
        walker.walk_expression(value_array_element.value, context);
    }

    'arena VariadicArrayElement as variadic_array_element => {
        walker.walk_expression(variadic_array_element.value, context);
    }

    _ MissingArrayElement as missing_array_element => {
        // Do nothing
    }

    'arena LegacyArray as legacy_array => {
        walker.walk_keyword(&legacy_array.array, context);
        for element in legacy_array.elements.iter() {
            walker.walk_array_element(element, context);
        }
    }

    'arena List as list => {
        walker.walk_keyword(&list.list, context);

        for element in list.elements.iter() {
            walker.walk_array_element(element, context);
        }
    }

    'arena ArrayAccess as array_access => {
        walker.walk_expression(array_access.array, context);
        walker.walk_expression(array_access.index, context);
    }

    'arena ArrayAppend as array_append => {
        walker.walk_expression(array_append.array, context);
    }

    'arena AnonymousClass as anonymous_class => {
        for attribute_list in anonymous_class.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        for modifier in anonymous_class.modifiers.iter() {
            walker.walk_modifier(modifier, context);
        }

        walker.walk_keyword(&anonymous_class.new, context);
        walker.walk_keyword(&anonymous_class.class, context);
        if let Some(argument_list) = &anonymous_class.argument_list {
            walker.walk_argument_list(argument_list, context);
        }

        if let Some(extends) = &anonymous_class.extends {
            walker.walk_extends(extends, context);
        }

        if let Some(implements) = &anonymous_class.implements {
            walker.walk_implements(implements, context);
        }

        for class_member in anonymous_class.members.iter() {
            walker.walk_class_like_member(class_member, context);
        }
    }

    'arena Closure as closure => {
        for attribute_list in closure.attribute_lists.iter() {
                walker.walk_attribute_list(attribute_list, context);
            }

        if let Some(keyword) = &closure.r#static {
            walker.walk_keyword(keyword, context);
        }

        walker.walk_keyword(&closure.function, context);
        walker.walk_function_like_parameter_list(&closure.parameter_list, context);
        if let Some(use_clause) = &closure.use_clause {
            walker.walk_closure_use_clause(use_clause, context);
        }

        if let Some(return_type_hint) = &closure.return_type_hint {
            walker.walk_function_like_return_type_hint(return_type_hint, context);
        }

        walker.walk_block(&closure.body, context);
    }

    'arena ClosureUseClause as closure_use_clause => {
        for variable in closure_use_clause.variables.iter() {
            walker.walk_closure_use_clause_variable(variable, context);
        }
    }

    'arena ClosureUseClauseVariable as closure_use_clause_variable => {
        walker.walk_direct_variable(&closure_use_clause_variable.variable, context);
    }

    'arena ArrowFunction as arrow_function => {
        for attribute_list in arrow_function.attribute_lists.iter() {
            walker.walk_attribute_list(attribute_list, context);
        }

        if let Some(keyword) = &arrow_function.r#static {
            walker.walk_keyword(keyword, context);
        }

        walker.walk_keyword(&arrow_function.r#fn, context);
        walker.walk_function_like_parameter_list(&arrow_function.parameter_list, context);

        if let Some(return_type_hint) = &arrow_function.return_type_hint {
            walker.walk_function_like_return_type_hint(return_type_hint, context);
        }

        walker.walk_expression(arrow_function.expression, context);
    }

    'arena Variable as variable => {
        match variable {
            Variable::Direct(direct_variable) => {
                walker.walk_direct_variable(direct_variable, context);
            }
            Variable::Indirect(indirect_variable) => {
                walker.walk_indirect_variable(indirect_variable, context);
            }
            Variable::Nested(nested_variable) => {
                walker.walk_nested_variable(nested_variable, context);
            }
        }
    }

    'arena DirectVariable as direct_variable => {
        // Do nothing by default
    }

    'arena IndirectVariable as indirect_variable => {
        walker.walk_expression(indirect_variable.expression, context);
    }

    'arena NestedVariable as nested_variable => {
        walker.walk_variable(nested_variable.variable, context);
    }

    'arena Identifier as identifier => {
        match identifier {
            Identifier::Local(local_identifier) => walker.walk_local_identifier(local_identifier, context),
            Identifier::Qualified(qualified_identifier) => walker.walk_qualified_identifier(qualified_identifier, context),
            Identifier::FullyQualified(fully_qualified_identifier) => walker.walk_fully_qualified_identifier(fully_qualified_identifier, context),
        };
    }

    'arena LocalIdentifier as local_identifier => {
        // Do nothing by default
    }

    'arena QualifiedIdentifier as qualified_identifier => {
        // Do nothing by default
    }

    'arena FullyQualifiedIdentifier as fully_qualified_identifier => {
        // Do nothing by default
    }

    'arena Match as r#match => {
        walker.walk_keyword(&r#match.r#match, context);
        walker.walk_expression(r#match.expression, context);
        for arm in r#match.arms.iter() {
            walker.walk_match_arm(arm, context);
        }
    }

    'arena MatchArm as match_arm => {
        match match_arm {
            MatchArm::Expression(expression_match_arm) => {
                walker.walk_match_expression_arm(expression_match_arm, context);
            }
            MatchArm::Default(default_match_arm) => {
                walker.walk_match_default_arm(default_match_arm, context);
            }
        }
    }

    'arena MatchExpressionArm as match_expression_arm => {
        for condition in match_expression_arm.conditions.iter() {
            walker.walk_expression(condition, context);
        }

        walker.walk_expression(match_expression_arm.expression, context);
    }

    'arena MatchDefaultArm as match_default_arm => {
        walker.walk_keyword(&match_default_arm.r#default, context);
        walker.walk_expression(match_default_arm.expression, context);
    }

    'arena Yield as r#yield => {
        match r#yield {
            Yield::Value(yield_value) => {
                walker.walk_yield_value(yield_value, context);
            }
            Yield::Pair(yield_pair) => {
                walker.walk_yield_pair(yield_pair, context);
            }
            Yield::From(yield_from) => {
                walker.walk_yield_from(yield_from, context);
            }
        }
    }

    'arena YieldValue as yield_value => {
        walker.walk_keyword(&yield_value.r#yield, context);

        if let Some(value) = &yield_value.value {
            walker.walk_expression(value, context);
        }
    }

    'arena YieldPair as yield_pair => {
        walker.walk_keyword(&yield_pair.r#yield, context);
        walker.walk_expression(yield_pair.key, context);
        walker.walk_expression(yield_pair.value, context);
    }

    'arena YieldFrom as yield_from => {
        walker.walk_keyword(&yield_from.r#yield, context);
        walker.walk_keyword(&yield_from.from, context);
        walker.walk_expression(yield_from.iterator, context);
    }

    'arena Construct as construct => {
        match construct {
            Construct::Isset(isset_construct) => {
                walker.walk_isset_construct(isset_construct, context);
            }
            Construct::Empty(empty_construct) => {
                walker.walk_empty_construct(empty_construct, context);
            }
            Construct::Eval(eval_construct) => {
                walker.walk_eval_construct(eval_construct, context);
            }
            Construct::Include(include_construct) => {
                walker.walk_include_construct(include_construct, context);
            }
            Construct::IncludeOnce(include_once_construct) => {
                walker.walk_include_once_construct(include_once_construct, context);
            }
            Construct::Require(require_construct) => {
                walker.walk_require_construct(require_construct, context);
            }
            Construct::RequireOnce(require_once_construct) => {
                walker.walk_require_once_construct(require_once_construct, context);
            }
            Construct::Print(print_construct) => {
                walker.walk_print_construct(print_construct, context);
            }
            Construct::Exit(exit_construct) => {
                walker.walk_exit_construct(exit_construct, context);
            }
            Construct::Die(die_construct) => {
                walker.walk_die_construct(die_construct, context);
            }
        }
    }

    'arena IssetConstruct as isset_construct => {
        walker.walk_keyword(&isset_construct.isset, context);
        for value in isset_construct.values.iter() {
            walker.walk_expression(value, context);
        }
    }

    'arena EmptyConstruct as empty_construct => {
        walker.walk_keyword(&empty_construct.empty, context);
        walker.walk_expression(empty_construct.value, context);
    }

    'arena EvalConstruct as eval_construct => {
        walker.walk_keyword(&eval_construct.eval, context);
        walker.walk_expression(eval_construct.value, context);
    }

    'arena IncludeConstruct as include_construct => {
        walker.walk_keyword(&include_construct.include, context);
        walker.walk_expression(include_construct.value, context);
    }

    'arena IncludeOnceConstruct as include_once_construct => {
        walker.walk_keyword(&include_once_construct.include_once, context);
        walker.walk_expression(include_once_construct.value, context);
    }

    'arena RequireConstruct as require_construct => {
        walker.walk_keyword(&require_construct.require, context);
        walker.walk_expression(require_construct.value, context);
    }

    'arena RequireOnceConstruct as require_once_construct => {
        walker.walk_keyword(&require_once_construct.require_once, context);
        walker.walk_expression(require_once_construct.value, context);
    }

    'arena PrintConstruct as print_construct => {
        walker.walk_keyword(&print_construct.print, context);
        walker.walk_expression(print_construct.value, context);
    }

    'arena ExitConstruct as exit_construct => {
        walker.walk_keyword(&exit_construct.exit, context);
        if let Some(arguments) = &exit_construct.arguments {
            walker.walk_argument_list(arguments, context);
        }
    }

    'arena DieConstruct as die_construct => {
        walker.walk_keyword(&die_construct.die, context);
        if let Some(arguments) = &die_construct.arguments {
            walker.walk_argument_list(arguments, context);
        }
    }

    'arena Throw as r#throw => {
        walker.walk_keyword(&r#throw.r#throw, context);
        walker.walk_expression(r#throw.exception, context);
    }

    'arena Clone as clone => {
        walker.walk_keyword(&clone.clone, context);
        walker.walk_expression(clone.object, context);
    }

    'arena Call as call => {
        match call {
            Call::Function(function_call) => {
                walker.walk_function_call(function_call, context);
            }
            Call::Method(method_call) => {
                walker.walk_method_call(method_call, context);
            }
            Call::NullSafeMethod(null_safe_method_call) => {
                walker.walk_null_safe_method_call(null_safe_method_call, context);
            }
            Call::StaticMethod(static_method_call) => {
                walker.walk_static_method_call(static_method_call, context);
            }
        }
    }

    'arena FunctionCall as function_call => {
        walker.walk_expression(function_call.function, context);
        walker.walk_argument_list(&function_call.argument_list, context);
    }

    'arena MethodCall as method_call => {
        walker.walk_expression(method_call.object, context);
        walker.walk_class_like_member_selector(&method_call.method, context);
        walker.walk_argument_list(&method_call.argument_list, context);
    }

    'arena NullSafeMethodCall as null_safe_method_call => {
        walker.walk_expression(null_safe_method_call.object, context);
        walker.walk_class_like_member_selector(&null_safe_method_call.method, context);
        walker.walk_argument_list(&null_safe_method_call.argument_list, context);
    }

    'arena StaticMethodCall as static_method_call => {
        walker.walk_expression(static_method_call.class, context);
        walker.walk_class_like_member_selector(&static_method_call.method, context);
        walker.walk_argument_list(&static_method_call.argument_list, context);
    }

    'arena ClassLikeMemberSelector as class_like_member_selector => {
        match class_like_member_selector {
            ClassLikeMemberSelector::Identifier(local_identifier) => {
                walker.walk_local_identifier(local_identifier, context);
            }
            ClassLikeMemberSelector::Variable(variable) => {
                walker.walk_variable(variable, context);
            }
            ClassLikeMemberSelector::Expression(class_like_member_expression_selector) => {
                walker.walk_class_like_member_expression_selector(
                    class_like_member_expression_selector,

                    context,
                );
            }
        }
    }

    'arena ClassLikeMemberExpressionSelector as class_like_member_expression_selector => {
        walker.walk_expression(class_like_member_expression_selector.expression, context);
    }

    'arena ConstantAccess as constant_access => {
        walker.walk_identifier(&constant_access.name, context);
    }

    'arena Access as access => {
        match access {
            Access::Property(property_access) => {
                walker.walk_property_access(property_access, context);
            }
            Access::NullSafeProperty(null_safe_property_access) => {
                walker.walk_null_safe_property_access(null_safe_property_access, context);
            }
            Access::StaticProperty(static_property_access) => {
                walker.walk_static_property_access(static_property_access, context);
            }
            Access::ClassConstant(class_constant_access) => {
                walker.walk_class_constant_access(class_constant_access, context);
            }
        }
    }

    'arena PropertyAccess as property_access => {
        walker.walk_expression(property_access.object, context);
        walker.walk_class_like_member_selector(&property_access.property, context);
    }

    'arena NullSafePropertyAccess as null_safe_property_access => {
        walker.walk_expression(null_safe_property_access.object, context);
        walker.walk_class_like_member_selector(&null_safe_property_access.property, context);
    }

    'arena StaticPropertyAccess as static_property_access => {
        walker.walk_expression(static_property_access.class, context);
        walker.walk_variable(&static_property_access.property, context);
    }

    'arena ClassConstantAccess as class_constant_access => {
        walker.walk_expression(class_constant_access.class, context);
        walker.walk_class_like_constant_selector(&class_constant_access.constant, context);
    }

    'arena ClassLikeConstantSelector as class_like_constant_selector => {
        match class_like_constant_selector {
            ClassLikeConstantSelector::Identifier(local_identifier) => {
                walker.walk_local_identifier(local_identifier, context);
            }
            ClassLikeConstantSelector::Expression(class_like_constant_expression_selector) => {
                walker.walk_class_like_member_expression_selector(
                    class_like_constant_expression_selector,

                    context,
                );
            }
        }
    }

    'arena ClosureCreation as closure_creation => {
        match closure_creation {
            ClosureCreation::Function(function_closure_creation) => {
                walker.walk_function_closure_creation(function_closure_creation, context);
            }
            ClosureCreation::Method(method_closure_creation) => {
                walker.walk_method_closure_creation(method_closure_creation, context);
            }
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                walker.walk_static_method_closure_creation(static_method_closure_creation, context);
            }
        }
    }

    'arena FunctionClosureCreation as function_closure_creation => {
        walker.walk_expression(function_closure_creation.function, context);
    }

    'arena MethodClosureCreation as method_closure_creation => {
        walker.walk_expression(method_closure_creation.object, context);
        walker.walk_class_like_member_selector(&method_closure_creation.method, context);
    }

    'arena StaticMethodClosureCreation as static_method_closure_creation => {
        walker.walk_expression(static_method_closure_creation.class, context);
        walker.walk_class_like_member_selector(&static_method_closure_creation.method, context);
    }

    'arena Keyword as parent_keyword => {
        // Do nothing by default
    }

    'arena Keyword as static_keyword => {
        // Do nothing by default
    }

    'arena Keyword as self_keyword => {
        // Do nothing by default
    }

    'arena Instantiation as instantiation => {
        walker.walk_keyword(&instantiation.new, context);
        walker.walk_expression(instantiation.class, context);
        if let Some(argument_list) = &instantiation.argument_list {
            walker.walk_argument_list(argument_list, context);
        }
    }

    'arena MagicConstant as magic_constant => {
        walker.walk_local_identifier(magic_constant.value(), context);
    }

    'arena Pipe as pipe => {
        walker.walk_expression(pipe.input, context);
        walker.walk_expression(pipe.callable, context);
    }

    'arena Hint as hint => {
        match hint {
            Hint::Identifier(identifier) => {
                walker.walk_identifier(identifier, context);
            }
            Hint::Parenthesized(parenthesized_hint) => {
                walker.walk_parenthesized_hint(parenthesized_hint, context);
            }
            Hint::Nullable(nullable_hint) => {
                walker.walk_nullable_hint(nullable_hint, context);
            }
            Hint::Union(union_hint) => {
                walker.walk_union_hint(union_hint, context);
            }
            Hint::Intersection(intersection_hint) => {
                walker.walk_intersection_hint(intersection_hint, context);
            }
            Hint::Null(keyword) |
            Hint::True(keyword) |
            Hint::False(keyword) |
            Hint::Array(keyword) |
            Hint::Callable(keyword) |
            Hint::Static(keyword) |
            Hint::Self_(keyword) |
            Hint::Parent(keyword) => {
                walker.walk_keyword(keyword, context);
            }
            Hint::Void(local_identifier) |
            Hint::Never(local_identifier) |
            Hint::Float(local_identifier) |
            Hint::Bool(local_identifier) |
            Hint::Integer(local_identifier) |
            Hint::String(local_identifier) |
            Hint::Object(local_identifier) |
            Hint::Mixed(local_identifier) |
            Hint::Iterable(local_identifier) => {
                walker.walk_local_identifier(local_identifier, context);
            }
        }
    }

    'arena ParenthesizedHint as parenthesized_hint => {
        walker.walk_hint(parenthesized_hint.hint, context);
    }

    'arena NullableHint as nullable_hint => {
        walker.walk_hint(nullable_hint.hint, context);
    }

    'arena UnionHint as union_hint => {
        walker.walk_hint(union_hint.left, context);
        walker.walk_hint(union_hint.right, context);
    }

    'arena IntersectionHint as intersection_hint => {
        walker.walk_hint(intersection_hint.left, context);
        walker.walk_hint(intersection_hint.right, context);
    }

    'arena Keyword as keyword => {
        // Do nothing by default
    }
}
