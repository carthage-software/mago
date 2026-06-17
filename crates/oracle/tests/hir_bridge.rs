use std::borrow::Cow;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::r#type as hir_type;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_syntax::parser::parse_file;

use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::object::named::ObjectAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectFlag;
use mago_oracle::ty::well_known;

use mago_flags::U8Flags;

fn lower_hir_type<'arena, S, A>(hir: &hir_type::Type<'_>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let atoms = lower_hir_kind(&hir.kind, builder);

    builder.union_of(&atoms)
}

fn lower_hir_kind<'arena, S, A>(
    kind: &hir_type::TypeKind<'_>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    match kind {
        hir_type::TypeKind::Named(identifier) => vec![builder.object_named(identifier.value)],
        hir_type::TypeKind::Union(members) => {
            let mut atoms = Vec::with_capacity(members.len());
            for member in *members {
                atoms.extend(lower_hir_kind(&member.kind, builder));
            }

            atoms
        }
        hir_type::TypeKind::Intersection(members) => {
            let mut atoms = Vec::new();
            for member in *members {
                atoms.extend(lower_hir_kind(&member.kind, builder));
            }

            let Some((head, conjuncts)) = atoms.split_first() else {
                return Vec::new();
            };

            vec![builder.intersected(*head, conjuncts)]
        }
        hir_type::TypeKind::Null => vec![well_known::NULL],
        hir_type::TypeKind::Array => vec![well_known::ARRAY_KEY_MIXED],
        hir_type::TypeKind::Callable => vec![well_known::CALLABLE],
        hir_type::TypeKind::Static(identifier) => {
            let name = builder.intern_class_like_path(identifier.value);

            vec![builder.object(ObjectAtom {
                name,
                type_arguments: None,
                flags: U8Flags::empty().with(ObjectFlag::IsStatic),
            })]
        }
        hir_type::TypeKind::Self_(identifier) | hir_type::TypeKind::Parent(identifier) => {
            vec![builder.object_named(identifier.value)]
        }
        hir_type::TypeKind::Void => vec![well_known::VOID],
        hir_type::TypeKind::Never => vec![well_known::NEVER],
        hir_type::TypeKind::Float => vec![well_known::FLOAT],
        hir_type::TypeKind::Bool(None) => vec![well_known::BOOL],
        hir_type::TypeKind::Bool(Some(true)) => vec![well_known::TRUE],
        hir_type::TypeKind::Bool(Some(false)) => vec![well_known::FALSE],
        hir_type::TypeKind::Integer => vec![well_known::INT],
        hir_type::TypeKind::String => vec![well_known::STRING],
        hir_type::TypeKind::Object => vec![well_known::OBJECT],
        hir_type::TypeKind::Mixed => vec![well_known::MIXED],
        hir_type::TypeKind::Iterable => vec![well_known::ITERABLE_MIXED_MIXED],
    }
}

const CODE: &str = "<?php

namespace App;

use Vendor\\Collection;

function transform(Collection&\\Countable $items, int|string $key, ?float $scale, callable $mapper): Collection|null
{
    return $items;
}

function reset(): void
{
}
";

fn collect_function_signatures(code: &str, run: impl FnOnce(&[(String, Vec<String>, Option<String>)])) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"bridge.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();

    let type_arena = LocalArena::new();
    let type_scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&type_arena, &type_scratch);

    let mut signatures: Vec<(String, Vec<String>, Option<String>)> = Vec::new();
    collect_from_statements(ir.statements, &mut builder, &mut signatures);
    run(&signatures);
}

fn collect_from_statements<'hir, S, A>(
    statements: &'hir [mago_hir::ir::statement::Statement<'hir, (), (), ()>],
    builder: &mut TypeBuilder<'_, '_, S, A>,
    signatures: &mut Vec<(String, Vec<String>, Option<String>)>,
) where
    S: Arena,
    A: Arena,
{
    for statement in statements {
        match &statement.kind {
            StatementKind::Namespace(namespace) => {
                if let StatementKind::Sequence(inner) = &namespace.statement.kind {
                    collect_from_statements(inner, builder, signatures);
                }
            }
            StatementKind::Sequence(inner) => collect_from_statements(inner, builder, signatures),
            StatementKind::Item(definition) => {
                let ItemStatementKind::Function(function) = &definition.kind else {
                    continue;
                };

                let name = String::from_utf8_lossy(function.name.value).into_owned();
                let mut parameters = Vec::new();
                for parameter in function.parameters.as_slice() {
                    if let Some(parameter_type) = parameter.r#type {
                        parameters.push(lower_hir_type(parameter_type, builder).to_string());
                    }
                }

                let return_type =
                    function.return_type.map(|return_type| lower_hir_type(return_type, builder).to_string());

                signatures.push((name, parameters, return_type));
            }
            _ => {}
        }
    }
}

#[test]
fn native_types_bridge_to_canonical_displays() {
    collect_function_signatures(CODE, |signatures| {
        let Some((name, parameters, return_type)) = signatures.first() else {
            panic!("the corpus must lower at least one function");
        };

        assert_eq!(name, "App\\transform");
        assert_eq!(
            parameters,
            &vec![
                "Vendor\\Collection&Countable".to_owned(),
                "int|string".to_owned(),
                "float|null".to_owned(),
                "callable".to_owned(),
            ],
        );
        assert_eq!(return_type.as_deref(), Some("Vendor\\Collection|null"));

        let Some((reset_name, reset_parameters, reset_return)) = signatures.get(1) else {
            panic!("the corpus must lower the second function");
        };
        assert_eq!(reset_name, "App\\reset");
        assert!(reset_parameters.is_empty());
        assert_eq!(reset_return.as_deref(), Some("void"));
    });
}

#[test]
fn bridged_types_are_consed_and_comparable() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(
        Cow::Borrowed(b"consed.php"),
        Cow::Owned(b"<?php function f(int|string $x, int|string $y): void {}".to_vec()),
    );
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();

    let type_arena = LocalArena::new();
    let type_scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&type_arena, &type_scratch);

    let mut lowered: Vec<Type<'_>> = Vec::new();
    for statement in ir.statements {
        let StatementKind::Item(definition) = &statement.kind else {
            continue;
        };
        let ItemStatementKind::Function(function) = &definition.kind else {
            continue;
        };
        for parameter in function.parameters.as_slice() {
            if let Some(parameter_type) = parameter.r#type {
                lowered.push(lower_hir_type(parameter_type, &mut builder));
            }
        }
    }

    let [first, second] = lowered.as_slice() else {
        panic!("both parameters must lower");
    };

    assert!(first.ptr_eq(second));
    assert!(first.ptr_eq(&well_known::TYPE_INT_OR_STRING));
}
