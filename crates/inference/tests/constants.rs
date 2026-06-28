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
use mago_oracle::linker::link;
use mago_oracle::symbol::part::origin::Origin;
use mago_syntax::parser::parse_file;

/// Links `definitions` into a symbol table (in the destination arena), then
/// infers the type of the last expression statement in `usage` against it.
fn ty(definitions: &[u8], usage: &[u8]) -> String {
    let source = LocalArena::new();
    let dest = LocalArena::new();
    let scratch = LocalArena::new();

    let definitions_file = File::ephemeral(Cow::Borrowed(b"defs.php"), Cow::Owned(definitions.to_vec()));
    let definitions_program = parse_file(&scratch, &definitions_file);
    let definitions_ir: IR<'_, (), (), ()> =
        Lowering::new(&dest, &scratch, &definitions_file, definitions_program, LowerSettings::default()).lower();
    let definitions_ir = dest.alloc(definitions_ir);
    let (_definitions, table) = bind(&dest, Origin::Project, definitions_ir);
    let symbols = link(&dest, &scratch, &[table]);

    let file = File::ephemeral(Cow::Borrowed(b"u.php"), Cow::Owned(usage.to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&source, &scratch, &file, program, LowerSettings::default()).lower();
    let ir = source.alloc(ir);
    let (bound, _definitions) = bind(&source, Origin::Project, ir);

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
fn resolves_global_constant_value() {
    assert_eq!(ty(b"<?php const FOO = 1;", b"<?php FOO;"), "int(1)");
    assert_eq!(ty(b"<?php const NAME = 'hi';", b"<?php NAME;"), "string('hi')");
    assert_eq!(ty(b"<?php const FLAG = true;", b"<?php FLAG;"), "true");
    assert_eq!(ty(b"<?php const PI = 3.14;", b"<?php PI;"), "float");
}

#[test]
fn fully_qualified_reference_resolves() {
    assert_eq!(ty(b"<?php const FOO = 7;", b"<?php \\FOO;"), "int(7)");
}

#[test]
fn declared_type_wins_over_value() {
    assert_eq!(ty(b"<?php /** @var int */ const FOO = 1;", b"<?php FOO;"), "int");
}

#[test]
fn unknown_constant_is_mixed() {
    assert_eq!(ty(b"<?php const FOO = 1;", b"<?php UNDEFINED;"), "mixed");
}
