use std::borrow::Cow;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_inference::Inference;
use mago_inference::flow::ControlFlow;
use mago_oracle::definition::binder::bind;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::symbol::part::origin::Origin;
use mago_syntax::parser::parse_file;

/// `(reachable, exit)` for every top-level statement.
fn flows(src: &[u8]) -> Vec<(bool, ControlFlow)> {
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

    typed.statements.iter().map(|statement| (statement.meta.reachable, statement.meta.exit)).collect()
}

#[test]
fn statements_after_return_are_unreachable() {
    use ControlFlow::*;

    assert_eq!(
        flows(b"<?php $a = 1; return; $b = 2;"),
        vec![(true, Fallthrough), (true, Fallthrough), (true, Return), (false, Fallthrough)],
    );
}

#[test]
fn diverging_expression_terminates_flow() {
    use ControlFlow::*;

    assert_eq!(flows(b"<?php throw $e; $x;"), vec![(true, Fallthrough), (true, Diverge), (false, Fallthrough)]);
    assert_eq!(flows(b"<?php exit(); $x;"), vec![(true, Fallthrough), (true, Diverge), (false, Fallthrough)]);
}

#[test]
fn straight_line_code_falls_through() {
    use ControlFlow::*;

    assert_eq!(flows(b"<?php $a = 1; $b = 2;"), vec![(true, Fallthrough), (true, Fallthrough), (true, Fallthrough)]);
}
