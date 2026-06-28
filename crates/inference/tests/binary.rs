#![allow(clippy::expect_used)]

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
    let source = LocalArena::new();
    let dest = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"b.php"), Cow::Owned(src.to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&source, &scratch, &file, program, LowerSettings::default()).lower();
    let ir = source.alloc(ir);
    let (bound, _d) = bind(&source, Origin::Project, ir);
    let symbols = SymbolTable::new_in(&dest);
    let typed = Inference::new(&source, &dest).infer(&symbols, &file, bound);
    let expr = typed
        .statements
        .iter()
        .find_map(|s| if let StatementKind::Expression(e) = &s.kind { Some(e) } else { None })
        .expect("expression statement");
    format!("{}", expr.meta)
}

#[test]
fn arithmetic_literals() {
    assert_eq!(ty(b"<?php 1 + 123;"), "int(124)");
    assert_eq!(ty(b"<?php 10 - 4;"), "int(6)");
    assert_eq!(ty(b"<?php 6 * 7;"), "int(42)");
    assert_eq!(ty(b"<?php 6 / 2;"), "int(3)"); // exact division -> int
    assert_eq!(ty(b"<?php 7 / 2;"), "float(3.5)"); // non-exact -> int|float
    assert_eq!(ty(b"<?php 7 % 3;"), "int(1)");
    assert_eq!(ty(b"<?php 2 ** 10;"), "int(1024)");
    assert_eq!(ty(b"<?php 1.5 + 2;"), "float(3.5)"); // float involved -> literal float
    assert_eq!(ty(b"<?php 1 / 0;"), "never"); // division by zero
}

#[test]
fn bitwise_and_concat_literals() {
    assert_eq!(ty(b"<?php 6 & 3;"), "int(2)");
    assert_eq!(ty(b"<?php 5 | 2;"), "int(7)");
    assert_eq!(ty(b"<?php 5 ^ 1;"), "int(4)");
    assert_eq!(ty(b"<?php 1 << 4;"), "int(16)");
    assert_eq!(ty(b"<?php 'a' . 'b';"), "string('ab')");
    assert_eq!(ty(b"<?php 'x' . 1;"), "string('x1')");
}

#[test]
fn comparison_and_logical_literals() {
    assert_eq!(ty(b"<?php 1 < 2;"), "true");
    assert_eq!(ty(b"<?php 2 <= 1;"), "false");
    assert_eq!(ty(b"<?php 1 === 1;"), "true");
    assert_eq!(ty(b"<?php 1 === 1.0;"), "false"); // strict: int !== float
    assert_eq!(ty(b"<?php 1 == 1.0;"), "true"); // loose numeric
    assert_eq!(ty(b"<?php true && false;"), "false");
    assert_eq!(ty(b"<?php true || false;"), "true");
    assert_eq!(ty(b"<?php 1 <=> 2;"), "int(-1)");
}

#[test]
fn php_coercion_truth_table() {
    // concat coercion (the headline)
    assert_eq!(ty(b"<?php [] . ' 123' . 0x01;"), "string('Array 1231')");
    assert_eq!(ty(b"<?php true . 'x';"), "string('1x')");
    assert_eq!(ty(b"<?php false . 'x';"), "string('x')");
    assert_eq!(ty(b"<?php null . 'x';"), "string('x')");

    // numeric strings in arithmetic
    assert_eq!(ty(b"<?php '123' + 1;"), "int(124)");
    assert_eq!(ty(b"<?php ' 123' + 1;"), "int(124)");
    assert_eq!(ty(b"<?php '1.5' + 1;"), "float(2.5)");
    assert_eq!(ty(b"<?php '123abc' + 1;"), "int(124)");
    assert_eq!(ty(b"<?php '0x1A' + 1;"), "int(1)");
    assert_eq!(ty(b"<?php '1e3' + 1;"), "float(1001)");
    assert_eq!(ty(b"<?php true + 1;"), "int(2)");
    assert_eq!(ty(b"<?php null + 1;"), "int(1)");

    // bitwise: both strings -> bytewise string; else numeric int
    assert_eq!(ty(b"<?php '5' & '3';"), "string('1')");
    assert_eq!(ty(b"<?php '5' | '2';"), "string('7')");
    assert_eq!(ty(b"<?php '10' & 6;"), "int(2)");

    // comparison numeric-aware
    assert_eq!(ty(b"<?php '123' <=> 124;"), "int(-1)");
    assert_eq!(ty(b"<?php '1' == 1;"), "true");
    assert_eq!(ty(b"<?php '01' == '1';"), "true");
}
