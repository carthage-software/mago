use ahash::HashSet;
use mago_codex::get_function;
use mago_codex::get_method_by_id;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_interner::StringIdentifier;
use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::Context;
use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;
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
pub mod r#try;
pub mod unset;

impl Analyzable for Statement {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let last_statement_span = context.statement_span;
        context.statement_span = self.span();

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
                context.scope.populate_from_use(context.interner, r#use);

                Ok(())
            }
            Statement::Namespace(namespace) => {
                match &namespace.name {
                    Some(name) => {
                        let name = context.interner.lookup(name.value());

                        context.scope = NamespaceScope::for_namespace(name);
                    }
                    None => {
                        context.scope = NamespaceScope::global();
                    }
                }

                analyze_statements(namespace.statements().as_slice(), context, block_context, artifacts)
            }
            Statement::Class(class) => {
                let class_name_id = context.resolved_names.get(&class.name);
                let class_name = context.interner.lookup(class_name_id);

                context.scope.add(NameKind::Default, class_name, None as Option<&str>);

                class.analyze(context, block_context, artifacts)
            }
            Statement::Interface(interface) => {
                let interface_name_id = context.resolved_names.get(&interface.name);
                let interface_name = context.interner.lookup(interface_name_id);

                context.scope.add(NameKind::Default, interface_name, None as Option<&str>);

                interface.analyze(context, block_context, artifacts)
            }
            Statement::Trait(r#trait) => {
                let trait_name_id = context.resolved_names.get(&r#trait.name);
                let trait_name = context.interner.lookup(trait_name_id);

                context.scope.add(NameKind::Default, trait_name, None as Option<&str>);

                r#trait.analyze(context, block_context, artifacts)
            }
            Statement::Enum(r#enum) => {
                let enum_name_id = context.resolved_names.get(&r#enum.name);
                let enum_name = context.interner.lookup(enum_name_id);

                context.scope.add(NameKind::Default, enum_name, None as Option<&str>);

                r#enum.analyze(context, block_context, artifacts)
            }
            Statement::Constant(constant) => {
                for item in constant.items.iter() {
                    let constant_item_name_id = context.resolved_names.get(&item.name);
                    let constant_item_name = context.interner.lookup(constant_item_name_id);

                    context.scope.add(NameKind::Constant, constant_item_name, None as Option<&str>);
                }

                constant.analyze(context, block_context, artifacts)
            }
            Statement::Function(function) => {
                let function_name_id = context.resolved_names.get(&function.name);
                let function_name = context.interner.lookup(function_name_id);

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
            Statement::Switch(r#switch) => {
                context.buffer.report(
                    TypingIssueKind::UnsupportedFeature,
                    Issue::warning("Analysis for `switch` statements is not yet implemented.")
                        .with_annotation(
                            Annotation::primary(r#switch.span())
                                .with_message("This `switch` statement will be skipped during analysis"),
                        )
                        .with_note(
                            "Support for `switch` statements is planned for a future release."
                        )
                        .with_note(
                            "In the meantime, types will not be narrowed and variables will not be updated within its branches, which may lead to other errors."
                        )
                        .with_help(
                            "For full type analysis, consider refactoring this logic into an `if` statement, or a `match` expression if possible.",
                        ),
                );

                Ok(())
            }
        };

        result?;

        if let Statement::Expression(expression) = self
            && context.settings.find_unused_expressions
        {
            detect_unused_statement_expressions(&expression.expression, self, context, artifacts);
        }

        context.statement_span = last_statement_span;
        block_context.conditionally_referenced_variable_ids = HashSet::default();

        Ok(())
    }
}

#[inline]
pub fn analyze_statements<'a>(
    statements: &[Statement],
    context: &mut Context<'a>,
    block: &mut BlockContext<'a>,
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
                    context.buffer.report(
                        TypingIssueKind::UselessControlFlow,
                        Issue::help("This control flow is unnecessary")
                            .with_annotation(
                                Annotation::primary(statement.span()).with_message("This statement has no effect."),
                            )
                            .with_note("This statement is unreachable because the block has already returned.")
                            .with_help("Consider removing this statement as it does not do anything in this context."),
                    );
                } else {
                    context.buffer.report(
                        TypingIssueKind::UnevaluatedCode,
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
fn detect_unused_statement_expressions(
    expression: &Expression,
    statement: &Statement,
    context: &mut Context<'_>,
    artifacts: &mut AnalysisArtifacts,
) {
    if let Some((issue_kind, name_id)) = has_unused_must_use(expression, context, artifacts) {
        let name_str = context.interner.lookup(&name_id);

        context.buffer.report(
            issue_kind,
            Issue::error(format!("The return value of '{name_str}' must be used."))
                .with_annotation(Annotation::primary(statement.span()).with_message("The result of this call is ignored"))
                .with_note(format!("The function or method '{name_str}' is marked with @must-use, indicating its return value is important and should not be discarded."))
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
            let Expression::Identifier(function_name) = function.as_ref() else {
                return;
            };

            let unqualified_name = function_name.value();
            let name = context.resolved_names.get(function_name);

            let Some(function) = get_function(context.codebase, context.interner, name).or_else(|| {
                if function_name.is_fully_qualified() {
                    None
                } else {
                    get_function(context.codebase, context.interner, unqualified_name)
                }
            }) else {
                return;
            };

            if function.is_pure && function.get_thrown_types().is_empty() && !function.has_throw {
                "Calling a pure function without using its result has no effect (consider using the result or removing the call)."
            } else {
                return;
            }
        }
        _ => return,
    };

    context.buffer.report(
        TypingIssueKind::UnusedStatement,
        Issue::note("Statement has no effect.")
            .with_annotation(Annotation::primary(expression.span()).with_message(useless_expression_message))
            .with_help(
                "Remove this statement or use the expression's value in an assignment, condition, or function call.",
            ),
    );
}

/// Checks if an expression is a call to a `@must-use` function/method
/// and returns the appropriate issue kind and the name identifier if the result is unused.
fn has_unused_must_use<'a>(
    expression: &'a Expression,
    context: &'a Context<'_>,
    artifacts: &'a AnalysisArtifacts,
) -> Option<(TypingIssueKind, StringIdentifier)> {
    let call_expression = match expression {
        Expression::Call(call_expr) => call_expr,
        _ => return None,
    };

    let functionlike_id_from_call =
        get_function_like_id_from_call(call_expression, context.resolved_names, &artifacts.expression_types)?;

    match functionlike_id_from_call {
        FunctionLikeIdentifier::Function(function_id) => {
            let function_metadata = get_function(context.codebase, context.interner, &function_id)?;
            if function_metadata.must_use { Some((TypingIssueKind::UnusedFunctionCall, function_id)) } else { None }
        }
        FunctionLikeIdentifier::Method(method_class, method_name) => {
            let method_metadata = get_method_by_id(
                context.codebase,
                context.interner,
                &MethodIdentifier::new(method_class, method_name),
            )?;

            if method_metadata.must_use { Some((TypingIssueKind::UnusedMethodCall, method_name)) } else { None }
        }
        FunctionLikeIdentifier::Closure(_) => None,
    }
}
