use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::ir::expression::AccessKind;
use crate::ir::expression::ArrayElementKind;
use crate::ir::expression::CalleeKind;
use crate::ir::expression::CompositeStringPart;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::MatchArmKind;
use crate::ir::expression::YieldKind;
use crate::ir::expression::selector::ConstantSelectorKind;
use crate::ir::expression::selector::MemberSelectorKind;
use crate::ir::item::annotation::effect::AssertAnnotationPatternKind;
use crate::ir::item::annotation::effect::AssertAnnotationTargetKind;
use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::expression::ItemExpressionKind;
use crate::ir::item::member::MemberItemKind;
use crate::ir::item::member::hook::HookBodyKind;
use crate::ir::item::member::trait_use::TraitUseAdaptation;
use crate::ir::item::statement::ItemStatementKind;
use crate::ir::literal::LiteralKind;
use crate::ir::statement::NamespaceBody;
use crate::ir::statement::StatementKind;
use crate::ir::statement::SwitchCaseKind;
use crate::ir::r#type::TypeKind;
use crate::ir::r#type::annotation::GlobalSelector;
use crate::ir::r#type::annotation::MemberReferenceSelector;
use crate::ir::r#type::annotation::ReferenceKind;
use crate::ir::r#type::annotation::ShapeTypeAnnotationKey;
use crate::ir::r#type::annotation::TypeAnnotationKind;
use crate::ir::variable::Variable;
use crate::node::Node;

impl<'ir, 'arena, I, S, E> Node<'ir, 'arena, I, S, E> {
    pub fn visit_children<F>(&self, mut f: F)
    where
        F: FnMut(Node<'ir, 'arena, I, S, E>),
    {
        match self {
            Self::Ir(node) => {
                for statement in node.statements {
                    f(Node::Statement(statement));
                }

                for error in node.errors {
                    f(Node::Error(error));
                }
            }
            Self::Statement(node) => {
                match &node.kind {
                    StatementKind::Shebang(_)
                    | StatementKind::Inline(_)
                    | StatementKind::HaltCompiler
                    | StatementKind::Noop => {}
                    StatementKind::Tag(n) => f(Node::Tag(n)),
                    StatementKind::Namespace(n) => f(Node::Namespace(n)),
                    StatementKind::Sequence(statements) => {
                        for statement in statements.iter() {
                            f(Node::Statement(statement));
                        }
                    }
                    StatementKind::Block(n) => f(Node::Block(n)),
                    StatementKind::Item(n) => f(Node::ItemStatement(n)),
                    StatementKind::Declare(n) => f(Node::Declare(n)),
                    StatementKind::Goto(name) | StatementKind::Label(name) => f(Node::Name(name)),
                    StatementKind::Try(n) => f(Node::Try(n)),
                    StatementKind::Foreach(n) => f(Node::Foreach(n)),
                    StatementKind::For(n) => f(Node::For(n)),
                    StatementKind::While(n) => f(Node::While(n)),
                    StatementKind::DoWhile(n) => f(Node::DoWhile(n)),
                    StatementKind::Continue(value) | StatementKind::Break(value) | StatementKind::Return(value) => {
                        if let Some(value) = value {
                            f(Node::Expression(value));
                        }
                    }
                    StatementKind::Switch(n) => f(Node::Switch(n)),
                    StatementKind::If(n) => f(Node::If(n)),
                    StatementKind::Expression(n) => f(Node::Expression(n)),
                    StatementKind::Echo(values) => {
                        for value in values.iter() {
                            f(Node::Expression(value));
                        }
                    }
                    StatementKind::Unset(values) => {
                        for value in values.iter() {
                            f(Node::Expression(value));
                        }
                    }
                    StatementKind::Use(items) => {
                        for item in items.iter() {
                            f(Node::UseItem(item));
                        }
                    }
                    StatementKind::Global(items) => {
                        for item in items.iter() {
                            f(Node::GlobalItem(item));
                        }
                    }
                    StatementKind::Static(items) => {
                        for item in items.iter() {
                            f(Node::StaticItem(item));
                        }
                    }
                    StatementKind::VariableBindingAnnotation(n) => f(Node::VariableBindingAnnotation(n)),
                }

                if let Some(terminator) = &node.terminator {
                    f(Node::Terminator(terminator));
                }
            }
            Self::Tag(_) => {}
            Self::Block(node) => {
                for statement in node.statements.iter() {
                    f(Node::Statement(statement));
                }
            }
            Self::Namespace(node) => {
                if let Some(name) = node.name {
                    f(Node::Identifier(name));
                }

                match &node.body {
                    NamespaceBody::BraceDelimited(block) => f(Node::Block(block)),
                    NamespaceBody::Implicit { terminator, statements } => {
                        f(Node::Terminator(terminator));
                        for statement in statements.iter() {
                            f(Node::Statement(statement));
                        }
                    }
                }
            }
            Self::Declare(node) => {
                for item in node.items.iter() {
                    f(Node::DeclareItem(item));
                }

                f(Node::Statement(node.statement));
            }
            Self::DeclareItem(node) => {
                f(Node::Name(&node.name));
                if let Some(value) = node.value {
                    f(Node::Expression(value));
                }
            }
            Self::Try(node) => {
                f(Node::Block(node.block));
                for catch_clause in node.catch_clauses {
                    f(Node::TryCatchClause(catch_clause));
                }
                if let Some(finally_clause) = node.finally_block {
                    f(Node::Block(finally_clause));
                }
            }
            Self::TryCatchClause(node) => {
                f(Node::Type(node.r#type));
                if let Some(variable) = &node.variable {
                    f(Node::DirectVariable(variable));
                }

                f(Node::Block(node.block));
            }
            Self::Foreach(node) => {
                f(Node::Expression(node.expression));
                if let Some(key) = node.key {
                    f(Node::Expression(key));
                }

                f(Node::Expression(node.value));
                f(Node::Statement(node.statement));
            }
            Self::For(node) => {
                for initialization in node.initializations {
                    f(Node::Expression(initialization));
                }

                for condition in node.conditions {
                    f(Node::Expression(condition));
                }

                for increment in node.increments {
                    f(Node::Expression(increment));
                }

                f(Node::Statement(node.statement));
            }
            Self::While(node) => {
                f(Node::Expression(node.condition));
                f(Node::Statement(node.statement));
            }
            Self::DoWhile(node) => {
                f(Node::Statement(node.statement));
                f(Node::Expression(node.condition));
            }
            Self::Switch(node) => {
                f(Node::Expression(node.subject));
                for case in node.cases.iter() {
                    f(Node::SwitchCase(case));
                }
            }
            Self::SwitchCase(node) => match &node.kind {
                SwitchCaseKind::Expression(expression, statements) => {
                    f(Node::Expression(expression));
                    for statement in statements.iter() {
                        f(Node::Statement(statement));
                    }
                }
                SwitchCaseKind::Default(statements) => {
                    for statement in statements.iter() {
                        f(Node::Statement(statement));
                    }
                }
            },
            Self::If(node) => {
                f(Node::Expression(node.condition));
                f(Node::Statement(node.then));
                if let Some(r#else) = node.r#else {
                    f(Node::Statement(r#else));
                }
            }
            Self::StaticItem(node) => {
                f(Node::DirectVariable(&node.variable));
                if let Some(type_annotation) = node.type_annotation {
                    f(Node::TypeAnnotation(type_annotation));
                }
                if let Some(value) = node.value {
                    f(Node::Expression(value));
                }
            }
            Self::GlobalItem(node) => {
                f(Node::Variable(&node.variable));
                if let Some(type_annotation) = node.type_annotation {
                    f(Node::TypeAnnotation(type_annotation));
                }
            }
            Self::UseItem(node) => f(Node::Identifier(&node.item)),
            Self::VariableBindingAnnotation(node) => {
                f(Node::DirectVariable(&node.variable));
                f(Node::TypeAnnotation(node.type_annotation));
            }
            Self::ItemStatement(node) => match &node.kind {
                ItemStatementKind::Class(n) => f(Node::Class(n)),
                ItemStatementKind::Interface(n) => f(Node::Interface(n)),
                ItemStatementKind::Trait(n) => f(Node::Trait(n)),
                ItemStatementKind::Enum(n) => f(Node::Enum(n)),
                ItemStatementKind::Constant(n) => f(Node::Constant(n)),
                ItemStatementKind::Function(n) => f(Node::Function(n)),
            },
            Self::Class(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for modifier in node.modifiers {
                    f(Node::Modifier(modifier));
                }

                f(Node::Identifier(&node.name));
                if let Some(extends) = node.extends {
                    f(Node::Extends(extends));
                }
                if let Some(implements) = node.implements {
                    f(Node::Implements(implements));
                }

                for member in node.members.iter() {
                    f(Node::MemberItem(member));
                }
            }
            Self::Interface(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                f(Node::Identifier(&node.name));
                if let Some(extends) = node.extends {
                    f(Node::Extends(extends));
                }

                for member in node.members.iter() {
                    f(Node::MemberItem(member));
                }
            }
            Self::Trait(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                f(Node::Identifier(&node.name));
                for member in node.members.iter() {
                    f(Node::MemberItem(member));
                }
            }
            Self::Enum(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                f(Node::Identifier(&node.name));
                if let Some(backing_type) = &node.backing_type {
                    f(Node::EnumBackingType(backing_type));
                }
                if let Some(implements) = node.implements {
                    f(Node::Implements(implements));
                }

                for member in node.members.iter() {
                    f(Node::MemberItem(member));
                }
            }
            Self::Constant(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                f(Node::Identifier(&node.name));
                f(Node::Expression(node.value));
            }
            Self::Function(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                f(Node::Identifier(&node.name));
                for parameter in node.parameters.iter() {
                    f(Node::Parameter(parameter));
                }
                if let Some(return_type) = node.return_type {
                    f(Node::Type(return_type));
                }

                f(Node::Block(node.body));
            }
            Self::MemberItem(node) => {
                match &node.kind {
                    MemberItemKind::Method(n) => f(Node::Method(n)),
                    MemberItemKind::Property(n) => f(Node::Property(n)),
                    MemberItemKind::HookedProperty(n) => f(Node::HookedProperty(n)),
                    MemberItemKind::TraitUse(n) => f(Node::TraitUse(n)),
                    MemberItemKind::Constant(n) => f(Node::ClassLikeConstant(n)),
                    MemberItemKind::EnumCase(n) => f(Node::EnumCase(n)),
                }

                if let Some(terminator) = &node.terminator {
                    f(Node::Terminator(terminator));
                }
            }
            Self::Method(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for modifier in node.modifiers {
                    f(Node::Modifier(modifier));
                }

                f(Node::Name(&node.name));
                for parameter in node.parameters.iter() {
                    f(Node::Parameter(parameter));
                }
                if let Some(return_type) = node.return_type {
                    f(Node::Type(return_type));
                }
                if let Some(body) = node.body {
                    f(Node::Block(body));
                }
            }
            Self::Property(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for modifier in node.modifiers {
                    f(Node::Modifier(modifier));
                }
                if let Some(r#type) = node.r#type {
                    f(Node::Type(r#type));
                }

                f(Node::DirectVariable(&node.variable));
                if let Some(default_value) = node.default_value {
                    f(Node::Expression(default_value));
                }
            }
            Self::HookedProperty(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for modifier in node.modifiers {
                    f(Node::Modifier(modifier));
                }
                if let Some(r#type) = node.r#type {
                    f(Node::Type(r#type));
                }

                f(Node::DirectVariable(&node.variable));
                if let Some(default_value) = node.default_value {
                    f(Node::Expression(default_value));
                }

                for hook in node.hooks.iter() {
                    f(Node::Hook(hook));
                }
            }
            Self::ClassLikeConstant(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for modifier in node.modifiers {
                    f(Node::Modifier(modifier));
                }
                if let Some(r#type) = node.r#type {
                    f(Node::Type(r#type));
                }

                f(Node::Name(&node.name));
                f(Node::Expression(node.value));
            }
            Self::EnumCase(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                f(Node::Name(&node.name));
                if let Some(value) = node.value {
                    f(Node::Expression(value));
                }
            }
            Self::Hook(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for modifier in node.modifiers {
                    f(Node::Modifier(modifier));
                }

                f(Node::Name(&node.name));
                for parameter in node.parameters.iter().flatten() {
                    f(Node::Parameter(parameter));
                }
                if let Some(body) = &node.body {
                    f(Node::HookBody(body));
                }
            }
            Self::HookBody(node) => match &node.kind {
                HookBodyKind::Expression(expression) => f(Node::Expression(expression)),
                HookBodyKind::Block(block) => f(Node::Block(block)),
            },
            Self::TraitUse(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for r#trait in node.traits {
                    f(Node::Identifier(r#trait));
                }

                for adaptation in node.adaptations.iter().flatten() {
                    f(Node::TraitUseAdaptation(adaptation));
                }
            }
            Self::TraitUseAdaptation(node) => match node {
                TraitUseAdaptation::Precedence(adaptation) => f(Node::TraitUsePrecedenceAdaptation(adaptation)),
                TraitUseAdaptation::Alias(adaptation) => f(Node::TraitUseAliasAdaptation(adaptation)),
            },
            Self::TraitUsePrecedenceAdaptation(node) => {
                f(Node::Identifier(&node.r#trait));
                f(Node::Name(&node.method));
                for instead_of in node.instead_of {
                    f(Node::Identifier(instead_of));
                }
            }
            Self::TraitUseAliasAdaptation(node) => {
                if let Some(r#trait) = &node.r#trait {
                    f(Node::Identifier(r#trait));
                }

                f(Node::Name(&node.method));
                if let Some(modifier) = &node.modifier {
                    f(Node::Modifier(modifier));
                }

                f(Node::Name(&node.alias));
            }
            Self::Parameter(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::VariableAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for modifier in node.modifiers {
                    f(Node::Modifier(modifier));
                }
                if let Some(r#type) = node.r#type {
                    f(Node::Type(r#type));
                }

                f(Node::DirectVariable(&node.variable));
                if let Some(default_value) = node.default_value {
                    f(Node::Expression(default_value));
                }

                for hook in node.hooks.iter().flatten() {
                    f(Node::Hook(hook));
                }
            }
            Self::ItemAnnotation(node) => {
                for type_alias in node.type_aliases {
                    f(Node::TypeAliasAnnotation(type_alias));
                }

                for imported_type_alias in node.imported_type_aliases {
                    f(Node::ImportedTypeAliasAnnotation(imported_type_alias));
                }

                for type_parameter in node.type_parameters {
                    f(Node::TypeParameterAnnotation(type_parameter));
                }

                for inherited in node.inherited_type_parameters {
                    f(Node::InheritedTypeParameterAnnotation(inherited));
                }

                for extends in node.extends {
                    f(Node::ExtendsAnnotation(extends));
                }

                for require_extends in node.require_extends {
                    f(Node::RequireExtendsAnnotation(require_extends));
                }

                for implements in node.implements {
                    f(Node::ImplementsAnnotation(implements));
                }

                for require_implements in node.require_implements {
                    f(Node::RequireImplementsAnnotation(require_implements));
                }

                for r#use in node.uses {
                    f(Node::UseAnnotation(r#use));
                }

                for sealing in node.sealings {
                    f(Node::SealedAnnotation(sealing));
                }

                for mixin in node.mixins {
                    f(Node::MixinAnnotation(mixin));
                }

                for method in node.methods {
                    f(Node::MethodAnnotation(method));
                }

                for property in node.properties {
                    f(Node::PropertyAnnotation(property));
                }

                for parameter in node.parameters {
                    f(Node::ParameterAnnotation(parameter));
                }

                for parameter_out in node.parameter_outs {
                    f(Node::ParameterOutAnnotation(parameter_out));
                }

                for where_constraint in node.where_constraints {
                    f(Node::WhereConstraintAnnotation(where_constraint));
                }

                for return_type in node.return_type {
                    f(Node::TypeAnnotation(return_type));
                }

                for throws in node.throws {
                    f(Node::ThrowsAnnotation(throws));
                }

                for assert in node.asserts.iter().chain(node.asserts_if_true).chain(node.asserts_if_false) {
                    f(Node::AssertAnnotation(assert));
                }

                for self_out in node.self_out {
                    f(Node::SelfOutAnnotation(self_out));
                }

                for variable in node.pure_unless_callable_impure {
                    f(Node::DirectVariable(variable));
                }

                for var in node.var {
                    f(Node::VariableAnnotation(var));
                }

                for error in node.errors {
                    f(Node::AnnotationError(error));
                }
            }
            Self::MethodAnnotation(node) => {
                if let Some(visibility) = &node.visibility {
                    f(Node::Visibility(visibility));
                }

                f(Node::Name(&node.name));
                for type_parameter in node.type_parameters.iter().flatten() {
                    f(Node::TypeParameterAnnotation(type_parameter));
                }

                for parameter in node.parameters.iter() {
                    f(Node::ParameterAnnotation(parameter));
                }
                if let Some(return_type) = node.return_type {
                    f(Node::TypeAnnotation(return_type));
                }
            }
            Self::ParameterAnnotation(node) => {
                if let Some(type_annotation) = node.r#type {
                    f(Node::TypeAnnotation(type_annotation));
                }
                if let Some(variable) = &node.variable {
                    f(Node::DirectVariable(variable));
                }
                if let Some(default_value) = node.default_value {
                    f(Node::Expression(default_value));
                }
            }
            Self::ParameterOutAnnotation(node) => {
                f(Node::TypeAnnotation(node.r#type));
                f(Node::DirectVariable(&node.variable));
            }
            Self::VariableAnnotation(node) => {
                f(Node::TypeAnnotation(node.type_annotation));
                if let Some(variable) = &node.variable {
                    f(Node::DirectVariable(variable));
                }

                for error in node.errors {
                    f(Node::AnnotationError(error));
                }
            }
            Self::Extends(node) => {
                for r#type in node.types {
                    f(Node::Identifier(r#type));
                }
            }
            Self::Implements(node) => {
                for r#type in node.types {
                    f(Node::Identifier(r#type));
                }
            }
            Self::Attribute(node) => {
                f(Node::Identifier(&node.class));
                for argument in node.arguments.iter().flatten() {
                    f(Node::PartialArgument(argument));
                }
            }
            Self::Argument(node) => match node {
                Argument::Value(expression) | Argument::Variadic(expression) => f(Node::Expression(expression)),
                Argument::Named(name, expression) => {
                    f(Node::Name(name));
                    f(Node::Expression(expression));
                }
            },
            Self::PartialArgument(node) => match node {
                PartialArgument::Value(expression) | PartialArgument::Variadic(expression) => {
                    f(Node::Expression(expression));
                }
                PartialArgument::Named(name, expression) => {
                    f(Node::Name(name));
                    f(Node::Expression(expression));
                }
                PartialArgument::NamedPlaceholder(name) => f(Node::Name(name)),
                PartialArgument::Placeholder(_) | PartialArgument::VariadicPlaceholder(_) => {}
            },
            Self::Expression(node) => match &node.kind {
                ExpressionKind::Parenthesized(n) => f(Node::Expression(n)),
                ExpressionKind::Binary(n) => f(Node::Binary(n)),
                ExpressionKind::UnaryPrefix(n) => f(Node::UnaryPrefix(n)),
                ExpressionKind::UnaryPostfix(n) => f(Node::UnaryPostfix(n)),
                ExpressionKind::Literal(n) => f(Node::Literal(n)),
                ExpressionKind::CompositeString(parts) | ExpressionKind::ShellExecute(parts) => {
                    for part in parts.iter() {
                        if let CompositeStringPart::Expression(expression) = part {
                            f(Node::Expression(expression));
                        }
                    }
                }
                ExpressionKind::Assignment(n) => f(Node::Assignment(n)),
                ExpressionKind::Annotation(n) => {
                    f(Node::VariableAnnotation(n.annotation));
                    f(Node::Expression(n.expression));
                }
                ExpressionKind::Conditional(n) => f(Node::Conditional(n)),
                ExpressionKind::ArrayLike(n) => f(Node::ArrayLike(n)),
                ExpressionKind::ArrayAppend(n)
                | ExpressionKind::Clone(n)
                | ExpressionKind::Empty(n)
                | ExpressionKind::Eval(n)
                | ExpressionKind::Include(n)
                | ExpressionKind::IncludeOnce(n)
                | ExpressionKind::Require(n)
                | ExpressionKind::RequireOnce(n)
                | ExpressionKind::Print(n)
                | ExpressionKind::Throw(n) => f(Node::Expression(n)),
                ExpressionKind::Item(n) => match &n.kind {
                    ItemExpressionKind::AnonymousClass(inner) => f(Node::AnonymousClass(inner)),
                    ItemExpressionKind::ArrowFunction(inner) => f(Node::ArrowFunction(inner)),
                    ItemExpressionKind::Closure(inner) => f(Node::Closure(inner)),
                },
                ExpressionKind::Call(n) => f(Node::Call(n)),
                ExpressionKind::PartialApplication(n) => f(Node::PartialApplication(n)),
                ExpressionKind::Access(n) => f(Node::Access(n)),
                ExpressionKind::Isset(values) => {
                    for value in values.iter() {
                        f(Node::Expression(value));
                    }
                }
                ExpressionKind::Exit(arguments) => {
                    for argument in arguments.iter().flatten() {
                        f(Node::Argument(argument));
                    }
                }
                ExpressionKind::Constant(identifier) | ExpressionKind::Identifier(identifier) => {
                    f(Node::Identifier(identifier));
                }
                ExpressionKind::Instantiation(n) => f(Node::Instantiation(n)),
                ExpressionKind::Variable(variable) => f(Node::Variable(variable)),
                ExpressionKind::Yield(n) => f(Node::Yield(n)),
                ExpressionKind::Match(n) => f(Node::Match(n)),
                ExpressionKind::Error(_) => {}
                ExpressionKind::MagicConstant(_) => {}
                ExpressionKind::Parent | ExpressionKind::Self_ | ExpressionKind::Static => {}
            },
            Self::Assignment(node) => {
                f(Node::Expression(node.left));
                f(Node::AssignmentOperator(&node.operator));
                f(Node::Expression(node.right));
            }
            Self::Binary(node) => {
                f(Node::Expression(node.left));
                f(Node::BinaryOperator(&node.operator));
                f(Node::Expression(node.right));
            }
            Self::UnaryPrefix(node) => {
                f(Node::UnaryPrefixOperator(&node.operator));
                f(Node::Expression(node.operand));
            }
            Self::UnaryPostfix(node) => {
                f(Node::Expression(node.operand));
                f(Node::UnaryPostfixOperator(&node.operator));
            }
            Self::Conditional(node) => {
                f(Node::Expression(node.condition));
                if let Some(then) = node.then {
                    f(Node::Expression(then));
                }

                f(Node::Expression(node.r#else));
            }
            Self::ArrayLike(node) => {
                for element in node.elements.iter() {
                    f(Node::ArrayElement(element));
                }
            }
            Self::ArrayElement(node) => match &node.kind {
                ArrayElementKind::KeyValue(key, value) => {
                    f(Node::Expression(key));
                    f(Node::Expression(value));
                }
                ArrayElementKind::Value(value) | ArrayElementKind::Variadic(value) => f(Node::Expression(value)),
                ArrayElementKind::Missing => {}
            },
            Self::Instantiation(node) => {
                f(Node::Expression(node.class));
                for argument in node.arguments.iter().flatten() {
                    f(Node::Argument(argument));
                }
            }
            Self::Call(node) => {
                f(Node::Callee(&node.callee));
                for argument in node.arguments.iter() {
                    f(Node::Argument(argument));
                }
            }
            Self::PartialApplication(node) => {
                f(Node::Callee(&node.callee));
                for argument in node.arguments.iter() {
                    f(Node::PartialArgument(argument));
                }
            }
            Self::Callee(node) => match &node.kind {
                CalleeKind::Function(expression) => f(Node::Expression(expression)),
                CalleeKind::Method(expression, selector)
                | CalleeKind::NullsafeMethod(expression, selector)
                | CalleeKind::StaticMethod(expression, selector) => {
                    f(Node::Expression(expression));
                    f(Node::MemberSelector(selector));
                }
            },
            Self::Access(node) => match &node.kind {
                AccessKind::Array(target, index) => {
                    f(Node::Expression(target));
                    f(Node::Expression(index));
                }
                AccessKind::Property(target, selector) | AccessKind::NullsafeProperty(target, selector) => {
                    f(Node::Expression(target));
                    f(Node::MemberSelector(selector));
                }
                AccessKind::StaticProperty(target, variable) => {
                    f(Node::Expression(target));
                    f(Node::Variable(variable));
                }
                AccessKind::ClassConstant(target, selector) => {
                    f(Node::Expression(target));
                    f(Node::ConstantSelector(selector));
                }
            },
            Self::MemberSelector(node) => match &node.kind {
                MemberSelectorKind::Missing => {}
                MemberSelectorKind::Name(name) => f(Node::Name(name)),
                MemberSelectorKind::Variable(variable) => f(Node::DirectVariable(variable)),
                MemberSelectorKind::Expression(expression) => f(Node::Expression(expression)),
            },
            Self::ConstantSelector(node) => match &node.kind {
                ConstantSelectorKind::Missing => {}
                ConstantSelectorKind::Name(name) => f(Node::Name(name)),
                ConstantSelectorKind::Expression(expression) => f(Node::Expression(expression)),
            },
            Self::Yield(node) => match &node.kind {
                YieldKind::Nothing => {}
                YieldKind::Expression(value) | YieldKind::From(value) => f(Node::Expression(value)),
                YieldKind::Pair(key, value) => {
                    f(Node::Expression(key));
                    f(Node::Expression(value));
                }
            },
            Self::Match(node) => {
                f(Node::Expression(node.subject));
                for arm in node.arms.iter() {
                    f(Node::MatchArm(arm));
                }
            }
            Self::MatchArm(node) => match &node.kind {
                MatchArmKind::Expression(conditions, body) => {
                    for condition in conditions.iter() {
                        f(Node::Expression(condition));
                    }
                    f(Node::Expression(body));
                }
                MatchArmKind::Default(body) => f(Node::Expression(body)),
            },
            Self::AnonymousClass(node) => {
                for modifier in node.modifiers {
                    f(Node::Modifier(modifier));
                }
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for argument in node.arguments.iter().flatten() {
                    f(Node::PartialArgument(argument));
                }
                if let Some(extends) = node.extends {
                    f(Node::Extends(extends));
                }
                if let Some(implements) = node.implements {
                    f(Node::Implements(implements));
                }

                for member in node.members.iter() {
                    f(Node::MemberItem(member));
                }
            }
            Self::Closure(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for parameter in node.parameters.iter() {
                    f(Node::Parameter(parameter));
                }
                if let Some(return_type) = node.return_type {
                    f(Node::Type(return_type));
                }

                for use_variable in node.use_variables.iter().flatten() {
                    f(Node::ClosureUseClauseVariable(use_variable));
                }

                f(Node::Block(node.body));
            }
            Self::ArrowFunction(node) => {
                if let Some(annotation) = node.annotation {
                    f(Node::ItemAnnotation(annotation));
                }

                for attribute in node.attributes {
                    f(Node::Attribute(attribute));
                }

                for parameter in node.parameters.iter() {
                    f(Node::Parameter(parameter));
                }
                if let Some(return_type) = node.return_type {
                    f(Node::Type(return_type));
                }

                f(Node::Expression(node.expression));
            }
            Self::ClosureUseClauseVariable(node) => f(Node::DirectVariable(&node.variable)),
            Self::Variable(node) => match node {
                Variable::Direct(direct) => f(Node::DirectVariable(direct)),
                Variable::Indirect(expression) => f(Node::Expression(expression)),
                Variable::Nested(nested) => f(Node::Variable(nested)),
            },
            Self::Literal(node) => match &node.kind {
                LiteralKind::String(n) => f(Node::LiteralString(n)),
                LiteralKind::Integer(n) => f(Node::LiteralInteger(n)),
                LiteralKind::Float(n) => f(Node::LiteralFloat(n)),
                LiteralKind::True | LiteralKind::False | LiteralKind::Null => {}
            },
            Self::Type(node) => match &node.kind {
                TypeKind::Parenthesized(ty) | TypeKind::Nullable(ty) => f(Node::Type(ty)),
                TypeKind::Named(identifier)
                | TypeKind::Static(identifier)
                | TypeKind::Self_(identifier)
                | TypeKind::Parent(identifier) => f(Node::Identifier(identifier)),
                TypeKind::Union(types) | TypeKind::Intersection(types) => {
                    for r#type in types.iter() {
                        f(Node::Type(r#type));
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
            },
            Self::TypeAnnotation(node) => visit_type_annotation_kind(&node.kind, &mut f),
            Self::NamedTypeAnnotation(node) => {
                let (ReferenceKind::Identifier(identifier)
                | ReferenceKind::Self_(identifier)
                | ReferenceKind::Static(identifier)
                | ReferenceKind::Parent(identifier)) = &node.kind;
                f(Node::Identifier(identifier));
                for type_argument in node.type_arguments.iter().flatten() {
                    f(Node::TypeAnnotation(type_argument));
                }
            }
            Self::GenericParameterTypeAnnotation(node) => {
                f(Node::Name(&node.name));
                visit_type_parameter_defining_entity(&node.defining_entity, &mut f);
                if let Some(bound) = node.bound {
                    f(Node::TypeAnnotation(bound));
                }
            }
            Self::StringTypeAnnotation(_) => {}
            Self::ObjectShapeTypeAnnotation(node) => {
                for field in node.fields.iter() {
                    f(Node::ShapeTypeAnnotationField(field));
                }
            }
            Self::ShapeTypeAnnotation(node) => {
                for field in node.fields.iter() {
                    f(Node::ShapeTypeAnnotationField(field));
                }
                if let Some(additional_fields) = &node.additional_fields {
                    f(Node::ShapeTypeAnnotationAdditionalFields(additional_fields));
                }
            }
            Self::ShapeTypeAnnotationField(node) => {
                if let ShapeTypeAnnotationKey::ClassLikeConstant(identifier, name) = &node.key {
                    f(Node::Identifier(identifier));
                    f(Node::Name(name));
                }

                f(Node::TypeAnnotation(&node.value));
            }
            Self::ShapeTypeAnnotationAdditionalFields(node) => {
                f(Node::TypeAnnotation(node.key_type));
                f(Node::TypeAnnotation(node.value_type));
            }
            Self::CallableTypeAnnotation(node) => {
                for parameter in node.parameters.iter().flatten() {
                    f(Node::CallableTypeAnnotationParameter(parameter));
                }
                if let Some(r#return) = node.r#return {
                    f(Node::TypeAnnotation(r#return));
                }
            }
            Self::CallableTypeAnnotationParameter(node) => {
                if let Some(r#type) = node.r#type {
                    f(Node::TypeAnnotation(r#type));
                }
                if let Some(variable) = &node.variable {
                    f(Node::DirectVariable(variable));
                }
            }
            Self::ConditionalTypeAnnotation(node) => {
                f(Node::TypeAnnotation(node.target));
                f(Node::TypeAnnotation(node.subject));
                f(Node::TypeAnnotation(node.then));
                f(Node::TypeAnnotation(node.r#else));
            }
            Self::TypeAliasAnnotation(node) => {
                f(Node::Name(&node.name));
                f(Node::TypeAnnotation(node.r#type));
            }
            Self::ImportedTypeAliasAnnotation(node) => {
                f(Node::Name(&node.name));
                f(Node::Identifier(&node.from));
                if let Some(r#as) = &node.r#as {
                    f(Node::Name(r#as));
                }
            }
            Self::TypeParameterAnnotation(node) => {
                f(Node::Name(&node.name));
                if let Some(bound) = node.bound {
                    f(Node::TypeAnnotation(bound));
                }
                if let Some(default) = node.default {
                    f(Node::TypeAnnotation(default));
                }
            }
            Self::InheritedTypeParameterAnnotation(node) => {
                visit_type_parameter_defining_entity(&node.defining_entity, &mut f);
                f(Node::Name(&node.name));
                if let Some(bound) = node.bound {
                    f(Node::TypeAnnotation(bound));
                }
                if let Some(default) = node.default {
                    f(Node::TypeAnnotation(default));
                }
            }
            Self::WhereConstraintAnnotation(node) => {
                f(Node::Name(&node.type_parameter));
                f(Node::TypeAnnotation(node.constraint));
            }
            Self::ExtendsAnnotation(node) => f(Node::NamedTypeAnnotation(&node.r#type)),
            Self::RequireExtendsAnnotation(node) => f(Node::NamedTypeAnnotation(&node.r#type)),
            Self::ImplementsAnnotation(node) => f(Node::NamedTypeAnnotation(&node.r#type)),
            Self::RequireImplementsAnnotation(node) => f(Node::NamedTypeAnnotation(&node.r#type)),
            Self::UseAnnotation(node) => f(Node::NamedTypeAnnotation(&node.r#type)),
            Self::SealedAnnotation(node) => {
                for r#type in node.types.iter() {
                    f(Node::NamedTypeAnnotation(r#type));
                }
            }
            Self::MixinAnnotation(node) => visit_type_annotation_kind(&node.r#type, &mut f),
            Self::PropertyAnnotation(node) => {
                if let Some(r#type) = node.r#type {
                    f(Node::TypeAnnotation(r#type));
                }

                f(Node::DirectVariable(&node.variable));
            }
            Self::ThrowsAnnotation(node) => f(Node::TypeAnnotation(node.r#type)),
            Self::AssertAnnotation(node) => {
                f(Node::AssertAnnotationPattern(&node.pattern));
                f(Node::AssertAnnotationTarget(&node.target));
            }
            Self::AssertAnnotationPattern(node) => {
                if let AssertAnnotationPatternKind::Type(r#type) = &node.kind {
                    f(Node::TypeAnnotation(r#type));
                }
            }
            Self::AssertAnnotationTarget(node) => match &node.kind {
                AssertAnnotationTargetKind::Variable(variable) => f(Node::DirectVariable(variable)),
                AssertAnnotationTargetKind::Method(variable, name)
                | AssertAnnotationTargetKind::Property(variable, name) => {
                    f(Node::DirectVariable(variable));
                    f(Node::Name(name));
                }
            },
            Self::SelfOutAnnotation(node) => f(Node::TypeAnnotation(node.r#type)),
            Self::AssignmentOperator(_)
            | Self::BinaryOperator(_)
            | Self::UnaryPrefixOperator(_)
            | Self::UnaryPostfixOperator(_)
            | Self::Terminator(_)
            | Self::Identifier(_)
            | Self::Name(_)
            | Self::DirectVariable(_)
            | Self::LiteralString(_)
            | Self::LiteralInteger(_)
            | Self::LiteralFloat(_)
            | Self::Modifier(_)
            | Self::Visibility(_)
            | Self::EnumBackingType(_)
            | Self::AnnotationError(_)
            | Self::Error(_) => {}
        }
    }
}

fn visit_type_parameter_defining_entity<'ir, 'arena, I, S, E, F>(
    entity: &'ir TypeParameterDefiningEntity<'arena>,
    f: &mut F,
) where
    I: 'arena,
    S: 'arena,
    E: 'arena,
    F: FnMut(Node<'ir, 'arena, I, S, E>),
{
    match entity {
        TypeParameterDefiningEntity::ClassLike(identifier) | TypeParameterDefiningEntity::Function(identifier) => {
            f(Node::Identifier(identifier));
        }
        TypeParameterDefiningEntity::Method(identifier, name) => {
            f(Node::Identifier(identifier));
            f(Node::Name(name));
        }
        TypeParameterDefiningEntity::Closure(_) => {}
    }
}

fn visit_type_annotation_kind<'ir, 'arena, I, S, E, F>(kind: &'ir TypeAnnotationKind<'arena>, f: &mut F)
where
    I: 'arena,
    S: 'arena,
    E: 'arena,
    F: FnMut(Node<'ir, 'arena, I, S, E>),
{
    match kind {
        TypeAnnotationKind::Named(n) => f(Node::NamedTypeAnnotation(n)),
        TypeAnnotationKind::GenericParameter(n) => f(Node::GenericParameterTypeAnnotation(n)),
        TypeAnnotationKind::Union(types) | TypeAnnotationKind::Intersection(types) => {
            for r#type in types.iter() {
                f(Node::TypeAnnotation(r#type));
            }
        }
        TypeAnnotationKind::Array(_, key, value) => {
            f(Node::TypeAnnotation(key));
            f(Node::TypeAnnotation(value));
        }
        TypeAnnotationKind::List(_, value) => f(Node::TypeAnnotation(value)),
        TypeAnnotationKind::Iterable(key, value) => {
            f(Node::TypeAnnotation(key));
            f(Node::TypeAnnotation(value));
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
        | TypeAnnotationKind::Slice(inner) => f(Node::TypeAnnotation(inner)),
        TypeAnnotationKind::String(n) => f(Node::StringTypeAnnotation(n)),
        TypeAnnotationKind::ObjectShape(n) => f(Node::ObjectShapeTypeAnnotation(n)),
        TypeAnnotationKind::MemberReference(identifier, selector) => {
            f(Node::Identifier(identifier));
            if let MemberReferenceSelector::Exact(name)
            | MemberReferenceSelector::StartsWith(name)
            | MemberReferenceSelector::EndsWith(name) = selector
            {
                f(Node::Name(name));
            }
        }
        TypeAnnotationKind::AliasReference(reference, name) => {
            let (ReferenceKind::Identifier(identifier)
            | ReferenceKind::Self_(identifier)
            | ReferenceKind::Static(identifier)
            | ReferenceKind::Parent(identifier)) = reference;
            f(Node::Identifier(identifier));
            f(Node::Name(name));
        }
        TypeAnnotationKind::Shape(n) => f(Node::ShapeTypeAnnotation(n)),
        TypeAnnotationKind::Callable(n) => f(Node::CallableTypeAnnotation(n)),
        TypeAnnotationKind::Variable(variable) => f(Node::DirectVariable(variable)),
        TypeAnnotationKind::Conditional(n) => f(Node::ConditionalTypeAnnotation(n)),
        TypeAnnotationKind::IntMask(types) | TypeAnnotationKind::TemplateType(types) => {
            for r#type in types.iter() {
                f(Node::TypeAnnotation(r#type));
            }
        }
        TypeAnnotationKind::IndexAccess(base, index) => {
            f(Node::TypeAnnotation(base));
            f(Node::TypeAnnotation(index));
        }
        TypeAnnotationKind::PropertiesOf(_, inner) => f(Node::TypeAnnotation(inner)),
        TypeAnnotationKind::GlobalSelector(selector) => {
            let (GlobalSelector::StartsWith(identifier) | GlobalSelector::EndsWith(identifier)) = selector;
            f(Node::Identifier(identifier));
        }
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
