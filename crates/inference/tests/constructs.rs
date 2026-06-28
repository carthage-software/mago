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
fn composite_string() {
    assert_eq!(ty(b"<?php $x = 1; \"n$x\";"), "string('n1')");
    assert_eq!(ty(b"<?php $x = 'b'; \"a{$x}c\";"), "string('abc')");
    assert_eq!(ty(b"<?php $x = 1.5; \"v$x\";"), "non-empty-string");
    assert_eq!(ty(b"<?php \"$undefined\";"), "string");
}

#[test]
fn shell_execute() {
    assert_eq!(ty(b"<?php `echo hi`;"), "false|null|string");
}

#[test]
fn conditional() {
    assert_eq!(ty(b"<?php 1 ? 'a' : 'b';"), "string('a')");
    assert_eq!(ty(b"<?php 0 ? 'a' : 'b';"), "string('b')");
    assert_eq!(ty(b"<?php $y ? 'a' : 'b';"), "string('a')|string('b')");
    assert_eq!(ty(b"<?php 0 ?: 'x';"), "string('x')");
    assert_eq!(ty(b"<?php 1 ?: 'x';"), "int(1)");
    assert_eq!(ty(b"<?php '' ?: 'y';"), "string('y')");
}

#[test]
fn empty_construct() {
    assert_eq!(ty(b"<?php empty(0);"), "true");
    assert_eq!(ty(b"<?php empty(1);"), "false");
    assert_eq!(ty(b"<?php empty([]);"), "true");
    assert_eq!(ty(b"<?php empty([1]);"), "false");
    assert_eq!(ty(b"<?php $x = 0; empty($x);"), "true");
    assert_eq!(ty(b"<?php $x = 'a'; empty($x);"), "false");
}

#[test]
fn isset_construct() {
    assert_eq!(ty(b"<?php $a = 1; isset($a);"), "true");
    assert_eq!(ty(b"<?php $a = null; isset($a);"), "false");
    assert_eq!(ty(b"<?php $a = 1; $b = 2; isset($a, $b);"), "true");
    assert_eq!(ty(b"<?php isset($undefined);"), "bool");
}

#[test]
fn print_eval_include() {
    assert_eq!(ty(b"<?php print 'x';"), "int(1)");
    assert_eq!(ty(b"<?php eval('code');"), "mixed");
    assert_eq!(ty(b"<?php include 'f.php';"), "mixed");
    assert_eq!(ty(b"<?php require 'f.php';"), "mixed");
    assert_eq!(ty(b"<?php require_once 'f.php';"), "mixed");
}

#[test]
fn exit_construct() {
    assert_eq!(ty(b"<?php exit;"), "never");
    assert_eq!(ty(b"<?php exit(1);"), "never");
    assert_eq!(ty(b"<?php die('bye');"), "never");
}

#[test]
fn magic_constant() {
    assert_eq!(ty(b"<?php __LINE__;"), "int(1)");
    assert_eq!(ty(b"<?php\n\n__LINE__;"), "int(3)");
    assert_eq!(ty(b"<?php __FILE__;"), "non-empty-literal-string");
    assert_eq!(ty(b"<?php __DIR__;"), "non-empty-literal-string");
    assert_eq!(ty(b"<?php __FUNCTION__;"), "string('')");
    assert_eq!(ty(b"<?php __CLASS__;"), "string('')");
    assert_eq!(ty(b"<?php __METHOD__;"), "string('')");
    assert_eq!(ty(b"<?php __NAMESPACE__;"), "string('')");
}
