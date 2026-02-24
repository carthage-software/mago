use std::borrow::Cow;

use bumpalo::Bump;
use mago_database::file::File;
use mago_linter::Linter;
use mago_linter::integration::IntegrationSet;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::RulesSettings;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;
use mago_syntax::parser::parse_file_content_with_settings;
use mago_syntax::settings::ParserSettings;
use mago_text_edit::Safety;
use mago_text_edit::TextEditor;

/// Lint an ephemeral file and apply fixes using the batched API (same code path
/// as `execute_stdin` in the CLI), returning the fixed source.
fn lint_and_fix_batched(code: &str, safety: Safety, only: Option<&[String]>) -> String {
    let arena = Bump::new();

    let file = File::ephemeral(Cow::Owned("test.php".to_string()), Cow::Owned(code.to_string()));

    let program = parse_file(&arena, &file);
    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let settings =
        Settings { integrations: IntegrationSet::all(), rules: RulesSettings::default(), ..Settings::default() };

    let php_version = settings.php_version;
    let registry = RuleRegistry::build(&settings, only, true);
    let linter = Linter::from_registry(&arena, std::sync::Arc::new(registry), php_version);

    let issues = linter.lint(&file, program, &resolved_names);

    let batches_by_file = issues.to_edit_batches();
    let Some(batches) = batches_by_file.into_iter().find(|(fid, _)| *fid == file.id).map(|(_, b)| b) else {
        return code.to_string();
    };

    let file_id = file.id;
    let parser_settings = ParserSettings::default();
    let check_arena = Bump::new();
    let mut editor = TextEditor::with_safety(code, safety);
    let checker = |c: &str| !parse_file_content_with_settings(&check_arena, file_id, c, parser_settings).has_errors();

    for (_rule_code, edits) in batches {
        let _ = editor.apply_batch(edits, Some(checker));
    }

    editor.finish()
}

#[test]
fn test_fixes_applied_via_batched_api() {
    let input = r#"<?php

foreach ($items as $item) {
    if ($item->isValid()) {
        doSomething($item);
    }
}
"#;

    let only = ["prefer-early-continue".to_string()];
    let result = lint_and_fix_batched(input, Safety::Safe, Some(&only));

    // The fix should have been applied (output differs from input)
    assert_ne!(result, input, "Expected the fixer to transform the code");
    assert!(result.contains("continue"), "Expected an early continue in the output");
}

#[test]
fn test_no_fixable_issues_returns_unchanged() {
    // Clean code with no lint issues for the chosen rule
    let input = "<?php\n\n$x = 1;\n";

    let only = ["prefer-early-continue".to_string()];
    let result = lint_and_fix_batched(input, Safety::Safe, Some(&only));

    assert_eq!(result, input, "Clean code should remain unchanged");
}
