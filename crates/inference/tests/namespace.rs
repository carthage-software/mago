use std::borrow::Cow;
use std::fmt::Display;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_inference::Inference;
use mago_oracle::definition::binder::bind;
use mago_oracle::linker::link;
use mago_oracle::symbol::part::origin::Origin;
use mago_syntax::parser::parse_file;

/// Finds the last expression statement's type, descending through `namespace`
/// blocks and statement sequences.
fn descend_last<I, S, E>(statements: &[Statement<'_, I, S, E>]) -> Option<String>
where
    E: Display,
{
    for statement in statements.iter().rev() {
        let found = match &statement.kind {
            StatementKind::Expression(expression) => Some(format!("{}", expression.meta)),
            StatementKind::Sequence(inner) => descend_last(inner),
            StatementKind::Namespace(namespace) => descend_last(std::slice::from_ref(namespace.statement)),
            _ => None,
        };

        if found.is_some() {
            return found;
        }
    }

    None
}

/// Links `definitions` into a symbol table, then infers the type of the last
/// expression statement in `usage` against it.
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
    let (bound, _d) = bind(&source, Origin::Project, ir);

    let typed = Inference::new(&source, &dest).infer(&symbols, &file, bound);

    descend_last(typed.statements).expect("expression statement")
}

#[test]
fn namespace_magic_constant_is_exact() {
    assert_eq!(ty(b"<?php", b"<?php __NAMESPACE__;"), "string('')");
    assert_eq!(ty(b"<?php", b"<?php namespace App; __NAMESPACE__;"), "string('App')");
    assert_eq!(ty(b"<?php", b"<?php namespace App\\Model; __NAMESPACE__;"), "string('App\\Model')");
}

#[test]
fn namespaced_constant_resolves() {
    assert_eq!(ty(b"<?php namespace App; const FOO = 1;", b"<?php namespace App; FOO;"), "int(1)");
}

#[test]
fn unqualified_constant_falls_back_to_global() {
    assert_eq!(ty(b"<?php const FOO = 5;", b"<?php namespace App; FOO;"), "int(5)");
}

#[test]
fn fully_qualified_constant_skips_namespace() {
    assert_eq!(ty(b"<?php const FOO = 5;", b"<?php namespace App; \\FOO;"), "int(5)");
}

#[test]
fn qualified_constant_does_not_fall_back() {
    assert_eq!(ty(b"<?php const FOO = 5;", b"<?php namespace App; Sub\\FOO;"), "mixed");
}
