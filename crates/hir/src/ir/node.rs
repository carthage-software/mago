#![allow(unused_variables)]

use serde::Serialize;
use strum::Display;

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

/// A type-erased discriminant for every kind of [`Node`] in the IR.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[non_exhaustive]
pub enum NodeKind {
    IR,
    Statement,
    Namespace,
    Declare,
    DeclareItem,
    Try,
    TryCatchClause,
    Foreach,
    For,
    While,
    DoWhile,
    Switch,
    SwitchCase,
    If,
    StaticItem,
    GlobalItem,
    VariableBindingAnnotation,
    DefinitionStatement,
    Class,
    Interface,
    Trait,
    Enum,
    Constant,
    ConstantItem,
    Function,
    Method,
    Property,
    PropertyItem,
    HookedProperty,
    ClassLikeConstant,
    ClassLikeConstantItem,
    EnumCase,
    Hook,
    Parameter,
    MethodAnnotation,
    ParameterAnnotation,
    Attribute,
    Argument,
    PartialArgument,
    Expression,
    DefinitionExpression,
    AnonymousClass,
    Closure,
    ArrowFunction,
    Variable,
    Assignment,
    Annotation,
    Binary,
    UnaryPrefix,
    UnaryPostfix,
    Conditional,
    ArrayElement,
    CompositeStringPart,
    Instantiation,
    Call,
    PartialApplication,
    Callee,
    Access,
    MemberSelector,
    ConstantSelector,
    Yield,
    Match,
    MatchArm,
    Identifier,
    Name,
    DirectVariable,
    TypeAnnotation,
    Literal,
}

/// A type-erased reference to any node in the IR, enabling uniform traversal and `kind`-based inspection.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize)]
pub enum Node<'ast, 'arena, S, D, E> {
    IR(&'ast IR<'arena, S, D, E>),
    Statement(&'ast Statement<'arena, S, D, E>),
    Namespace(&'ast Namespace<'arena, S, D, E>),
    Declare(&'ast Declare<'arena, S, D, E>),
    DeclareItem(&'ast DeclareItem<'arena, S, D, E>),
    Try(&'ast Try<'arena, S, D, E>),
    TryCatchClause(&'ast TryCatchClause<'arena, S, D, E>),
    Foreach(&'ast Foreach<'arena, S, D, E>),
    For(&'ast For<'arena, S, D, E>),
    While(&'ast While<'arena, S, D, E>),
    DoWhile(&'ast DoWhile<'arena, S, D, E>),
    Switch(&'ast Switch<'arena, S, D, E>),
    SwitchCase(&'ast SwitchCase<'arena, S, D, E>),
    If(&'ast If<'arena, S, D, E>),
    StaticItem(&'ast StaticItem<'arena, S, D, E>),
    GlobalItem(&'ast GlobalItem<'arena, S, D, E>),
    VariableBindingAnnotation(&'ast VariableBindingAnnotation<'arena>),
    DefinitionStatement(&'ast DefinitionStatement<'arena, S, D, E>),
    Class(&'ast Class<'arena, S, D, E>),
    Interface(&'ast Interface<'arena, S, D, E>),
    Trait(&'ast Trait<'arena, S, D, E>),
    Enum(&'ast Enum<'arena, S, D, E>),
    Constant(&'ast Constant<'arena, S, D, E>),
    ConstantItem(&'ast ConstantItem<'arena, S, D, E>),
    Function(&'ast Function<'arena, S, D, E>),
    Method(&'ast Method<'arena, S, D, E>),
    Property(&'ast Property<'arena, S, D, E>),
    PropertyItem(&'ast PropertyItem<'arena, S, D, E>),
    HookedProperty(&'ast HookedProperty<'arena, S, D, E>),
    ClassLikeConstant(&'ast ClassLikeConstant<'arena, S, D, E>),
    ClassLikeConstantItem(&'ast ClassLikeConstantItem<'arena, S, D, E>),
    EnumCase(&'ast EnumCase<'arena, S, D, E>),
    Hook(&'ast Hook<'arena, S, D, E>),
    Parameter(&'ast Parameter<'arena, S, D, E>),
    MethodAnnotation(&'ast MethodAnnotation<'arena, S, D, E>),
    ParameterAnnotation(&'ast ParameterAnnotation<'arena, S, D, E>),
    Attribute(&'ast Attribute<'arena, S, D, E>),
    Argument(&'ast Argument<'arena, S, D, E>),
    PartialArgument(&'ast PartialArgument<'arena, S, D, E>),
    Expression(&'ast Expression<'arena, S, D, E>),
    DefinitionExpression(&'ast DefinitionExpression<'arena, S, D, E>),
    AnonymousClass(&'ast AnonymousClass<'arena, S, D, E>),
    Closure(&'ast Closure<'arena, S, D, E>),
    ArrowFunction(&'ast ArrowFunction<'arena, S, D, E>),
    Variable(&'ast Variable<'arena, S, D, E>),
    Assignment(&'ast Assignment<'arena, S, D, E>),
    Annotation(&'ast Annotation<'arena, S, D, E>),
    Binary(&'ast Binary<'arena, S, D, E>),
    UnaryPrefix(&'ast UnaryPrefix<'arena, S, D, E>),
    UnaryPostfix(&'ast UnaryPostfix<'arena, S, D, E>),
    Conditional(&'ast Conditional<'arena, S, D, E>),
    ArrayElement(&'ast ArrayElement<'arena, S, D, E>),
    CompositeStringPart(&'ast CompositeStringPart<'arena, S, D, E>),
    Instantiation(&'ast Instantiation<'arena, S, D, E>),
    Call(&'ast Call<'arena, S, D, E>),
    PartialApplication(&'ast PartialApplication<'arena, S, D, E>),
    Callee(&'ast Callee<'arena, S, D, E>),
    Access(&'ast Access<'arena, S, D, E>),
    MemberSelector(&'ast MemberSelector<'arena, S, D, E>),
    ConstantSelector(&'ast ConstantSelector<'arena, S, D, E>),
    Yield(&'ast Yield<'arena, S, D, E>),
    Match(&'ast Match<'arena, S, D, E>),
    MatchArm(&'ast MatchArm<'arena, S, D, E>),
    Identifier(&'ast Identifier<'arena>),
    Name(&'ast Name<'arena>),
    DirectVariable(&'ast DirectVariable<'arena>),
    TypeAnnotation(&'ast TypeAnnotation<'arena>),
    Literal(&'ast Literal<'arena>),
}

impl<'ast, 'arena, S, D, E> Node<'ast, 'arena, S, D, E> {
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        match self {
            Self::IR(_) => NodeKind::IR,
            Self::Statement(_) => NodeKind::Statement,
            Self::Namespace(_) => NodeKind::Namespace,
            Self::Declare(_) => NodeKind::Declare,
            Self::DeclareItem(_) => NodeKind::DeclareItem,
            Self::Try(_) => NodeKind::Try,
            Self::TryCatchClause(_) => NodeKind::TryCatchClause,
            Self::Foreach(_) => NodeKind::Foreach,
            Self::For(_) => NodeKind::For,
            Self::While(_) => NodeKind::While,
            Self::DoWhile(_) => NodeKind::DoWhile,
            Self::Switch(_) => NodeKind::Switch,
            Self::SwitchCase(_) => NodeKind::SwitchCase,
            Self::If(_) => NodeKind::If,
            Self::StaticItem(_) => NodeKind::StaticItem,
            Self::GlobalItem(_) => NodeKind::GlobalItem,
            Self::VariableBindingAnnotation(_) => NodeKind::VariableBindingAnnotation,
            Self::DefinitionStatement(_) => NodeKind::DefinitionStatement,
            Self::Class(_) => NodeKind::Class,
            Self::Interface(_) => NodeKind::Interface,
            Self::Trait(_) => NodeKind::Trait,
            Self::Enum(_) => NodeKind::Enum,
            Self::Constant(_) => NodeKind::Constant,
            Self::ConstantItem(_) => NodeKind::ConstantItem,
            Self::Function(_) => NodeKind::Function,
            Self::Method(_) => NodeKind::Method,
            Self::Property(_) => NodeKind::Property,
            Self::PropertyItem(_) => NodeKind::PropertyItem,
            Self::HookedProperty(_) => NodeKind::HookedProperty,
            Self::ClassLikeConstant(_) => NodeKind::ClassLikeConstant,
            Self::ClassLikeConstantItem(_) => NodeKind::ClassLikeConstantItem,
            Self::EnumCase(_) => NodeKind::EnumCase,
            Self::Hook(_) => NodeKind::Hook,
            Self::Parameter(_) => NodeKind::Parameter,
            Self::MethodAnnotation(_) => NodeKind::MethodAnnotation,
            Self::ParameterAnnotation(_) => NodeKind::ParameterAnnotation,
            Self::Attribute(_) => NodeKind::Attribute,
            Self::Argument(_) => NodeKind::Argument,
            Self::PartialArgument(_) => NodeKind::PartialArgument,
            Self::Expression(_) => NodeKind::Expression,
            Self::DefinitionExpression(_) => NodeKind::DefinitionExpression,
            Self::AnonymousClass(_) => NodeKind::AnonymousClass,
            Self::Closure(_) => NodeKind::Closure,
            Self::ArrowFunction(_) => NodeKind::ArrowFunction,
            Self::Variable(_) => NodeKind::Variable,
            Self::Assignment(_) => NodeKind::Assignment,
            Self::Annotation(_) => NodeKind::Annotation,
            Self::Binary(_) => NodeKind::Binary,
            Self::UnaryPrefix(_) => NodeKind::UnaryPrefix,
            Self::UnaryPostfix(_) => NodeKind::UnaryPostfix,
            Self::Conditional(_) => NodeKind::Conditional,
            Self::ArrayElement(_) => NodeKind::ArrayElement,
            Self::CompositeStringPart(_) => NodeKind::CompositeStringPart,
            Self::Instantiation(_) => NodeKind::Instantiation,
            Self::Call(_) => NodeKind::Call,
            Self::PartialApplication(_) => NodeKind::PartialApplication,
            Self::Callee(_) => NodeKind::Callee,
            Self::Access(_) => NodeKind::Access,
            Self::MemberSelector(_) => NodeKind::MemberSelector,
            Self::ConstantSelector(_) => NodeKind::ConstantSelector,
            Self::Yield(_) => NodeKind::Yield,
            Self::Match(_) => NodeKind::Match,
            Self::MatchArm(_) => NodeKind::MatchArm,
            Self::Identifier(_) => NodeKind::Identifier,
            Self::Name(_) => NodeKind::Name,
            Self::DirectVariable(_) => NodeKind::DirectVariable,
            Self::TypeAnnotation(_) => NodeKind::TypeAnnotation,
            Self::Literal(_) => NodeKind::Literal,
        }
    }

    #[inline]
    pub fn visit_children<F>(&self, mut visit: F)
    where
        F: FnMut(Node<'ast, 'arena, S, D, E>),
    {
        match *self {
            Self::IR(node) => {
                for statement in node.statements {
                    visit(Node::Statement(statement));
                }
            }
            Self::Statement(node) => match &node.kind {
                StatementKind::Inline(_) | StatementKind::HaltCompiler | StatementKind::Noop => {}
                StatementKind::Namespace(namespace) => visit(Node::Namespace(namespace)),
                StatementKind::Sequence(statements) => {
                    for statement in statements.iter() {
                        visit(Node::Statement(statement));
                    }
                }
                StatementKind::Definition(definition) => visit(Node::DefinitionStatement(definition)),
                StatementKind::Declare(declare) => visit(Node::Declare(declare)),
                StatementKind::Goto(name) | StatementKind::Label(name) => visit(Node::Name(name)),
                StatementKind::Try(try_statement) => visit(Node::Try(try_statement)),
                StatementKind::Foreach(foreach) => visit(Node::Foreach(foreach)),
                StatementKind::For(for_loop) => visit(Node::For(for_loop)),
                StatementKind::While(while_loop) => visit(Node::While(while_loop)),
                StatementKind::DoWhile(do_while) => visit(Node::DoWhile(do_while)),
                StatementKind::Continue(value) | StatementKind::Break(value) | StatementKind::Return(value) => {
                    if let Some(expression) = value {
                        visit(Node::Expression(expression));
                    }
                }
                StatementKind::Switch(switch) => visit(Node::Switch(switch)),
                StatementKind::If(if_statement) => visit(Node::If(if_statement)),
                StatementKind::Expression(expression) => visit(Node::Expression(expression)),
                StatementKind::Echo(values) | StatementKind::Unset(values) => {
                    for value in values.iter() {
                        visit(Node::Expression(value));
                    }
                }
                StatementKind::Global(items) => {
                    for item in items.iter() {
                        visit(Node::GlobalItem(item));
                    }
                }
                StatementKind::Static(items) => {
                    for item in items.iter() {
                        visit(Node::StaticItem(item));
                    }
                }
                StatementKind::VariableBindingAnnotation(binding) => visit(Node::VariableBindingAnnotation(binding)),
            },

            Self::Namespace(node) => {
                if let Some(name) = node.name {
                    visit(Node::Identifier(name));
                }

                visit(Node::Statement(node.statement));
            }
            Self::Declare(node) => {
                for item in node.items {
                    visit(Node::DeclareItem(item));
                }

                visit(Node::Statement(node.statement));
            }
            Self::DeclareItem(node) => {
                visit(Node::Name(&node.name));
                if let Some(value) = node.value {
                    visit(Node::Expression(value));
                }
            }
            Self::Try(node) => {
                visit(Node::Statement(node.statement));
                for catch_clause in node.catch_clauses {
                    visit(Node::TryCatchClause(catch_clause));
                }

                if let Some(finally_clause) = node.finally_clause {
                    visit(Node::Statement(finally_clause));
                }
            }
            Self::TryCatchClause(node) => {
                if let Some(variable) = &node.variable {
                    visit(Node::DirectVariable(variable));
                }

                visit(Node::Statement(node.statement));
            }
            Self::Foreach(node) => {
                visit(Node::Expression(node.expression));
                if let Some(key) = node.key {
                    visit(Node::Expression(key));
                }

                visit(Node::Expression(node.value));
                visit(Node::Statement(node.statement));
            }
            Self::For(node) => {
                for initialization in node.initializations {
                    visit(Node::Expression(initialization));
                }

                for condition in node.conditions {
                    visit(Node::Expression(condition));
                }

                for increment in node.increments {
                    visit(Node::Expression(increment));
                }

                visit(Node::Statement(node.statement));
            }
            Self::While(node) => {
                visit(Node::Expression(node.condition));
                visit(Node::Statement(node.statement));
            }
            Self::DoWhile(node) => {
                visit(Node::Statement(node.statement));
                visit(Node::Expression(node.condition));
            }
            Self::Switch(node) => {
                visit(Node::Expression(node.subject));
                for case in node.cases {
                    visit(Node::SwitchCase(case));
                }
            }
            Self::SwitchCase(node) => match node {
                SwitchCase::Expression(expression, statement) => {
                    visit(Node::Expression(expression));
                    visit(Node::Statement(statement));
                }
                SwitchCase::Default(statement) => visit(Node::Statement(statement)),
            },

            Self::If(node) => {
                visit(Node::Expression(node.condition));
                visit(Node::Statement(node.then));
                if let Some(r#else) = node.r#else {
                    visit(Node::Statement(r#else));
                }
            }
            Self::StaticItem(node) => {
                visit(Node::DirectVariable(&node.variable));
                if let Some(type_annotation) = node.type_annotation {
                    visit(Node::TypeAnnotation(type_annotation));
                }

                if let Some(value) = node.value {
                    visit(Node::Expression(value));
                }
            }
            Self::GlobalItem(node) => {
                visit(Node::Variable(&node.variable));
                if let Some(type_annotation) = node.type_annotation {
                    visit(Node::TypeAnnotation(type_annotation));
                }
            }
            Self::VariableBindingAnnotation(node) => {
                visit(Node::DirectVariable(&node.variable));
                visit(Node::TypeAnnotation(node.type_annotation));
            }
            Self::DefinitionStatement(node) => match &node.kind {
                DefinitionStatementKind::Class(class) => visit(Node::Class(class)),
                DefinitionStatementKind::Interface(interface) => visit(Node::Interface(interface)),
                DefinitionStatementKind::Trait(trait_definition) => visit(Node::Trait(trait_definition)),
                DefinitionStatementKind::Enum(enum_definition) => visit(Node::Enum(enum_definition)),
                DefinitionStatementKind::Constant(constant) => visit(Node::Constant(constant)),
                DefinitionStatementKind::Function(function) => visit(Node::Function(function)),
            },

            Self::Class(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                visit(Node::Identifier(&node.name));
                for constant in node.constants {
                    visit(Node::ClassLikeConstant(constant));
                }

                for property in node.properties {
                    visit(Node::Property(property));
                }

                for hooked_property in node.hooked_properties {
                    visit(Node::HookedProperty(hooked_property));
                }

                for method in node.methods {
                    visit(Node::Method(method));
                }

                for method_annotation in node.method_annotations {
                    visit(Node::MethodAnnotation(method_annotation));
                }
            }
            Self::Interface(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                visit(Node::Identifier(&node.name));
                for constant in node.constants {
                    visit(Node::ClassLikeConstant(constant));
                }

                for hooked_property in node.hooked_properties {
                    visit(Node::HookedProperty(hooked_property));
                }

                for method in node.methods {
                    visit(Node::Method(method));
                }

                for method_annotation in node.method_annotations {
                    visit(Node::MethodAnnotation(method_annotation));
                }
            }
            Self::Trait(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                visit(Node::Identifier(&node.name));
                for constant in node.constants {
                    visit(Node::ClassLikeConstant(constant));
                }

                for property in node.properties {
                    visit(Node::Property(property));
                }

                for hooked_property in node.hooked_properties {
                    visit(Node::HookedProperty(hooked_property));
                }

                for method in node.methods {
                    visit(Node::Method(method));
                }

                for method_annotation in node.method_annotations {
                    visit(Node::MethodAnnotation(method_annotation));
                }
            }
            Self::Enum(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                visit(Node::Identifier(&node.name));
                for constant in node.constants {
                    visit(Node::ClassLikeConstant(constant));
                }

                for enum_case in node.enum_cases {
                    visit(Node::EnumCase(enum_case));
                }

                for method in node.methods {
                    visit(Node::Method(method));
                }

                for method_annotation in node.method_annotations {
                    visit(Node::MethodAnnotation(method_annotation));
                }
            }
            Self::Constant(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                if let Some(type_annotation) = node.type_annotation {
                    visit(Node::TypeAnnotation(type_annotation));
                }

                for item in node.items {
                    visit(Node::ConstantItem(item));
                }
            }
            Self::ConstantItem(node) => {
                visit(Node::Identifier(&node.name));
                visit(Node::Expression(node.value));
            }
            Self::Function(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                visit(Node::Identifier(&node.name));
                for parameter in node.parameters {
                    visit(Node::Parameter(parameter));
                }

                if let Some(return_type_annotation) = node.return_type_annotation {
                    visit(Node::TypeAnnotation(return_type_annotation));
                }

                visit(Node::Statement(node.body));
            }
            Self::Method(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                visit(Node::Name(&node.name));
                for parameter in node.parameters {
                    visit(Node::Parameter(parameter));
                }

                if let Some(return_type_annotation) = node.return_type_annotation {
                    visit(Node::TypeAnnotation(return_type_annotation));
                }

                if let Some(body) = node.body {
                    visit(Node::Statement(body));
                }
            }
            Self::Property(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                if let Some(type_annotation) = node.type_annotation {
                    visit(Node::TypeAnnotation(type_annotation));
                }

                for item in node.items {
                    visit(Node::PropertyItem(item));
                }
            }
            Self::PropertyItem(node) => {
                visit(Node::DirectVariable(&node.variable));
                if let Some(default_value) = node.default_value {
                    visit(Node::Expression(default_value));
                }
            }
            Self::HookedProperty(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                if let Some(type_annotation) = node.type_annotation {
                    visit(Node::TypeAnnotation(type_annotation));
                }

                visit(Node::PropertyItem(&node.item));
                for hook in node.hooks {
                    visit(Node::Hook(hook));
                }
            }
            Self::ClassLikeConstant(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                if let Some(type_annotation) = node.type_annotation {
                    visit(Node::TypeAnnotation(type_annotation));
                }

                for item in node.items {
                    visit(Node::ClassLikeConstantItem(item));
                }
            }
            Self::ClassLikeConstantItem(node) => {
                visit(Node::Name(&node.name));
                visit(Node::Expression(node.value));
            }
            Self::EnumCase(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                visit(Node::Name(&node.name));
                if let Some(value) = node.value {
                    visit(Node::Expression(value));
                }
            }
            Self::Hook(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                visit(Node::Name(&node.name));
                for parameter in node.parameters {
                    visit(Node::Parameter(parameter));
                }

                if let Some(body) = &node.body {
                    match body {
                        HookBody::Expression(expression) => visit(Node::Expression(expression)),
                        HookBody::Statements(statements) => {
                            for statement in statements.iter() {
                                visit(Node::Statement(statement));
                            }
                        }
                    }
                }
            }
            Self::Parameter(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                if let Some(type_annotation) = node.type_annotation {
                    visit(Node::TypeAnnotation(type_annotation));
                }

                if let Some(out_annotation) = node.out_annotation {
                    visit(Node::TypeAnnotation(out_annotation));
                }

                visit(Node::DirectVariable(&node.variable));
                if let Some(default_value) = node.default_value {
                    visit(Node::Expression(default_value));
                }

                for hook in node.hooks {
                    visit(Node::Hook(hook));
                }
            }
            Self::MethodAnnotation(node) => {
                visit(Node::Name(&node.name));
                for parameter in node.parameters {
                    visit(Node::ParameterAnnotation(parameter));
                }

                if let Some(return_type) = node.return_type {
                    visit(Node::TypeAnnotation(return_type));
                }
            }
            Self::ParameterAnnotation(node) => {
                visit(Node::DirectVariable(&node.variable));
                if let Some(default_value) = node.default_value {
                    visit(Node::Expression(default_value));
                }
            }
            Self::Attribute(node) => {
                visit(Node::Identifier(&node.class));
                for argument in node.arguments {
                    visit(Node::Argument(argument));
                }
            }
            Self::Argument(node) => match node {
                Argument::Value(expression) | Argument::Variadic(expression) => {
                    visit(Node::Expression(expression));
                }
                Argument::Named(name, expression) => {
                    visit(Node::Name(name));
                    visit(Node::Expression(expression));
                }
            },

            Self::PartialArgument(node) => match node {
                PartialArgument::Value(expression) | PartialArgument::Variadic(expression) => {
                    visit(Node::Expression(expression));
                }
                PartialArgument::Named(name, expression) => {
                    visit(Node::Name(name));
                    visit(Node::Expression(expression));
                }
                PartialArgument::NamedPlaceholder(name) => visit(Node::Name(name)),
                PartialArgument::Placeholder | PartialArgument::VariadicPlaceholder => {}
            },

            Self::Expression(node) => match &node.kind {
                ExpressionKind::Binary(binary) => visit(Node::Binary(binary)),
                ExpressionKind::UnaryPrefix(unary_prefix) => visit(Node::UnaryPrefix(unary_prefix)),
                ExpressionKind::UnaryPostfix(unary_postfix) => visit(Node::UnaryPostfix(unary_postfix)),
                ExpressionKind::Literal(literal) => visit(Node::Literal(literal)),
                ExpressionKind::CompositeString(parts) | ExpressionKind::ShellExecute(parts) => {
                    for part in parts.iter() {
                        visit(Node::CompositeStringPart(part));
                    }
                }
                ExpressionKind::Assignment(assignment) => visit(Node::Assignment(assignment)),
                ExpressionKind::Annotation(annotation) => visit(Node::Annotation(annotation)),
                ExpressionKind::Conditional(conditional) => visit(Node::Conditional(conditional)),
                ExpressionKind::Array(elements) | ExpressionKind::List(elements) => {
                    for element in elements.iter() {
                        visit(Node::ArrayElement(element));
                    }
                }
                ExpressionKind::ArrayAppend(expression)
                | ExpressionKind::Clone(expression)
                | ExpressionKind::Empty(expression)
                | ExpressionKind::Eval(expression)
                | ExpressionKind::Include(expression)
                | ExpressionKind::IncludeOnce(expression)
                | ExpressionKind::Require(expression)
                | ExpressionKind::RequireOnce(expression)
                | ExpressionKind::Print(expression)
                | ExpressionKind::Throw(expression) => visit(Node::Expression(expression)),
                ExpressionKind::Definition(definition) => visit(Node::DefinitionExpression(definition)),
                ExpressionKind::Call(call) => visit(Node::Call(call)),
                ExpressionKind::PartialApplication(partial_application) => {
                    visit(Node::PartialApplication(partial_application))
                }
                ExpressionKind::Access(access) => visit(Node::Access(access)),
                ExpressionKind::Isset(values) => {
                    for value in values.iter() {
                        visit(Node::Expression(value));
                    }
                }
                ExpressionKind::Exit(arguments) => {
                    for argument in arguments.iter() {
                        visit(Node::Argument(argument));
                    }
                }
                ExpressionKind::Constant(identifier) | ExpressionKind::Identifier(identifier) => {
                    visit(Node::Identifier(identifier));
                }
                ExpressionKind::Instantiation(instantiation) => visit(Node::Instantiation(instantiation)),
                ExpressionKind::Variable(variable) => visit(Node::Variable(variable)),
                ExpressionKind::Yield(r#yield) => visit(Node::Yield(r#yield)),
                ExpressionKind::Match(r#match) => visit(Node::Match(r#match)),
                ExpressionKind::MagicConstant(_)
                | ExpressionKind::Parent
                | ExpressionKind::Self_
                | ExpressionKind::Static
                | ExpressionKind::SyntaxError => {}
            },

            Self::DefinitionExpression(node) => match &node.kind {
                DefinitionExpressionKind::AnonymousClass(anonymous_class) => {
                    visit(Node::AnonymousClass(anonymous_class))
                }
                DefinitionExpressionKind::ArrowFunction(arrow_function) => visit(Node::ArrowFunction(arrow_function)),
                DefinitionExpressionKind::Closure(closure) => visit(Node::Closure(closure)),
            },

            Self::AnonymousClass(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                for argument in node.arguments {
                    visit(Node::Argument(argument));
                }

                for constant in node.constants {
                    visit(Node::ClassLikeConstant(constant));
                }

                for property in node.properties {
                    visit(Node::Property(property));
                }

                for hooked_property in node.hooked_properties {
                    visit(Node::HookedProperty(hooked_property));
                }

                for method in node.methods {
                    visit(Node::Method(method));
                }
            }
            Self::Closure(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                for parameter in node.parameters {
                    visit(Node::Parameter(parameter));
                }

                for use_variable in node.use_variables {
                    visit(Node::DirectVariable(&use_variable.variable));
                }

                if let Some(return_type_annotation) = node.return_type_annotation {
                    visit(Node::TypeAnnotation(return_type_annotation));
                }

                visit(Node::Statement(node.body));
            }
            Self::ArrowFunction(node) => {
                for attribute in node.attributes {
                    visit(Node::Attribute(attribute));
                }

                for parameter in node.parameters {
                    visit(Node::Parameter(parameter));
                }

                if let Some(return_type_annotation) = node.return_type_annotation {
                    visit(Node::TypeAnnotation(return_type_annotation));
                }

                visit(Node::Expression(node.expression));
            }
            Self::Variable(node) => match node {
                Variable::Direct(direct) => visit(Node::DirectVariable(direct)),
                Variable::Indirect(expression) => visit(Node::Expression(expression)),
                Variable::Nested(nested) => visit(Node::Variable(nested)),
            },

            Self::Assignment(node) => {
                visit(Node::Expression(node.left));
                visit(Node::Expression(node.right));
            }
            Self::Annotation(node) => {
                visit(Node::Expression(node.expression));
                visit(Node::TypeAnnotation(node.type_annotation));
            }
            Self::Binary(node) => {
                visit(Node::Expression(node.left));
                visit(Node::Expression(node.right));
            }
            Self::UnaryPrefix(node) => {
                visit(Node::Expression(node.operand));
            }
            Self::UnaryPostfix(node) => {
                visit(Node::Expression(node.operand));
            }
            Self::Conditional(node) => {
                visit(Node::Expression(node.condition));
                if let Some(then) = node.then {
                    visit(Node::Expression(then));
                }

                visit(Node::Expression(node.r#else));
            }
            Self::ArrayElement(node) => match node {
                ArrayElement::KeyValue(key, value) => {
                    visit(Node::Expression(key));
                    visit(Node::Expression(value));
                }
                ArrayElement::Value(value) | ArrayElement::Variadic(value) => {
                    visit(Node::Expression(value));
                }
                ArrayElement::Missing => {}
            },

            Self::CompositeStringPart(node) => match node {
                CompositeStringPart::Literal(_) => {}
                CompositeStringPart::Expression(expression) => visit(Node::Expression(expression)),
            },

            Self::Instantiation(node) => {
                visit(Node::Expression(node.class));
                for argument in node.arguments {
                    visit(Node::Argument(argument));
                }
            }
            Self::Call(node) => {
                visit(Node::Callee(&node.callee));
                for argument in node.arguments {
                    visit(Node::Argument(argument));
                }
            }
            Self::PartialApplication(node) => {
                visit(Node::Callee(&node.callee));
                for argument in node.arguments {
                    visit(Node::PartialArgument(argument));
                }
            }
            Self::Callee(node) => match node {
                Callee::Function(expression) => visit(Node::Expression(expression)),
                Callee::Method(expression, selector)
                | Callee::NullsafeMethod(expression, selector)
                | Callee::StaticMethod(expression, selector) => {
                    visit(Node::Expression(expression));
                    visit(Node::MemberSelector(selector));
                }
            },

            Self::Access(node) => match node {
                Access::Array(target, index) => {
                    visit(Node::Expression(target));
                    visit(Node::Expression(index));
                }
                Access::Property(target, selector) | Access::NullsafeProperty(target, selector) => {
                    visit(Node::Expression(target));
                    visit(Node::MemberSelector(selector));
                }
                Access::StaticProperty(target, variable) => {
                    visit(Node::Expression(target));
                    visit(Node::Variable(variable));
                }
                Access::ClassConstant(target, selector) => {
                    visit(Node::Expression(target));
                    visit(Node::ConstantSelector(selector));
                }
            },

            Self::MemberSelector(node) => match node {
                MemberSelector::Name(name) => visit(Node::Name(name)),
                MemberSelector::Variable(variable) => visit(Node::DirectVariable(variable)),
                MemberSelector::Expression(expression) => visit(Node::Expression(expression)),
            },

            Self::ConstantSelector(node) => match node {
                ConstantSelector::Name(name) => visit(Node::Name(name)),
                ConstantSelector::Expression(expression) => visit(Node::Expression(expression)),
            },

            Self::Yield(node) => match node {
                Yield::Nothing => {}
                Yield::Expression(value) | Yield::From(value) => visit(Node::Expression(value)),
                Yield::Pair(key, value) => {
                    visit(Node::Expression(key));
                    visit(Node::Expression(value));
                }
            },

            Self::Match(node) => {
                visit(Node::Expression(node.subject));
                for arm in node.arms {
                    visit(Node::MatchArm(arm));
                }
            }
            Self::MatchArm(node) => match node {
                MatchArm::Expression(conditions, body) => {
                    for condition in conditions.iter() {
                        visit(Node::Expression(condition));
                    }
                    visit(Node::Expression(body));
                }
                MatchArm::Default(body) => visit(Node::Expression(body)),
            },

            Self::Identifier(node) => {}
            Self::Name(node) => {}
            Self::DirectVariable(node) => {}
            Self::TypeAnnotation(node) => {}
            Self::Literal(node) => {}
        }
    }

    #[inline]
    #[must_use]
    pub fn children(&self) -> Vec<Node<'ast, 'arena, S, D, E>> {
        let mut children = Vec::new();
        self.visit_children(|child| children.push(child));
        children
    }
}
