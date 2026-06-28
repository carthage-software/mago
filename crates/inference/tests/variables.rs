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

/// Infers the type of the *last* expression statement, so multi-statement
/// snippets can build up an environment before the assertion.
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
    let expr = typed
        .statements
        .iter()
        .filter_map(|s| if let StatementKind::Expression(e) = &s.kind { Some(e) } else { None })
        .next_back()
        .expect("expression statement");
    format!("{}", expr.meta)
}

#[test]
fn variable_read_write() {
    assert_eq!(ty(b"<?php $a = 1; $a;"), "int(1)");
    assert_eq!(ty(b"<?php $a = 1; $b = 123; $a + $b;"), "int(124)");
    assert_eq!(ty(b"<?php $a = 'x'; $b = 'y'; $a . $b;"), "string('xy')");
    assert_eq!(ty(b"<?php $a = 1; $a = 'two'; $a;"), "string('two')");
    assert_eq!(ty(b"<?php $undefined;"), "mixed");
}

#[test]
fn assignment_in_short_circuit_is_conditional() {
    // `$b = 5` runs only when `$cond` is falsy, so afterwards `$b` is its old
    // value unioned with the conditional assignment, not clobbered to `int(5)`.
    assert_eq!(ty(b"<?php $b = 'x'; $cond || ($b = 5); $b;"), "int(5)|string('x')");
}

#[test]
fn increment_decrement() {
    assert_eq!(ty(b"<?php $i = 5; ++$i;"), "int(6)");
    assert_eq!(ty(b"<?php $i = 5; --$i;"), "int(4)");
    assert_eq!(ty(b"<?php $i = 5; $i++;"), "int(5)");
    assert_eq!(ty(b"<?php $i = 5; $i++; $i;"), "int(6)");
    assert_eq!(ty(b"<?php $i = 5; $i--; $i;"), "int(4)");
    assert_eq!(ty(b"<?php $f = 1.5; ++$f;"), "float(2.5)");
    assert_eq!(ty(b"<?php $n = null; ++$n;"), "int(1)");
    assert_eq!(ty(b"<?php $n = null; --$n;"), "null");
    assert_eq!(ty(b"<?php $s = '9'; ++$s;"), "int(10)");
}

#[test]
fn string_increment() {
    assert_eq!(ty(b"<?php $s = 'a'; ++$s;"), "string('b')");
    assert_eq!(ty(b"<?php $s = 'Az'; ++$s;"), "string('Ba')");
    assert_eq!(ty(b"<?php $s = 'Zz'; ++$s;"), "string('AAa')");
    assert_eq!(ty(b"<?php $s = 'a9'; ++$s;"), "string('b0')");
    assert_eq!(ty(b"<?php $s = 'abc'; --$s;"), "string('abc')");
}

#[test]
fn nested_casts() {
    assert_eq!(ty(b"<?php (int) (string) (int) (string) (1 + 2);"), "int(3)");
}

#[test]
fn never_propagation() {
    assert_eq!(ty(b"<?php 'abc' + 1;"), "never");
    assert_eq!(ty(b"<?php [1] - 2;"), "never");
    assert_eq!(ty(b"<?php [1] + 1;"), "never");
    assert_eq!(ty(b"<?php [1] * 2;"), "never");
    assert_eq!(ty(b"<?php ('abc' + 1) + 2;"), "never");
    assert_eq!(ty(b"<?php (int) ('abc' + 1);"), "never");
    assert_eq!(ty(b"<?php ['abc' + 1];"), "never");
    assert_eq!(ty(b"<?php match ('abc' + 1) { 1 => 'x' };"), "never");
}

#[test]
fn array_addition() {
    assert_eq!(ty(b"<?php [1] + [2];"), "list{0: int(1)}");
    assert_eq!(ty(b"<?php [] + [1];"), "list{0: int(1)}");
    assert_eq!(ty(b"<?php [1, 2] + [3, 4, 5];"), "list{0: int(1), 1: int(2), 2: int(5)}");
    assert_eq!(ty(b"<?php ['a' => 1] + ['b' => 2];"), "array{'a': int(1), 'b': int(2)}");
    assert_eq!(
        ty(b"<?php ['a' => 1, 'b' => 2] + ['b' => 3, 'c' => 4];"),
        "array{'a': int(1), 'b': int(2), 'c': int(4)}"
    );
}
