use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_text_edit::TextEditor;
use mago_twig_linter::Linter;
use mago_twig_linter::settings::Settings;
use mago_twig_syntax::parser::Parser;
use mago_twig_syntax::settings::ParserSettings;

/// The nesting limit `mago twig:lint` parses with.
const NESTING_LIMIT: u16 = 32;

/// The most fix passes a template may need before its fixes settle.
const MAX_PASSES: usize = 4;

/// Settings that load every rule: the newest Twig version and a name in each disallow list.
fn every_rule() -> Settings {
    let mut settings = Settings { version: "3.30".parse().unwrap_or_default(), ..Settings::default() };
    settings.rules.disallowed_functions.config.names = vec!["source".into()];
    settings.rules.disallowed_filters.config.names = vec!["unsafe_filter".into()];
    settings.rules.disallowed_tags.config.names = vec!["flush".into()];
    settings
}

/// Lints `source` with every rule and applies the fixes until none remain.
///
/// Returns the number of passes that changed the template, or `None` when `source` does not parse.
///
/// # Panics
///
/// Panics if a fix produces a template that does not parse, or if the fixes do not settle.
pub fn assert_fixes_settle(source: &[u8]) -> Option<usize> {
    let settings = every_rule();
    let mut current = source.to_vec();
    for pass in 0..MAX_PASSES {
        let arena = LocalArena::new();
        let file = File::ephemeral(Cow::Borrowed(b"fixed.twig"), Cow::Owned(current.clone()));
        let template = Parser::new(&arena, file.id, &file.contents, ParserSettings::default())
            .with_recursion_limit(NESTING_LIMIT)
            .parse(&file.contents, file.id);
        if template.has_errors() {
            assert!(
                pass == 0,
                "Fix pass {pass} produced a template that does not parse:\n{}\n{:?}",
                String::from_utf8_lossy(&current),
                template.errors
            );

            return None;
        }

        let issues = Linter::new(&arena, &settings, None, true).lint(&file, template);
        for issue in issues.iter().filter(|issue| issue.code.as_deref().is_some_and(|code| code.starts_with("twig-"))) {
            let code = issue.code.as_deref().unwrap_or_default();
            assert!(
                issue.annotations.iter().all(|annotation| annotation.message.is_some()),
                "Issue '{code}' has an annotation without a message"
            );
            assert!(!issue.notes.is_empty() && issue.help.is_some(), "Issue '{code}' lacks a note or help");
        }

        let mut editor = TextEditor::new(&current);
        for batches in issues.to_edit_batches().into_values() {
            for (_, edits) in batches {
                editor.apply_unique_batch(edits, None::<fn(&[u8]) -> bool>);
            }
        }

        let fixed = editor.finish();
        if fixed == current {
            return Some(pass);
        }

        current = fixed;
    }

    panic!("Fixes did not settle after {MAX_PASSES} passes:\n{}", String::from_utf8_lossy(&current));
}
