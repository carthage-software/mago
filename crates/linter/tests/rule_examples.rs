use std::borrow::Cow;

use bumpalo::Bump;
use mago_atom::AtomSet;
use mago_atom::atom;
use mago_database::file::File;
use mago_linter::Linter;
use mago_linter::integration::IntegrationSet;
use mago_linter::registry::RuleRegistry;
use mago_linter::rule::DisallowedFunctionsConfig;
use mago_linter::settings::RuleSettings;
use mago_linter::settings::RulesSettings;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

#[test]
fn test_all_rule_examples() {
    let settings = Settings::default();
    let registry = RuleRegistry::build(&settings, None, true);
    let rules = registry.rules();

    let mut failures = Vec::new();

    for rule in rules {
        let rule_code = rule.code();
        let rule_meta = rule.meta();

        let bad_result = test_code_snippet(rule_code, rule_meta.bad_example, true);
        if let Err(e) = bad_result {
            failures.push(format!("Rule '{}': Bad example issue - {}", rule_code, e));
        }

        let good_result = test_code_snippet(rule_code, rule_meta.good_example, false);
        if let Err(e) = good_result {
            failures.push(format!("Rule '{}': Good example issue - {}", rule_code, e));
        }
    }

    if !failures.is_empty() {
        panic!("\n\n{} rule example(s) failed:\n\n{}\n\n", failures.len(), failures.join("\n"));
    }
}

/// Test a code snippet and verify it produces (or doesn't produce) issues
fn test_code_snippet(rule_code: &str, code: &str, should_have_issues: bool) -> Result<(), String> {
    let arena = Bump::new();

    let file = File::ephemeral(Cow::Owned("test.php".to_string()), Cow::Owned(code.to_string()));

    let (program, parse_error) = parse_file(&arena, &file);

    if let Some(err) = parse_error {
        return Err(format!("Parse error: {:?}", err));
    }

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let settings = Settings {
        integrations: IntegrationSet::all(),
        rules: RulesSettings {
            disallowed_functions: RuleSettings {
                config: DisallowedFunctionsConfig {
                    extensions: AtomSet::from_iter([atom("curl")]),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..RulesSettings::default()
        },
        ..Settings::default()
    };

    let php_version = settings.php_version;
    let registry = RuleRegistry::build(&settings, Some(&[rule_code.to_string()]), true);
    if registry.rules().is_empty() {
        return Err(format!("No rules found for code '{}'", rule_code));
    }

    let linter = Linter::from_registry(&arena, std::sync::Arc::new(registry), php_version);

    let issues = linter.lint(&file, program, &resolved_names);

    let has_issues = !issues.is_empty();

    if should_have_issues && !has_issues {
        return Err("Expected bad example to produce issues, but none were found.".to_string());
    }

    if !should_have_issues && has_issues {
        return Err(format!("Expected good example to NOT produce issues, but found {} issue(s).", issues.len(),));
    }

    Ok(())
}
