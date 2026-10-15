use std::sync::Arc;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_linter::Linter;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::settings::ParserSettings;

fn lint(source: &str) -> Vec<(String, u32, u32)> {
    let arena = LocalArena::new();
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let settings = ParserSettings { recover_incomplete: true, ..Default::default() };
    let program = parse_file_borrowed_with_settings(&arena, &file, settings);
    let resolved = NameResolver::new(&arena).resolve(program);
    let settings = Settings::default();
    let registry = RuleRegistry::build(&settings, None, true);
    let linter = Linter::from_registry(&arena, Arc::new(registry), settings.php_version);
    linter
        .lint(&file, program, &resolved)
        .into_iter()
        .filter_map(|issue| {
            let primary = issue.annotations.iter().find(|annotation| annotation.is_primary())?.span;
            Some((issue.code?, primary.start.offset, primary.end.offset))
        })
        .collect()
}

fn codes(source: &str) -> Vec<String> {
    lint(source).into_iter().map(|(code, ..)| code).collect()
}

#[test]
fn lints_outside_the_recovered_part_of_a_statement_are_kept() {
    const IF_CHAIN: &str = " {\n        if ($a == 2) {\n            return 2;\n        }\n    } elseif ($a < 0) {\n        foreach ($items as $item) echo $item;\n    } else {\n        return 3;\n    }\n\n    return 1;\n}\n";
    const ELSEIF: &str = " {\n        foreach ($items as $item) echo $item;\n    } else {\n        return 3;\n    }\n\n    return 1;\n}\n";
    const SWITCH: &str = " {\n        case 1:\n            foreach ($items as $item) echo $item;\n            break;\n        default:\n            if ($a == 2) {\n                return 2;\n            }\n    }\n\n    return 1;\n}\n";
    const WHILE: &str = " {\n        if ($a == 2) {\n            return 2;\n        }\n        foreach ($items as $item) echo $item;\n        $a--;\n    }\n\n    return 1;\n}\n";
    const ALTERNATIVE_WHILE: &str = ":\n        if ($a == 2) {\n            return 2;\n        }\n        $a--;\n    endwhile;\n\n    return 1;\n}\n";
    const DO_WHILE: &str = ";\n\n    return 1;\n}\n";
    const LATER_CLAUSE: &str = "\n\n    return 1;\n}\n";
    const FUNCTION: &str = "<?php\n\nfunction run(int $a, array $items): int\n{\n    ";
    const TEMPLATE: &str = "<?php declare(strict_types=1); ?>\n<ul>\n<?php foreach ($items as $item): ?>\n  <?php ";
    let branches =
        format!("{FUNCTION}if ($a > 5) {{\n        if ($a == 6) {{\n            return 6;\n        }}\n    }} ");
    let do_body = format!(
        "{FUNCTION}do {{\n        if ($a == 4) {{\n            return 4;\n        }}\n        foreach ($items as $item) echo $item;\n    }} "
    );
    let completed =
        format!("{FUNCTION}if ($a > 5) {{\n        echo 5;\n    }} elseif ($a == 6) {{\n        echo 6;\n    }}");
    for (before, head, complete, edited, after, kept) in [
        (
            FUNCTION,
            "if ($a > 1",
            ")",
            "",
            IF_CHAIN,
            ["identity-comparison", "block-statement", "no-else-clause"].as_slice(),
        ),
        (
            branches.as_str(),
            "elseif ($a < 0",
            ")",
            "",
            ELSEIF,
            ["identity-comparison", "block-statement", "no-else-clause"].as_slice(),
        ),
        (FUNCTION, "switch ($a", ")", "", SWITCH, ["block-statement", "identity-comparison"].as_slice()),
        (FUNCTION, "while ($a > 1", ")", "", WHILE, ["identity-comparison", "block-statement"].as_slice()),
        (
            FUNCTION,
            "while ($a > 1",
            ")",
            "",
            ALTERNATIVE_WHILE,
            ["identity-comparison", "no-alternative-syntax"].as_slice(),
        ),
        (
            TEMPLATE,
            "if ($item->visible",
            ")",
            "",
            ": ?>\n    <li><?php echo $item == 1 ? 'a' : 'b'; ?></li>\n    <?php foreach ($item->children as $child) echo $child; ?>\n  <?php endif; ?>\n<?php endforeach; ?>\n</ul>\n",
            ["identity-comparison", "block-statement"].as_slice(),
        ),
        (do_body.as_str(), "while ($a > 1", ")", "", DO_WHILE, ["identity-comparison", "block-statement"].as_slice()),
        (branches.as_str(), "", "", "elseif ($a > 6", LATER_CLAUSE, ["identity-comparison"].as_slice()),
        (completed.as_str(), "", "", " else", "\n}\n", ["identity-comparison", "no-else-clause"].as_slice()),
    ] {
        let complete_source = format!("{before}{head}{complete}{after}");
        let edited_source = format!("{before}{head}{edited}{after}");
        let start = before.len() as u32;
        let at = start + head.len() as u32;
        let (complete_length, edited_length) = (complete.len() as u32, edited.len() as u32);
        let shift = |offset: u32| match offset {
            offset if offset <= at => Some(offset),
            offset if offset >= at + edited_length => Some(offset - edited_length + complete_length),
            _ => None,
        };
        let mut suppressed = lint(&complete_source);
        let published = lint(&edited_source);
        for (code, issue_start, issue_end) in &published {
            let Some(issue) = shift(*issue_start).zip(shift(*issue_end)).map(|(start, end)| (code.clone(), start, end))
            else {
                panic!("{edited_source}: {code} at {issue_start}..{issue_end}");
            };
            let Some(index) = suppressed.iter().position(|expected| *expected == issue) else {
                panic!("{edited_source}: {code} at {issue_start}..{issue_end}");
            };
            suppressed.swap_remove(index);
        }
        for (code, issue_start, issue_end) in &suppressed {
            assert!(*issue_end > start && *issue_start < at + complete_length, "{edited_source}: lost {code}");
        }
        for code in kept {
            assert!(published.iter().any(|(published, ..)| published == code), "{edited_source}: missing {code}");
        }
    }
}

#[test]
fn statements_completed_by_editor_recovery_are_not_linted() {
    let unclosed = [
        "if ($a",
        "if (true",
        "if ($a) {} elseif ($a",
        "if ($this->",
        "while ($a",
        "while ($kind === Kind::Alpha",
        "while ($this->",
        "do {} while ($a",
        "switch ($a",
    ];
    let bodiless =
        ["if ($a)", "if ($a) {} else", "while ($a)", "foreach ($items as $item)", "for ($i = 0; $i < 1; $i++)"];
    for (heads, tails) in [
        (unclosed.as_slice(), ["\n        return 1;\n    }\n}\n", "\n    }\n}\n", ""].as_slice()),
        (bodiless.as_slice(), ["\n    }\n}\n", ""].as_slice()),
    ] {
        for (head, tail) in heads.iter().flat_map(|head| tails.iter().map(move |tail| (*head, *tail))) {
            let method = |head: &str| {
                format!(
                    "<?php\n\nenum Kind\n{{\n    case Alpha;\n}}\n\nfinal class C\n{{\n    public function run(int $a, Kind $kind, array $items): int\n    {{\n        {head}{tail}"
                )
            };
            let script = |head: &str| format!("<?php\n\nfinal class D\n{{\n}}\n\n{head}");
            for (source, reference) in [(method(head), method("")), (script(head), script(""))] {
                let mut expected = codes(&reference);
                for code in codes(&source) {
                    let Some(index) = expected.iter().position(|expected| *expected == code) else {
                        panic!("{source}: {code}");
                    };
                    expected.swap_remove(index);
                }
            }
        }
    }
}
