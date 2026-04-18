use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use bumpalo::Bump;
use clap::ColorChoice;
use clap::Parser;
use indoc::indoc;

use mago_database::file::FileId;
use mago_pattern::Match;
use mago_pattern::compile;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_span::Position;
use mago_span::Span;
use mago_text_edit::TextEdit;

use crate::commands::args::reporting::ReportingArgs;
use crate::config::Configuration;
use crate::error::Error;
use crate::utils::create_orchestrator;

/// Issue code emitted for every pattern match, so reporting filters (`--retain-code`,
/// `--minimum-report-level`) can target query output specifically.
const QUERY_ISSUE_CODE: &str = "query-match";

#[derive(Parser, Debug)]
#[command(
    name = "query",
    about = "Run a GritQL-style pattern across the codebase in parallel (experimental).",
    long_about = indoc! {"
        Match a pattern against the AST of every PHP file in the workspace (or the supplied
        paths) and emit one issue per match through the shared reporting pipeline.

        Metavariables use the `^name` sigil. `^...name` is a sequence metavariable that
        captures a variadic run of sibling arguments or elements.

        Rewrites use a top-level `` `lhs` => `rhs` `` form: the `=>` between
        backtick-delimited snippets means \"rewrite lhs to rhs\". A `=>` INSIDE a snippet
        body is ordinary PHP syntax (array key/value, match arm, arrow function) and is not
        interpreted as a rewrite. Pass `--fix` to apply rewrites, or `--fix --dry-run` to
        preview them as a coloured unified diff.

        NOTE: this command is experimental. Its behaviour, arguments, and output may change
        without warning between releases.
    "}
)]
pub struct QueryCommand {
    /// The pattern to match.
    #[arg(required = true)]
    pub pattern: String,

    /// Query specific files or directories, overriding the source configuration. When
    /// omitted, the command runs against every PHP file in the workspace.
    #[arg()]
    pub path: Vec<PathBuf>,

    #[clap(flatten)]
    pub reporting: ReportingArgs,
}

impl QueryCommand {
    pub fn execute(self, mut configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        let pattern_arena = Bump::new();
        let compiled = match compile(&pattern_arena, &self.pattern) {
            Ok(c) => Arc::new(c),
            Err(err) => {
                tracing::error!("Failed to compile pattern: {err}");
                return Ok(ExitCode::from(2));
            }
        };

        let editor_url = configuration.editor_url.take();

        let mut orchestrator = create_orchestrator(&configuration, color_choice, false, true, false);
        if !self.path.is_empty() {
            orchestrator.set_source_paths(self.path.iter().map(|p| p.to_string_lossy().to_string()));
        }

        let mut database = orchestrator.load_database(&configuration.source.workspace, false, None, None)?;
        let service = orchestrator.get_query_service(database.read_only(), compiled);
        let result = service.run()?;

        let pattern_preview = short_pattern_preview(&self.pattern);
        let mut issues = IssueCollection::new();
        for (file_id, status) in result.iter() {
            for m in &status.matches {
                issues.push(build_issue(&pattern_preview, *file_id, m));
            }
        }

        let processor = self.reporting.get_processor(color_choice, editor_url, configuration.linter.minimum_fail_level);
        let (exit_code, _) = processor.process_issues(&orchestrator, &mut database, issues, None, false)?;

        Ok(exit_code)
    }
}

/// Builds a single Help-level issue for one pattern match, attaching every rewrite as
/// a `TextEdit` so the reporting pipeline's `--fix` / `--fix --dry-run` paths can apply
/// or preview them.
fn build_issue(pattern_preview: &str, file_id: FileId, m: &Match) -> Issue {
    let start = Position::new(m.range.start as u32);
    let end = Position::new(m.range.end as u32);
    let span = Span::new(file_id, start, end);

    let mut annotation = Annotation::primary(span);
    if !m.captures.is_empty() {
        let joined = m.captures.iter().map(|(name, value)| format!("^{name} = {value}")).collect::<Vec<_>>().join(", ");
        annotation = annotation.with_message(joined);
    }

    let mut issue = Issue::new(Level::Help, format!("Pattern matched: `{pattern_preview}`"))
        .with_code(QUERY_ISSUE_CODE)
        .with_annotation(annotation);

    for rewrite in &m.rewrites {
        let edit = TextEdit::replace(rewrite.range.start as u32..rewrite.range.end as u32, rewrite.replacement.clone());
        issue = issue.with_edit(file_id, edit);
    }

    issue
}

/// One-line preview of the pattern source, collapsing whitespace so long multi-line
/// patterns stay readable in issue messages.
fn short_pattern_preview(pattern: &str) -> String {
    let collapsed: String = pattern.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() > 80 { format!("{}…", &collapsed[..79]) } else { collapsed }
}
