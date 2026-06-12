use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::literal::Literal;
use mago_hir::ir::literal::LiteralKind;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::r#type::TypeKind;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_hir::walker::MutWalker;
use mago_syntax::parser::parse_file;

const CODE: &str = "<?php

namespace App;

use Vendor\\Collection;

function first(Collection $items): Collection
{
    return $items;
}

function second(Collection $items): Collection
{
    return $items;
}
";

#[test]
fn resolved_names_are_interned_once_per_file() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(CODE.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    let mut class_names = Vec::new();
    for statement in ir.statements {
        let StatementKind::Namespace(namespace) = &statement.kind else {
            continue;
        };
        let StatementKind::Sequence(statements) = &namespace.statement.kind else {
            panic!("the namespace body must lower to a statement sequence");
        };

        for statement in statements.iter() {
            let StatementKind::Item(item) = &statement.kind else {
                continue;
            };
            let ItemStatementKind::Function(function) = &item.kind else {
                continue;
            };

            for parameter in function.parameters.iter() {
                let Some(r#type) = parameter.r#type else {
                    panic!("every fixture parameter must carry a type hint");
                };
                let TypeKind::Named(identifier) = r#type.kind else {
                    panic!("every fixture parameter hint must lower to a named type");
                };

                class_names.push(identifier.value);
            }

            let Some(return_type) = function.return_type else {
                panic!("every fixture function must carry a return type");
            };
            let TypeKind::Named(identifier) = return_type.kind else {
                panic!("every fixture return type must lower to a named type");
            };

            class_names.push(identifier.value);
        }
    }

    let [first, rest @ ..] = class_names.as_slice() else {
        panic!("the fixture must produce resolved class names");
    };

    assert_eq!(class_names.len(), 4, "expected two parameter hints and two return types");
    assert_eq!(*first, b"Vendor\\Collection", "the alias must resolve to the imported name");
    for other in rest {
        assert!(
            std::ptr::eq(*first, *other),
            "every occurrence of the resolved name must share one interned allocation",
        );
    }
}

const LITERALS: &str = "<?php

function render(string $field): void
{
    if ($field === 'password') {
        render_field('password');
    }
}
";

#[test]
fn repeated_string_literals_share_one_interned_allocation() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"literals.php"), Cow::Owned(LITERALS.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    struct Literals<'arena> {
        values: Vec<&'arena [u8]>,
    }

    impl<'arena> MutWalker<'arena, (), (), (), ()> for Literals<'arena> {
        fn walk_in_literal(&mut self, literal: &Literal<'arena>, _context: &mut ()) {
            if let LiteralKind::String(string) = literal.kind
                && let Some(value) = string.value
            {
                self.values.push(value);
            }
        }
    }

    let mut literals = Literals { values: Vec::new() };
    literals.walk_ir(&ir, &mut ());

    let [first, second] = literals.values.as_slice() else {
        panic!("the fixture must lower exactly two string literals, got {}", literals.values.len());
    };

    assert_eq!(*first, b"password");
    assert!(std::ptr::eq(*first, *second), "both occurrences of the literal must share one interned allocation",);
}
