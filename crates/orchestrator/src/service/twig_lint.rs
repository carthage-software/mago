use std::sync::Arc;

use mago_allocator::Arena;

use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_span::HasSpan;
use mago_twig_linter::Linter;
use mago_twig_linter::registry::RuleRegistry;
use mago_twig_linter::settings::Settings;
use mago_twig_syntax::error::ParseError;
use mago_twig_syntax::parser::Parser;
use mago_twig_syntax::settings::ParserSettings;

use crate::OrchestratorError;
use crate::service::lint::LintMode;
use crate::service::pipeline::StatelessParallelPipeline;
use crate::service::pipeline::StatelessReducer;

/// The deepest Twig nesting the service parses. Deeper templates are reported, not linted.
const NESTING_LIMIT: u16 = 32;

/// Service responsible for running the Twig linting pipeline.
///
/// Every selected file is linted as Twig, whatever its name. No PHP parser or codebase
/// scan is involved.
#[derive(Debug)]
pub struct TwigLintService {
    /// The read-only database containing the templates to lint.
    database: ReadDatabase,

    /// The linter settings to configure the linting process.
    settings: Settings,

    /// Whether to display progress bars during linting.
    use_progress_bars: bool,
}

impl TwigLintService {
    /// Creates a new instance of the `TwigLintService`.
    ///
    /// # Arguments
    ///
    /// * `database` - The read-only database containing the templates to lint.
    /// * `settings` - The linter settings to configure the linting process.
    /// * `use_progress_bars` - Whether to display progress bars during linting.
    #[must_use]
    pub fn new(database: ReadDatabase, settings: Settings, use_progress_bars: bool) -> Self {
        Self { database, settings, use_progress_bars }
    }

    /// Creates a `RuleRegistry` based on the current settings.
    ///
    /// # Arguments
    ///
    /// * `only` - An optional list of specific rules to include.
    /// * `include_disabled` - Whether to include disabled rules in the registry.
    #[must_use]
    pub fn create_registry(&self, only: Option<&[String]>, include_disabled: bool) -> RuleRegistry {
        let registry = RuleRegistry::build(&self.settings, only, include_disabled);
        if let Some(only) = only
            && registry.is_empty()
        {
            tracing::warn!("No Twig rules found for the specified 'only' filter: {:?}", only);
        }

        registry
    }

    /// Lints every template in the database in parallel.
    ///
    /// Syntax errors are always reported. Rules run only in [`LintMode::Full`], and only on
    /// templates that parse cleanly.
    ///
    /// # Arguments
    ///
    /// * `mode` - Whether to run the rules or only check syntax.
    /// * `only` - An optional list of specific rules to run.
    /// * `include_disabled` - Whether to run rules that are disabled in the settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the parallel pipeline fails.
    pub fn lint(
        self,
        mode: LintMode,
        only: Option<&[String]>,
        include_disabled: bool,
    ) -> Result<IssueCollection, OrchestratorError> {
        const PROGRESS_BAR_THEME: &str = "🧹 Linting Twig";

        let context = TwigLintContext { registry: Arc::new(self.create_registry(only, include_disabled)), mode };

        let pipeline = StatelessParallelPipeline::new(
            PROGRESS_BAR_THEME,
            self.database,
            context,
            Box::new(TwigLintResultReducer),
            self.use_progress_bars,
        );

        pipeline.run(|context, arena, file| Ok(lint_file(&context, arena, &file)))
    }
}

fn lint_file<A>(context: &TwigLintContext, arena: &A, file: &File) -> IssueCollection
where
    A: Arena,
{
    let template = Parser::new(arena, file.id, &file.contents, ParserSettings::default())
        .with_recursion_limit(NESTING_LIMIT)
        .parse(&file.contents, file.id);

    if template.has_errors() {
        return template.errors.iter().map(syntax_issue).collect();
    }

    if context.mode == LintMode::SemanticsOnly {
        return IssueCollection::new();
    }

    Linter::from_registry(arena, Arc::clone(&context.registry)).lint(file, template)
}

fn syntax_issue(error: &ParseError<'_>) -> Issue {
    match error {
        ParseError::RecursionLimitExceeded(span) => Issue::error(format!(
            "Twig nesting exceeds Mago's limit of {NESTING_LIMIT}; Mago did not lint this template."
        ))
        .with_code("twig-depth-limit")
        .with_annotation(Annotation::primary(*span).with_message("Nesting limit reached here"))
        .with_note(format!(
            "Mago stops parsing a template nested more than {NESTING_LIMIT} levels deep, so that deep input cannot exhaust its stack. Twig may still accept the template."
        ))
        .with_help(
            "Move the deeply nested part into a macro, an included template, or a variable set with `{% set %}`.",
        ),
        _ => Issue::error(error.to_string())
            .with_code("twig-syntax-error")
            .with_annotation(Annotation::primary(error.span()).with_message("Invalid Twig syntax"))
            .with_note("Mago could not parse this template, so no lint rule ran on it.")
            .with_help("Fix the syntax error; Mago lints the template once it parses."),
    }
}

#[derive(Clone)]
struct TwigLintContext {
    pub registry: Arc<RuleRegistry>,
    pub mode: LintMode,
}

#[derive(Debug)]
struct TwigLintResultReducer;

impl StatelessReducer<IssueCollection, IssueCollection> for TwigLintResultReducer {
    fn reduce(&self, results: Vec<IssueCollection>) -> Result<IssueCollection, OrchestratorError> {
        let mut final_issues = IssueCollection::new();
        final_issues.reserve(results.iter().map(IssueCollection::len).sum());
        for issues in results {
            final_issues.extend(issues);
        }

        Ok(final_issues)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::borrow::Cow;

    use mago_allocator::LocalArena;

    use super::*;

    fn lint(source: &[u8], mode: LintMode, only: &[&str]) -> IssueCollection {
        let settings = Settings::default();
        let only: Vec<String> = only.iter().map(|code| (*code).to_owned()).collect();
        let context = TwigLintContext {
            registry: Arc::new(RuleRegistry::build(&settings, (!only.is_empty()).then_some(only.as_slice()), false)),
            mode,
        };

        let file = File::ephemeral(Cow::Borrowed(b"template.twig"), Cow::Owned(source.to_vec()));
        lint_file(&context, &LocalArena::new(), &file)
    }

    #[test]
    fn deep_templates_report_a_limit_and_flat_chains_still_get_linted() {
        std::thread::Builder::new()
            .stack_size(4 * 1024 * 1024)
            .spawn(|| {
                for source in [
                    format!("{{{{ {}value{} }}}}", "(".repeat(100), ")".repeat(100)),
                    format!("{}value{}", "{% if flag %}".repeat(100), "{% endif %}".repeat(100)),
                ] {
                    let issues = lint(source.as_bytes(), LintMode::Full, &[]);
                    assert!(issues.iter().any(|issue| issue.code.as_deref() == Some("twig-depth-limit")));
                    assert!(issues.iter().all(|issue| issue.code.as_deref() != Some("twig-syntax-error")));
                    assert!(issues.iter().all(is_explained));
                }

                let source = format!("{{{{ {}dump(value) }}}}", "1 + ".repeat(100_000));
                let issues = lint(source.as_bytes(), LintMode::Full, &[]);
                assert_eq!(issues.len(), 1);
                assert!(issues.iter().all(|issue| issue.code.as_deref() == Some("twig-no-dump")));
            })
            .expect("the test thread starts")
            .join()
            .expect("the test thread does not panic");
    }

    #[test]
    fn syntax_only_mode_reports_syntax_errors_without_lint_rules_or_pragmas() {
        assert!(
            lint(b"{# @mago-expect lint:twig-no-dump #}\n{{ dump(user) }}", LintMode::SemanticsOnly, &[]).is_empty()
        );

        let issues = lint(b"{# @mago-ignore lint:twig-syntax-error #}\n{{ ( }}", LintMode::SemanticsOnly, &[]);
        assert!(!issues.is_empty());
        assert!(issues.iter().all(|issue| issue.code.as_deref() == Some("twig-syntax-error")));
        assert!(issues.iter().all(|issue| issue.edits.is_empty()));
    }

    fn is_explained(issue: &Issue) -> bool {
        issue.annotations.iter().all(|annotation| annotation.message.is_some())
            && !issue.notes.is_empty()
            && issue.help.is_some()
    }

    #[test]
    fn syntax_errors_explain_themselves() {
        let issues = lint(b"{{ ( }}", LintMode::Full, &[]);
        assert!(!issues.is_empty());
        assert!(issues.iter().all(is_explained));
    }

    #[test]
    fn syntax_errors_skip_the_rules() {
        let issues = lint(b"{{ dump(user) }}{{ ( }}", LintMode::Full, &["twig-no-dump"]);
        assert!(!issues.is_empty());
        assert!(issues.iter().all(|issue| issue.code.as_deref() == Some("twig-syntax-error")));
    }
}
