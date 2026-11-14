use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_twig_linter::Linter;
use mago_twig_linter::registry::RuleRegistry;
use mago_twig_linter::settings::Settings;
use mago_twig_syntax::parser::Parser;
use mago_twig_syntax::settings::ParserSettings;

#[test]
fn test_all_rule_examples() {
    let settings = example_settings();
    let registry = RuleRegistry::build(&settings, None, true);
    assert_eq!(registry.len(), 25, "every rule is loaded for the newest Twig version");

    let mut failures = Vec::new();
    for rule in registry.rules() {
        let rule_code = rule.code();
        let rule_meta = rule.meta();

        if let Err(e) = test_code_snippet(rule_code, rule_meta.bad_example, true) {
            failures.push(format!("Rule '{rule_code}': Bad example issue - {e}"));
        }

        if let Err(e) = test_code_snippet(rule_code, rule_meta.good_example, false) {
            failures.push(format!("Rule '{rule_code}': Good example issue - {e}"));
        }
    }

    assert!(failures.is_empty(), "\n\n{} rule example(s) failed:\n\n{}\n\n", failures.len(), failures.join("\n"));
}

fn example_settings() -> Settings {
    let mut settings = Settings { version: "3.30".parse().unwrap_or_default(), ..Settings::default() };
    settings.rules.disallowed_functions.config.names = vec!["source".into()];
    settings.rules.disallowed_filters.config.names = vec!["raw".into()];
    settings.rules.disallowed_tags.config.names = vec!["flush".into()];
    settings
}

/// Test a code snippet and verify it produces (or doesn't produce) issues.
fn test_code_snippet(rule_code: &str, code: &str, should_have_issues: bool) -> Result<(), String> {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"example.twig"), Cow::Owned(code.as_bytes().to_vec()));
    let template =
        Parser::new(&arena, file.id, &file.contents, ParserSettings::default()).parse(&file.contents, file.id);
    if template.has_errors() {
        return Err("Failed to parse code snippet.".to_string());
    }

    let only = [rule_code.to_string()];
    let linter = Linter::new(&arena, &example_settings(), Some(&only), false);
    let issues = linter.lint(&file, template);
    let has_issues = issues.iter().any(|issue| issue.code.as_deref() == Some(rule_code));

    match (should_have_issues, has_issues) {
        (true, false) => Err("Expected issues but found none.".to_string()),
        (false, true) => Err(format!("Expected no issues but found {}.", issues.len())),
        _ => Ok(()),
    }
}
