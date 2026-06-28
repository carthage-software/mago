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

fn inferred_expression_type(src: &[u8]) -> String {
    let source = LocalArena::new();
    let dest = LocalArena::new();
    let scratch = LocalArena::new();

    let file = File::ephemeral(Cow::Borrowed(b"inf.php"), Cow::Owned(src.to_vec()));
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
        .expect("an expression statement");

    format!("{}", expr.meta)
}

#[test]
fn infers_literals() {
    assert_eq!(inferred_expression_type(b"<?php 1;"), "int(1)");
    assert_eq!(inferred_expression_type(b"<?php 1.5;"), "float(1.5)");
    assert_eq!(inferred_expression_type(b"<?php true;"), "true");
    assert_eq!(inferred_expression_type(b"<?php null;"), "null");
    assert_eq!(inferred_expression_type(b"<?php 'foo';"), "string('foo')");
    assert_eq!(inferred_expression_type(b"<?php '';"), "string('')");
}

#[test]
fn infers_assignment_value_type() {
    // the value of an assignment expression is the RHS type
    assert_eq!(inferred_expression_type(b"<?php $a = 1;"), "int(1)");
    assert_eq!(inferred_expression_type(b"<?php $a = 'hi';"), "string('hi')");
}

#[test]
fn assignment_binds_target_variable_type() {
    let source = LocalArena::new();
    let dest = LocalArena::new();
    let scratch = LocalArena::new();

    let file = File::ephemeral(Cow::Borrowed(b"inf.php"), Cow::Owned(b"<?php $a = 1;".to_vec()));
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
        .expect("an expression statement");

    let ExpressionKind::Assignment(assignment) = &expr.kind else { panic!("expected an assignment") };
    // the LHS target `$a` carries the assigned type
    assert_eq!(format!("{}", assignment.left.meta), "int(1)");
}
