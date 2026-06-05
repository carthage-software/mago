use std::borrow::Cow;

use bumpalo::Bump;

use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::expression::definition::Closure;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::definition::DefinitionStatementKind;
use mago_hir::ir::statement::definition::Function;
use mago_hir::ir::r#type::annotation::TypeAnnotationKind;
use mago_hir::lower::DefineConstantLowering;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_hir::walker::MutWalker;
use mago_syntax::parser::parse_file;

#[derive(Default)]
struct Collector {
    function_return_kinds: Vec<String>,
    closure_inherited_templates: Vec<Vec<String>>,
}

impl<'arena> MutWalker<'arena, (), (), (), ()> for Collector {
    fn walk_in_function(&mut self, function: &Function<'arena, (), (), ()>, _context: &mut ()) {
        if let Some(annotation) = function.return_type_annotation {
            self.function_return_kinds.push(kind_name(&annotation.kind));
        }
    }

    fn walk_in_closure(&mut self, closure: &Closure<'arena, (), (), ()>, _context: &mut ()) {
        self.closure_inherited_templates.push(
            closure
                .inherited_type_parameters
                .iter()
                .map(|template| String::from_utf8_lossy(template.name.value).into_owned())
                .collect(),
        );
    }
}

fn kind_name(kind: &TypeAnnotationKind<'_>) -> String {
    match kind {
        TypeAnnotationKind::AliasReference(..) => "AliasReference".to_owned(),
        TypeAnnotationKind::Named(..) => "Named".to_owned(),
        other => format!("{other:?}"),
    }
}

fn collect(code: &str, settings: LowerSettings) -> Collector {
    let arena = Bump::new();
    let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &file, program, settings).lower();

    let mut collector = Collector::default();
    collector.walk_ir(&ir, &mut ());
    collector
}

const PROGRAM_WIDE_ALIAS: &str = "<?php
/** @phpstan-type Foo int */
class A {}

/** @return Foo */
function f() { return 1; }
";

#[test]
fn program_wide_aliases_resolve_a_foreign_alias_when_enabled() {
    let settings = LowerSettings { program_wide_type_aliases: true, ..LowerSettings::default() };
    let collector = collect(PROGRAM_WIDE_ALIAS, settings);
    assert_eq!(collector.function_return_kinds, vec!["AliasReference".to_owned()]);
}

#[test]
fn program_wide_aliases_do_not_resolve_a_foreign_alias_when_disabled() {
    let settings = LowerSettings { program_wide_type_aliases: false, ..LowerSettings::default() };
    let collector = collect(PROGRAM_WIDE_ALIAS, settings);
    assert_eq!(collector.function_return_kinds, vec!["Named".to_owned()]);
}

const RE_EXPORT_ALIAS: &str = "<?php
/** @phpstan-type Foo int */
class A {}

/** @phpstan-import-type Foo from A */
class B {}

/** @return Foo */
function f() { return 1; }
";

#[test]
fn re_export_aliases_resolve_an_imported_alias_when_enabled() {
    let settings =
        LowerSettings { program_wide_type_aliases: false, re_export_type_aliases: true, ..LowerSettings::default() };
    let collector = collect(RE_EXPORT_ALIAS, settings);
    assert_eq!(collector.function_return_kinds, vec!["AliasReference".to_owned()]);
}

#[test]
fn re_export_aliases_do_not_resolve_an_imported_alias_when_disabled() {
    let settings =
        LowerSettings { program_wide_type_aliases: false, re_export_type_aliases: false, ..LowerSettings::default() };
    let collector = collect(RE_EXPORT_ALIAS, settings);
    assert_eq!(collector.function_return_kinds, vec!["Named".to_owned()]);
}

const STATIC_TEMPLATE: &str = "<?php
/** @template T */
class Box {
    /** @param T $value */
    public static function make($value): void {
        $f = function () use ($value) { return $value; };
    }
}
";

#[test]
fn closures_inherit_class_templates_in_static_methods_when_enabled() {
    let settings = LowerSettings { inherit_static_templates: true, ..LowerSettings::default() };
    let collector = collect(STATIC_TEMPLATE, settings);
    assert_eq!(collector.closure_inherited_templates, vec![vec!["T".to_owned()]]);
}

#[test]
fn closures_do_not_inherit_class_templates_in_static_methods_when_disabled() {
    let settings = LowerSettings { inherit_static_templates: false, ..LowerSettings::default() };
    let collector = collect(STATIC_TEMPLATE, settings);
    assert_eq!(collector.closure_inherited_templates, vec![Vec::<String>::new()]);
}

const DEFINE_CALL: &str = "<?php
define('FOO', 1);
";

fn first_statement_shape(code: &str, settings: LowerSettings) -> String {
    let arena = Bump::new();
    let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &file, program, settings).lower();

    for statement in ir.statements {
        match &statement.kind {
            StatementKind::Definition(definition) => {
                return match definition.kind {
                    DefinitionStatementKind::Constant(_) => "constant".to_owned(),
                    _ => "definition".to_owned(),
                };
            }
            StatementKind::Expression(_) => return "expression".to_owned(),
            StatementKind::Sequence(sequence) => {
                let constants = sequence
                    .iter()
                    .filter(|statement| {
                        matches!(&statement.kind, StatementKind::Definition(definition)
                            if matches!(definition.kind, DefinitionStatementKind::Constant(_)))
                    })
                    .count();
                let expressions =
                    sequence.iter().filter(|statement| matches!(&statement.kind, StatementKind::Expression(_))).count();
                return format!("sequence(constants={constants},expressions={expressions})");
            }
            _ => {}
        }
    }

    "none".to_owned()
}

#[test]
fn define_lowers_to_a_constant_statement_by_default() {
    assert_eq!(first_statement_shape(DEFINE_CALL, LowerSettings::default()), "constant");
}

#[test]
fn define_can_keep_the_call_alongside_the_constant() {
    let settings = LowerSettings {
        define_constant_lowering: DefineConstantLowering::StatementAndCall,
        ..LowerSettings::default()
    };
    assert_eq!(first_statement_shape(DEFINE_CALL, settings), "sequence(constants=1,expressions=1)");
}

#[test]
fn define_is_left_as_a_call_when_disabled() {
    let settings =
        LowerSettings { define_constant_lowering: DefineConstantLowering::Disabled, ..LowerSettings::default() };
    assert_eq!(first_statement_shape(DEFINE_CALL, settings), "expression");
}

#[test]
fn defaults_match_the_cst_behavior() {
    assert_eq!(collect(PROGRAM_WIDE_ALIAS, LowerSettings::default()).function_return_kinds, vec!["AliasReference"]);
    assert_eq!(collect(RE_EXPORT_ALIAS, LowerSettings::default()).function_return_kinds, vec!["AliasReference"]);
    assert_eq!(
        collect(STATIC_TEMPLATE, LowerSettings::default()).closure_inherited_templates,
        vec![vec!["T".to_owned()]]
    );
}
