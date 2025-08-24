use ahash::HashSet;

use mago_atom::Atom;
use mago_codex::get_function;
use mago_codex::get_method_by_id;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::Context;
use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::heuristic;
use crate::utils::docblock::populate_docblock_variables;
use crate::utils::expression::get_function_like_id_from_call;

pub mod attributes;
pub mod class_like;
pub mod constant;
pub mod echo;
pub mod function_like;
pub mod global;
pub mod r#if;
pub mod r#loop;
pub mod r#return;
pub mod r#static;
pub mod switch;
pub mod r#try;
pub mod unset;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Statement<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let last_statement_span = context.statement_span;
        context.statement_span = self.span();

        let should_populate_docblock_variables = matches!(
            self,
            Statement::Block(_)
                | Statement::Expression(_)
                | Statement::Try(_)
                | Statement::Continue(_)
                | Statement::Break(_)
                | Statement::Return(_)
                | Statement::Echo(_)
                | Statement::Unset(_)
                | Statement::Noop(_)
        );

        if should_populate_docblock_variables {
            let override_existing_variables = !matches!(self, Statement::Expression(ExpressionStatement { expression, .. }) if expression.is_assignment());

            populate_docblock_variables(context, block_context, artifacts, override_existing_variables);
        }

        let result = match self {
            Statement::Inline(_)
            | Statement::OpeningTag(_)
            | Statement::Declare(_)
            | Statement::Noop(_)
            | Statement::ClosingTag(_)
            | Statement::HaltCompiler(_) => {
                // ignore
                Ok(())
            }
            Statement::Goto(_) | Statement::Label(_) => {
                // not supported, unlikely to be supported
                Ok(())
            }
            Statement::Use(r#use) => {
                context.scope.populate_from_use(r#use);

                Ok(())
            }
            Statement::Namespace(namespace) => {
                match &namespace.name {
                    Some(name) => {
                        context.scope = NamespaceScope::for_namespace(name.value());
                    }
                    None => {
                        context.scope = NamespaceScope::global();
                    }
                }

                analyze_statements(namespace.statements().as_slice(), context, block_context, artifacts)
            }
            Statement::Class(class) => {
                let class_name = context.resolved_names.get(&class.name);

                context.scope.add(NameKind::Default, class_name, None as Option<&str>);

                class.analyze(context, block_context, artifacts)
            }
            Statement::Interface(interface) => {
                let interface_name = context.resolved_names.get(&interface.name);

                context.scope.add(NameKind::Default, interface_name, None as Option<&str>);

                interface.analyze(context, block_context, artifacts)
            }
            Statement::Trait(r#trait) => {
                let trait_name = context.resolved_names.get(&r#trait.name);

                context.scope.add(NameKind::Default, trait_name, None as Option<&str>);

                r#trait.analyze(context, block_context, artifacts)
            }
            Statement::Enum(r#enum) => {
                let enum_name = context.resolved_names.get(&r#enum.name);

                context.scope.add(NameKind::Default, enum_name, None as Option<&str>);

                r#enum.analyze(context, block_context, artifacts)
            }
            Statement::Constant(constant) => {
                for item in constant.items.iter() {
                    let constant_item_name = context.resolved_names.get(&item.name);

                    context.scope.add(NameKind::Constant, constant_item_name, None as Option<&str>);
                }

                constant.analyze(context, block_context, artifacts)
            }
            Statement::Function(function) => {
                let function_name = context.resolved_names.get(&function.name);

                context.scope.add(NameKind::Function, function_name, None as Option<&str>);

                function.analyze(context, block_context, artifacts)
            }
            Statement::Block(block) => {
                analyze_statements(block.statements.as_slice(), context, block_context, artifacts)
            }
            Statement::Expression(expression) => expression.expression.analyze(context, block_context, artifacts),
            Statement::Try(r#try) => r#try.analyze(context, block_context, artifacts),
            Statement::Foreach(foreach) => foreach.analyze(context, block_context, artifacts),
            Statement::For(r#for) => r#for.analyze(context, block_context, artifacts),
            Statement::While(r#while) => r#while.analyze(context, block_context, artifacts),
            Statement::DoWhile(do_while) => do_while.analyze(context, block_context, artifacts),
            Statement::Continue(r#continue) => r#continue.analyze(context, block_context, artifacts),
            Statement::Break(r#break) => r#break.analyze(context, block_context, artifacts),
            Statement::If(r#if) => r#if.analyze(context, block_context, artifacts),
            Statement::Return(r#return) => r#return.analyze(context, block_context, artifacts),
            Statement::Echo(echo) => echo.analyze(context, block_context, artifacts),
            Statement::Global(global) => global.analyze(context, block_context, artifacts),
            Statement::Static(r#static) => r#static.analyze(context, block_context, artifacts),
            Statement::Unset(unset) => unset.analyze(context, block_context, artifacts),
            Statement::Switch(r#switch) => r#switch.analyze(context, block_context, artifacts),
        };

        result?;

        if let Statement::Expression(expression) = self
            && context.settings.find_unused_expressions
        {
            detect_unused_statement_expressions(expression.expression, self, context, artifacts);
        }

        context.statement_span = last_statement_span;
        block_context.conditionally_referenced_variable_ids = HashSet::default();

        heuristic::check_statement(self, context);

        Ok(())
    }
}

#[inline]
pub fn analyze_statements<'ctx, 'arena>(
    statements: &[Statement<'arena>],
    context: &mut Context<'ctx, 'arena>,
    block: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    for statement in statements {
        if block.has_returned {
            if context.settings.find_unused_expressions {
                let is_harmless = match &statement {
                    Statement::Break(_) => true,
                    Statement::Continue(_) => true,
                    Statement::Return(return_statement) => return_statement.value.is_none(),
                    _ => false,
                };

                if is_harmless {
                    context.collector.report_with_code(
                        IssueCode::UselessControlFlow,
                        Issue::help("This control flow is unnecessary")
                            .with_annotation(
                                Annotation::primary(statement.span()).with_message("This statement has no effect."),
                            )
                            .with_note("This statement is unreachable because the block has already returned.")
                            .with_help("Consider removing this statement as it does not do anything in this context."),
                    );
                } else {
                    context.collector.report_with_code(
                        IssueCode::UnevaluatedCode,
                        Issue::help("Unreachable code detected.")
                            .with_annotation(Annotation::primary(statement.span()).with_message("This code will never be executed."))
                            .with_note("Execution cannot reach this point due to preceding code (e.g., return, throw, break, continue, exit, or an infinite loop).")
                            .with_help("Consider removing this unreachable code."),
                    );
                }
            }

            if !context.settings.analyze_dead_code {
                continue;
            }
        }

        statement.analyze(context, block, artifacts)?;
    }

    Ok(())
}

/// Checks statement expressions for unused results or lack of side effects.
fn detect_unused_statement_expressions<'ctx, 'ast, 'arena>(
    expression: &'ast Expression<'arena>,
    statement: &'ast Statement<'arena>,
    context: &mut Context<'ctx, 'arena>,
    artifacts: &mut AnalysisArtifacts,
) {
    if let Some((issue_kind, name)) = has_unused_must_use(expression, context, artifacts) {
        context.collector.report_with_code(
            issue_kind,
            Issue::error(format!("The return value of '{name}' must be used."))
                .with_annotation(Annotation::primary(statement.span()).with_message("The result of this call is ignored"))
                .with_note(format!("The function or method '{name}' is marked with @must-use or #[NoDiscard], indicating its return value is important and should not be discarded."))
                .with_help("Assign the result to a variable, pass it to another function, or use it in an expression.")
        );

        return;
    }

    let useless_expression_message: &str = match expression {
        Expression::Literal(_) => "Evaluating a literal as a statement has no effect.",
        Expression::CompositeString(_) => "Evaluating a string as a statement has no effect.",
        Expression::Array(_) | Expression::LegacyArray(_) | Expression::List(_) => {
            "Creating an array or list as a statement has no effect."
        }
        Expression::Variable(_) => "Accessing a variable as a statement has no effect.",
        Expression::ConstantAccess(_) => "Accessing a constant as a statement has no effect.",
        Expression::Identifier(_) => {
            "Using an identifier directly as a statement likely has no effect (perhaps a typo?)."
        }
        Expression::Access(_) => {
            "Accessing a property or constant as a statement might have no effect (unless it's meant to trigger a magic method call)."
        }
        Expression::AnonymousClass(_) => "Defining an anonymous class without assigning it has no effect.",
        Expression::Closure(_) | Expression::ArrowFunction(_) | Expression::ClosureCreation(_) => {
            "Defining a closure or arrow function without assigning or calling it has no effect."
        }
        Expression::Parent(_) | Expression::Static(_) | Expression::Self_(_) => {
            "Using 'parent', 'static', or 'self' directly as a statement has no effect."
        }
        Expression::MagicConstant(_) => "Evaluating a magic constant as a statement has no effect.",
        Expression::Binary(_) => "A binary operation used as a statement likely has no effect.",
        Expression::Call(Call::Function(FunctionCall { function, .. })) => {
            let Expression::Identifier(function_name) = function else {
                return;
            };

            let unqualified_name = function_name.value();
            let name = context.resolved_names.get(function_name);

            let Some(function) = get_function(context.codebase, name).or_else(|| {
                if !function_name.is_local() { None } else { get_function(context.codebase, unqualified_name) }
            }) else {
                return;
            };

            // If the function has side effects, we don't report it as useless.
            if !function.flags.is_pure() {
                return;
            }

            // If the function does throw or has thrown types, we don't report it as useless.
            if !function.thrown_types.is_empty() || function.flags.has_throw() {
                return;
            }

            // If the function has parameters that are by reference, we don't report it as useless.
            if function.parameters.iter().any(|param| param.flags.is_by_reference()) {
                return;
            }

            "Calling a pure function without using its result has no effect (consider using the result or removing the call)."
        }
        _ => return,
    };

    context.collector.report_with_code(
        IssueCode::UnusedStatement,
        Issue::note("Expression has no effect as a statement")
            .with_annotation(Annotation::primary(expression.span()).with_message(useless_expression_message))
            .with_note("This expression does not produce a side effect or return value that is used.")
            .with_help(
                "To fix this, assign the value to a variable, return it, or remove the statement if it is truly unnecessary.",
            ),
    );
}

/// Checks if an expression is a call to a `@must-use` function/method
/// and returns the appropriate issue kind and the name identifier if the result is unused.
fn has_unused_must_use<'ctx, 'ast, 'arena>(
    expression: &'ast Expression<'arena>,
    context: &Context<'ctx, 'arena>,
    artifacts: &AnalysisArtifacts,
) -> Option<(IssueCode, Atom)> {
    let call_expression = match expression {
        Expression::Call(call_expr) => call_expr,
        _ => return None,
    };

    let functionlike_id_from_call =
        get_function_like_id_from_call(call_expression, context.resolved_names, &artifacts.expression_types)?;

    match functionlike_id_from_call {
        FunctionLikeIdentifier::Function(function_id) => {
            let function_metadata = get_function(context.codebase, &function_id)?;

            let must_use = function_metadata.flags.must_use()
                || function_metadata.attributes.iter().any(|attr| attr.name.eq_ignore_ascii_case("NoDiscard"));

            if must_use { Some((IssueCode::UnusedFunctionCall, function_id)) } else { None }
        }
        FunctionLikeIdentifier::Method(method_class, method_name) => {
            let method_metadata =
                get_method_by_id(context.codebase, &MethodIdentifier::new(method_class, method_name))?;

            let must_use = method_metadata.flags.must_use()
                || method_metadata.attributes.iter().any(|attr| attr.name.eq_ignore_ascii_case("NoDiscard"));

            if must_use { Some((IssueCode::UnusedMethodCall, method_name)) } else { None }
        }
        FunctionLikeIdentifier::Closure(_, _) => None,
    }
}
