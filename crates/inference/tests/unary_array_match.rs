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

fn ty(src: &[u8]) -> String {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"u.php"), Cow::Owned(src.to_vec()));
    let program = parse_file(&arena, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &arena, &file, program, LowerSettings::default()).lower();
    let ir = arena.alloc(ir);
    let (bound, _d) = bind(&arena, Origin::Project, ir);
    let symbols = SymbolTable::new_in(&arena);
    let typed = Inference::new(&arena, &arena).infer(&symbols, &file, bound);
    let expr = typed
        .statements
        .iter()
        .rev()
        .find_map(|s| if let StatementKind::Expression(e) = &s.kind { Some(e) } else { None })
        .expect("expression statement");
    format!("{}", expr.meta)
}

#[test]
fn unary_prefix() {
    assert_eq!(ty(b"<?php -5;"), "int(-5)");
    assert_eq!(ty(b"<?php -1.5;"), "float(-1.5)");
    assert_eq!(ty(b"<?php -'5';"), "int(-5)");
    assert_eq!(ty(b"<?php -true;"), "int(-1)");
    assert_eq!(ty(b"<?php +'5';"), "int(5)");
    assert_eq!(ty(b"<?php +'1.5';"), "float(1.5)");
    assert_eq!(ty(b"<?php +true;"), "int(1)");
    assert_eq!(ty(b"<?php !true;"), "false");
    assert_eq!(ty(b"<?php !0;"), "true");
    assert_eq!(ty(b"<?php !'a';"), "false");
    assert_eq!(ty(b"<?php ~5;"), "int(-6)");
    assert_eq!(ty(b"<?php ~1.9;"), "int(-2)");
    assert_eq!(ty(b"<?php 2 ** -1;"), "float(0.5)");
}

#[test]
fn casts() {
    assert_eq!(ty(b"<?php (int) 1.9;"), "int(1)");
    assert_eq!(ty(b"<?php (int) '123abc';"), "int(123)");
    assert_eq!(ty(b"<?php (int) true;"), "int(1)");
    assert_eq!(ty(b"<?php (int) null;"), "int(0)");
    assert_eq!(ty(b"<?php (float) 3;"), "float(3)");
    assert_eq!(ty(b"<?php (bool) 0;"), "false");
    assert_eq!(ty(b"<?php (bool) 'a';"), "true");
    assert_eq!(ty(b"<?php (string) 1;"), "string('1')");
    assert_eq!(ty(b"<?php (string) true;"), "string('1')");
    assert_eq!(ty(b"<?php (string) null;"), "string('')");
}

#[test]
fn array_casts() {
    assert_eq!(ty(b"<?php (int) (float) [1];"), "int(1)");
    assert_eq!(ty(b"<?php (int) [];"), "int(0)");
    assert_eq!(ty(b"<?php (int) [1, 2];"), "int(1)");
    assert_eq!(ty(b"<?php (float) [1];"), "float(1)");
    assert_eq!(ty(b"<?php (float) [];"), "float(0)");
    assert_eq!(ty(b"<?php (bool) [];"), "false");
    assert_eq!(ty(b"<?php (bool) [1];"), "true");
    assert_eq!(ty(b"<?php ![];"), "true");
    assert_eq!(ty(b"<?php ![1];"), "false");
}

#[test]
fn array_literals() {
    assert_eq!(ty(b"<?php [];"), "array{}");
    assert_eq!(ty(b"<?php [1, 2, 3];"), "list{0: int(1), 1: int(2), 2: int(3)}");
    assert_eq!(ty(b"<?php ['a' => 1, 'b' => 2];"), "array{'a': int(1), 'b': int(2)}");
    assert_eq!(ty(b"<?php [5 => 'x', 'y'];"), "array{5: string('x'), 6: string('y')}");
    assert_eq!(ty(b"<?php ['1' => 'a', '01' => 'b'];"), "array{1: string('a'), '01': string('b')}");
    assert_eq!(ty(b"<?php [true => 'a', null => 'b'];"), "array{1: string('a'), '': string('b')}");
    assert_eq!(ty(b"<?php ['a' => 1, 'a' => 2];"), "array{'a': int(2)}");
}

#[test]
fn match_expression() {
    assert_eq!(ty(b"<?php match (1) { 1 => 'a', 2 => 'b' };"), "string('a')");
    assert_eq!(ty(b"<?php match (3) { 1, 2 => 'a', default => 'd' };"), "string('d')");
    assert_eq!(ty(b"<?php match (false) { true => 1 };"), "never");
}
