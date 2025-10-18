use std::rc::Rc;

use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_null;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::assignment;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Variable<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            Variable::Direct(var) => var.analyze(context, block_context, artifacts),
            Variable::Indirect(var) => var.analyze(context, block_context, artifacts),
            Variable::Nested(var) => var.analyze(context, block_context, artifacts),
        }
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for DirectVariable<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let resulting_type = read_variable(context, block_context, artifacts, self.name, self.span());

        artifacts.set_rc_expression_type(self, resulting_type);

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for IndirectVariable<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.expression.analyze(context, block_context, artifacts)?;

        let resulting_type = match artifacts.get_expression_type(&self.expression) {
            Some(expression_type) if expression_type.is_single() => {
                match expression_type.get_single_literal_string_value() {
                    Some(value) => {
                        let variable_name = format!("${value}");

                        read_variable(context, block_context, artifacts, &variable_name, self.span())
                    }
                    _ => Rc::new(get_mixed()),
                }
            }
            _ => Rc::new(get_mixed()),
        };

        artifacts.set_rc_expression_type(self, resulting_type);

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for NestedVariable<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.variable.analyze(context, block_context, artifacts)?;

        let resulting_type = match artifacts.get_expression_type(&self.variable) {
            Some(expression_type) if expression_type.is_single() => {
                match expression_type.get_single_literal_string_value() {
                    Some(value) => {
                        let variable_name = format!("${value}");

                        read_variable(context, block_context, artifacts, &variable_name, self.span())
                    }
                    _ => Rc::new(get_mixed()),
                }
            }
            _ => Rc::new(get_mixed()),
        };

        artifacts.set_rc_expression_type(self, resulting_type);

        Ok(())
    }
}

fn read_variable<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    variable_name: &str,
    variable_span: Span,
) -> Rc<TUnion> {
    let _ = block_context.has_variable(variable_name);

    let variable_type = match block_context.locals.get(variable_name) {
        Some(variable_type) => variable_type.clone(),
        None => {
            if block_context.variables_possibly_in_scope.contains(variable_name) {
                context.collector.report_with_code(
                    IssueCode::PossiblyUndefinedVariable,
                    Issue::warning(format!(
                        "Variable `{variable_name}` might not have been defined on all execution paths leading to this point.",
                    ))
                    .with_annotation(
                        Annotation::primary(variable_span)
                            .with_message(format!("`{variable_name}` might be undefined here")),
                    )
                    .with_note("This can happen if the variable is assigned within a conditional block and there's an execution path to this usage where that block is skipped.")
                    .with_note("Accessing an undefined variable will result in an `E_WARNING` (PHP 8+) or `E_NOTICE` (PHP 7) and it will be treated as `null`.")
                    .with_help(format!("Initialize `{variable_name}` before conditional paths, or use `isset()` to check its existence."))
                );

                Rc::new(get_mixed())
            } else if block_context.inside_variable_reference {
                context.collector.report_with_code(
                    IssueCode::ReferenceToUndefinedVariable,
                    Issue::help(format!("Reference created from a previously undefined variable `{variable_name}`.",))
                        .with_annotation(
                            Annotation::primary(variable_span)
                                .with_message(format!("`{variable_name}` is created here and initialized to `null` because it's used as a reference")),
                        )
                        .with_note(
                            "When a reference is taken from an undefined variable, PHP creates it with a `null` value."
                        )
                        .with_note(
                            "This is often used for output parameters but can hide typos if you intended to use an existing variable."
                        )
                        .with_help(
                            format!("If this is intentional, consider initializing `{variable_name}` to `null` first for code clarity. Otherwise, check for typos.")
                        ),
                );

                // This variable does not currently exist, but is being referenced.
                // therefore, we need to analyze it as if it was being assigned `null`.
                assignment::analyze_assignment_to_variable(
                    context,
                    block_context,
                    artifacts,
                    variable_span,
                    None,
                    get_null(),
                    variable_name,
                    false,
                );

                Rc::new(get_mixed())
            } else if block_context.inside_unset {
                Rc::new(get_null())
            } else if block_context.inside_isset {
                Rc::new(get_mixed())
            } else {
                let mut issue = Issue::error(format!("Undefined variable: `{variable_name}`.")).with_annotation(
                    Annotation::primary(variable_span)
                        .with_message(format!("Variable `{variable_name}` used here but not defined")),
                );

                let mut has_confusable_characters = false;
                if let Some(confusable_note) = generate_confusable_character_note(variable_name) {
                    has_confusable_characters = true;
                    issue = issue.with_note(confusable_note);
                }

                let similar_suggestions = find_similar_variable_names(block_context, variable_name);

                let mut help_message =
                    format!("Ensure `{variable_name}` is assigned a value before this use, or check its scope.");
                if !similar_suggestions.is_empty() {
                    let suggestions_str = similar_suggestions.join("`, `");
                    issue = issue.with_note(format!(
                        "Did you perhaps mean one of these defined variables: `{suggestions_str}`?"
                    ));

                    help_message = format!(
                        "Check for typos (like those suggested above), ensure `{variable_name}` is assigned, or verify its scope."
                    );
                } else if !has_confusable_characters {
                    // Only add generic typo help if no confusable chars and no specific suggestions.
                    help_message = format!(
                        "Ensure `{variable_name}` is assigned before use, or check for typos and variable scope."
                    );
                }

                context.collector.report_with_code(IssueCode::UndefinedVariable, issue.with_help(help_message));

                Rc::new(get_mixed())
            }
        }
    };

    if variable_type.possibly_undefined_from_try && !block_context.inside_isset {
        context.collector.report_with_code(
            IssueCode::PossiblyUndefinedVariable,
            Issue::warning(format!(
                "Variable `{variable_name}` might be undefined here because its assignment occurs within a `try` block.",
            ))
            .with_annotation(
                Annotation::primary(variable_span)
                    .with_message(format!("`{variable_name}` might be undefined due to an exception in the preceding `try` block")),
            )
            .with_note(
                "This variable is assigned inside a `try` block. If an exception was thrown before this assignment was reached, the variable would not be defined in this context."
            )
            .with_note(
                "Accessing an undefined variable will result in an `E_WARNING` (PHP 8+) or `E_NOTICE` (PHP 7) and it will be treated as `null`."
            )
            .with_help(format!(
                "Initialize `{variable_name}` before the `try` block if it should always exist, or use `isset()` to check its existence.",
            )),
        );
    }

    variable_type
}

fn find_similar_variable_names<'ctx>(context: &BlockContext<'ctx>, target: &str) -> Vec<String> {
    let mut suggestions: Vec<(usize, &String)> = Vec::new();

    for local in context.locals.keys() {
        if local.is_empty() {
            continue;
        }

        let distance = strsim::levenshtein(target, local);

        if distance > 0 && distance <= 3 {
            suggestions.push((distance, local));
        }
    }

    suggestions.sort_by_key(|k| k.0);
    suggestions.into_iter().map(|(_, name)| name).cloned().collect()
}

fn generate_confusable_character_note(variable_name: &str) -> Option<String> {
    let mut has_non_std_ascii_alphanumeric = false;
    let mut confusable_examples = Vec::new();

    for c in variable_name.chars().skip(1) {
        if !c.is_ascii_alphanumeric() && c != '_' {
            if c.is_alphabetic() {
                has_non_std_ascii_alphanumeric = true;
                if c == '\u{0430}' {
                    confusable_examples.push("'а' (Cyrillic 'a')");
                } else if c == '\u{03BF}' {
                    confusable_examples.push("'ο' (Greek 'o')");
                }
            } else if c > '\x7F' {
                has_non_std_ascii_alphanumeric = true;
            }
        }
    }

    if has_non_std_ascii_alphanumeric {
        let mut note = format!("Variable name `{variable_name}` contains non-standard ASCII alphanumeric characters.");
        if !confusable_examples.is_empty() {
            note.push_str(&format!(" For example, it might contain {}.", confusable_examples.join(" or ")));
        }

        note.push_str(" Please verify all characters are intended.");

        Some(note)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = possibly_undefined_variable_from_foreach,
        code = indoc! {r#"
            <?php

            /**
             * @param array<string, string> $arr
             */
            function iter(array $arr)
            {
                $value = 1;
                unset($value);
                foreach ($arr as $key => $value) {
                    $y = 1;
                    echo 'Key: ' . $key . ', Value: ' . $value . "\n";
                    echo 'Y: ' . $y . "\n";
                }

                echo (string) $key;
                echo (string) $value;
                echo (string) $y;
            }
        "#},
        issues = [
            IssueCode::PossiblyUndefinedVariable, // $key
            IssueCode::PossiblyUndefinedVariable, // $value
            IssueCode::PossiblyUndefinedVariable, // $y
        ]
    }

    test_analysis! {
        name = defined_variable_from_foreach,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-array<string, string> $arr
             */
            function iter(array $arr)
            {
                $value = 1;
                unset($value);
                foreach ($arr as $key => $value) {
                    $y = 1;
                    echo 'Key: ' . $key . ', Value: ' . $value . "\n";
                    echo 'Y: ' . $y . "\n";
                }

                echo (string) $key;
                echo (string) $value;
                echo (string) $y;
            }
        "#},
        issues = [
            IssueCode::RedundantCast, // $key is known to be a string
            IssueCode::RedundantCast, // $value is known to be a string
        ]
    }
}
