use ahash::HashSet;
use indexmap::IndexMap;

use mago_algebra::clause::Clause;
use mago_algebra::find_satisfying_assignments;
use mago_atom::atom;
use mago_codex::get_anonymous_class;
use mago_codex::ttype::get_literal_string;
use mago_codex::ttype::get_named_object;
use mago_codex::ttype::get_never;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::var_has_root;
use crate::error::AnalysisError;
use crate::formula::get_formula;
use crate::heuristic;
use crate::reconciler::reconcile_keyed_types;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::class_like::analyze_class_like;
use crate::utils::misc::check_for_paradox;

pub mod access;
pub mod argument_list;
pub mod array;
pub mod array_access;
pub mod arrow_function;
pub mod assignment;
pub mod binary;
pub mod call;
pub mod clone;
pub mod closure;
pub mod closure_creation;
pub mod composite_string;
pub mod conditional;
pub mod constant_access;
pub mod construct;
pub mod instantiation;
pub mod literal;
pub mod magic_constant;
pub mod r#match;
pub mod throw;
pub mod unary;
pub mod variable;
pub mod r#yield;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Expression<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let result = match self {
            Expression::Parenthesized(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Literal(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Binary(expr) => expr.analyze(context, block_context, artifacts),
            Expression::UnaryPrefix(expr) => expr.analyze(context, block_context, artifacts),
            Expression::UnaryPostfix(expr) => expr.analyze(context, block_context, artifacts),
            Expression::CompositeString(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Assignment(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Conditional(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Array(expr) => expr.analyze(context, block_context, artifacts),
            Expression::LegacyArray(expr) => expr.analyze(context, block_context, artifacts),
            Expression::ArrayAccess(expr) => expr.analyze(context, block_context, artifacts),
            Expression::ArrayAppend(_) => {
                context.collector.report_with_code(
                    IssueCode::ArrayAppendInReadContext,
                    Issue::error("Array append syntax `[]` cannot be used in a read context.")
                    .with_annotation(
                        Annotation::primary(self.span()).with_message("This syntax is for appending elements, not for reading a value.")
                    )
                    .with_note("The `[]` syntax after an array (e.g., `$array[]`) is used exclusively on the left-hand side of an assignment to append a new element (e.g., `$array[] = $value;`). It does not represent a readable value itself.")
                    .with_help("If you intended to access an array element, provide an index (e.g., `$array[0]`, `$array['key']`). If you intended to append, use this syntax on the left side of an assignment."),
                );

                Ok(())
            }
            Expression::AnonymousClass(anonymous_class) => {
                analyze_attributes(
                    context,
                    block_context,
                    artifacts,
                    anonymous_class.attribute_lists.as_slice(),
                    AttributeTarget::ClassLike,
                )?;

                let Some(class_like_metadata) = get_anonymous_class(context.codebase, self.span()) else {
                    return Ok(());
                };

                analyze_class_like(
                    context,
                    artifacts,
                    None,
                    anonymous_class.span(),
                    anonymous_class.extends.as_ref(),
                    anonymous_class.implements.as_ref(),
                    class_like_metadata,
                    anonymous_class.members.as_slice(),
                )?;

                heuristic::check_class_like(class_like_metadata, anonymous_class.members.as_slice(), context);

                artifacts.set_expression_type(&self, get_named_object(class_like_metadata.name, None));

                Ok(())
            }
            Expression::Closure(expr) => expr.analyze(context, block_context, artifacts),
            Expression::ArrowFunction(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Variable(expr) => expr.analyze(context, block_context, artifacts),
            Expression::ConstantAccess(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Match(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Yield(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Construct(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Throw(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Clone(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Call(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Access(expr) => expr.analyze(context, block_context, artifacts),
            Expression::ClosureCreation(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Instantiation(expr) => expr.analyze(context, block_context, artifacts),
            Expression::MagicConstant(expr) => expr.analyze(context, block_context, artifacts),
            Expression::Pipe(expr) => expr.analyze(context, block_context, artifacts),
            Expression::List(list_expr) => {
                context.collector.report_with_code(
                    IssueCode::ListUsedInReadContext,
                    Issue::error("`list()` construct cannot be used as a value.")
                        .with_annotation(
                            Annotation::primary(list_expr.span())
                                .with_message("`list()` used here in a read context"),
                        )
                        .with_note(
                            "`list()` is a language construct for destructuring an array on the left side of an assignment."
                        )
                        .with_help(
                            "It is not a function and does not return a value. To create an array, use `[]` or `array()`."
                        ),
                );

                artifacts.set_expression_type(&list_expr, get_never());

                Ok(())
            }
            Expression::Self_(keyword) | Expression::Static(keyword) | Expression::Parent(keyword) => {
                let keyword_str = keyword.value;

                context.collector.report_with_code(
                    IssueCode::InvalidScopeKeywordContext,
                    Issue::error(format!("The `{keyword_str}` keyword cannot be used as a standalone value."))
                        .with_annotation(
                            Annotation::primary(keyword.span)
                                .with_message(format!("`{keyword_str}` used as a value here")),
                        )
                        .with_note(
                            format!("The `{keyword_str}` keyword is used to refer to a class scope and must be used with the `::` operator.")
                        )
                        .with_help(
                            format!("Use `{keyword_str}::CONSTANT`, `{keyword_str}::method()`, or `new {keyword_str}()` instead.")
                        ),
                );

                artifacts.set_expression_type(&self, get_never());

                Ok(())
            }
            Expression::Identifier(identifier) => {
                if !identifier.is_local() {
                    unreachable!(
                        "Parser should not produce a bare `Identifier` as a standalone expression in this context. \nIf you see this, it indicates a bug in the parser or the analysis logic. \nPlease report this issue with the following identifier: `{}` line `{}`, column `{}`.",
                        context.source_file.name,
                        context.source_file.line_number(self.offset()),
                        context.source_file.column_number(self.offset()),
                    );
                }

                artifacts.set_expression_type(&self, get_literal_string(atom(identifier.value())));

                Ok(())
            }
        };

        result?;

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Parenthesized<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.expression.analyze(context, block_context, artifacts)?;
        if let Some(u) = artifacts.get_expression_type(&self.expression) {
            artifacts.set_expression_type(&self, u.clone());
        }

        Ok(())
    }
}

pub fn find_expression_logic_issues<'ctx, 'arena>(
    expression: &Expression<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) {
    let mut if_block_context = block_context.clone();
    let mut cond_referenced_var_ids = if_block_context.conditionally_referenced_variable_ids.clone();

    let Some(mut expression_clauses) = get_formula(
        expression.span(),
        expression.span(),
        expression,
        context.get_assertion_context_from_block(block_context),
        artifacts,
    ) else {
        context.collector.report_with_code(
           IssueCode::ExpressionIsTooComplex,
           Issue::warning("Expression is too complex for complete logical analysis.")
               .with_annotation(
                   Annotation::primary(expression.span())
                       .with_message("This expression is too complex for the analyzer to fully understand its logical implications"),
               )
               .with_note(
                   "To prevent performance issues, the analyzer limits how many logical paths it explores for a single expression."
               )
               .with_note(
                   "As a result, some logical paradoxes or redundant checks within this expression may not be detected."
               )
               .with_help(
                   "Consider refactoring this expression into smaller, intermediate variables to improve analysis and readability.",
               ),
        );

        return;
    };

    let mut mixed_var_ids = Vec::new();
    for (var_id, var_type) in &block_context.locals {
        if var_type.is_mixed() && block_context.locals.contains_key(var_id) {
            mixed_var_ids.push(var_id);
        }
    }

    expression_clauses = expression_clauses
        .into_iter()
        .map(|c| {
            let keys = &c.possibilities.keys().collect::<Vec<&String>>();

            let mut new_mixed_var_ids = vec![];
            for i in mixed_var_ids.clone() {
                if !keys.contains(&i) {
                    new_mixed_var_ids.push(i);
                }
            }
            mixed_var_ids = new_mixed_var_ids;

            for key in keys {
                for mixed_var_id in &mixed_var_ids {
                    if var_has_root(key, mixed_var_id) {
                        return Clause::new(
                            IndexMap::default(),
                            expression.span(),
                            expression.span(),
                            Some(true),
                            None,
                            None,
                        );
                    }
                }
            }

            c
        })
        .collect::<Vec<Clause>>();

    let expression_span = expression.span();

    // this will see whether any of the clauses in set A conflict with the clauses in set B
    check_for_paradox(&mut context.collector, &block_context.clauses, &expression_clauses, expression_span);

    expression_clauses.extend(block_context.clauses.iter().map(|v| (**v).clone()).collect::<Vec<_>>());

    let (reconcilable_if_types, active_if_types) = find_satisfying_assignments(
        expression_clauses.iter().as_slice(),
        Some(expression.span()),
        &mut cond_referenced_var_ids,
    );

    reconcile_keyed_types(
        context,
        &reconcilable_if_types,
        active_if_types,
        &mut if_block_context,
        &mut HashSet::default(),
        &cond_referenced_var_ids,
        &expression_span,
        true,
        false,
    );
}
