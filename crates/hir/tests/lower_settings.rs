use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::item::annotation::ItemAnnotationTag;
use mago_hir::ir::item::expression::closure::Closure;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::item::statement::function::Function;
use mago_hir::ir::item::statement::function::FunctionFlag;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::r#type::annotation::TypeAnnotationKind;
use mago_hir::ir::variable::DirectVariable;
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
        if let Some(annotation) = function.annotation.and_then(|annotation| annotation.return_type.first()) {
            self.function_return_kinds.push(kind_name(&annotation.kind));
        }
    }

    fn walk_in_closure(&mut self, closure: &Closure<'arena, (), (), ()>, _context: &mut ()) {
        let inherited_type_parameters =
            closure.annotation.map(|annotation| annotation.inherited_type_parameters).unwrap_or(&[]);
        self.closure_inherited_templates.push(
            inherited_type_parameters
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
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, settings).lower();
    drop(scratch);

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
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, settings).lower();
    drop(scratch);

    for statement in ir.statements {
        match &statement.kind {
            StatementKind::Item(definition) => {
                return match definition.kind {
                    ItemStatementKind::Constant(_) => "constant".to_owned(),
                    _ => "definition".to_owned(),
                };
            }
            StatementKind::Expression(_) => return "expression".to_owned(),
            StatementKind::Sequence(sequence) => {
                let constants = sequence
                    .iter()
                    .filter(|statement| {
                        matches!(&statement.kind, StatementKind::Item(definition)
                            if matches!(definition.kind, ItemStatementKind::Constant(_)))
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

fn global_names(globals: &[DirectVariable<'_>]) -> Vec<String> {
    globals.iter().map(|variable| String::from_utf8_lossy(variable.name).into_owned()).collect()
}

fn top_level_functions<'arena>(ir: &IR<'arena, (), (), ()>) -> Vec<&'arena Function<'arena, (), (), ()>> {
    let mut functions = Vec::new();
    for statement in ir.statements {
        if let StatementKind::Item(item) = &statement.kind {
            if let ItemStatementKind::Function(function) = item.kind {
                functions.push(function);
            }
        }
    }

    functions
}

const BODY_EFFECTS: &str = "<?php
function with_effects() {
    global $a;
    throw new Exception();
    yield 1;
}

function only_nested() {
    $c = function () {
        global $b;
        throw new Exception();
        yield 2;
    };
}
";

#[test]
fn body_effects_stop_at_function_like_boundaries() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(BODY_EFFECTS.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    let functions = top_level_functions(&ir);
    assert_eq!(functions.len(), 2);

    let with_effects = functions[0];
    assert!(with_effects.flags.contains(FunctionFlag::Throws));
    assert!(with_effects.flags.contains(FunctionFlag::Yields));
    assert_eq!(global_names(with_effects.direct_accessed_globals), vec!["$a".to_owned()]);

    let only_nested = functions[1];
    assert!(!only_nested.flags.contains(FunctionFlag::Throws));
    assert!(!only_nested.flags.contains(FunctionFlag::Yields));
    assert!(only_nested.direct_accessed_globals.is_empty());
}

const DEPRECATION_CONFLICT: &str = "<?php
/**
 * @deprecated
 * @not-deprecated
 */
function f() {}
";

#[test]
fn deprecated_and_not_deprecated_are_both_recorded() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(DEPRECATION_CONFLICT.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    let functions = top_level_functions(&ir);
    assert_eq!(functions.len(), 1);

    let annotation = functions[0].annotation;
    assert!(annotation.is_some_and(|annotation| annotation.tags.contains(ItemAnnotationTag::Deprecated)));
    assert!(annotation.is_some_and(|annotation| annotation.tags.contains(ItemAnnotationTag::NotDeprecated)));
}

const FORWARD_TEMPLATE: &str = "<?php
/**
 * @return T
 * @template T
 */
function f() {}
";

#[test]
fn templates_resolve_when_used_before_declaration_in_a_docblock() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(FORWARD_TEMPLATE.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    let functions = top_level_functions(&ir);
    assert_eq!(functions.len(), 1);

    let return_kind = functions[0]
        .annotation
        .and_then(|annotation| annotation.return_type.first())
        .map(|return_type| matches!(return_type.kind, TypeAnnotationKind::GenericParameter(_)));
    assert_eq!(return_kind, Some(true));
}
