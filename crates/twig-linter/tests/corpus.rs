#![allow(clippy::unwrap_used)]

use std::path::Path;

#[path = "support/fixes.rs"]
mod fixes;

#[derive(Default)]
struct Tally {
    parsed: usize,
    fixed: usize,
}

impl Tally {
    fn check(&mut self, source: &[u8]) {
        if let Some(passes) = fixes::assert_fixes_settle(source) {
            self.parsed += 1;
            self.fixed += usize::from(passes > 0);
        }
    }
}

fn visit(directory: &Path, tally: &mut Tally) {
    let mut entries = std::fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().path()).collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            visit(&path, tally);
            continue;
        }

        if path.extension().is_none_or(|extension| extension != "test") {
            tally.check(&std::fs::read(&path).unwrap());
            continue;
        }

        let source = std::fs::read_to_string(&path).unwrap();
        let mut in_template = false;
        let mut template = String::new();
        for line in source.split_inclusive('\n') {
            let marker = line.trim_end_matches(['\r', '\n']);
            if marker.starts_with("--") && marker.ends_with("--") {
                if in_template {
                    tally.check(template.as_bytes());
                    template.clear();
                }

                in_template = marker == "--TEMPLATE--" || marker.starts_with("--TEMPLATE(");
            } else if in_template {
                template.push_str(line);
            }
        }

        if in_template {
            tally.check(template.as_bytes());
        }
    }
}

#[test]
fn every_rule_and_fix_holds_across_the_twig_corpus() {
    std::thread::Builder::new().stack_size(4 * 1024 * 1024).spawn(check_corpus).unwrap().join().unwrap();
}

fn check_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut tally = Tally::default();
    for directory in [
        "../twig-syntax/tests/fixtures",
        "../twig-syntax/fuzz/seeds/parser",
        "../twig-formatter/tests/cases",
        "../twig-formatter/tests/runtime/cases",
        "tests/fixtures",
        "tests/runtime/cases",
    ] {
        visit(&root.join(directory), &mut tally);
    }

    assert!(tally.parsed > 1100, "Expected the full Twig corpus, checked {} templates", tally.parsed);
    assert!(tally.fixed > 500, "Expected fixes across the corpus, fixed {} templates", tally.fixed);
}
