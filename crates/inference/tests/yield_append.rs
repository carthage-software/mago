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
fn array_append_in_value_position_is_never() {
    assert_eq!(ty(b"<?php $arr[];"), "never");
    assert_eq!(ty(b"<?php $x = $arr[]; $x;"), "never");
}

#[test]
fn yield_is_mixed_for_now() {
    assert_eq!(ty(b"<?php yield;"), "mixed");
    assert_eq!(ty(b"<?php yield 1;"), "mixed");
    assert_eq!(ty(b"<?php yield $k => $v;"), "mixed");
    assert_eq!(ty(b"<?php yield from $gen;"), "mixed");
}
