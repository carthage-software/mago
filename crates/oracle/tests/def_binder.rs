#![allow(clippy::expect_used)]

use std::borrow::Cow;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_oracle::definition::DefinitionTable;
use mago_oracle::definition::binder::bind;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::part::origin::Origin;
use mago_syntax::parser::parse_file;

/// Parse and lower `code` to an untyped IR, fold it into a symbol-keyed IR plus a
/// definition table, and hand both to `check`. Everything stays in the local
/// arena; only the assertions escape.
fn with_bind<'arena>(
    arena: &'arena LocalArena,
    code: &str,
    check: impl FnOnce(&IR<'arena, SymbolId, (), ()>, &DefinitionTable<'arena, LocalArena, (), ()>),
) {
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"def_binder.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'arena, (), (), ()> = Lowering::new(arena, &scratch, &file, program, LowerSettings::default()).lower();
    let ir = arena.alloc(ir);

    let (typed, table) = bind(arena, Origin::Project, ir);

    check(&typed, &table);
}

#[test]
fn indexes_each_top_level_definition_kind() {
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

    let arena = LocalArena::new();
    with_bind(&arena, CODE, |_ir, table| {
        assert_eq!(table.constants.len(), 1, "one top-level constant");
        assert_eq!(table.functions.len(), 1, "one top-level function");
        assert_eq!(table.classes.len(), 1, "one named class");
        assert_eq!(table.interfaces.len(), 1, "one interface");
        assert_eq!(table.traits.len(), 1, "one trait");
        assert_eq!(table.enums.len(), 1, "one enum");
        assert_eq!(table.anonymous_classes.len(), 1, "one anonymous class");
        assert_eq!(table.closures.len(), 1, "one closure");
        assert_eq!(table.arrow_functions.len(), 1, "one arrow function");

        let function = table.functions.get(&SymbolId::function_like(b"top_function")).expect("top_function is indexed");
        assert!(function.name.value.ends_with(b"top_function"));
    });
}

#[test]
fn writes_the_symbol_id_into_the_item_meta_hole() {
    const CODE: &str = "<?php
class Widget {
    const A = 1, B = 2;
    public int $x = 0, $y = 1;
    public function run(): void {}
}
";

    let arena = LocalArena::new();
    with_bind(&arena, CODE, |_ir, table| {
        let class = table.classes.get(&SymbolId::class_like(b"Widget")).expect("Widget is indexed");

        // The flattened multi-declarators each carry their own distinct id.
        let mut member_ids = Vec::new();
        for member in class.members.iter() {
            member_ids.push(member.meta);
        }

        assert_eq!(member_ids.len(), 5, "two consts + two properties + one method");
        let unique: std::collections::HashSet<_> = member_ids.iter().copied().collect();
        assert_eq!(unique.len(), 5, "every flattened declarator gets a distinct symbol id");

        assert!(member_ids.contains(&SymbolId::class_like_constant(b"Widget", b"A")));
        assert!(member_ids.contains(&SymbolId::class_like_constant(b"Widget", b"B")));
        assert!(member_ids.contains(&SymbolId::property(b"Widget", b"$x")));
        assert!(member_ids.contains(&SymbolId::property(b"Widget", b"$y")));
        assert!(member_ids.contains(&SymbolId::method(b"Widget", b"run")));
    });
}

#[test]
fn indexes_definitions_nested_in_bodies() {
    const CODE: &str = "<?php

function outer(): void {
    $inner = function (): object {
        return new class {};
    };
}
";

    let arena = LocalArena::new();
    with_bind(&arena, CODE, |_ir, table| {
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
fn empty_program_indexes_no_definitions() {
    let arena = LocalArena::new();
    with_bind(&arena, "<?php echo 1;", |_ir, table| {
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
