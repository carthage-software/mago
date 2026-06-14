use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_allocator::copy::CopyInto;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_oracle::def::DefinitionTable;
use mago_oracle::def::scanner::scan_definitions;
use mago_syntax::parser::parse_file;

/// Parse and lower `code` to an untyped IR, scan it for definitions, and hand
/// the resulting table to `check`. The arena (output), scratch (parse/lower
/// scratch), and IR all stay local; only the assertions escape.
fn with_scan(code: &str, check: impl FnOnce(&DefinitionTable<'_, (), (), ()>)) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"def_scanner.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();

    let table = scan_definitions(&arena, &ir);
    check(&table);
}

#[test]
fn collects_each_top_level_definition_kind() {
    const CODE: &str = "<?php

const TOP_CONSTANT = 1;

function top_function(): void {}

class TopClass {}

interface TopInterface {}

trait TopTrait {}

enum TopEnum {}

$assigned_closure = function (): int { return 1; };
$assigned_arrow = fn (): int => 2;
$assigned_anonymous = new class {};
";

    with_scan(CODE, |table| {
        assert_eq!(table.constants.len(), 1, "one top-level constant");
        assert_eq!(table.functions.len(), 1, "one top-level function");
        assert_eq!(table.classes.len(), 1, "one named class");
        assert_eq!(table.interfaces.len(), 1, "one interface");
        assert_eq!(table.traits.len(), 1, "one trait");
        assert_eq!(table.enums.len(), 1, "one enum");
        assert_eq!(table.anonymous_classes.len(), 1, "one anonymous class");
        assert_eq!(table.closures.len(), 1, "one closure");
        assert_eq!(table.arrow_functions.len(), 1, "one arrow function");

        assert!(
            table.functions[0].name.value.ends_with(b"top_function"),
            "the collected function must be `top_function`, got {:?}",
            String::from_utf8_lossy(table.functions[0].name.value),
        );
    });
}

#[test]
fn collects_definitions_nested_in_bodies() {
    const CODE: &str = "<?php

function outer(): void {
    $inner = function (): object {
        return new class {};
    };
}
";

    with_scan(CODE, |table| {
        assert_eq!(table.functions.len(), 1, "the outer function");
        assert_eq!(table.closures.len(), 1, "the closure nested in the function body");
        assert_eq!(table.anonymous_classes.len(), 1, "the anonymous class nested in the closure body");

        assert_eq!(table.constants.len(), 0);
        assert_eq!(table.classes.len(), 0);
        assert_eq!(table.interfaces.len(), 0);
        assert_eq!(table.traits.len(), 0);
        assert_eq!(table.enums.len(), 0);
        assert_eq!(table.arrow_functions.len(), 0);
    });
}

#[test]
fn definition_table_copy_into_outlives_the_source_arena() {
    const CODE: &str = "<?php

const COPIED_CONSTANT = 1;

function copied_function(): void {}

class CopiedClass {}
";

    let target = LocalArena::new();

    let copied = {
        let arena = LocalArena::new();
        let scratch = LocalArena::new();
        let file = File::ephemeral(Cow::Borrowed(b"copy.php"), Cow::Owned(CODE.as_bytes().to_vec()));
        let program = parse_file(&scratch, &file);
        let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();

        scan_definitions(&arena, &ir).copy_into(&target)
    };

    assert_eq!(copied.constants.len(), 1);
    assert_eq!(copied.functions.len(), 1);
    assert_eq!(copied.classes.len(), 1);
    assert!(
        copied.functions[0].name.value.ends_with(b"copied_function"),
        "the deep-copied function name must survive the source arena, got {:?}",
        String::from_utf8_lossy(copied.functions[0].name.value),
    );
}

#[test]
fn empty_program_collects_no_definitions() {
    with_scan("<?php echo 1;", |table| {
        assert_eq!(table.constants.len(), 0);
        assert_eq!(table.functions.len(), 0);
        assert_eq!(table.classes.len(), 0);
        assert_eq!(table.interfaces.len(), 0);
        assert_eq!(table.traits.len(), 0);
        assert_eq!(table.enums.len(), 0);
        assert_eq!(table.anonymous_classes.len(), 0);
        assert_eq!(table.closures.len(), 0);
        assert_eq!(table.arrow_functions.len(), 0);
    });
}
