use std::borrow::Cow;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::expression::Binary;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_inference::Inference;
use mago_inference::extension::Extensions;
use mago_inference::flow::Flow;
use mago_oracle::definition::binder::bind;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::link;
use mago_oracle::symbol::part::origin::Origin;
use mago_oracle::ty::Type;
use mago_syntax::parser::parse_file;

pub use mago_hir::ir::expression::ExpressionKind;
pub use mago_inference::flow::ControlFlow;

pub type TypedIr<'arena> = IR<'arena, SymbolId, Flow, Type<'arena>>;
pub type TypedStatement<'arena> = Statement<'arena, SymbolId, Flow, Type<'arena>>;
pub type TypedExpression<'arena> = Expression<'arena, SymbolId, Flow, Type<'arena>>;
pub type TypedBinary<'arena> = Binary<'arena, SymbolId, Flow, Type<'arena>>;

/// Owns the arenas a single inference run needs so that the returned typed `IR`
/// (which borrows the arena) can outlive the call. Only the `test_inference!`
/// macro constructs and drives it.
pub struct Test {
    arena: LocalArena,
    scratch: LocalArena,
}

impl Default for Test {
    fn default() -> Self {
        Self::new()
    }
}

impl Test {
    pub fn new() -> Self {
        Self { arena: LocalArena::new(), scratch: LocalArena::new() }
    }

    /// Links `definitions` into a symbol table, then infers `code` against it,
    /// returning the typed IR. `definitions` is plain PHP whose declarations
    /// (constants, functions, classes) populate the symbol table.
    pub fn infer<'arena>(&'arena self, definitions: &str, code: &str) -> TypedIr<'arena> {
        self.infer_with(definitions, code, Extensions::default())
    }

    /// As [`Self::infer`], inferring against the given extensions.
    pub fn infer_with<'arena>(
        &'arena self,
        definitions: &str,
        code: &str,
        extensions: Extensions<'arena, LocalArena>,
    ) -> TypedIr<'arena> {
        let definitions_file =
            File::ephemeral(Cow::Borrowed(b"definitions.php"), Cow::Owned(definitions.as_bytes().to_vec()));
        let definitions_program = parse_file(&self.scratch, &definitions_file);
        let definitions_ir: IR<'_, (), (), ()> =
            Lowering::new(&self.arena, &self.scratch, &definitions_file, definitions_program, LowerSettings::default())
                .lower();
        let definitions_ir = self.arena.alloc(definitions_ir);
        let (_definitions, definitions_table) = bind(&self.arena, Origin::Project, definitions_ir);

        let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(code.as_bytes().to_vec()));
        let program = parse_file(&self.scratch, &file);
        let ir: IR<'_, (), (), ()> =
            Lowering::new(&self.arena, &self.scratch, &file, program, LowerSettings::default()).lower();
        let ir = self.arena.alloc(ir);
        let (bound, code_table) = bind(&self.arena, Origin::Project, ir);

        let symbols = link(&self.arena, &self.scratch, &[definitions_table, code_table]);

        Inference::new(&self.arena, &self.arena).infer(&symbols, &file, bound, extensions)
    }
}

/// The last top-level statement; panics when the program is empty.
pub fn get_last_statement(ir: TypedIr<'_>) -> &TypedStatement<'_> {
    match ir.statements.last() {
        Some(statement) => statement,
        None => panic!("expected the program to contain at least one statement"),
    }
}

/// The top-level statement at `index`; panics when there is none.
pub fn get_nth_statement(ir: TypedIr<'_>, index: usize) -> &TypedStatement<'_> {
    match ir.statements.get(index) {
        Some(statement) => statement,
        None => panic!("expected the program to contain a statement at index {index}"),
    }
}

/// The last expression statement's expression, descending through transparent
/// grouping (namespace blocks and statement sequences). Panics when none exist.
pub fn get_last_expression(ir: TypedIr<'_>) -> &TypedExpression<'_> {
    match last_expression_in(ir.statements) {
        Some(expression) => expression,
        None => panic!("expected the program to contain an expression statement"),
    }
}

/// The last expression statement as a binary expression; panics otherwise.
pub fn get_last_binary(ir: TypedIr<'_>) -> &TypedBinary<'_> {
    match get_last_expression(ir).kind {
        ExpressionKind::Binary(binary) => binary,
        _ => panic!("expected the last expression to be a binary"),
    }
}

/// The expression of an expression statement; panics for any other statement.
pub fn expression_of<'arena>(statement: &'arena TypedStatement<'arena>) -> &'arena TypedExpression<'arena> {
    match statement.kind {
        StatementKind::Expression(expression) => expression,
        _ => panic!("expected an expression statement"),
    }
}

/// The formatted type of the last expression statement — the assertion used by
/// the `cases` form of `test_inference!`.
pub fn last_type(ir: TypedIr<'_>) -> String {
    get_last_expression(ir).meta.to_string()
}

fn last_expression_in<'arena>(statements: &'arena [TypedStatement<'arena>]) -> Option<&'arena TypedExpression<'arena>> {
    for statement in statements.iter().rev() {
        let found = match statement.kind {
            StatementKind::Expression(expression) => Some(expression),
            StatementKind::Sequence(inner) => last_expression_in(inner),
            StatementKind::Namespace(namespace) => last_expression_in(std::slice::from_ref(namespace.statement)),
            _ => None,
        };

        if found.is_some() {
            return found;
        }
    }

    None
}

/// Declares an inference test. Two forms:
///
/// ```ignore
/// // batch: assert the last expression's type for each snippet
/// test_inference! { name = adds_ints, cases = { "<?php 1 + 2;" => "int(3)" } }
///
/// // closure: navigate the typed IR for flow/structural assertions
/// test_inference! {
///     name = returns,
///     code = "<?php return;",
///     expect = |ir| { assert_eq!(get_last_statement(ir).meta.exit, ControlFlow::Return); }
/// }
/// ```
///
/// `def = "<?php const FOO = 1;"` is optional in both forms and populates the
/// symbol table the code is inferred against (default: an empty `"<?php"`).
macro_rules! test_inference {
    (name = $name:ident, code = $code:expr, expect = |$ir:ident| $body:block $(,)?) => {
        test_inference!(name = $name, def = "<?php", code = $code, expect = |$ir| $body);
    };
    (name = $name:ident, def = $def:expr, code = $code:expr, expect = |$ir:ident| $body:block $(,)?) => {
        #[test]
        fn $name() {
            let test = $crate::harness::Test::new();
            let $ir = test.infer($def, $code);
            $body
        }
    };
    (name = $name:ident, cases = { $($code:expr => $expected:expr),+ $(,)? } $(,)?) => {
        test_inference!(name = $name, def = "<?php", cases = { $($code => $expected),+ });
    };
    (name = $name:ident, def = $def:expr, cases = { $($code:expr => $expected:expr),+ $(,)? } $(,)?) => {
        #[test]
        fn $name() {
            $({
                let test = $crate::harness::Test::new();
                let ir = test.infer($def, $code);
                assert_eq!($crate::harness::last_type(ir), $expected, "for code: {}", $code);
            })+
        }
    };
}
