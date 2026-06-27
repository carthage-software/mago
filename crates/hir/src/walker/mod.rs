use crate::ir::IR;
use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::ir::error::Error;
use crate::ir::error::ErrorKind;
use crate::ir::error::annotation::AnnotationError;
use crate::ir::error::annotation::AnnotationErrorKind;
use crate::ir::expression::Access;
use crate::ir::expression::AccessKind;
use crate::ir::expression::ArrayElement;
use crate::ir::expression::ArrayElementKind;
use crate::ir::expression::Assignment;
use crate::ir::expression::Binary;
use crate::ir::expression::Call;
use crate::ir::expression::Callee;
use crate::ir::expression::CalleeKind;
use crate::ir::expression::CompositeStringPart;
use crate::ir::expression::Conditional;
use crate::ir::expression::Expression;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::Instantiation;
use crate::ir::expression::MagicConstant;
use crate::ir::expression::MagicConstantKind;
use crate::ir::expression::Match;
use crate::ir::expression::MatchArm;
use crate::ir::expression::MatchArmKind;
use crate::ir::expression::PartialApplication;
use crate::ir::expression::UnaryPostfix;
use crate::ir::expression::UnaryPrefix;
use crate::ir::expression::Yield;
use crate::ir::expression::YieldKind;
use crate::ir::expression::annotation::Annotation;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::expression::selector::ConstantSelectorKind;
use crate::ir::expression::selector::MemberSelector;
use crate::ir::expression::selector::MemberSelectorKind;
use crate::ir::identifier::Identifier;
use crate::ir::identifier::IdentifierKind;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::annotation::alias::ImportedTypeAliasAnnotation;
use crate::ir::item::annotation::alias::TypeAliasAnnotation;
use crate::ir::item::annotation::effect::AssertAnnotation;
use crate::ir::item::annotation::effect::AssertAnnotationPattern;
use crate::ir::item::annotation::effect::AssertAnnotationPatternKind;
use crate::ir::item::annotation::effect::AssertAnnotationTarget;
use crate::ir::item::annotation::effect::AssertAnnotationTargetKind;
use crate::ir::item::annotation::effect::SelfOutAnnotation;
use crate::ir::item::annotation::effect::ThrowsAnnotation;
use crate::ir::item::annotation::generics::InheritedTypeParameterAnnotation;
use crate::ir::item::annotation::generics::TypeParameterAnnotation;
use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::annotation::generics::WhereConstraintAnnotation;
use crate::ir::item::annotation::inheritance::ExtendsAnnotation;
use crate::ir::item::annotation::inheritance::ImplementsAnnotation;
use crate::ir::item::annotation::inheritance::MixinAnnotation;
use crate::ir::item::annotation::inheritance::RequireExtendsAnnotation;
use crate::ir::item::annotation::inheritance::RequireImplementsAnnotation;
use crate::ir::item::annotation::inheritance::SealedAnnotation;
use crate::ir::item::annotation::inheritance::UseAnnotation;
use crate::ir::item::annotation::member::MethodAnnotation;
use crate::ir::item::annotation::member::PropertyAnnotation;
use crate::ir::item::annotation::member::PropertyAnnotationKind;
use crate::ir::item::annotation::parameter::ParameterAnnotation;
use crate::ir::item::annotation::parameter::ParameterOutAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::expression::ItemExpression;
use crate::ir::item::expression::ItemExpressionKind;
use crate::ir::item::expression::anonymous_class::AnonymousClass;
use crate::ir::item::expression::arrow_function::ArrowFunction;
use crate::ir::item::expression::closure::Closure;
use crate::ir::item::expression::closure::ClosureUseClauseVariable;
use crate::ir::item::inheritance::Extends;
use crate::ir::item::inheritance::Implements;
use crate::ir::item::member::MemberItem;
use crate::ir::item::member::MemberItemKind;
use crate::ir::item::member::constant::ClassLikeConstant;
use crate::ir::item::member::enum_case::EnumCase;
use crate::ir::item::member::hook::Hook;
use crate::ir::item::member::hook::HookBody;
use crate::ir::item::member::hook::HookBodyKind;
use crate::ir::item::member::method::Method;
use crate::ir::item::member::property::HookedProperty;
use crate::ir::item::member::property::Property;
use crate::ir::item::member::trait_use::TraitUse;
use crate::ir::item::member::trait_use::TraitUseAdaptation;
use crate::ir::item::modifier::Modifier;
use crate::ir::item::modifier::ModifierKind;
use crate::ir::item::modifier::Visibility;
use crate::ir::item::modifier::VisibilityKind;
use crate::ir::item::parameter::Parameter;
use crate::ir::item::statement::ItemStatement;
use crate::ir::item::statement::ItemStatementKind;
use crate::ir::item::statement::class::Class;
use crate::ir::item::statement::constant::Constant;
use crate::ir::item::statement::r#enum::Enum;
use crate::ir::item::statement::r#enum::EnumBackingType;
use crate::ir::item::statement::r#enum::EnumBackingTypeKind;
use crate::ir::item::statement::function::Function;
use crate::ir::item::statement::interface::Interface;
use crate::ir::item::statement::r#trait::Trait;
use crate::ir::literal::Literal;
use crate::ir::literal::LiteralFloat;
use crate::ir::literal::LiteralInteger;
use crate::ir::literal::LiteralKind;
use crate::ir::literal::LiteralString;
use crate::ir::literal::LiteralStringKind;
use crate::ir::name::Name;
use crate::ir::statement::Declare;
use crate::ir::statement::DeclareItem;
use crate::ir::statement::DoWhile;
use crate::ir::statement::For;
use crate::ir::statement::Foreach;
use crate::ir::statement::GlobalItem;
use crate::ir::statement::If;
use crate::ir::statement::Namespace;
use crate::ir::statement::Statement;
use crate::ir::statement::StatementKind;
use crate::ir::statement::StaticItem;
use crate::ir::statement::Switch;
use crate::ir::statement::SwitchCase;
use crate::ir::statement::Try;
use crate::ir::statement::TryCatchClause;
use crate::ir::statement::UseItem;
use crate::ir::statement::While;
use crate::ir::statement::annotation::VariableBindingAnnotation;
use crate::ir::r#type::Type;
use crate::ir::r#type::TypeKind;
use crate::ir::r#type::annotation::CallableTypeAnnotation;
use crate::ir::r#type::annotation::CallableTypeAnnotationParameter;
use crate::ir::r#type::annotation::CallableTypeKind;
use crate::ir::r#type::annotation::ConditionalTypeAnnotation;
use crate::ir::r#type::annotation::GenericParameterTypeAnnotation;
use crate::ir::r#type::annotation::GlobalSelector;
use crate::ir::r#type::annotation::MemberReferenceSelector;
use crate::ir::r#type::annotation::NamedTypeAnnotation;
use crate::ir::r#type::annotation::ObjectShapeTypeAnnotation;
use crate::ir::r#type::annotation::ReferenceKind;
use crate::ir::r#type::annotation::ShapeTypeAnnotation;
use crate::ir::r#type::annotation::ShapeTypeAnnotationAdditionalFields;
use crate::ir::r#type::annotation::ShapeTypeAnnotationField;
use crate::ir::r#type::annotation::ShapeTypeAnnotationKey;
use crate::ir::r#type::annotation::StringTypeAnnotation;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::r#type::annotation::TypeAnnotationKind;
use crate::ir::variable::DirectVariable;
use crate::ir::variable::Variable;
use crate::ir::variable::annotation::VariableAnnotation;

macro_rules! gen_mut_walker_methods {
    (generic $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&mut self, [<_ $name>]: &'arena $node<'arena, I, S, E>, _context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&mut self, $name: &'arena $node<'arena, I, S, E>, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&mut self, [<_ $name>]: &'arena $node<'arena, I, S, E>, _context: &mut C) {}
        }
    };
    (arena $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&mut self, [<_ $name>]: &'arena $node<'arena>, _context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&mut self, $name: &'arena $node<'arena>, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&mut self, [<_ $name>]: &'arena $node<'arena>, _context: &mut C) {}
        }
    };
    (plain $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&mut self, [<_ $name>]: &'arena $node, _context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&mut self, $name: &'arena $node, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&mut self, [<_ $name>]: &'arena $node, _context: &mut C) {}
        }
    };
}

macro_rules! gen_const_walker_methods {
    (generic $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&self, [<_ $name>]: &'arena $node<'arena, I, S, E>, _context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&self, $name: &'arena $node<'arena, I, S, E>, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&self, [<_ $name>]: &'arena $node<'arena, I, S, E>, _context: &mut C) {}
        }
    };
    (arena $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&self, [<_ $name>]: &'arena $node<'arena>, _context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&self, $name: &'arena $node<'arena>, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&self, [<_ $name>]: &'arena $node<'arena>, _context: &mut C) {}
        }
    };
    (plain $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&self, [<_ $name>]: &'arena $node, _context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&self, $name: &'arena $node, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&self, [<_ $name>]: &'arena $node, _context: &mut C) {}
        }
    };
}

macro_rules! generate_walker {
    (
        using($walker:ident, $context:ident):
        $( $prefix:ident $node:ident as $name:ident => $body:block )*
    ) => {
        /// A trait for traversing the [`IR`] with mutable access to the walker.
        pub trait MutWalker<'arena, I, S, E, C>
        {
            $( gen_mut_walker_methods!($prefix $node $name $walker $context $body); )*
        }

        /// A trait for traversing the [`IR`] with shared access to the walker.
        pub trait Walker<'arena, I, S, E, C>
        {
            $( gen_const_walker_methods!($prefix $node $name $walker $context $body); )*
        }
    };
}

generate_walker! {
    using(walker, context):

    generic IR as ir => {
        for statement in ir.statements {
            walker.walk_statement(statement, context);
        }

        for error in ir.errors {
            walker.walk_error(error, context);
        }
    }

    generic Statement as statement => {
        walker.walk_statement_kind(&statement.kind, context);
    }

    generic StatementKind as statement_kind => {
        match statement_kind {
            StatementKind::Inline(_) | StatementKind::HaltCompiler | StatementKind::Noop => {}
            StatementKind::Namespace(node) => walker.walk_namespace(node, context),
            StatementKind::Sequence(statements) => {
                for statement in statements.iter() {
                    walker.walk_statement(statement, context);
                }
            }
            StatementKind::Item(node) => walker.walk_item_statement(node, context),
            StatementKind::Declare(node) => walker.walk_declare(node, context),
            StatementKind::Goto(name) | StatementKind::Label(name) => walker.walk_name(name, context),
            StatementKind::Try(node) => walker.walk_try_statement(node, context),
            StatementKind::Foreach(node) => walker.walk_foreach(node, context),
            StatementKind::For(node) => walker.walk_for_loop(node, context),
            StatementKind::While(node) => walker.walk_while_loop(node, context),
            StatementKind::DoWhile(node) => walker.walk_do_while(node, context),
            StatementKind::Continue(value) | StatementKind::Break(value) | StatementKind::Return(value) => {
                if let Some(value) = value {
                    walker.walk_expression(value, context);
                }
            }
            StatementKind::Switch(node) => walker.walk_switch(node, context),
            StatementKind::If(node) => walker.walk_if_statement(node, context),
            StatementKind::Expression(node) => walker.walk_expression(node, context),
            StatementKind::Echo(values) => {
                for value in values.iter() {
                    walker.walk_expression(value, context);
                }
            }
            StatementKind::Unset(values) => {
                for value in values.iter() {
                    walker.walk_expression(value, context);
                }
            }
            StatementKind::Use(items) => {
                for item in items.iter() {
                    walker.walk_use_item(item, context);
                }
            }
            StatementKind::Global(items) => {
                for item in items.iter() {
                    walker.walk_global_item(item, context);
                }
            }
            StatementKind::Static(items) => {
                for item in items.iter() {
                    walker.walk_static_item(item, context);
                }
            }
            StatementKind::VariableBindingAnnotation(node) => walker.walk_variable_binding_annotation(node, context),
        }
    }

    generic Namespace as namespace => {
        if let Some(name) = namespace.name {
            walker.walk_identifier(name, context);
        }

        walker.walk_statement(namespace.statement, context);
    }

    generic Declare as declare => {
        for item in declare.items.iter() {
            walker.walk_declare_item(item, context);
        }

        walker.walk_statement(declare.statement, context);
    }

    generic DeclareItem as declare_item => {
        walker.walk_name(&declare_item.name, context);
        if let Some(value) = declare_item.value {
            walker.walk_expression(value, context);
        }
    }

    generic Try as try_statement => {
        walker.walk_statement(try_statement.statement, context);
        for catch_clause in try_statement.catch_clauses {
            walker.walk_try_catch_clause(catch_clause, context);
        }

        if let Some(finally_clause) = try_statement.finally_clause {
            walker.walk_statement(finally_clause, context);
        }
    }

    generic TryCatchClause as try_catch_clause => {
        walker.walk_type(try_catch_clause.r#type, context);
        if let Some(variable) = &try_catch_clause.variable {
            walker.walk_direct_variable(variable, context);
        }

        walker.walk_statement(try_catch_clause.statement, context);
    }

    generic Foreach as foreach => {
        walker.walk_expression(foreach.expression, context);
        if let Some(key) = foreach.key {
            walker.walk_expression(key, context);
        }

        walker.walk_expression(foreach.value, context);
        walker.walk_statement(foreach.statement, context);
    }

    generic For as for_loop => {
        for initialization in for_loop.initializations {
            walker.walk_expression(initialization, context);
        }

        for condition in for_loop.conditions {
            walker.walk_expression(condition, context);
        }

        for increment in for_loop.increments {
            walker.walk_expression(increment, context);
        }

        walker.walk_statement(for_loop.statement, context);
    }

    generic While as while_loop => {
        walker.walk_expression(while_loop.condition, context);
        walker.walk_statement(while_loop.statement, context);
    }

    generic DoWhile as do_while => {
        walker.walk_statement(do_while.statement, context);
        walker.walk_expression(do_while.condition, context);
    }

    generic Switch as switch => {
        walker.walk_expression(switch.subject, context);
        for case in switch.cases.iter() {
            walker.walk_switch_case(case, context);
        }
    }

    generic SwitchCase as switch_case => {
        match switch_case {
            SwitchCase::Expression(expression, statement) => {
                walker.walk_expression(expression, context);
                walker.walk_statement(statement, context);
            }
            SwitchCase::Default(statement) => walker.walk_statement(statement, context),
        }
    }

    generic If as if_statement => {
        walker.walk_expression(if_statement.condition, context);
        walker.walk_statement(if_statement.then, context);
        if let Some(r#else) = if_statement.r#else {
            walker.walk_statement(r#else, context);
        }
    }

    generic StaticItem as static_item => {
        walker.walk_direct_variable(&static_item.variable, context);
        if let Some(type_annotation) = static_item.type_annotation {
            walker.walk_type_annotation(type_annotation, context);
        }

        if let Some(value) = static_item.value {
            walker.walk_expression(value, context);
        }
    }

    generic GlobalItem as global_item => {
        walker.walk_variable(&global_item.variable, context);
        if let Some(type_annotation) = global_item.type_annotation {
            walker.walk_type_annotation(type_annotation, context);
        }
    }

    arena UseItem as use_item => {
        walker.walk_identifier(&use_item.item, context);
    }

    arena VariableBindingAnnotation as variable_binding_annotation => {
        walker.walk_direct_variable(&variable_binding_annotation.variable, context);
        walker.walk_type_annotation(variable_binding_annotation.type_annotation, context);
    }

    generic ItemStatement as item_statement => {
        walker.walk_item_statement_kind(&item_statement.kind, context);
    }

    generic ItemStatementKind as item_statement_kind => {
        match item_statement_kind {
            ItemStatementKind::Class(node) => walker.walk_class(node, context),
            ItemStatementKind::Interface(node) => walker.walk_interface(node, context),
            ItemStatementKind::Trait(node) => walker.walk_trait(node, context),
            ItemStatementKind::Enum(node) => walker.walk_enum(node, context),
            ItemStatementKind::Constant(node) => walker.walk_constant(node, context),
            ItemStatementKind::Function(node) => walker.walk_function(node, context),
        }
    }

    generic Class as class => {
        if let Some(annotation) = class.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in class.attributes {
            walker.walk_attribute(attribute, context);
        }

        for modifier in class.modifiers {
            walker.walk_modifier(modifier, context);
        }

        walker.walk_identifier(&class.name, context);
        if let Some(extends) = class.extends {
            walker.walk_extends(extends, context);
        }

        if let Some(implements) = class.implements {
            walker.walk_implements(implements, context);
        }

        for member in class.members.iter() {
            walker.walk_member_item(member, context);
        }
    }

    generic Interface as interface => {
        if let Some(annotation) = interface.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in interface.attributes {
            walker.walk_attribute(attribute, context);
        }

        walker.walk_identifier(&interface.name, context);
        if let Some(extends) = interface.extends {
            walker.walk_extends(extends, context);
        }

        for member in interface.members.iter() {
            walker.walk_member_item(member, context);
        }
    }

    generic Trait as r#trait => {
        if let Some(annotation) = r#trait.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in r#trait.attributes {
            walker.walk_attribute(attribute, context);
        }

        walker.walk_identifier(&r#trait.name, context);
        for member in r#trait.members.iter() {
            walker.walk_member_item(member, context);
        }
    }

    generic Enum as r#enum => {
        if let Some(annotation) = r#enum.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in r#enum.attributes {
            walker.walk_attribute(attribute, context);
        }

        walker.walk_identifier(&r#enum.name, context);
        if let Some(backing_type) = &r#enum.backing_type {
            walker.walk_enum_backing_type(backing_type, context);
        }

        if let Some(implements) = r#enum.implements {
            walker.walk_implements(implements, context);
        }

        for member in r#enum.members.iter() {
            walker.walk_member_item(member, context);
        }
    }

    generic Constant as constant => {
        if let Some(annotation) = constant.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in constant.attributes {
            walker.walk_attribute(attribute, context);
        }

        walker.walk_identifier(&constant.name, context);
        walker.walk_expression(constant.value, context);
    }

    generic Function as function => {
        if let Some(annotation) = function.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in function.attributes {
            walker.walk_attribute(attribute, context);
        }

        walker.walk_identifier(&function.name, context);
        for parameter in function.parameters.iter() {
            walker.walk_parameter(parameter, context);
        }

        if let Some(return_type) = function.return_type {
            walker.walk_type(return_type, context);
        }

        walker.walk_statement(function.body, context);
    }

    generic MemberItem as member_item => {
        walker.walk_member_item_kind(&member_item.kind, context);
    }

    generic MemberItemKind as member_item_kind => {
        match member_item_kind {
            MemberItemKind::Method(node) => walker.walk_method(node, context),
            MemberItemKind::Property(node) => walker.walk_property(node, context),
            MemberItemKind::HookedProperty(node) => walker.walk_hooked_property(node, context),
            MemberItemKind::TraitUse(node) => walker.walk_trait_use(node, context),
            MemberItemKind::Constant(node) => walker.walk_class_like_constant(node, context),
            MemberItemKind::EnumCase(node) => walker.walk_enum_case(node, context),
        }
    }

    generic Method as method => {
        if let Some(annotation) = method.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in method.attributes {
            walker.walk_attribute(attribute, context);
        }

        for modifier in method.modifiers {
            walker.walk_modifier(modifier, context);
        }

        walker.walk_name(&method.name, context);
        for parameter in method.parameters.iter() {
            walker.walk_parameter(parameter, context);
        }

        if let Some(return_type) = method.return_type {
            walker.walk_type(return_type, context);
        }

        if let Some(body) = method.body {
            walker.walk_statement(body, context);
        }
    }

    generic Property as property => {
        if let Some(annotation) = property.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in property.attributes {
            walker.walk_attribute(attribute, context);
        }

        for modifier in property.modifiers {
            walker.walk_modifier(modifier, context);
        }

        if let Some(r#type) = property.r#type {
            walker.walk_type(r#type, context);
        }

        walker.walk_direct_variable(&property.variable, context);
        if let Some(default_value) = property.default_value {
            walker.walk_expression(default_value, context);
        }
    }

    generic HookedProperty as hooked_property => {
        if let Some(annotation) = hooked_property.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in hooked_property.attributes {
            walker.walk_attribute(attribute, context);
        }

        for modifier in hooked_property.modifiers {
            walker.walk_modifier(modifier, context);
        }

        if let Some(r#type) = hooked_property.r#type {
            walker.walk_type(r#type, context);
        }

        walker.walk_direct_variable(&hooked_property.variable, context);
        if let Some(default_value) = hooked_property.default_value {
            walker.walk_expression(default_value, context);
        }
        for hook in hooked_property.hooks.iter() {
            walker.walk_hook(hook, context);
        }
    }

    generic ClassLikeConstant as class_like_constant => {
        if let Some(annotation) = class_like_constant.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in class_like_constant.attributes {
            walker.walk_attribute(attribute, context);
        }

        for modifier in class_like_constant.modifiers {
            walker.walk_modifier(modifier, context);
        }

        if let Some(r#type) = class_like_constant.r#type {
            walker.walk_type(r#type, context);
        }

        walker.walk_name(&class_like_constant.name, context);
        walker.walk_expression(class_like_constant.value, context);
    }

    generic EnumCase as enum_case => {
        if let Some(annotation) = enum_case.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in enum_case.attributes {
            walker.walk_attribute(attribute, context);
        }

        walker.walk_name(&enum_case.name, context);
        if let Some(value) = enum_case.value {
            walker.walk_expression(value, context);
        }
    }

    generic Hook as hook => {
        if let Some(annotation) = hook.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in hook.attributes {
            walker.walk_attribute(attribute, context);
        }

        for modifier in hook.modifiers {
            walker.walk_modifier(modifier, context);
        }

        walker.walk_name(&hook.name, context);
        for parameter in hook.parameters.iter().flatten() {
            walker.walk_parameter(parameter, context);
        }

        if let Some(body) = &hook.body {
            walker.walk_hook_body(body, context);
        }
    }

    generic HookBody as hook_body => {
        walker.walk_hook_body_kind(&hook_body.kind, context);
    }

    generic HookBodyKind as hook_body_kind => {
        match hook_body_kind {
            HookBodyKind::Expression(expression) => walker.walk_expression(expression, context),
            HookBodyKind::Statements(statements) => {
                for statement in statements.iter() {
                    walker.walk_statement(statement, context);
                }
            }
        }
    }

    generic TraitUse as trait_use => {
        if let Some(annotation) = trait_use.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for r#trait in trait_use.traits {
            walker.walk_identifier(r#trait, context);
        }

        for adaptation in trait_use.adaptations.iter().flatten() {
            walker.walk_trait_use_adaptation(adaptation, context);
        }
    }

    arena TraitUseAdaptation as trait_use_adaptation => {
        match trait_use_adaptation {
            TraitUseAdaptation::Precedence(adaptation) => {
                walker.walk_identifier(&adaptation.r#trait, context);
                walker.walk_name(&adaptation.method, context);
                for instead_of in adaptation.instead_of {
                    walker.walk_identifier(instead_of, context);
                }
            }
            TraitUseAdaptation::Alias(adaptation) => {
                if let Some(r#trait) = &adaptation.r#trait {
                    walker.walk_identifier(r#trait, context);
                }
                walker.walk_name(&adaptation.method, context);
                if let Some(modifier) = &adaptation.modifier {
                    walker.walk_modifier(modifier, context);
                }
                walker.walk_name(&adaptation.alias, context);
            }
        }
    }

    generic Parameter as parameter => {
        if let Some(annotation) = parameter.annotation {
            walker.walk_variable_annotation(annotation, context);
        }

        for attribute in parameter.attributes {
            walker.walk_attribute(attribute, context);
        }

        for modifier in parameter.modifiers {
            walker.walk_modifier(modifier, context);
        }

        if let Some(r#type) = parameter.r#type {
            walker.walk_type(r#type, context);
        }

        walker.walk_direct_variable(&parameter.variable, context);
        if let Some(default_value) = parameter.default_value {
            walker.walk_expression(default_value, context);
        }

        for hook in parameter.hooks.iter().flatten() {
            walker.walk_hook(hook, context);
        }
    }

    generic ItemAnnotation as item_annotation => {
        for type_alias in item_annotation.type_aliases {
            walker.walk_type_alias_annotation(type_alias, context);
        }

        for imported_type_alias in item_annotation.imported_type_aliases {
            walker.walk_imported_type_alias_annotation(imported_type_alias, context);
        }

        for type_parameter in item_annotation.type_parameters {
            walker.walk_type_parameter_annotation(type_parameter, context);
        }

        for inherited in item_annotation.inherited_type_parameters {
            walker.walk_inherited_type_parameter_annotation(inherited, context);
        }

        for extends in item_annotation.extends {
            walker.walk_extends_annotation(extends, context);
        }

        for require_extends in item_annotation.require_extends {
            walker.walk_require_extends_annotation(require_extends, context);
        }

        for implements in item_annotation.implements {
            walker.walk_implements_annotation(implements, context);
        }

        for require_implements in item_annotation.require_implements {
            walker.walk_require_implements_annotation(require_implements, context);
        }

        for r#use in item_annotation.uses {
            walker.walk_use_annotation(r#use, context);
        }

        for sealing in item_annotation.sealings {
            walker.walk_sealed_annotation(sealing, context);
        }

        for mixin in item_annotation.mixins {
            walker.walk_mixin_annotation(mixin, context);
        }

        for method in item_annotation.methods {
            walker.walk_method_annotation(method, context);
        }

        for property in item_annotation.properties {
            walker.walk_property_annotation(property, context);
        }

        for parameter in item_annotation.parameters {
            walker.walk_parameter_annotation(parameter, context);
        }

        for parameter_out in item_annotation.parameter_outs {
            walker.walk_parameter_out_annotation(parameter_out, context);
        }

        for where_constraint in item_annotation.where_constraints {
            walker.walk_where_constraint_annotation(where_constraint, context);
        }

        for return_type in item_annotation.return_type {
            walker.walk_type_annotation(return_type, context);
        }

        for throws in item_annotation.throws {
            walker.walk_throws_annotation(throws, context);
        }

        for assert in
            item_annotation.asserts.iter().chain(item_annotation.asserts_if_true).chain(item_annotation.asserts_if_false)
        {
            walker.walk_assert_annotation(assert, context);
        }

        for self_out in item_annotation.self_out {
            walker.walk_self_out_annotation(self_out, context);
        }

        for variable in item_annotation.pure_unless_callable_impure {
            walker.walk_direct_variable(variable, context);
        }

        for var in item_annotation.var {
            walker.walk_variable_annotation(var, context);
        }

        for error in item_annotation.errors {
            walker.walk_annotation_error(error, context);
        }
    }

    generic MethodAnnotation as method_annotation => {
        if let Some(visibility) = &method_annotation.visibility {
            walker.walk_visibility(visibility, context);
        }

        walker.walk_name(&method_annotation.name, context);
        for type_parameter in method_annotation.type_parameters.iter().flatten() {
            walker.walk_type_parameter_annotation(type_parameter, context);
        }

        for parameter in method_annotation.parameters.iter() {
            walker.walk_parameter_annotation(parameter, context);
        }

        if let Some(return_type) = method_annotation.return_type {
            walker.walk_type_annotation(return_type, context);
        }
    }

    generic ParameterAnnotation as parameter_annotation => {
        if let Some(type_annotation) = parameter_annotation.r#type {
            walker.walk_type_annotation(type_annotation, context);
        }

        if let Some(variable) = &parameter_annotation.variable {
            walker.walk_direct_variable(variable, context);
        }
        if let Some(default_value) = parameter_annotation.default_value {
            walker.walk_expression(default_value, context);
        }
    }

    arena ParameterOutAnnotation as parameter_out_annotation => {
        walker.walk_type_annotation(parameter_out_annotation.r#type, context);
        walker.walk_direct_variable(&parameter_out_annotation.variable, context);
    }

    arena VariableAnnotation as variable_annotation => {
        walker.walk_type_annotation(variable_annotation.type_annotation, context);
        if let Some(variable) = &variable_annotation.variable {
            walker.walk_direct_variable(variable, context);
        }

        for error in variable_annotation.errors {
            walker.walk_annotation_error(error, context);
        }
    }

    arena Extends as extends => {
        for r#type in extends.types {
            walker.walk_identifier(r#type, context);
        }
    }

    arena Implements as implements => {
        for r#type in implements.types {
            walker.walk_identifier(r#type, context);
        }
    }

    generic Attribute as attribute => {
        walker.walk_identifier(&attribute.class, context);
        for argument in attribute.arguments.iter().flatten() {
            walker.walk_partial_argument(argument, context);
        }
    }

    generic Argument as argument => {
        match argument {
            Argument::Value(expression) | Argument::Variadic(expression) => {
                walker.walk_expression(expression, context);
            }
            Argument::Named(name, expression) => {
                walker.walk_name(name, context);
                walker.walk_expression(expression, context);
            }
        }
    }

    generic PartialArgument as partial_argument => {
        match partial_argument {
            PartialArgument::Value(expression) | PartialArgument::Variadic(expression) => {
                walker.walk_expression(expression, context);
            }
            PartialArgument::Named(name, expression) => {
                walker.walk_name(name, context);
                walker.walk_expression(expression, context);
            }
            PartialArgument::NamedPlaceholder(name) => walker.walk_name(name, context),
            PartialArgument::Placeholder(_) | PartialArgument::VariadicPlaceholder(_) => {}
        }
    }

    generic Expression as expression => {
        walker.walk_expression_kind(&expression.kind, context);
    }

    generic ExpressionKind as expression_kind => {
        match expression_kind {
            ExpressionKind::Parenthesized(node) => walker.walk_expression(node, context),
            ExpressionKind::Binary(node) => walker.walk_binary(node, context),
            ExpressionKind::UnaryPrefix(node) => walker.walk_unary_prefix(node, context),
            ExpressionKind::UnaryPostfix(node) => walker.walk_unary_postfix(node, context),
            ExpressionKind::Literal(node) => walker.walk_literal(node, context),
            ExpressionKind::CompositeString(parts) | ExpressionKind::ShellExecute(parts) => {
                for part in parts.iter() {
                    walker.walk_composite_string_part(part, context);
                }
            }
            ExpressionKind::Assignment(node) => walker.walk_assignment(node, context),
            ExpressionKind::Annotation(node) => walker.walk_annotation(node, context),
            ExpressionKind::Conditional(node) => walker.walk_conditional(node, context),
            ExpressionKind::Array(elements) | ExpressionKind::List(elements) => {
                for element in elements.iter() {
                    walker.walk_array_element(element, context);
                }
            }
            ExpressionKind::ArrayAppend(node)
            | ExpressionKind::Clone(node)
            | ExpressionKind::Empty(node)
            | ExpressionKind::Eval(node)
            | ExpressionKind::Include(node)
            | ExpressionKind::IncludeOnce(node)
            | ExpressionKind::Require(node)
            | ExpressionKind::RequireOnce(node)
            | ExpressionKind::Print(node)
            | ExpressionKind::Throw(node) => walker.walk_expression(node, context),
            ExpressionKind::Item(node) => walker.walk_item_expression(node, context),
            ExpressionKind::Call(node) => walker.walk_call(node, context),
            ExpressionKind::PartialApplication(node) => walker.walk_partial_application(node, context),
            ExpressionKind::Access(node) => walker.walk_access(node, context),
            ExpressionKind::Isset(values) => {
                for value in values.iter() {
                    walker.walk_expression(value, context);
                }
            }
            ExpressionKind::Exit(arguments) => {
                for argument in arguments.iter().flatten() {
                    walker.walk_argument(argument, context);
                }
            }
            ExpressionKind::Constant(identifier) | ExpressionKind::Identifier(identifier) => {
                walker.walk_identifier(identifier, context);
            }
            ExpressionKind::Instantiation(node) => walker.walk_instantiation(node, context),
            ExpressionKind::Variable(variable) => walker.walk_variable(variable, context),
            ExpressionKind::Yield(node) => walker.walk_yield_expression(node, context),
            ExpressionKind::Match(node) => walker.walk_match_expression(node, context),
            ExpressionKind::Error(_) => {}
            ExpressionKind::MagicConstant(node) => walker.walk_magic_constant(node, context),
            ExpressionKind::Parent | ExpressionKind::Self_ | ExpressionKind::Static => {}
        }
    }

    generic Assignment as assignment => {
        walker.walk_expression(assignment.left, context);
        walker.walk_expression(assignment.right, context);
    }

    generic Annotation as annotation => {
        walker.walk_variable_annotation(annotation.annotation, context);
        walker.walk_expression(annotation.expression, context);
    }

    generic Binary as binary => {
        walker.walk_expression(binary.left, context);
        walker.walk_expression(binary.right, context);
    }

    generic UnaryPrefix as unary_prefix => {
        walker.walk_expression(unary_prefix.operand, context);
    }

    generic UnaryPostfix as unary_postfix => {
        walker.walk_expression(unary_postfix.operand, context);
    }

    generic Conditional as conditional => {
        walker.walk_expression(conditional.condition, context);
        if let Some(then) = conditional.then {
            walker.walk_expression(then, context);
        }

        walker.walk_expression(conditional.r#else, context);
    }

    generic ArrayElement as array_element => {
        walker.walk_array_element_kind(&array_element.kind, context);
    }

    generic ArrayElementKind as array_element_kind => {
        match array_element_kind {
            ArrayElementKind::KeyValue(key, value) => {
                walker.walk_expression(key, context);
                walker.walk_expression(value, context);
            }
            ArrayElementKind::Value(value) | ArrayElementKind::Variadic(value) => {
                walker.walk_expression(value, context);
            }
            ArrayElementKind::Missing => {}
        }
    }

    generic CompositeStringPart as composite_string_part => {
        match composite_string_part {
            CompositeStringPart::Literal(_) => {}
            CompositeStringPart::Expression(expression) => walker.walk_expression(expression, context),
        }
    }

    generic Instantiation as instantiation => {
        walker.walk_expression(instantiation.class, context);
        for argument in instantiation.arguments.iter().flatten() {
            walker.walk_argument(argument, context);
        }
    }

    generic Call as call => {
        walker.walk_callee(&call.callee, context);
        for argument in call.arguments.iter() {
            walker.walk_argument(argument, context);
        }
    }

    generic PartialApplication as partial_application => {
        walker.walk_callee(&partial_application.callee, context);
        for argument in partial_application.arguments.iter() {
            walker.walk_partial_argument(argument, context);
        }
    }

    generic Callee as callee => {
        walker.walk_callee_kind(&callee.kind, context);
    }

    generic CalleeKind as callee_kind => {
        match callee_kind {
            CalleeKind::Function(expression) => walker.walk_expression(expression, context),
            CalleeKind::Method(expression, selector)
            | CalleeKind::NullsafeMethod(expression, selector)
            | CalleeKind::StaticMethod(expression, selector) => {
                walker.walk_expression(expression, context);
                walker.walk_member_selector(selector, context);
            }
        }
    }

    generic Access as access => {
        walker.walk_access_kind(&access.kind, context);
    }

    generic AccessKind as access_kind => {
        match access_kind {
            AccessKind::Array(target, index) => {
                walker.walk_expression(target, context);
                walker.walk_expression(index, context);
            }
            AccessKind::Property(target, selector) | AccessKind::NullsafeProperty(target, selector) => {
                walker.walk_expression(target, context);
                walker.walk_member_selector(selector, context);
            }
            AccessKind::StaticProperty(target, variable) => {
                walker.walk_expression(target, context);
                walker.walk_variable(variable, context);
            }
            AccessKind::ClassConstant(target, selector) => {
                walker.walk_expression(target, context);
                walker.walk_constant_selector(selector, context);
            }
        }
    }

    generic MemberSelector as member_selector => {
        walker.walk_member_selector_kind(&member_selector.kind, context);
    }

    generic MemberSelectorKind as member_selector_kind => {
        match member_selector_kind {
            MemberSelectorKind::Missing => {}
            MemberSelectorKind::Name(name) => walker.walk_name(name, context),
            MemberSelectorKind::Variable(variable) => walker.walk_direct_variable(variable, context),
            MemberSelectorKind::Expression(expression) => walker.walk_expression(expression, context),
        }
    }

    generic ConstantSelector as constant_selector => {
        walker.walk_constant_selector_kind(&constant_selector.kind, context);
    }

    generic ConstantSelectorKind as constant_selector_kind => {
        match constant_selector_kind {
            ConstantSelectorKind::Missing => {}
            ConstantSelectorKind::Name(name) => walker.walk_name(name, context),
            ConstantSelectorKind::Expression(expression) => walker.walk_expression(expression, context),
        }
    }

    generic Yield as yield_expression => {
        walker.walk_yield_kind(&yield_expression.kind, context);
    }

    generic YieldKind as yield_kind => {
        match yield_kind {
            YieldKind::Nothing => {}
            YieldKind::Expression(value) | YieldKind::From(value) => walker.walk_expression(value, context),
            YieldKind::Pair(key, value) => {
                walker.walk_expression(key, context);
                walker.walk_expression(value, context);
            }
        }
    }

    generic Match as match_expression => {
        walker.walk_expression(match_expression.subject, context);
        for arm in match_expression.arms.iter() {
            walker.walk_match_arm(arm, context);
        }
    }

    generic MatchArm as match_arm => {
        walker.walk_match_arm_kind(&match_arm.kind, context);
    }

    generic MatchArmKind as match_arm_kind => {
        match match_arm_kind {
            MatchArmKind::Expression(conditions, body) => {
                for condition in conditions.iter() {
                    walker.walk_expression(condition, context);
                }
                walker.walk_expression(body, context);
            }
            MatchArmKind::Default(body) => walker.walk_expression(body, context),
        }
    }

    generic ItemExpression as item_expression => {
        walker.walk_item_expression_kind(&item_expression.kind, context);
    }

    generic ItemExpressionKind as item_expression_kind => {
        match item_expression_kind {
            ItemExpressionKind::AnonymousClass(node) => walker.walk_anonymous_class(node, context),
            ItemExpressionKind::ArrowFunction(node) => walker.walk_arrow_function(node, context),
            ItemExpressionKind::Closure(node) => walker.walk_closure(node, context),
        }
    }

    generic AnonymousClass as anonymous_class => {
        for modifier in anonymous_class.modifiers {
            walker.walk_modifier(modifier, context);
        }

        if let Some(annotation) = anonymous_class.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in anonymous_class.attributes {
            walker.walk_attribute(attribute, context);
        }

        for argument in anonymous_class.arguments.iter().flatten() {
            walker.walk_partial_argument(argument, context);
        }

        if let Some(extends) = anonymous_class.extends {
            walker.walk_extends(extends, context);
        }

        if let Some(implements) = anonymous_class.implements {
            walker.walk_implements(implements, context);
        }

        for member in anonymous_class.members.iter() {
            walker.walk_member_item(member, context);
        }
    }

    generic Closure as closure => {
        if let Some(annotation) = closure.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in closure.attributes {
            walker.walk_attribute(attribute, context);
        }

        for parameter in closure.parameters.iter() {
            walker.walk_parameter(parameter, context);
        }

        if let Some(return_type) = closure.return_type {
            walker.walk_type(return_type, context);
        }

        for use_variable in closure.use_variables.iter().flatten() {
            walker.walk_closure_use_clause_variable(use_variable, context);
        }

        walker.walk_statement(closure.body, context);
    }

    generic ArrowFunction as arrow_function => {
        if let Some(annotation) = arrow_function.annotation {
            walker.walk_item_annotation(annotation, context);
        }

        for attribute in arrow_function.attributes {
            walker.walk_attribute(attribute, context);
        }

        for parameter in arrow_function.parameters.iter() {
            walker.walk_parameter(parameter, context);
        }

        if let Some(return_type) = arrow_function.return_type {
            walker.walk_type(return_type, context);
        }

        walker.walk_expression(arrow_function.expression, context);
    }

    generic Variable as variable => {
        match variable {
            Variable::Direct(direct) => walker.walk_direct_variable(direct, context),
            Variable::Indirect(expression) => walker.walk_expression(expression, context),
            Variable::Nested(nested) => walker.walk_variable(nested, context),
        }
    }

    arena Identifier as identifier => {
        walker.walk_identifier_kind(&identifier.kind, context);
    }

    plain IdentifierKind as identifier_kind => {}

    arena Name as name => {}

    arena DirectVariable as direct_variable => {}

    arena TypeAnnotation as type_annotation => {
        walker.walk_type_annotation_kind(&type_annotation.kind, context);
    }

    arena TypeAnnotationKind as type_annotation_kind => {
        match type_annotation_kind {
            TypeAnnotationKind::Named(node) => walker.walk_named_type_annotation(node, context),
            TypeAnnotationKind::GenericParameter(node) => {
                walker.walk_generic_parameter_type_annotation(node, context);
            }
            TypeAnnotationKind::Union(types) | TypeAnnotationKind::Intersection(types) => {
                for r#type in types.iter() {
                    walker.walk_type_annotation(r#type, context);
                }
            }
            TypeAnnotationKind::Array(_, key, value) => {
                walker.walk_type_annotation(key, context);
                walker.walk_type_annotation(value, context);
            }
            TypeAnnotationKind::List(_, value) => walker.walk_type_annotation(value, context),
            TypeAnnotationKind::Iterable(key, value) => {
                walker.walk_type_annotation(key, context);
                walker.walk_type_annotation(value, context);
            }
            TypeAnnotationKind::ClassLikeString(inner)
            | TypeAnnotationKind::ClassString(inner)
            | TypeAnnotationKind::InterfaceString(inner)
            | TypeAnnotationKind::EnumString(inner)
            | TypeAnnotationKind::TraitString(inner)
            | TypeAnnotationKind::KeyOf(inner)
            | TypeAnnotationKind::ValueOf(inner)
            | TypeAnnotationKind::IntMaskOf(inner)
            | TypeAnnotationKind::New(inner)
            | TypeAnnotationKind::Negated(inner)
            | TypeAnnotationKind::Posited(inner)
            | TypeAnnotationKind::Slice(inner) => walker.walk_type_annotation(inner, context),
            TypeAnnotationKind::String(node) => walker.walk_string_type_annotation(node, context),
            TypeAnnotationKind::ObjectShape(node) => walker.walk_object_shape_type_annotation(node, context),
            TypeAnnotationKind::MemberReference(identifier, selector) => {
                walker.walk_identifier(identifier, context);
                walker.walk_member_reference_selector(selector, context);
            }
            TypeAnnotationKind::AliasReference(reference, name) => {
                walker.walk_reference_kind(reference, context);
                walker.walk_name(name, context);
            }
            TypeAnnotationKind::Shape(node) => walker.walk_shape_type_annotation(node, context),
            TypeAnnotationKind::Callable(node) => walker.walk_callable_type_annotation(node, context),
            TypeAnnotationKind::Variable(variable) => walker.walk_direct_variable(variable, context),
            TypeAnnotationKind::Conditional(node) => walker.walk_conditional_type_annotation(node, context),
            TypeAnnotationKind::IntMask(types) | TypeAnnotationKind::TemplateType(types) => {
                for r#type in types.iter() {
                    walker.walk_type_annotation(r#type, context);
                }
            }
            TypeAnnotationKind::IndexAccess(base, index) => {
                walker.walk_type_annotation(base, context);
                walker.walk_type_annotation(index, context);
            }
            TypeAnnotationKind::PropertiesOf(_, inner) => walker.walk_type_annotation(inner, context),
            TypeAnnotationKind::GlobalSelector(selector) => walker.walk_global_selector(selector, context),
            TypeAnnotationKind::Mixed(_)
            | TypeAnnotationKind::Null
            | TypeAnnotationKind::Void
            | TypeAnnotationKind::Never
            | TypeAnnotationKind::Resource(_)
            | TypeAnnotationKind::Bool(_)
            | TypeAnnotationKind::Float(_)
            | TypeAnnotationKind::Int(_)
            | TypeAnnotationKind::StringableObject
            | TypeAnnotationKind::Object
            | TypeAnnotationKind::Numeric
            | TypeAnnotationKind::ArrayKey
            | TypeAnnotationKind::Scalar
            | TypeAnnotationKind::Empty
            | TypeAnnotationKind::EmptyScalar
            | TypeAnnotationKind::ThisVariable
            | TypeAnnotationKind::IntRange(_, _)
            | TypeAnnotationKind::Wildcard => {}
        }
    }

    arena NamedTypeAnnotation as named_type_annotation => {
        walker.walk_reference_kind(&named_type_annotation.kind, context);
        for type_argument in named_type_annotation.type_arguments.iter().flatten() {
            walker.walk_type_annotation(type_argument, context);
        }
    }

    arena GenericParameterTypeAnnotation as generic_parameter_type_annotation => {
        walker.walk_name(&generic_parameter_type_annotation.name, context);
        walker.walk_type_parameter_defining_entity(&generic_parameter_type_annotation.defining_entity, context);
        if let Some(bound) = generic_parameter_type_annotation.bound {
            walker.walk_type_annotation(bound, context);
        }
    }

    arena StringTypeAnnotation as string_type_annotation => {}

    arena ObjectShapeTypeAnnotation as object_shape_type_annotation => {
        for field in object_shape_type_annotation.fields.iter() {
            walker.walk_shape_type_annotation_field(field, context);
        }
    }

    arena ShapeTypeAnnotation as shape_type_annotation => {
        for field in shape_type_annotation.fields.iter() {
            walker.walk_shape_type_annotation_field(field, context);
        }

        if let Some(additional_fields) = &shape_type_annotation.additional_fields {
            walker.walk_shape_type_annotation_additional_fields(additional_fields, context);
        }
    }

    arena ShapeTypeAnnotationField as shape_type_annotation_field => {
        walker.walk_shape_type_annotation_key(&shape_type_annotation_field.key, context);
        walker.walk_type_annotation(&shape_type_annotation_field.value, context);
    }

    arena ShapeTypeAnnotationAdditionalFields as shape_type_annotation_additional_fields => {
        walker.walk_type_annotation(shape_type_annotation_additional_fields.key_type, context);
        walker.walk_type_annotation(shape_type_annotation_additional_fields.value_type, context);
    }

    arena CallableTypeAnnotation as callable_type_annotation => {
        walker.walk_callable_type_kind(&callable_type_annotation.kind, context);
        for parameter in callable_type_annotation.parameters.iter().flatten() {
            walker.walk_callable_type_annotation_parameter(parameter, context);
        }

        if let Some(r#return) = callable_type_annotation.r#return {
            walker.walk_type_annotation(r#return, context);
        }
    }

    arena CallableTypeAnnotationParameter as callable_type_annotation_parameter => {
        if let Some(r#type) = callable_type_annotation_parameter.r#type {
            walker.walk_type_annotation(r#type, context);
        }

        if let Some(variable) = &callable_type_annotation_parameter.variable {
            walker.walk_direct_variable(variable, context);
        }
    }

    arena ConditionalTypeAnnotation as conditional_type_annotation => {
        walker.walk_type_annotation(conditional_type_annotation.target, context);
        walker.walk_type_annotation(conditional_type_annotation.subject, context);
        walker.walk_type_annotation(conditional_type_annotation.then, context);
        walker.walk_type_annotation(conditional_type_annotation.r#else, context);
    }

    arena ReferenceKind as reference_kind => {
        match reference_kind {
            ReferenceKind::Identifier(identifier)
            | ReferenceKind::Self_(identifier)
            | ReferenceKind::Static(identifier)
            | ReferenceKind::Parent(identifier) => walker.walk_identifier(identifier, context),
        }
    }

    arena MemberReferenceSelector as member_reference_selector => {
        match member_reference_selector {
            MemberReferenceSelector::Wildcard => {}
            MemberReferenceSelector::Exact(name)
            | MemberReferenceSelector::StartsWith(name)
            | MemberReferenceSelector::EndsWith(name) => walker.walk_name(name, context),
        }
    }

    arena GlobalSelector as global_selector => {
        match global_selector {
            GlobalSelector::StartsWith(identifier) | GlobalSelector::EndsWith(identifier) => {
                walker.walk_identifier(identifier, context);
            }
        }
    }

    arena ShapeTypeAnnotationKey as shape_type_annotation_key => {
        match shape_type_annotation_key {
            ShapeTypeAnnotationKey::String(_) | ShapeTypeAnnotationKey::Integer(_) => {}
            ShapeTypeAnnotationKey::ClassLikeConstant(identifier, name) => {
                walker.walk_identifier(identifier, context);
                walker.walk_name(name, context);
            }
        }
    }

    plain CallableTypeKind as callable_type_kind => {}

    arena Type as r#type => {
        walker.walk_type_kind(&r#type.kind, context);
    }

    arena TypeKind as type_kind => {
        match type_kind {
            TypeKind::Parenthesized(ty) => walker.walk_type(ty, context),
            TypeKind::Nullable(ty) => walker.walk_type(ty, context),
            TypeKind::Named(identifier)
            | TypeKind::Static(identifier)
            | TypeKind::Self_(identifier)
            | TypeKind::Parent(identifier) => walker.walk_identifier(identifier, context),
            TypeKind::Union(types) | TypeKind::Intersection(types) => {
                for r#type in types.iter() {
                    walker.walk_type(r#type, context);
                }
            }
            TypeKind::Null
            | TypeKind::Array
            | TypeKind::Callable
            | TypeKind::Void
            | TypeKind::Never
            | TypeKind::Float
            | TypeKind::Bool(_)
            | TypeKind::Integer
            | TypeKind::String
            | TypeKind::Object
            | TypeKind::Mixed
            | TypeKind::Iterable => {}
        }
    }

    plain EnumBackingType as enum_backing_type => {
        walker.walk_enum_backing_type_kind(&enum_backing_type.kind, context);
    }

    plain EnumBackingTypeKind as enum_backing_type_kind => {}

    plain Modifier as modifier => {
        walker.walk_modifier_kind(&modifier.kind, context);
    }

    plain ModifierKind as modifier_kind => {}

    plain Visibility as visibility => {
        walker.walk_visibility_kind(&visibility.kind, context);
    }

    plain VisibilityKind as visibility_kind => {}

    plain AnnotationError as annotation_error => {
        walker.walk_annotation_error_kind(&annotation_error.kind, context);
    }

    plain AnnotationErrorKind as annotation_error_kind => {}

    arena TypeAliasAnnotation as type_alias_annotation => {
        walker.walk_name(&type_alias_annotation.name, context);
        walker.walk_type_annotation(type_alias_annotation.r#type, context);
    }

    arena ImportedTypeAliasAnnotation as imported_type_alias_annotation => {
        walker.walk_name(&imported_type_alias_annotation.name, context);
        walker.walk_identifier(&imported_type_alias_annotation.from, context);
        if let Some(r#as) = &imported_type_alias_annotation.r#as {
            walker.walk_name(r#as, context);
        }
    }

    arena TypeParameterAnnotation as type_parameter_annotation => {
        walker.walk_name(&type_parameter_annotation.name, context);
        if let Some(bound) = type_parameter_annotation.bound {
            walker.walk_type_annotation(bound, context);
        }

        if let Some(default) = type_parameter_annotation.default {
            walker.walk_type_annotation(default, context);
        }
    }

    arena InheritedTypeParameterAnnotation as inherited_type_parameter_annotation => {
        walker.walk_type_parameter_defining_entity(&inherited_type_parameter_annotation.defining_entity, context);
        walker.walk_name(&inherited_type_parameter_annotation.name, context);
        if let Some(bound) = inherited_type_parameter_annotation.bound {
            walker.walk_type_annotation(bound, context);
        }

        if let Some(default) = inherited_type_parameter_annotation.default {
            walker.walk_type_annotation(default, context);
        }
    }

    arena TypeParameterDefiningEntity as type_parameter_defining_entity => {
        match type_parameter_defining_entity {
            TypeParameterDefiningEntity::ClassLike(identifier) | TypeParameterDefiningEntity::Function(identifier) => {
                walker.walk_identifier(identifier, context);
            }
            TypeParameterDefiningEntity::Method(identifier, name) => {
                walker.walk_identifier(identifier, context);
                walker.walk_name(name, context);
            }
            TypeParameterDefiningEntity::Closure(_) => {}
        }
    }

    arena WhereConstraintAnnotation as where_constraint_annotation => {
        walker.walk_name(&where_constraint_annotation.type_parameter, context);
        walker.walk_type_annotation(where_constraint_annotation.constraint, context);
    }

    arena ExtendsAnnotation as extends_annotation => {
        walker.walk_named_type_annotation(&extends_annotation.r#type, context);
    }

    arena RequireExtendsAnnotation as require_extends_annotation => {
        walker.walk_named_type_annotation(&require_extends_annotation.r#type, context);
    }

    arena ImplementsAnnotation as implements_annotation => {
        walker.walk_named_type_annotation(&implements_annotation.r#type, context);
    }

    arena RequireImplementsAnnotation as require_implements_annotation => {
        walker.walk_named_type_annotation(&require_implements_annotation.r#type, context);
    }

    arena UseAnnotation as use_annotation => {
        walker.walk_named_type_annotation(&use_annotation.r#type, context);
    }

    arena SealedAnnotation as sealed_annotation => {
        for r#type in sealed_annotation.types.iter() {
            walker.walk_named_type_annotation(r#type, context);
        }
    }

    arena MixinAnnotation as mixin_annotation => {
        walker.walk_type_annotation_kind(&mixin_annotation.r#type, context);
    }

    arena PropertyAnnotation as property_annotation => {
        walker.walk_property_annotation_kind(&property_annotation.kind, context);
        if let Some(r#type) = property_annotation.r#type {
            walker.walk_type_annotation(r#type, context);
        }

        walker.walk_direct_variable(&property_annotation.variable, context);
    }

    plain PropertyAnnotationKind as property_annotation_kind => {}

    arena ThrowsAnnotation as throws_annotation => {
        walker.walk_type_annotation(throws_annotation.r#type, context);
    }

    arena AssertAnnotation as assert_annotation => {
        walker.walk_assert_annotation_pattern(&assert_annotation.pattern, context);
        walker.walk_assert_annotation_target(&assert_annotation.target, context);
    }

    arena AssertAnnotationPattern as assert_annotation_pattern => {
        walker.walk_assert_annotation_pattern_kind(&assert_annotation_pattern.kind, context);
    }

    arena AssertAnnotationPatternKind as assert_annotation_pattern_kind => {
        match assert_annotation_pattern_kind {
            AssertAnnotationPatternKind::Type(r#type) => walker.walk_type_annotation(r#type, context),
            AssertAnnotationPatternKind::Truthy
            | AssertAnnotationPatternKind::Falsy
            | AssertAnnotationPatternKind::NonEmpty => {}
        }
    }

    arena AssertAnnotationTarget as assert_annotation_target => {
        walker.walk_assert_annotation_target_kind(&assert_annotation_target.kind, context);
    }

    arena AssertAnnotationTargetKind as assert_annotation_target_kind => {
        match assert_annotation_target_kind {
            AssertAnnotationTargetKind::Variable(variable) => walker.walk_direct_variable(variable, context),
            AssertAnnotationTargetKind::Method(variable, name)
            | AssertAnnotationTargetKind::Property(variable, name) => {
                walker.walk_direct_variable(variable, context);
                walker.walk_name(name, context);
            }
        }
    }

    arena SelfOutAnnotation as self_out_annotation => {
        walker.walk_type_annotation(self_out_annotation.r#type, context);
    }

    arena ClosureUseClauseVariable as closure_use_clause_variable => {
        walker.walk_direct_variable(&closure_use_clause_variable.variable, context);
    }

    arena Literal as literal => {
        walker.walk_literal_kind(&literal.kind, context);
    }

    arena LiteralKind as literal_kind => {
        match literal_kind {
            LiteralKind::String(node) => walker.walk_literal_string(node, context),
            LiteralKind::Integer(node) => walker.walk_literal_integer(node, context),
            LiteralKind::Float(node) => walker.walk_literal_float(node, context),
            LiteralKind::True | LiteralKind::False | LiteralKind::Null => {}
        }
    }

    arena LiteralString as literal_string => {
        walker.walk_literal_string_kind(&literal_string.kind, context);
    }

    plain LiteralStringKind as literal_string_kind => {}

    arena LiteralInteger as literal_integer => {}

    arena LiteralFloat as literal_float => {}

    plain MagicConstant as magic_constant => {
        walker.walk_magic_constant_kind(&magic_constant.kind, context);
    }

    plain MagicConstantKind as magic_constant_kind => {}

    plain Error as error => {
        walker.walk_error_kind(&error.kind, context);
    }

    plain ErrorKind as error_kind => {}
}
