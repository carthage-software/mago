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
fn ternary_unions_both_branches() {
    assert_eq!(ty(b"<?php /** @var 1|2 */ $a = 1; $a > 0 ? 'yes' : 'no';"), "string('no')|string('yes')");
}

#[test]
fn ternary_narrows_branch_environments() {
    assert_eq!(ty(b"<?php /** @var 1|2 */ $a = 1; $a === 1 ? $a : $a;"), "int(1)|int(2)");
}

#[test]
fn ternary_arithmetic_distributes_over_narrowed_union() {
    assert_eq!(ty(b"<?php /** @var 1|2 */ $a = 1; ($a === 1 || $a === 2) ? ($a + 1) : 99;"), "int(2)|int(3)");
}

#[test]
fn ternary_with_always_true_condition_kills_else() {
    assert_eq!(
        ty(b"<?php /** @var 1|2 */ $a = 1; $c = ($a === 1 || ($b = 3)) ? ($a + 1) : ($a + $b); $c;"),
        "int(2)|int(3)"
    );
}

#[test]
fn ternary_with_reachable_else_unions_all_outcomes() {
    assert_eq!(
        ty(b"<?php /** @var 1|2 */ $a = 1; /** @var bool */ $x = true; ($a === 1 || $x) ? ($a + 1) : 99;"),
        "int(2)|int(3)|int(99)"
    );
}

#[test]
fn elvis_drops_falsy_from_condition() {
    assert_eq!(
        ty(b"<?php /** @var string|null */ $a = null; $a ?: 'default';"),
        "(string&!string('')&!string('0'))|string('default')"
    );
}
