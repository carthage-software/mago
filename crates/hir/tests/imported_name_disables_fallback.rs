use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::expression::CalleeKind;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::identifier::Identifier;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_syntax::parser::parse_file;

fn with_first_function_callee(code: &str, check: impl FnOnce(&Identifier<'_>)) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"t.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();

    assert!(ir.errors.is_empty(), "`{code}` lowered with errors: {:?}", ir.errors);

    let callee = find_function_callee(ir.statements);
    let Some(callee) = callee else {
        panic!("`{code}` has no function call");
    };

    check(callee);
}

fn find_function_callee<'ir, 'arena>(
    statements: &'ir [Statement<'arena, (), (), ()>],
) -> Option<&'ir Identifier<'arena>> {
    for statement in statements {
        let found = match &statement.kind {
            StatementKind::Namespace(namespace) => find_function_callee(std::slice::from_ref(namespace.statement)),
            StatementKind::Sequence(inner) => find_function_callee(inner),
            StatementKind::Expression(expression) => match &expression.kind {
                ExpressionKind::Call(call) => match &call.callee.kind {
                    CalleeKind::Function(callee) => match &callee.kind {
                        ExpressionKind::Identifier(identifier) => Some(identifier),
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };

        if found.is_some() {
            return found;
        }
    }

    None
}

#[test]
fn imported_function_call_disables_the_fallback() {
    with_first_function_callee("<?php namespace A { use function A\\x; x(); }", |callee| {
        assert_eq!(callee.value, b"A\\x");
        assert!(callee.imported, "a `use function` import resolves exactly, with no fallback");
    });
}

#[test]
fn unqualified_function_call_keeps_the_fallback() {
    with_first_function_callee("<?php namespace A { x(); }", |callee| {
        assert_eq!(callee.value, b"A\\x");
        assert!(!callee.imported, "a bare call resolves to `A\\x` but PHP may fall back to `\\x`");
    });
}

#[test]
fn qualified_function_call_is_not_imported() {
    with_first_function_callee("<?php namespace A { A\\x(); }", |callee| {
        assert_eq!(callee.value, b"A\\A\\x");
        assert!(!callee.imported, "a qualified name is resolved relative to the namespace, not imported");
    });
}

#[test]
fn fully_qualified_function_call_is_not_imported() {
    with_first_function_callee("<?php namespace A { \\x(); }", |callee| {
        assert_eq!(callee.value, b"x");
        assert!(!callee.imported, "a fully-qualified name is already exact, not imported");
    });
}

#[test]
fn aliased_function_call_is_imported() {
    with_first_function_callee("<?php namespace A { use function B\\y as x; x(); }", |callee| {
        assert_eq!(callee.value, b"B\\y");
        assert!(callee.imported, "a renamed `use function` import resolves to its target, no fallback");
    });
}
