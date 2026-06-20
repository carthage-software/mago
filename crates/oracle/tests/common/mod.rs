#![allow(dead_code)]

//! Test harness shared by the algebra test suites: a [`Fixture`] owning a
//! [`TypeBuilder`], exposing factory methods that build atoms and types in the
//! fixture's arena and free helpers that run the lattice, join, meet, and
//! subtract operations against a real [`SymbolTable`] built from PHP.

use std::collections::BTreeMap;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_flags::U8Flags;

use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_oracle::definition::DefinitionTable;
use mago_oracle::definition::binder::bind;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::link;
use mago_oracle::linker::link_with;
use mago_oracle::path::Path;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::symbol::class_like::ClassLikeKind;
use mago_oracle::symbol::part::origin::Origin;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::atom::payload::callable::Parameter;
use mago_oracle::ty::atom::payload::callable::ParameterFlag;
use mago_oracle::ty::atom::payload::callable::Signature;
use mago_oracle::ty::atom::payload::callable::SignatureFlag;
use mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity;
use mago_oracle::ty::atom::payload::generic_parameter::GenericParameterAtom;
use mago_oracle::ty::atom::payload::object::has_method::HasMethodAtom;
use mago_oracle::ty::atom::payload::object::has_property::HasPropertyAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectFlag;
use mago_oracle::ty::atom::payload::object::shape::KnownProperty;
use mago_oracle::ty::atom::payload::object::shape::ObjectShapeAtom;
use mago_oracle::ty::atom::payload::object::shape::ObjectShapeFlag;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use mago_oracle::ty::join;
use mago_oracle::ty::join::JoinOptions;
use mago_oracle::ty::lattice;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::well_known;
use mago_syntax::parser::parse_file;
use std::borrow::Cow;

/// The per-test environment: a consing builder over fresh arenas. Symbol
/// tables are built per test via [`symbol_table`] / [`empty_symbol_table`].
/// Construct via [`fixture`].
pub struct Fixture<'scratch, 'arena> {
    pub builder: TypeBuilder<'scratch, 'arena, LocalArena, LocalArena>,
    pub arena: &'arena LocalArena,
}

/// Run `run` against a fresh [`Fixture`] backed by function-local arenas.
pub fn fixture<F>(run: F)
where
    F: for<'scratch, 'arena> FnOnce(&mut Fixture<'scratch, 'arena>),
{
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut instance = Fixture { builder: TypeBuilder::new(&arena, &scratch), arena: &arena };
    run(&mut instance);
}

/// An empty symbol table (one that knows nothing) in `arena`.
pub fn empty_symbol_table(arena: &LocalArena) -> SymbolTable<'_, LocalArena> {
    SymbolTable::new_in(arena)
}

/// Parse, lower, and bind one PHP source into a definition table in `arena`.
fn define<'arena>(arena: &'arena LocalArena, source: &str) -> DefinitionTable<'arena, LocalArena, (), ()> {
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'arena, (), (), ()> = Lowering::new(arena, &scratch, &file, program, LowerSettings::default()).lower();
    let ir = arena.alloc(ir);
    let (_typed, table) = bind(arena, Origin::Project, ir);
    table
}

/// Link one PHP source into a fully-resolved symbol table in `arena`.
pub fn symbol_table<'arena>(arena: &'arena LocalArena, source: &str) -> SymbolTable<'arena, LocalArena> {
    let scratch = LocalArena::new();
    let definition = define(arena, source);
    link(arena, &scratch, std::slice::from_ref(&definition))
}

/// Link one PHP source on top of an existing `base` symbol table, returning the
/// merged table in `arena`.
pub fn symbol_table_onto<'arena>(
    arena: &'arena LocalArena,
    base: &SymbolTable<'arena, LocalArena>,
    source: &str,
) -> SymbolTable<'arena, LocalArena> {
    let scratch = LocalArena::new();
    let definition = define(arena, source);
    link_with(arena, &scratch, base, std::slice::from_ref(&definition))
}

/// Assert that two atom slices are multiset-equal (order-insensitive).
#[track_caller]
pub fn assert_multiset_eq<'arena>(actual: &[Atom<'arena>], expected: &[Atom<'arena>]) {
    let mut left: Vec<Atom<'arena>> = actual.to_vec();
    let mut right: Vec<Atom<'arena>> = expected.to_vec();
    left.sort_unstable();
    right.sort_unstable();
    assert_eq!(left, right, "\n  actual:   {actual:?}\n  expected: {expected:?}");
}

impl<'arena> Fixture<'_, 'arena> {
    pub fn name(&mut self, text: &str) -> Path<'arena> {
        self.builder.intern_class_like_path(text.as_bytes())
    }

    pub fn name_atom(&mut self, text: &str) -> Path<'arena> {
        self.name(text)
    }

    pub fn never(&self) -> Atom<'arena> {
        well_known::NEVER
    }

    pub fn null(&self) -> Atom<'arena> {
        well_known::NULL
    }

    pub fn void(&self) -> Atom<'arena> {
        well_known::VOID
    }

    pub fn placeholder(&self) -> Atom<'arena> {
        well_known::PLACEHOLDER
    }

    pub fn mixed(&self) -> Atom<'arena> {
        well_known::MIXED
    }

    pub fn mixed_truthy(&self) -> Atom<'arena> {
        well_known::TRUTHY_MIXED
    }

    pub fn mixed_falsy(&self) -> Atom<'arena> {
        well_known::FALSY_MIXED
    }

    pub fn mixed_nonnull(&self) -> Atom<'arena> {
        well_known::NON_NULL_MIXED
    }

    pub fn t_true(&self) -> Atom<'arena> {
        well_known::TRUE
    }

    pub fn t_false(&self) -> Atom<'arena> {
        well_known::FALSE
    }

    pub fn t_bool(&self) -> Atom<'arena> {
        well_known::BOOL
    }

    pub fn t_int(&self) -> Atom<'arena> {
        well_known::INT
    }

    pub fn t_lit_int(&self, value: i64) -> Atom<'arena> {
        Atom::int_literal(value)
    }

    pub fn t_int_from(&mut self, from: i64) -> Atom<'arena> {
        self.builder.int_range(Some(from), None)
    }

    pub fn t_int_to(&mut self, to: i64) -> Atom<'arena> {
        self.builder.int_range(None, Some(to))
    }

    pub fn t_int_range(&mut self, low: i64, high: i64) -> Atom<'arena> {
        self.builder.int_range(Some(low), Some(high))
    }

    pub fn t_int_unspec_lit(&self) -> Atom<'arena> {
        well_known::LITERAL_INT
    }

    pub fn t_positive_int(&self) -> Atom<'arena> {
        well_known::POSITIVE_INT
    }

    pub fn t_negative_int(&self) -> Atom<'arena> {
        well_known::NEGATIVE_INT
    }

    pub fn t_non_negative_int(&self) -> Atom<'arena> {
        well_known::NON_NEGATIVE_INT
    }

    pub fn t_non_positive_int(&self) -> Atom<'arena> {
        well_known::NON_POSITIVE_INT
    }

    pub fn t_float(&self) -> Atom<'arena> {
        well_known::FLOAT
    }

    pub fn t_lit_float(&self, value: f64) -> Atom<'arena> {
        Atom::float_literal(value)
    }

    pub fn t_unspec_lit_float(&self) -> Atom<'arena> {
        well_known::LITERAL_FLOAT
    }

    pub fn t_string(&self) -> Atom<'arena> {
        well_known::STRING
    }

    pub fn t_lit_string(&mut self, value: &str) -> Atom<'arena> {
        self.builder.string_literal(value.as_bytes())
    }

    pub fn t_non_empty_string(&self) -> Atom<'arena> {
        well_known::NON_EMPTY_STRING
    }

    pub fn t_numeric_string(&self) -> Atom<'arena> {
        well_known::NUMERIC_STRING
    }

    pub fn t_lower_string(&self) -> Atom<'arena> {
        well_known::LOWERCASE_STRING
    }

    pub fn t_upper_string(&self) -> Atom<'arena> {
        well_known::UPPERCASE_STRING
    }

    pub fn t_truthy_string(&self) -> Atom<'arena> {
        well_known::TRUTHY_STRING
    }

    pub fn t_unspec_lit_string(&self, non_empty: bool) -> Atom<'arena> {
        if non_empty { well_known::NON_EMPTY_LITERAL_STRING } else { well_known::LITERAL_STRING }
    }

    pub fn t_callable_string(&self) -> Atom<'arena> {
        well_known::CALLABLE_STRING
    }

    pub fn t_array_key(&self) -> Atom<'arena> {
        well_known::ARRAY_KEY
    }

    pub fn t_numeric(&self) -> Atom<'arena> {
        well_known::NUMERIC
    }

    pub fn t_scalar(&self) -> Atom<'arena> {
        well_known::SCALAR
    }

    pub fn t_class_string(&self) -> Atom<'arena> {
        well_known::CLASS_STRING
    }

    pub fn t_interface_string(&self) -> Atom<'arena> {
        well_known::INTERFACE_STRING
    }

    pub fn t_enum_string(&self) -> Atom<'arena> {
        well_known::ENUM_STRING
    }

    pub fn t_trait_string(&self) -> Atom<'arena> {
        well_known::TRAIT_STRING
    }

    pub fn t_lit_class_string(&mut self, name: &str) -> Atom<'arena> {
        self.builder.class_string_literal(name.as_bytes())
    }

    pub fn t_class_string_of(&mut self, constraint: Type<'arena>) -> Atom<'arena> {
        self.builder.class_like_string(ClassLikeStringAtom {
            kind: ClassLikeKind::Class,
            specifier: ClassLikeStringSpecifier::OfType { constraint },
        })
    }

    pub fn t_interface_string_of(&mut self, constraint: Type<'arena>) -> Atom<'arena> {
        self.builder.class_like_string(ClassLikeStringAtom {
            kind: ClassLikeKind::Interface,
            specifier: ClassLikeStringSpecifier::OfType { constraint },
        })
    }

    pub fn t_resource(&self) -> Atom<'arena> {
        well_known::RESOURCE
    }

    pub fn t_open_resource(&self) -> Atom<'arena> {
        well_known::OPEN_RESOURCE
    }

    pub fn t_closed_resource(&self) -> Atom<'arena> {
        well_known::CLOSED_RESOURCE
    }

    pub fn t_object_any(&self) -> Atom<'arena> {
        well_known::OBJECT
    }

    pub fn t_named(&mut self, name: &str) -> Atom<'arena> {
        self.builder.object_named(name.as_bytes())
    }

    pub fn t_enum(&mut self, name: &str) -> Atom<'arena> {
        self.builder.enum_any(name.as_bytes())
    }

    pub fn t_enum_case(&mut self, name: &str, case: &str) -> Atom<'arena> {
        self.builder.enum_case(name.as_bytes(), case.as_bytes())
    }

    pub fn t_generic_named(&mut self, name: &str, arguments: Vec<Type<'arena>>) -> Atom<'arena> {
        let name = self.builder.intern_class_like_path(name.as_bytes());

        let type_arguments = Some(self.builder.types(&arguments));

        self.builder.object(ObjectAtom { name, type_arguments, flags: U8Flags::empty() })
    }

    pub fn t_named_intersected(&mut self, head: &str, conjuncts: &[Atom<'arena>]) -> Atom<'arena> {
        let head = self.builder.object_named(head.as_bytes());

        self.builder.intersected(head, conjuncts)
    }

    pub fn t_named_static(&mut self, name: &str) -> Atom<'arena> {
        let name = self.builder.intern_class_like_path(name.as_bytes());

        self.builder.object(ObjectAtom {
            name,
            type_arguments: None,
            flags: U8Flags::empty().with(ObjectFlag::IsStatic),
        })
    }

    pub fn t_named_this(&mut self, name: &str) -> Atom<'arena> {
        let name = self.builder.intern_class_like_path(name.as_bytes());

        self.builder.object(ObjectAtom {
            name,
            type_arguments: None,
            flags: U8Flags::empty().with(ObjectFlag::IsStatic).with(ObjectFlag::IsThis),
        })
    }

    pub fn t_has_method(&mut self, name: &str) -> Atom<'arena> {
        let method_name = self.builder.intern(name.as_bytes());

        Atom::HasMethod(HasMethodAtom { method_name })
    }

    pub fn t_has_property(&mut self, name: &str) -> Atom<'arena> {
        let property_name = self.builder.intern(name.as_bytes());

        Atom::HasProperty(HasPropertyAtom { property_name })
    }

    pub fn t_object_shape(&mut self, properties: &[(&str, Type<'arena>, bool)], sealed: bool) -> Atom<'arena> {
        let mut entries: Vec<KnownProperty<'arena>> = Vec::with_capacity(properties.len());
        for (name, value, optional) in properties {
            let name = self.builder.intern(name.as_bytes());
            entries.push(KnownProperty { name, value: *value, optional: *optional });
        }

        let known_properties = if entries.is_empty() { None } else { Some(self.builder.known_properties(&entries)) };
        let mut flags = U8Flags::empty();
        flags.set_value(ObjectShapeFlag::Sealed, sealed);

        self.builder.object_shape(ObjectShapeAtom { known_properties, flags })
    }

    pub fn t_template(&mut self, class_name: &str, template_name: &str) -> Atom<'arena> {
        self.t_template_of(class_name, template_name, well_known::TYPE_MIXED)
    }

    pub fn t_template_of(&mut self, class_name: &str, template_name: &str, constraint: Type<'arena>) -> Atom<'arena> {
        let name = self.builder.intern(template_name.as_bytes());
        let class = self.builder.intern_class_like_path(class_name.as_bytes());

        self.builder.generic_parameter(GenericParameterAtom {
            name,
            defining_entity: DefiningEntity::ClassLike(class),
            constraint,
        })
    }

    pub fn t_empty_array(&self) -> Atom<'arena> {
        well_known::EMPTY_ARRAY
    }

    pub fn t_list(&mut self, element: Type<'arena>, non_empty: bool) -> Atom<'arena> {
        self.builder.list_of(element, non_empty)
    }

    pub fn t_keyed_unsealed(&mut self, key: Type<'arena>, value: Type<'arena>, non_empty: bool) -> Atom<'arena> {
        self.builder.keyed_unsealed(key, value, non_empty)
    }

    pub fn t_keyed_sealed(
        &mut self,
        items: BTreeMap<ArrayKey<'arena>, (bool, Type<'arena>)>,
        non_empty: bool,
    ) -> Atom<'arena> {
        let entries: Vec<KnownItem<'arena>> =
            items.into_iter().map(|(key, (optional, value))| KnownItem { key, value, optional }).collect();

        self.builder.keyed_sealed(&entries, non_empty)
    }

    pub fn t_iterable(&mut self, key: Type<'arena>, value: Type<'arena>) -> Atom<'arena> {
        self.builder
            .iterable(mago_oracle::ty::atom::payload::iterable::IterableAtom { key_type: key, value_type: value })
    }

    pub fn t_callable_mixed(&mut self) -> Atom<'arena> {
        self.builder.callable_mixed()
    }

    pub fn t_closure_mixed(&mut self) -> Atom<'arena> {
        self.builder.closure_mixed()
    }

    pub fn t_callable_any(&self) -> Atom<'arena> {
        well_known::CALLABLE
    }

    pub fn t_callable_sig(
        &mut self,
        parameters: &[(Type<'arena>, bool, bool, bool)],
        return_type: Type<'arena>,
        pure: bool,
    ) -> Atom<'arena> {
        let mut entries: Vec<Parameter<'arena>> = Vec::with_capacity(parameters.len());
        for (index, (ty, has_default, by_reference, variadic)) in parameters.iter().enumerate() {
            let name = self.builder.intern(format!("p{index}").as_bytes());
            let mut flags = U8Flags::empty();
            flags.set_value(ParameterFlag::HasDefault, *has_default);
            flags.set_value(ParameterFlag::ByReference, *by_reference);
            flags.set_value(ParameterFlag::Variadic, *variadic);
            entries.push(Parameter { name, r#type: *ty, flags });
        }

        let trailing_variadic = entries.last().is_some_and(|entry| entry.flags.contains(ParameterFlag::Variadic));
        let parameter_list = if entries.is_empty() { None } else { Some(self.builder.parameters(&entries)) };
        let mut flags = U8Flags::empty();
        flags.set_value(SignatureFlag::IsVariadic, trailing_variadic);
        flags.set_value(SignatureFlag::IsPure, pure);

        let signature =
            self.builder.signature(Signature { parameters: parameter_list, return_type, throws: None, flags });

        Atom::Callable(CallableAtom::Signature(signature))
    }

    pub fn t_callable(&mut self, parameters: &[Type<'arena>], return_type: Type<'arena>) -> Atom<'arena> {
        let entries: Vec<(Type<'arena>, bool, bool, bool)> =
            parameters.iter().map(|ty| (*ty, false, false, false)).collect();

        self.t_callable_sig(&entries, return_type, false)
    }

    pub fn ak_int(&self, value: i64) -> ArrayKey<'arena> {
        ArrayKey::Int(value)
    }

    pub fn ak_str(&mut self, value: &str) -> ArrayKey<'arena> {
        ArrayKey::String(self.builder.intern(value.as_bytes()))
    }

    pub fn u(&mut self, atom: Atom<'arena>) -> Type<'arena> {
        self.builder.union_of(&[atom])
    }

    pub fn u_many(&mut self, atoms: Vec<Atom<'arena>>) -> Type<'arena> {
        self.builder.union_of(&atoms)
    }

    pub fn ui(&mut self, value: i64) -> Type<'arena> {
        let atom = self.t_lit_int(value);
        self.u(atom)
    }

    pub fn us(&mut self, value: &str) -> Type<'arena> {
        let atom = self.t_lit_string(value);
        self.u(atom)
    }
}

fn class_id(text: &str) -> SymbolId {
    SymbolId::class_like(text.as_bytes())
}

pub fn is_contained<'arena>(
    f: &mut Fixture<'_, 'arena>,
    input: Type<'arena>,
    container: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> bool {
    let mut report = LatticeReport::new();
    let options = LatticeOptions::default().with_template_default_coercion();

    lattice::refines(input, container, symbols, options, &mut report, &mut f.builder)
}

pub fn is_contained_capturing<'arena>(
    f: &mut Fixture<'_, 'arena>,
    input: Type<'arena>,
    container: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> (bool, LatticeReport<'arena>) {
    let mut report = LatticeReport::new();
    let options = LatticeOptions::default().with_template_default_coercion();
    let verdict = lattice::refines(input, container, symbols, options, &mut report, &mut f.builder);

    (verdict, report)
}

pub fn is_contained_with<'arena>(
    f: &mut Fixture<'_, 'arena>,
    input: Type<'arena>,
    container: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
    ignore_null: bool,
    ignore_false: bool,
    inside_assertion: bool,
) -> bool {
    let options = LatticeOptions { ignore_null, ignore_false, inside_assertion, ..LatticeOptions::default() }
        .with_template_default_coercion();
    let mut report = LatticeReport::new();

    lattice::refines(input, container, symbols, options, &mut report, &mut f.builder)
}

pub fn atomic_is_contained<'arena>(
    f: &mut Fixture<'_, 'arena>,
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> bool {
    let input_type = f.builder.union_of(&[input]);
    let container_type = f.builder.union_of(&[container]);

    is_contained(f, input_type, container_type, symbols)
}

pub fn atomic_is_contained_capturing<'arena>(
    f: &mut Fixture<'_, 'arena>,
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> (bool, LatticeReport<'arena>) {
    let input_type = f.builder.union_of(&[input]);
    let container_type = f.builder.union_of(&[container]);

    is_contained_capturing(f, input_type, container_type, symbols)
}

pub fn overlaps<'arena>(
    f: &mut Fixture<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> bool {
    let mut report = LatticeReport::new();

    lattice::overlaps(a, b, symbols, LatticeOptions::default(), &mut report, &mut f.builder)
}

pub fn atomic_overlaps<'arena>(
    f: &mut Fixture<'_, 'arena>,
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, LocalArena>,
) -> bool {
    let a_type = f.builder.union_of(&[a]);
    let b_type = f.builder.union_of(&[b]);

    overlaps(f, a_type, b_type, symbols)
}

#[track_caller]
pub fn assert_subtype<'arena>(f: &mut Fixture<'_, 'arena>, input: Type<'arena>, container: Type<'arena>) {
    let symbols = empty_symbol_table(f.arena);
    assert!(is_contained(f, input, container, &symbols), "expected {input} <: {container} but it is not");
}

#[track_caller]
pub fn assert_not_subtype<'arena>(f: &mut Fixture<'_, 'arena>, input: Type<'arena>, container: Type<'arena>) {
    let symbols = empty_symbol_table(f.arena);
    assert!(!is_contained(f, input, container, &symbols), "expected NOT ({input} <: {container}) but it is");
}

#[track_caller]
pub fn assert_atomic_subtype<'arena>(f: &mut Fixture<'_, 'arena>, input: Atom<'arena>, container: Atom<'arena>) {
    let symbols = empty_symbol_table(f.arena);
    assert!(atomic_is_contained(f, input, container, &symbols), "expected atomic {input} <: {container}");
}

#[track_caller]
pub fn assert_atomic_not_subtype<'arena>(f: &mut Fixture<'_, 'arena>, input: Atom<'arena>, container: Atom<'arena>) {
    let symbols = empty_symbol_table(f.arena);
    assert!(!atomic_is_contained(f, input, container, &symbols), "expected NOT (atomic {input} <: {container})");
}

pub fn combine_default<'arena>(f: &mut Fixture<'_, 'arena>, atoms: Vec<Atom<'arena>>) -> Vec<Atom<'arena>> {
    join::compute(&atoms, &mut f.builder).as_slice().to_vec()
}

pub fn combine_with_int_threshold<'arena>(
    f: &mut Fixture<'_, 'arena>,
    atoms: Vec<Atom<'arena>>,
    threshold: u16,
) -> Vec<Atom<'arena>> {
    let options = JoinOptions::structural().with_int_literal_collapse_threshold(threshold);

    join::compute_with(&atoms, &options, &mut f.builder).as_slice().to_vec()
}

pub fn combine_with_string_threshold<'arena>(
    f: &mut Fixture<'_, 'arena>,
    atoms: Vec<Atom<'arena>>,
    threshold: u16,
) -> Vec<Atom<'arena>> {
    let options = JoinOptions::structural().with_string_literal_collapse_threshold(threshold);

    join::compute_with(&atoms, &options, &mut f.builder).as_slice().to_vec()
}

pub fn combine_with_float_threshold<'arena>(
    f: &mut Fixture<'_, 'arena>,
    atoms: Vec<Atom<'arena>>,
    threshold: u16,
) -> Vec<Atom<'arena>> {
    let options = JoinOptions::structural().with_float_literal_collapse_threshold(threshold);

    join::compute_with(&atoms, &options, &mut f.builder).as_slice().to_vec()
}

pub fn combine_with_array_threshold<'arena>(
    f: &mut Fixture<'_, 'arena>,
    atoms: Vec<Atom<'arena>>,
    threshold: u16,
) -> Vec<Atom<'arena>> {
    let options = JoinOptions::structural().with_array_shape_collapse_threshold(threshold);

    join::compute_with(&atoms, &options, &mut f.builder).as_slice().to_vec()
}

pub fn combine_overwrite<'arena>(f: &mut Fixture<'_, 'arena>, atoms: Vec<Atom<'arena>>) -> Vec<Atom<'arena>> {
    let options = JoinOptions::default().with_overwrite_empty_array(true);

    join::compute_with(&atoms, &options, &mut f.builder).as_slice().to_vec()
}

#[track_caller]
pub fn assert_single<'arena, F>(f: &mut Fixture<'_, 'arena>, input: Vec<Atom<'arena>>, predicate: F)
where
    F: Fn(&Atom<'arena>) -> bool,
{
    let result = combine_default(f, input);
    assert_eq!(result.len(), 1, "expected single atom, got: {result:?}");
    assert!(predicate(&result[0]), "predicate failed for: {:?}", result[0]);
}

#[track_caller]
pub fn assert_self_idempotent<'arena>(f: &mut Fixture<'_, 'arena>, atom: Atom<'arena>, n: usize) {
    let out = combine_default(f, vec![atom; n]);
    assert_eq!(out.len(), 1, "self-combination should produce 1 atom for {atom:?}, got {out:?}");
    assert_eq!(out[0], atom, "self-combination should preserve identity for {atom:?}");
}

#[track_caller]
pub fn assert_combines_to<'arena>(
    f: &mut Fixture<'_, 'arena>,
    input: Vec<Atom<'arena>>,
    mut expected: Vec<Atom<'arena>>,
) {
    let mut actual = combine_default(f, input);
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected, "\n  actual:   {actual:?}\n  expected: {expected:?}");
}
