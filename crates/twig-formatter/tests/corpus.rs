#![allow(clippy::unwrap_used, clippy::use_debug)]

use std::borrow::Cow;
use std::path::Path;

use mago_allocator::LocalArena;
use mago_database::file::FileId;
use mago_twig_formatter::Formatter;
use mago_twig_formatter::settings::FormatSettings;
use mago_twig_syntax::parser::parse_file_content;

mod support;

fn check(source: &[u8], name: &Path) -> bool {
    let arena = LocalArena::new();
    let template = parse_file_content(&arena, FileId::zero(), source);
    if template.has_errors() {
        return false;
    }

    for width in [30, 120] {
        let formatter = Formatter::new(&arena, FormatSettings { print_width: width, ..FormatSettings::default() });
        let output = formatter.format(template).unwrap();
        support::assert_preserves_syntax(source, output);
        let again = formatter.format_code(Cow::Borrowed(b"corpus.twig"), Cow::Owned(output.to_vec())).unwrap();
        assert_eq!(
            output,
            again,
            "Unstable formatting: {} (width {width})\n{}",
            name.display(),
            String::from_utf8_lossy(output)
        );
    }

    true
}

fn visit(directory: &Path, checked: &mut usize) {
    let mut entries = std::fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().path()).collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            visit(&path, checked);
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "twig") {
            *checked += usize::from(check(&std::fs::read(&path).unwrap(), &path));
            continue;
        }

        if path.extension().is_none_or(|extension| extension != "test") {
            continue;
        }

        let source = std::fs::read_to_string(&path).unwrap();
        let expects_error = source.contains("--EXCEPTION--");
        let mut in_template = false;
        let mut template = String::new();
        for line in source.split_inclusive('\n') {
            let marker = line.trim_end_matches(['\r', '\n']);
            if marker.starts_with("--") && marker.ends_with("--") {
                if in_template {
                    let accepted = check(template.as_bytes(), &path);
                    assert!(accepted || expects_error, "Rejected valid Twig fixture: {}", path.display());
                    *checked += usize::from(accepted);
                    template.clear();
                }

                in_template = marker == "--TEMPLATE--" || marker.starts_with("--TEMPLATE(");
            } else if in_template {
                template.push_str(line);
            }
        }

        if in_template {
            let accepted = check(template.as_bytes(), &path);
            assert!(accepted || expects_error, "Rejected valid Twig fixture: {}", path.display());
            *checked += usize::from(accepted);
        }
    }
}

#[test]
#[allow(clippy::print_stderr)]
fn preserves_syntax_and_text_across_twig_corpus() {
    let mut checked = 0;
    visit(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../twig-syntax/tests/fixtures"), &mut checked);
    assert!(checked > 500, "Expected the full Twig corpus, checked {checked} templates");
    eprintln!("Checked {checked} templates from the bundled Twig corpus at widths 30 and 120.");
    if let Some(paths) = std::env::var_os("MAGO_TWIG_CORPUS") {
        for path in std::env::split_paths(&paths) {
            let mut checked = 0;
            visit(&path, &mut checked);
            eprintln!("Checked {checked} parsed templates from {}.", path.display());
        }
    }
}
