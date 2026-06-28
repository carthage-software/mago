use std::borrow::Cow;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_inference::Inference;
use mago_oracle::definition::binder::bind;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::symbol::part::origin::Origin;
use mago_syntax::parser::parse_file;

#[test]
fn folds_empty_program_into_typed_ir() {
    let source = LocalArena::new();
    let dest = LocalArena::new();
    let scratch = LocalArena::new();

    let file = File::ephemeral(Cow::Borrowed(b"inference.php"), Cow::Owned(b"".to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&source, &scratch, &file, program, LowerSettings::default()).lower();
    let ir = source.alloc(ir);

    let (bound, _definitions) = bind(&source, Origin::Project, ir);

    let symbols = SymbolTable::new_in(&dest);

    let typed = Inference::new(&source, &dest).infer(&symbols, &file, bound);

    assert!(
        typed.statements.is_empty(),
        "an empty program folds into an empty typed IR without touching the meta hooks"
    );
}
