#![allow(unused_variables)]

use crate::ir::IR;
use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::ir::attribute::Attribute;
use crate::ir::expression::Access;
use crate::ir::expression::ArrayElement;
use crate::ir::expression::Assignment;
use crate::ir::expression::Binary;
use crate::ir::expression::Call;
use crate::ir::expression::Callee;
use crate::ir::expression::CompositeStringPart;
use crate::ir::expression::Conditional;
use crate::ir::expression::Expression;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::Instantiation;
use crate::ir::expression::Match;
use crate::ir::expression::MatchArm;
use crate::ir::expression::PartialApplication;
use crate::ir::expression::UnaryPostfix;
use crate::ir::expression::UnaryPrefix;
use crate::ir::expression::Yield;
use crate::ir::expression::annotation::Annotation;
use crate::ir::expression::definition::AnonymousClass;
use crate::ir::expression::definition::ArrowFunction;
use crate::ir::expression::definition::Closure;
use crate::ir::expression::definition::DefinitionExpression;
use crate::ir::expression::definition::DefinitionExpressionKind;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::expression::selector::MemberSelector;
use crate::ir::hook::Hook;
use crate::ir::hook::HookBody;
use crate::ir::identifier::Identifier;
use crate::ir::literal::Literal;
use crate::ir::member::ClassLikeConstant;
use crate::ir::member::ClassLikeConstantItem;
use crate::ir::member::EnumCase;
use crate::ir::member::HookedProperty;
use crate::ir::member::Method;
use crate::ir::member::Property;
use crate::ir::member::PropertyItem;
use crate::ir::member::annotation::MethodAnnotation;
use crate::ir::name::Name;
use crate::ir::parameter::Parameter;
use crate::ir::parameter::annotation::ParameterAnnotation;
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
use crate::ir::statement::While;
use crate::ir::statement::annotation::VariableBindingAnnotation;
use crate::ir::statement::definition::Class;
use crate::ir::statement::definition::Constant;
use crate::ir::statement::definition::ConstantItem;
use crate::ir::statement::definition::DefinitionStatement;
use crate::ir::statement::definition::DefinitionStatementKind;
use crate::ir::statement::definition::Enum;
use crate::ir::statement::definition::Function;
use crate::ir::statement::definition::Interface;
use crate::ir::statement::definition::Trait;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use crate::ir::variable::Variable;

macro_rules! gen_mut_walker_methods {
    (generic $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&mut self, $name: &$node<'arena, S, D, E>, context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&mut self, $name: &$node<'arena, S, D, E>, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&mut self, $name: &$node<'arena, S, D, E>, context: &mut C) {}
        }
    };
    (arena $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&mut self, $name: &$node<'arena>, context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&mut self, $name: &$node<'arena>, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&mut self, $name: &$node<'arena>, context: &mut C) {}
        }
    };
}

macro_rules! gen_const_walker_methods {
    (generic $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&self, $name: &$node<'arena, S, D, E>, context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&self, $name: &$node<'arena, S, D, E>, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&self, $name: &$node<'arena, S, D, E>, context: &mut C) {}
        }
    };
    (arena $node:ident $name:ident $walker:ident $context:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $name>](&self, $name: &$node<'arena>, context: &mut C) {}
            #[inline]
            fn [<walk_ $name>](&self, $name: &$node<'arena>, $context: &mut C) {
                let $walker = self;
                $walker.[<walk_in_ $name>]($name, $context);
                $body
                $walker.[<walk_out_ $name>]($name, $context);
            }
            #[inline]
            fn [<walk_out_ $name>](&self, $name: &$node<'arena>, context: &mut C) {}
        }
    };
}

macro_rules! generate_walker {
    (
        using($walker:ident, $context:ident):
        $( $prefix:ident $node:ident as $name:ident => $body:block )*
    ) => {
        /// A trait for traversing the [`IR`] with mutable access to the walker.
        pub trait MutWalker<'arena, S, D, E, C>
        {
            $( gen_mut_walker_methods!($prefix $node $name $walker $context $body); )*
        }

        /// A trait for traversing the [`IR`] with shared access to the walker.
        pub trait Walker<'arena, S, D, E, C>
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
    }

    generic Statement as statement => {
        match &statement.kind {
            StatementKind::Inline(_) | StatementKind::HaltCompiler | StatementKind::Noop => {}
            StatementKind::Namespace(node) => walker.walk_namespace(node, context),
            StatementKind::Sequence(statements) => {
                for statement in statements.iter() {
                    walker.walk_statement(statement, context);
                }
            }
            StatementKind::Definition(node) => walker.walk_definition_statement(node, context),
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
            StatementKind::Echo(values) | StatementKind::Unset(values) => {
                for value in values.iter() {
                    walker.walk_expression(value, context);
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
        for item in declare.items {
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
        for case in switch.cases {
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

    arena VariableBindingAnnotation as variable_binding_annotation => {
        walker.walk_direct_variable(&variable_binding_annotation.variable, context);
        walker.walk_type_annotation(variable_binding_annotation.type_annotation, context);
    }

    generic DefinitionStatement as definition_statement => {
        match &definition_statement.kind {
            DefinitionStatementKind::Class(node) => walker.walk_class(node, context),
            DefinitionStatementKind::Interface(node) => walker.walk_interface(node, context),
            DefinitionStatementKind::Trait(node) => walker.walk_trait_definition(node, context),
            DefinitionStatementKind::Enum(node) => walker.walk_enum_definition(node, context),
            DefinitionStatementKind::Constant(node) => walker.walk_constant(node, context),
            DefinitionStatementKind::Function(node) => walker.walk_function(node, context),
        }
    }

    generic Class as class => {
        for attribute in class.attributes {
            walker.walk_attribute(attribute, context);
        }
        walker.walk_identifier(&class.name, context);
        for constant in class.constants {
            walker.walk_class_like_constant(constant, context);
        }
        for property in class.properties {
            walker.walk_property(property, context);
        }
        for hooked_property in class.hooked_properties {
            walker.walk_hooked_property(hooked_property, context);
        }
        for method in class.methods {
            walker.walk_method(method, context);
        }
        for method_annotation in class.method_annotations {
            walker.walk_method_annotation(method_annotation, context);
        }
    }

    generic Interface as interface => {
        for attribute in interface.attributes {
            walker.walk_attribute(attribute, context);
        }
        walker.walk_identifier(&interface.name, context);
        for constant in interface.constants {
            walker.walk_class_like_constant(constant, context);
        }
        for hooked_property in interface.hooked_properties {
            walker.walk_hooked_property(hooked_property, context);
        }
        for method in interface.methods {
            walker.walk_method(method, context);
        }
        for method_annotation in interface.method_annotations {
            walker.walk_method_annotation(method_annotation, context);
        }
    }

    generic Trait as trait_definition => {
        for attribute in trait_definition.attributes {
            walker.walk_attribute(attribute, context);
        }
        walker.walk_identifier(&trait_definition.name, context);
        for constant in trait_definition.constants {
            walker.walk_class_like_constant(constant, context);
        }
        for property in trait_definition.properties {
            walker.walk_property(property, context);
        }
        for hooked_property in trait_definition.hooked_properties {
            walker.walk_hooked_property(hooked_property, context);
        }
        for method in trait_definition.methods {
            walker.walk_method(method, context);
        }
        for method_annotation in trait_definition.method_annotations {
            walker.walk_method_annotation(method_annotation, context);
        }
    }

    generic Enum as enum_definition => {
        for attribute in enum_definition.attributes {
            walker.walk_attribute(attribute, context);
        }
        walker.walk_identifier(&enum_definition.name, context);
        for constant in enum_definition.constants {
            walker.walk_class_like_constant(constant, context);
        }
        for enum_case in enum_definition.enum_cases {
            walker.walk_enum_case(enum_case, context);
        }
        for method in enum_definition.methods {
            walker.walk_method(method, context);
        }
        for method_annotation in enum_definition.method_annotations {
            walker.walk_method_annotation(method_annotation, context);
        }
    }

    generic Constant as constant => {
        for attribute in constant.attributes {
            walker.walk_attribute(attribute, context);
        }
        if let Some(type_annotation) = constant.type_annotation {
            walker.walk_type_annotation(type_annotation, context);
        }
        for item in constant.items {
            walker.walk_constant_item(item, context);
        }
    }

    generic ConstantItem as constant_item => {
        walker.walk_identifier(&constant_item.name, context);
        walker.walk_expression(constant_item.value, context);
    }

    generic Function as function => {
        for attribute in function.attributes {
            walker.walk_attribute(attribute, context);
        }
        walker.walk_identifier(&function.name, context);
        for parameter in function.parameters {
            walker.walk_parameter(parameter, context);
        }
        if let Some(return_type_annotation) = function.return_type_annotation {
            walker.walk_type_annotation(return_type_annotation, context);
        }
        walker.walk_statement(function.body, context);
    }

    generic Method as method => {
        for attribute in method.attributes {
            walker.walk_attribute(attribute, context);
        }
        walker.walk_name(&method.name, context);
        for parameter in method.parameters {
            walker.walk_parameter(parameter, context);
        }
        if let Some(return_type_annotation) = method.return_type_annotation {
            walker.walk_type_annotation(return_type_annotation, context);
        }
        if let Some(body) = method.body {
            walker.walk_statement(body, context);
        }
    }

    generic Property as property => {
        for attribute in property.attributes {
            walker.walk_attribute(attribute, context);
        }
        if let Some(type_annotation) = property.type_annotation {
            walker.walk_type_annotation(type_annotation, context);
        }
        for item in property.items {
            walker.walk_property_item(item, context);
        }
    }

    generic PropertyItem as property_item => {
        walker.walk_direct_variable(&property_item.variable, context);
        if let Some(default_value) = property_item.default_value {
            walker.walk_expression(default_value, context);
        }
    }

    generic HookedProperty as hooked_property => {
        for attribute in hooked_property.attributes {
            walker.walk_attribute(attribute, context);
        }
        if let Some(type_annotation) = hooked_property.type_annotation {
            walker.walk_type_annotation(type_annotation, context);
        }
        walker.walk_property_item(&hooked_property.item, context);
        for hook in hooked_property.hooks {
            walker.walk_hook(hook, context);
        }
    }

    generic ClassLikeConstant as class_like_constant => {
        for attribute in class_like_constant.attributes {
            walker.walk_attribute(attribute, context);
        }
        if let Some(type_annotation) = class_like_constant.type_annotation {
            walker.walk_type_annotation(type_annotation, context);
        }
        for item in class_like_constant.items {
            walker.walk_class_like_constant_item(item, context);
        }
    }

    generic ClassLikeConstantItem as class_like_constant_item => {
        walker.walk_name(&class_like_constant_item.name, context);
        walker.walk_expression(class_like_constant_item.value, context);
    }

    generic EnumCase as enum_case => {
        for attribute in enum_case.attributes {
            walker.walk_attribute(attribute, context);
        }
        walker.walk_name(&enum_case.name, context);
        if let Some(value) = enum_case.value {
            walker.walk_expression(value, context);
        }
    }

    generic Hook as hook => {
        for attribute in hook.attributes {
            walker.walk_attribute(attribute, context);
        }
        walker.walk_name(&hook.name, context);
        for parameter in hook.parameters {
            walker.walk_parameter(parameter, context);
        }
        if let Some(body) = &hook.body {
            match body {
                HookBody::Expression(expression) => walker.walk_expression(expression, context),
                HookBody::Statements(statements) => {
                    for statement in statements.iter() {
                        walker.walk_statement(statement, context);
                    }
                }
            }
        }
    }

    generic Parameter as parameter => {
        for attribute in parameter.attributes {
            walker.walk_attribute(attribute, context);
        }
        if let Some(type_annotation) = parameter.type_annotation {
            walker.walk_type_annotation(type_annotation, context);
        }
        if let Some(out_annotation) = parameter.out_annotation {
            walker.walk_type_annotation(out_annotation, context);
        }
        walker.walk_direct_variable(&parameter.variable, context);
        if let Some(default_value) = parameter.default_value {
            walker.walk_expression(default_value, context);
        }
        for hook in parameter.hooks {
            walker.walk_hook(hook, context);
        }
    }

    generic MethodAnnotation as method_annotation => {
        walker.walk_name(&method_annotation.name, context);
        for parameter in method_annotation.parameters {
            walker.walk_parameter_annotation(parameter, context);
        }
        if let Some(return_type) = method_annotation.return_type {
            walker.walk_type_annotation(return_type, context);
        }
    }

    generic ParameterAnnotation as parameter_annotation => {
        walker.walk_direct_variable(&parameter_annotation.variable, context);
        if let Some(default_value) = parameter_annotation.default_value {
            walker.walk_expression(default_value, context);
        }
    }

    generic Attribute as attribute => {
        walker.walk_identifier(&attribute.class, context);
        for argument in attribute.arguments {
            walker.walk_argument(argument, context);
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
            PartialArgument::Placeholder | PartialArgument::VariadicPlaceholder => {}
        }
    }

    generic Expression as expression => {
        match &expression.kind {
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
            ExpressionKind::Definition(node) => walker.walk_definition_expression(node, context),
            ExpressionKind::Call(node) => walker.walk_call(node, context),
            ExpressionKind::PartialApplication(node) => walker.walk_partial_application(node, context),
            ExpressionKind::Access(node) => walker.walk_access(node, context),
            ExpressionKind::Isset(values) => {
                for value in values.iter() {
                    walker.walk_expression(value, context);
                }
            }
            ExpressionKind::Exit(arguments) => {
                for argument in arguments.iter() {
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
            ExpressionKind::MagicConstant(_)
            | ExpressionKind::Parent
            | ExpressionKind::Self_
            | ExpressionKind::Static
            | ExpressionKind::SyntaxError => {}
        }
    }

    generic Assignment as assignment => {
        walker.walk_expression(assignment.left, context);
        walker.walk_expression(assignment.right, context);
    }

    generic Annotation as annotation => {
        walker.walk_expression(annotation.expression, context);
        walker.walk_type_annotation(annotation.type_annotation, context);
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
        match array_element {
            ArrayElement::KeyValue(key, value) => {
                walker.walk_expression(key, context);
                walker.walk_expression(value, context);
            }
            ArrayElement::Value(value) | ArrayElement::Variadic(value) => {
                walker.walk_expression(value, context);
            }
            ArrayElement::Missing => {}
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
        for argument in instantiation.arguments {
            walker.walk_argument(argument, context);
        }
    }

    generic Call as call => {
        walker.walk_callee(&call.callee, context);
        for argument in call.arguments {
            walker.walk_argument(argument, context);
        }
    }

    generic PartialApplication as partial_application => {
        walker.walk_callee(&partial_application.callee, context);
        for argument in partial_application.arguments {
            walker.walk_partial_argument(argument, context);
        }
    }

    generic Callee as callee => {
        match callee {
            Callee::Function(expression) => walker.walk_expression(expression, context),
            Callee::Method(expression, selector)
            | Callee::NullsafeMethod(expression, selector)
            | Callee::StaticMethod(expression, selector) => {
                walker.walk_expression(expression, context);
                walker.walk_member_selector(selector, context);
            }
        }
    }

    generic Access as access => {
        match access {
            Access::Array(target, index) => {
                walker.walk_expression(target, context);
                walker.walk_expression(index, context);
            }
            Access::Property(target, selector) | Access::NullsafeProperty(target, selector) => {
                walker.walk_expression(target, context);
                walker.walk_member_selector(selector, context);
            }
            Access::StaticProperty(target, variable) => {
                walker.walk_expression(target, context);
                walker.walk_variable(variable, context);
            }
            Access::ClassConstant(target, selector) => {
                walker.walk_expression(target, context);
                walker.walk_constant_selector(selector, context);
            }
        }
    }

    generic MemberSelector as member_selector => {
        match member_selector {
            MemberSelector::Name(name) => walker.walk_name(name, context),
            MemberSelector::Variable(variable) => walker.walk_direct_variable(variable, context),
            MemberSelector::Expression(expression) => walker.walk_expression(expression, context),
        }
    }

    generic ConstantSelector as constant_selector => {
        match constant_selector {
            ConstantSelector::Name(name) => walker.walk_name(name, context),
            ConstantSelector::Expression(expression) => walker.walk_expression(expression, context),
        }
    }

    generic Yield as yield_expression => {
        match yield_expression {
            Yield::Nothing => {}
            Yield::Expression(value) | Yield::From(value) => walker.walk_expression(value, context),
            Yield::Pair(key, value) => {
                walker.walk_expression(key, context);
                walker.walk_expression(value, context);
            }
        }
    }

    generic Match as match_expression => {
        walker.walk_expression(match_expression.subject, context);
        for arm in match_expression.arms {
            walker.walk_match_arm(arm, context);
        }
    }

    generic MatchArm as match_arm => {
        match match_arm {
            MatchArm::Expression(conditions, body) => {
                for condition in conditions.iter() {
                    walker.walk_expression(condition, context);
                }
                walker.walk_expression(body, context);
            }
            MatchArm::Default(body) => walker.walk_expression(body, context),
        }
    }

    generic DefinitionExpression as definition_expression => {
        match &definition_expression.kind {
            DefinitionExpressionKind::AnonymousClass(node) => walker.walk_anonymous_class(node, context),
            DefinitionExpressionKind::ArrowFunction(node) => walker.walk_arrow_function(node, context),
            DefinitionExpressionKind::Closure(node) => walker.walk_closure(node, context),
        }
    }

    generic AnonymousClass as anonymous_class => {
        for attribute in anonymous_class.attributes {
            walker.walk_attribute(attribute, context);
        }
        for argument in anonymous_class.arguments {
            walker.walk_argument(argument, context);
        }
        for constant in anonymous_class.constants {
            walker.walk_class_like_constant(constant, context);
        }
        for property in anonymous_class.properties {
            walker.walk_property(property, context);
        }
        for hooked_property in anonymous_class.hooked_properties {
            walker.walk_hooked_property(hooked_property, context);
        }
        for method in anonymous_class.methods {
            walker.walk_method(method, context);
        }
    }

    generic Closure as closure => {
        for attribute in closure.attributes {
            walker.walk_attribute(attribute, context);
        }
        for parameter in closure.parameters {
            walker.walk_parameter(parameter, context);
        }
        for use_variable in closure.use_variables {
            walker.walk_direct_variable(&use_variable.variable, context);
        }
        if let Some(return_type_annotation) = closure.return_type_annotation {
            walker.walk_type_annotation(return_type_annotation, context);
        }
        walker.walk_statement(closure.body, context);
    }

    generic ArrowFunction as arrow_function => {
        for attribute in arrow_function.attributes {
            walker.walk_attribute(attribute, context);
        }
        for parameter in arrow_function.parameters {
            walker.walk_parameter(parameter, context);
        }
        if let Some(return_type_annotation) = arrow_function.return_type_annotation {
            walker.walk_type_annotation(return_type_annotation, context);
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

    arena Identifier as identifier => {}

    arena Name as name => {}

    arena DirectVariable as direct_variable => {}

    arena TypeAnnotation as type_annotation => {}

    arena Literal as literal => {}
}
