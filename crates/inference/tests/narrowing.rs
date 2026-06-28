use std::borrow::Cow;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::statement::StatementKind;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_inference::Inference;
use mago_oracle::definition::binder::bind;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::symbol::part::origin::Origin;
use mago_syntax::parser::parse_file;

/// The type of the right operand of the last (binary) expression statement.
fn second_operand(src: &[u8]) -> String {
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
    let expr = typed
        .statements
        .iter()
        .filter_map(|s| if let StatementKind::Expression(e) = &s.kind { Some(e) } else { None })
        .next_back()
        .expect("expression statement");

    match &expr.kind {
        ExpressionKind::Binary(binary) => format!("{}", binary.right.meta),
        _ => panic!("expected a binary expression"),
    }
}

/// The type of the last expression statement.
fn result_type(src: &[u8]) -> String {
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
fn tdd_folds_multi_variable_contradiction() {
    // `(a∨b) ∧ (¬a∧¬b)` is unsatisfiable across two variables — the per-variable
    // narrowing cannot see it (the disjunctive `a∨b` narrows nothing), but the
    // decision diagram collapses it to `false`.
    assert_eq!(result_type(b"<?php ($a || $b) && (!$a && !$b);"), "false");
}

#[test]
fn tdd_folds_single_variable_tautology() {
    assert_eq!(result_type(b"<?php $a || !$a;"), "true");
}

#[test]
fn or_narrows_right_operand_when_left_is_false() {
    // $a is string|null. The right `$a === null` is reached only when the left
    // was false, i.e. `$a !== null` ⇒ `$a` is `string`, so `string === null`
    // is always false.
    assert_eq!(second_operand(b"<?php /** @var string|null */ $a = null; $a === null || $a === null;"), "false");
}

#[test]
fn and_narrows_right_operand_when_left_is_true() {
    // The right `$a === null` is reached only when the left was true, so `$a`
    // is `null` and `null === null` is true.
    assert_eq!(second_operand(b"<?php /** @var string|null */ $a = null; $a === null && $a === null;"), "true");
}

#[test]
fn disjoint_identical_is_false_without_narrowing() {
    assert_eq!(second_operand(b"<?php $x = 'a'; true && $x === null;"), "false");
}
