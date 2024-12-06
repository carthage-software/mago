use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

pub use codespan_reporting::term::termcolor::*;

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::term;
use codespan_reporting::term::Config;

use fennec_source::SourceIdentifier;
use fennec_source::SourceManager;

use crate::Issue;
use crate::IssueCollection;
use crate::Level;

#[derive(Clone)]
pub struct Reporter {
    writer: Arc<Mutex<StandardStream>>,
    config: Arc<Config>,
    manager: SourceManager,
}

impl Reporter {
    pub fn new(manager: SourceManager) -> Self {
        Self {
            writer: Arc::new(Mutex::new(StandardStream::stdout(ColorChoice::Auto))),
            config: Arc::new(Config::default()),
            manager,
        }
    }

    pub fn report(&self, issue: Issue) {
        let mut writer = Gaurd(self.writer.lock().expect("failed to aquire lock"));

        let diagnostic: Diagnostic<SourceIdentifier> = issue.into();

        term::emit(&mut writer, &self.config, &self.manager, &diagnostic).unwrap();
    }

    pub fn report_all(&self, issues: impl IntoIterator<Item = Issue>) -> Option<Level> {
        let collection = IssueCollection::from(issues);
        let mut writer = Gaurd(self.writer.lock().expect("failed to aquire lock"));

        let highest_level = collection.get_highest_level();
        let mut errors = 0;
        let mut warnings = 0;
        let mut notes = 0;
        let mut help = 0;
        let mut suggestions = 0;

        for issue in collection {
            match &issue.level {
                Level::Note => {
                    notes += 1;
                }
                Level::Help => {
                    help += 1;
                }
                Level::Warning => {
                    warnings += 1;
                }
                Level::Error => {
                    errors += 1;
                }
            }

            if !issue.suggestions.is_empty() {
                suggestions += 1;
            }

            let diagnostic: Diagnostic<SourceIdentifier> = issue.into();

            term::emit(&mut writer, &self.config, &self.manager, &diagnostic).unwrap();
        }

        if let Some(highest_level) = highest_level {
            let total_issues = errors + warnings + notes + help;
            let mut message_notes = vec![];
            if errors > 0 {
                message_notes.push(format!("{} error(s)", errors));
            }

            if warnings > 0 {
                message_notes.push(format!("{} warning(s)", warnings));
            }

            if notes > 0 {
                message_notes.push(format!("{} note(s)", notes));
            }

            if help > 0 {
                message_notes.push(format!("{} help message(s)", help));
            }

            let mut diagnostic: Diagnostic<SourceIdentifier> = Diagnostic::new(highest_level.into())
                .with_message(format!("found {} issues: {}", total_issues, message_notes.join(", ")));

            if suggestions > 0 {
                diagnostic =
                    diagnostic.with_notes(vec![format!("{} issues contain auto-fix suggestions", suggestions)]);
            }

            term::emit(&mut writer, &self.config, &self.manager, &diagnostic).unwrap();
        }

        highest_level
    }
}

unsafe impl Send for Reporter {}
unsafe impl Sync for Reporter {}

impl std::fmt::Debug for Reporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reporter").field("config", &self.config).field("manager", &self.manager).finish_non_exhaustive()
    }
}

struct Gaurd<'a>(MutexGuard<'a, StandardStream>);

impl<'a> WriteColor for Gaurd<'a> {
    fn set_color(&mut self, spec: &term::termcolor::ColorSpec) -> std::io::Result<()> {
        self.0.set_color(spec)
    }

    fn reset(&mut self) -> std::io::Result<()> {
        self.0.reset()
    }

    fn supports_color(&self) -> bool {
        self.0.supports_color()
    }
}

impl<'a> std::io::Write for Gaurd<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
