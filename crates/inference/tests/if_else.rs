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

/// Infers the type of the *last* expression statement, so a snippet can run an
/// `if`/`elseif`/`else` and then read a variable to observe the joined type.
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
fn join_unions_both_branch_assignments() {
    assert_eq!(
        ty(b"<?php /** @var bool */ $c = true; if ($c) { $x = 1; } else { $x = 'a'; } $x;"),
        "int(1)|string('a')"
    );
}

#[test]
fn narrows_then_branch_environment() {
    assert_eq!(
        ty(b"<?php /** @var string|null */ $a = null; if ($a === null) { $b = $a; } else { $b = $a; } $b;"),
        "null|string"
    );
}

#[test]
fn returning_then_branch_leaves_else_type() {
    assert_eq!(ty(b"<?php /** @var int|null */ $a = null; if ($a === null) { return; } $a;"), "int");
}

#[test]
fn elseif_chain_narrows_each_arm() {
    assert_eq!(
        ty(b"<?php /** @var 1|2|3 */ $a = 1; if ($a === 1) { $b = 'one'; } elseif ($a === 2) { $b = 'two'; } else { $b = 'rest'; } $b;"),
        "string('one')|string('rest')|string('two')"
    );
}

#[test]
fn no_else_unions_modified_with_passthrough() {
    assert_eq!(ty(b"<?php $x = 'a'; /** @var bool */ $c = true; if ($c) { $x = 1; } $x;"), "int(1)|string('a')");
}

#[test]
fn always_true_condition_takes_only_then() {
    assert_eq!(ty(b"<?php $x = 'a'; if (true) { $x = 1; } else { $x = 2; } $x;"), "int(1)");
}

#[test]
fn exhaustive_elseif_chain_proves_else_unreachable() {
    assert_eq!(
        ty(
            b"<?php /** @var bool */ $a = true; if ($a === true) { $x = 1; } elseif ($a === false) { $x = 2; } else { $x = 'dead'; } $x;"
        ),
        "int(1)|int(2)"
    );
}
