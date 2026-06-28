use std::borrow::Cow;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::statement::StatementKind;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_inference::Inference;
use mago_oracle::definition::binder::bind;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::symbol::part::origin::Origin;
use mago_syntax::parser::parse_file;

/// Infers the type of the *last* expression statement.
fn ty(src: &[u8]) -> String {
    let source = LocalArena::new();
    let dest = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"u.php"), Cow::Owned(src.to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&source, &scratch, &file, program, LowerSettings::default()).lower();
    let ir = source.alloc(ir);
    let (bound, _d) = bind(&source, Origin::Project, ir);
    let symbols = SymbolTable::new_in(&dest);
    let typed = Inference::new(&source, &dest).infer(&symbols, &file, bound);

    typed
        .statements
        .iter()
        .filter_map(|s| if let StatementKind::Expression(e) = &s.kind { Some(e) } else { None })
        .next_back()
        .map(|e| format!("{}", e.meta))
        .expect("expression statement")
}

#[test]
fn unions_results_of_reachable_arms() {
    assert_eq!(
        ty(b"<?php /** @var 1|2|3 */ $a = 1; match ($a) { 1 => 'one', 2 => 'two', 3 => 'three' };"),
        "string('one')|string('three')|string('two')"
    );
}

#[test]
fn narrows_subject_variable_inside_arm() {
    assert_eq!(ty(b"<?php /** @var 1|2 */ $a = 1; match ($a) { 1 => $a, 2 => $a + 1 };"), "int(1)|int(3)");
}

#[test]
fn unreachable_arm_is_excluded_from_union() {
    assert_eq!(
        ty(b"<?php /** @var 1|2 */ $a = 1; match ($a) { 1 => 'one', 2 => 'two', 3 => 'never' };"),
        "string('one')|string('two')"
    );
}

#[test]
fn exhausted_subject_makes_default_unreachable() {
    assert_eq!(
        ty(b"<?php /** @var 1|2 */ $a = 1; match ($a) { 1 => 'one', 2 => 'two', default => 'd' };"),
        "string('one')|string('two')"
    );
}

#[test]
fn non_exhaustive_without_default_drops_unmatched_path() {
    assert_eq!(ty(b"<?php /** @var int */ $a = 1; match ($a) { 1 => 'one' };"), "string('one')");
}

#[test]
fn default_handles_the_unmatched_remainder() {
    assert_eq!(
        ty(b"<?php /** @var int */ $a = 1; match ($a) { 1 => 'one', default => 'rest' };"),
        "string('one')|string('rest')"
    );
}

#[test]
fn arm_assignment_joins_after_match() {
    assert_eq!(
        ty(b"<?php /** @var 1|2 */ $a = 1; match ($a) { 1 => $b = 'x', 2 => $b = 'y' }; $b;"),
        "string('x')|string('y')"
    );
}
