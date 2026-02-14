use std::borrow::Cow;
use std::sync::Arc;

use bumpalo::Bump;
use mago_database::file::File;
use mago_linter::Linter;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;
use mago_text_edit::TextEditor;

fn lint_and_fix(code: &str) -> String {
    let arena = Bump::new();
    let file = File::ephemeral(Cow::Owned("test.php".to_string()), Cow::Owned(code.to_string()));
    let program = parse_file(&arena, &file);

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let settings = Settings::default();
    let registry = RuleRegistry::build(&settings, Some(&["no-trailing-space".to_string()]), true);
    let linter = Linter::from_registry(&arena, Arc::new(registry), settings.php_version);
    let mut issues = linter.lint(&file, program, &resolved_names);

    let mut editor = TextEditor::new(code);
    for (_, edits) in issues.take_edits() {
        for edit in edits {
            editor.apply(edit, None::<fn(&str) -> bool>);
        }
    }

    editor.finish()
}

#[test]
fn test_fix_with_crlf_multibyte() {
    let input = "<?php\r\n\r\n/**\r\n * あ \r\n */\r\n";
    let expected = "<?php\r\n\r\n/**\r\n * あ\r\n */\r\n";

    let result = lint_and_fix(input);
    assert_eq!(result, expected);
}
