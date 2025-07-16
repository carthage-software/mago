use ahash::HashMap;

use mago_codex::ttype::TType;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_never;
use mago_codex::ttype::template::TemplateResult;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlAction;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::LanguageConstructKind;
use crate::invocation::analyzer::analyze_invocation;
use crate::invocation::return_type_fetcher::fetch_invocation_return_type;
use crate::issue::TypingIssueKind;

impl Analyzable for Construct {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            Construct::Isset(isset_construct) => isset_construct.analyze(context, block_context, artifacts),
            Construct::Include(construct) => construct.analyze(context, block_context, artifacts),
            Construct::IncludeOnce(construct) => construct.analyze(context, block_context, artifacts),
            Construct::Require(construct) => construct.analyze(context, block_context, artifacts),
            Construct::RequireOnce(construct) => construct.analyze(context, block_context, artifacts),
            Construct::Print(print_construct) => print_construct.analyze(context, block_context, artifacts),
            Construct::Exit(exit_construct) => exit_construct.analyze(context, block_context, artifacts),
            Construct::Die(die_construct) => die_construct.analyze(context, block_context, artifacts),
            Construct::Eval(_) => {
                context.buffer.report(
                    TypingIssueKind::UnsupportedFeature,
                    Issue::warning("Analysis for eval expression is not yet implemented.")
                        .with_annotation(
                            Annotation::primary(self.span())
                                .with_message("This expression will be skipped during analysis"),
                        )
                        .with_note("Support for eval construct expression is planned for a future release.")
                        .with_note(
                            "If you need this feature, please open an issue on the project's GitHub repository.",
                        ),
                );

                Ok(())
            }
            Construct::Empty(_) => {
                context.buffer.report(
                    TypingIssueKind::UnsupportedFeature,
                    Issue::warning("Analysis for empty expression is not yet implemented.")
                        .with_annotation(
                            Annotation::primary(self.span())
                                .with_message("This expression will be skipped during analysis"),
                        )
                        .with_note("Support for empty construct expression is planned for a future release.")
                        .with_note(
                            "If you need this feature, please open an issue on the project's GitHub repository.",
                        ),
                );

                Ok(())
            }
        }
    }
}

impl Analyzable for IncludeConstruct {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_include(
            context,
            block_context,
            artifacts,
            self.span(),
            self.include.span,
            &self.value,
            true,  // is_include
            false, // is_once
        )
    }
}

impl Analyzable for IncludeOnceConstruct {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_include(
            context,
            block_context,
            artifacts,
            self.span(),
            self.include_once.span,
            &self.value,
            true, // is_include
            true, // is_once
        )
    }
}

impl Analyzable for RequireConstruct {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_include(
            context,
            block_context,
            artifacts,
            self.span(),
            self.require.span,
            &self.value,
            false, // is_include
            false, // is_once
        )
    }
}

impl Analyzable for RequireOnceConstruct {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_include(
            context,
            block_context,
            artifacts,
            self.span(),
            self.require_once.span,
            &self.value,
            false, // is_include
            true,  // is_once
        )
    }
}

impl Analyzable for IssetConstruct {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        for value in self.values.iter() {
            let is_valid_expression =
                matches!(value, Expression::Variable(_) | Expression::Access(_) | Expression::ArrayAccess(_));

            if !is_valid_expression {
                // report invalid expression in isset, e.g `isset(1 + 2)`
            }

            let was_inside_isset = block_context.inside_isset;
            block_context.inside_isset = true;
            value.analyze(context, block_context, artifacts)?;
            block_context.inside_isset = was_inside_isset;
        }

        artifacts.set_expression_type(self, get_bool());
        Ok(())
    }
}

impl Analyzable for PrintConstruct {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let target = InvocationTarget::for_language_construct(LanguageConstructKind::Print, self.print.span);
        let arguments = InvocationArgumentsSource::Slice(std::slice::from_ref(&self.value));
        let invocation = Invocation::new(target, arguments, self.span());

        let mut template_result = TemplateResult::default();
        let mut argument_types = HashMap::default();

        analyze_invocation(
            context,
            block_context,
            artifacts,
            &invocation,
            None,
            &mut template_result,
            &mut argument_types,
        )?;

        let return_type =
            fetch_invocation_return_type(context, artifacts, &invocation, &template_result, &argument_types);

        artifacts.set_expression_type(self, return_type);

        Ok(())
    }
}

impl Analyzable for ExitConstruct {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if let Some(arguments_list) = self.arguments.as_ref() {
            let target = InvocationTarget::for_language_construct(LanguageConstructKind::Exit, self.exit.span);
            let arguments = InvocationArgumentsSource::ArgumentList(arguments_list);
            let invocation = Invocation::new(target, arguments, self.span());

            let mut template_result = TemplateResult::default();
            analyze_invocation(
                context,
                block_context,
                artifacts,
                &invocation,
                None,
                &mut template_result,
                &mut HashMap::default(),
            )?;
        }

        block_context.has_returned = true;
        block_context.control_actions.insert(ControlAction::End);

        artifacts.set_expression_type(self, get_never());

        Ok(())
    }
}

impl Analyzable for DieConstruct {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if let Some(arguments_list) = self.arguments.as_ref() {
            let target = InvocationTarget::for_language_construct(LanguageConstructKind::Exit, self.die.span);
            let arguments = InvocationArgumentsSource::ArgumentList(arguments_list);
            let invocation = Invocation::new(target, arguments, self.span());

            let mut template_result = TemplateResult::default();
            analyze_invocation(
                context,
                block_context,
                artifacts,
                &invocation,
                None,
                &mut template_result,
                &mut HashMap::default(),
            )?;
        }

        block_context.has_returned = true;
        block_context.control_actions.insert(ControlAction::End);

        artifacts.set_expression_type(self, get_never());

        Ok(())
    }
}

fn analyze_include<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    construct_span: Span,
    keyword_span: Span,
    included_file: &Expression,
    is_include: bool,
    is_once: bool,
) -> Result<(), AnalysisError> {
    let was_inside_call = block_context.inside_call;
    block_context.inside_call = true;
    included_file.analyze(context, block_context, artifacts)?;
    block_context.inside_call = was_inside_call;

    let construct_name_str = if is_include {
        if is_once { "include_once" } else { "include" }
    } else if is_once {
        "require_once"
    } else {
        "require"
    };

    if !context.settings.allow_include {
        context.buffer.report(
            TypingIssueKind::DisallowedInclude,
            Issue::error(format!(
                "File inclusion via `{construct_name_str}` is disallowed by your project configuration.",
            ))
            .with_annotation(Annotation::primary(keyword_span).with_message("This operation is disallowed"))
            .with_note("Including files can introduce security vulnerabilities and make dependencies less explicit.")
            .with_help("Refactor to use a class autoloader or dependency injection instead of manual file includes."),
        );
    }

    let included_file_type = artifacts.get_expression_type(included_file).cloned().unwrap_or_else(get_mixed_any);
    if !included_file_type.is_string() {
        context.buffer.report(
            TypingIssueKind::InvalidIncludeArgument,
            Issue::error(format!(
                "Argument for `{construct_name_str}` must be a string representing a file path, but found type `{}`.",
                included_file_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(included_file.span()).with_message(format!(
                "This expression has type `{}`",
                included_file_type.get_id(Some(context.interner))
            )))
            .with_help("Ensure the expression provided to this construct evaluates to a string."),
        );
    }

    if block_context.scope.is_mutation_free() {
        context.buffer.report(
            TypingIssueKind::ImpureInclude,
            Issue::error(format!(
                "Impure use of `{construct_name_str}` in a context declared as mutation-free or pure.",
            ))
            .with_annotation(Annotation::primary(keyword_span).with_message("Impure operation in pure context"))
            .with_note("Including files executes code and can cause side effects (e.g., I/O, changing global state), which is not allowed in a mutation-free context.")
            .with_help("Refactor the pure function to not rely on file includes, or remove the `@pure` / `@mutation-free` designation if side effects are intended."),
        );
    }

    if is_include {
        context.buffer.report(
            TypingIssueKind::IncludeInsteadOfRequire,
            Issue::help("Consider using `require` or `require_once` for better error handling.")
                .with_annotation(Annotation::primary(keyword_span).with_message(format!(
                    "Using `{construct_name_str}` here will only emit a warning on failure.",
                )))
                .with_note("`require` will throw a fatal `Error` if the file cannot be found, immediately halting execution, which is often safer.")
                .with_help(format!(
                    "Change `{construct_name_str}` to `{}` to make file inclusion mandatory.",
                    if is_once { "require_once" } else { "require" }
                )),
        );
    }

    if !is_once {
        context.buffer.report(
            TypingIssueKind::IncludeInsteadOfOnceVariant,
            Issue::help(format!(
                "Consider using `{construct_name_str}_once` to prevent multiple inclusions of the same file.",
            ))
            .with_annotation(Annotation::primary(keyword_span).with_message("This may include the file multiple times"))
            .with_note("Including the same file multiple times can lead to 'cannot redeclare function' or 'cannot redeclare class' fatal errors if the file defines symbols.")
            .with_help(format!("Change to `{construct_name_str}_once` to ensure the file is included only one time.")),
        );
    }

    artifacts.set_expression_type(&construct_span, get_mixed());

    Ok(())
}
