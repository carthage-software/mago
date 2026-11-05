#![allow(clippy::expect_used)]

use std::borrow::Cow;
use std::sync::LazyLock;

use mago_allocator::LocalArena;
use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
use mago_codex::populator::populate_codebase;
use mago_codex::scanner::scan_program;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_prelude::Prelude;
use mago_syntax::parser::parse_file;
use mago_word::WordSet;

static PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);

const SOURCE: &str = "<?php

declare(strict_types=1);

function save(int $attempt): void
{
    if ($attempt === 1) {
        throw new RuntimeException('first');
    }

    if ($attempt === 2) {
        throw new RuntimeException('second');
    }

    if ($attempt === 3) {
        throw new RuntimeException('third');
    }

    if ($attempt === 4) {
        throw new RuntimeException('fourth');
    }

    if ($attempt === 5) {
        throw new RuntimeException('fifth');
    }

    throw new RuntimeException('last');
}
";

/// The throw sites of an unhandled exception must be reported in source order, since
/// `@mago-expect` and `@mago-ignore` pragmas are matched against the first primary span.
///
/// See <https://github.com/carthage-software/mago/issues/2660>.
#[test]
fn unhandled_thrown_type_reports_throw_sites_in_source_order() {
    let file = File::ephemeral(Cow::Borrowed(b"throw-order.php"), Cow::Borrowed(SOURCE.as_bytes()));
    let arena = LocalArena::new();
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "the fixture should parse");

    let resolved_names = NameResolver::new(&arena).resolve(program);
    let settings = Settings { check_throws: true, ..Default::default() };
    let mut codebase = PRELUDE.metadata.clone();
    codebase.extend(scan_program(&arena, &file, program, &resolved_names, settings.version));
    let mut symbol_references = PRELUDE.symbol_references.clone();
    populate_codebase(&mut codebase, &mut symbol_references, WordSet::default(), foldhash::HashSet::default());

    let registry = PluginRegistry::with_library_providers();
    let analyzer = Analyzer::new(&arena, &file, &resolved_names, &codebase, &registry, settings);
    let mut result = AnalysisResult::new(symbol_references);
    analyzer.analyze(program, &mut result).expect("analysis failed");

    let issue = result
        .issues
        .iter()
        .find(|issue| issue.code.as_deref() == Some("unhandled-thrown-type"))
        .expect("expected an unhandled-thrown-type issue");

    let offsets = issue
        .annotations
        .iter()
        .filter(|annotation| annotation.kind.is_primary())
        .map(|annotation| annotation.span.start.offset)
        .collect::<Vec<_>>();

    assert_eq!(offsets.len(), 6, "expected one primary annotation per throw site");
    assert!(offsets.is_sorted(), "throw sites are not in source order: {offsets:?}");
}
