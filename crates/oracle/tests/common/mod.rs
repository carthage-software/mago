#![allow(dead_code)]

//! Test harness shared by the algebra test suites: a [`Fixture`] owning a
//! [`TypeBuilder`] plus a [`MockWorld`], exposing factory methods that build
//! atoms and types in the fixture's arena and free helpers that run the
//! lattice, join, meet, and subtract operations against them.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use mago_allocator::LocalArena;
use mago_flags::U8Flags;

use mago_oracle::name::Name;
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
use mago_oracle::ty::atom::payload::derived::Visibility;
use mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity;
use mago_oracle::ty::atom::payload::generic_parameter::GenericParameterAtom;
use mago_oracle::ty::atom::payload::object::has_method::HasMethodAtom;
use mago_oracle::ty::atom::payload::object::has_property::HasPropertyAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectAtom;
use mago_oracle::ty::atom::payload::object::named::ObjectFlag;
use mago_oracle::ty::atom::payload::object::shape::KnownProperty;
use mago_oracle::ty::atom::payload::object::shape::ObjectShapeAtom;
use mago_oracle::ty::atom::payload::object::shape::ObjectShapeFlag;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeKind;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use mago_oracle::ty::join;
use mago_oracle::ty::join::JoinOptions;
use mago_oracle::ty::lattice;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::well_known;
use mago_oracle::world::ClassConstant;
use mago_oracle::world::ClassProperty;
use mago_oracle::world::EnumBacking;
use mago_oracle::world::NullWorld;
use mago_oracle::world::TemplateParameter;
use mago_oracle::world::Variance;
use mago_oracle::world::World;

/// The per-test environment: a consing builder over fresh arenas and a
/// [`MockWorld`]. Construct via [`fixture`].
pub struct Fixture<'scratch, 'arena> {
    pub builder: TypeBuilder<'scratch, 'arena, LocalArena, LocalArena>,
    pub world: MockWorld<'arena>,
}

/// Run `run` against a fresh [`Fixture`] backed by function-local arenas.
pub fn fixture<F>(run: F)
where
    F: for<'scratch, 'arena> FnOnce(&mut Fixture<'scratch, 'arena>),
{
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut instance = Fixture { builder: TypeBuilder::new(&arena, &scratch), world: MockWorld::new() };
    run(&mut instance);
}

/// An empty world: nothing knows about anything.
pub const fn empty_world() -> NullWorld {
    NullWorld
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
    pub fn name(&mut self, text: &str) -> Name<'arena> {
        self.builder.name(text.as_bytes())
    }

    pub fn name_atom(&mut self, text: &str) -> Name<'arena> {
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
        let name = self.builder.name(name.as_bytes());
        let type_arguments = Some(self.builder.types(&arguments));

        self.builder.object(ObjectAtom { name, type_arguments, flags: U8Flags::empty() })
    }

    pub fn t_named_intersected(&mut self, head: &str, conjuncts: &[Atom<'arena>]) -> Atom<'arena> {
        let head = self.builder.object_named(head.as_bytes());

        self.builder.intersected(head, conjuncts)
    }

    pub fn t_named_static(&mut self, name: &str) -> Atom<'arena> {
        let name = self.builder.name(name.as_bytes());

        self.builder.object(ObjectAtom {
            name,
            type_arguments: None,
            flags: U8Flags::empty().with(ObjectFlag::IsStatic),
        })
    }

    pub fn t_named_this(&mut self, name: &str) -> Atom<'arena> {
        let name = self.builder.name(name.as_bytes());

        self.builder.object(ObjectAtom {
            name,
            type_arguments: None,
            flags: U8Flags::empty().with(ObjectFlag::IsStatic).with(ObjectFlag::IsThis),
        })
    }

    pub fn t_has_method(&mut self, name: &str) -> Atom<'arena> {
        let method_name = self.builder.name(name.as_bytes());

        Atom::HasMethod(HasMethodAtom { method_name })
    }

    pub fn t_has_property(&mut self, name: &str) -> Atom<'arena> {
        let property_name = self.builder.name(name.as_bytes());

        Atom::HasProperty(HasPropertyAtom { property_name })
    }

    pub fn t_object_shape(&mut self, properties: &[(&str, Type<'arena>, bool)], sealed: bool) -> Atom<'arena> {
        let mut entries: Vec<KnownProperty<'arena>> = Vec::with_capacity(properties.len());
        for (name, value, optional) in properties {
            let name = self.builder.name(name.as_bytes());
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
        let name = self.builder.name(template_name.as_bytes());
        let class = self.builder.name(class_name.as_bytes());

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
            let name = self.builder.name(format!("p{index}").as_bytes());
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
        ArrayKey::String(self.builder.name(value.as_bytes()))
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

/// A [`World`] backed by hand-built tables: hierarchy edges, trait usage,
/// per-class type parameters, and per-extension type arguments.
///
/// Registration methods take `&'static str` names (tests use literals);
/// query lookups compare exact bytes against any-lifetime [`Name`]s.
#[derive(Debug)]
pub struct MockWorld<'arena> {
    ancestors: HashMap<Vec<u8>, HashSet<Vec<u8>>>,
    traits_used: HashMap<Vec<u8>, HashSet<Vec<u8>>>,
    templates: HashMap<Vec<u8>, Vec<TemplateParameter<'arena>>>,
    extended: HashMap<(Vec<u8>, Vec<u8>), Vec<Type<'arena>>>,
    methods: HashMap<Vec<u8>, HashSet<Vec<u8>>>,
    properties: HashMap<Vec<u8>, Vec<ClassProperty<'arena>>>,
    enums: HashMap<Vec<u8>, EnumBacking<'arena>>,
    class_like_kinds: HashMap<Vec<u8>, ClassLikeKind>,
    final_classes: HashSet<Vec<u8>>,
    aliases: HashMap<(Vec<u8>, Vec<u8>), Type<'arena>>,
    class_constants: HashMap<Vec<u8>, Vec<ClassConstant<'arena>>>,
    enum_cases: HashMap<Vec<u8>, Vec<Name<'arena>>>,
    global_constants: HashMap<Vec<u8>, Type<'arena>>,
    sealed_inheritors: HashMap<Vec<u8>, Vec<Name<'static>>>,
    sealed_parent: HashMap<Vec<u8>, Name<'static>>,
}

fn key(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

impl<'arena> MockWorld<'arena> {
    pub fn new() -> Self {
        Self {
            ancestors: HashMap::new(),
            traits_used: HashMap::new(),
            templates: HashMap::new(),
            extended: HashMap::new(),
            methods: HashMap::new(),
            properties: HashMap::new(),
            enums: HashMap::new(),
            class_like_kinds: HashMap::new(),
            final_classes: HashSet::new(),
            aliases: HashMap::new(),
            class_constants: HashMap::new(),
            enum_cases: HashMap::new(),
            global_constants: HashMap::new(),
            sealed_inheritors: HashMap::new(),
            sealed_parent: HashMap::new(),
        }
    }

    /// Add a single `child extends/implements parent` edge and recompute the
    /// transitive closure.
    pub fn add_edge(&mut self, child: &str, parent: &str) -> &mut Self {
        self.ancestors.entry(key(child)).or_default().insert(key(child));
        self.ancestors.entry(key(parent)).or_default().insert(key(parent));
        if let Some(entry) = self.ancestors.get_mut(child.as_bytes()) {
            entry.insert(key(parent));
        }

        self.recompute_closure();
        self
    }

    /// Build from a list of `(child, parent)` pairs in one shot.
    pub fn from_edges(edges: &[(&str, &str)]) -> Self {
        let mut world = Self::new();
        for (child, parent) in edges {
            world.add_edge(child, parent);
        }

        world
    }

    /// Register a `class uses TraitName;` relation. Also records the edge
    /// for the ancestor closure.
    pub fn add_trait_use(&mut self, class: &str, trait_name: &str) -> &mut Self {
        self.add_edge(class, trait_name);
        self.traits_used.entry(key(class)).or_default().insert(key(trait_name));
        self
    }

    /// Register a class-like with no ancestors (so reflexive queries like
    /// `descends_from(C, C)` still answer yes).
    pub fn declare(&mut self, name: &str) -> &mut Self {
        self.ancestors.entry(key(name)).or_default().insert(key(name));
        self
    }

    /// Tag `name` as an interface. Implicitly declares `name`.
    pub fn declare_interface(&mut self, name: &str) -> &mut Self {
        self.declare(name);
        self.class_like_kinds.insert(key(name), ClassLikeKind::Interface);
        self
    }

    /// Tag `name` as a trait. Implicitly declares `name`.
    pub fn declare_trait(&mut self, name: &str) -> &mut Self {
        self.declare(name);
        self.class_like_kinds.insert(key(name), ClassLikeKind::Trait);
        self
    }

    /// Mark `name` as `final`. Implicitly declares `name`.
    pub fn with_final(&mut self, name: &str) -> &mut Self {
        self.declare(name);
        self.final_classes.insert(key(name));
        self
    }

    /// Declare `class_like`'s type parameters in declaration order. Each is
    /// a `(name, variance)` pair; bounds default to `None`.
    pub fn with_templates(&mut self, class_like: &str, parameters: &[(&'static str, Variance)]) -> &mut Self {
        self.declare(class_like);
        self.templates.insert(
            key(class_like),
            parameters
                .iter()
                .map(|(name, variance)| TemplateParameter {
                    name: Name::new(name.as_bytes()),
                    variance: *variance,
                    upper_bound: None,
                })
                .collect(),
        );
        self
    }

    /// Set the upper bound (`@template T of Foo`) on `class_like`'s `name`d
    /// template parameter.
    pub fn with_template_bound(&mut self, class_like: &str, name: &str, bound: Type<'arena>) -> &mut Self {
        if let Some(parameters) = self.templates.get_mut(class_like.as_bytes())
            && let Some(parameter) =
                parameters.iter_mut().find(|parameter| parameter.name.as_bytes() == name.as_bytes())
        {
            parameter.upper_bound = Some(bound);
        }

        self
    }

    /// Declare what type arguments `child` passes to `ancestor`. Positional,
    /// in `ancestor`'s declaration order. Implicitly registers
    /// `child extends ancestor`.
    pub fn with_extended(&mut self, child: &str, ancestor: &str, arguments: Vec<Type<'arena>>) -> &mut Self {
        self.add_edge(child, ancestor);
        self.extended.insert((key(child), key(ancestor)), arguments);
        self
    }

    /// Declare that `class` has a method `name` (directly; inheritance is
    /// walked at query time).
    pub fn with_method(&mut self, class: &str, name: &str) -> &mut Self {
        self.declare(class);
        self.methods.entry(key(class)).or_default().insert(key(name));
        self
    }

    /// Declare a public property `name: type` on `class`.
    pub fn with_property(&mut self, class: &str, name: &'static str, r#type: Type<'arena>) -> &mut Self {
        self.with_visible_property(class, name, r#type, Visibility::Public)
    }

    /// Declare a property with explicit visibility.
    pub fn with_visible_property(
        &mut self,
        class: &str,
        name: &'static str,
        r#type: Type<'arena>,
        visibility: Visibility,
    ) -> &mut Self {
        self.declare(class);
        self.properties.entry(key(class)).or_default().push(ClassProperty {
            name: Name::new(name.as_bytes()),
            r#type,
            visibility,
        });
        self
    }

    /// Declare a pure enum: cases expose only `name`.
    pub fn with_pure_enum(&mut self, name: &str) -> &mut Self {
        self.declare(name);
        self.enums.insert(key(name), EnumBacking::Pure);
        self
    }

    /// Declare a backed enum: cases expose `name` and `value`, where `value`
    /// is of `backing` (typically `int` or `string`).
    pub fn with_backed_enum(&mut self, name: &str, backing: Type<'arena>) -> &mut Self {
        self.declare(name);
        self.enums.insert(key(name), EnumBacking::Backed(backing));
        self
    }

    /// Declare an enum case, in declaration order. Enumerated by
    /// [`World::enum_cases`] for wildcard member references (`Suit::*`).
    pub fn with_enum_case(&mut self, enum_name: &str, case: &'static str) -> &mut Self {
        self.declare(enum_name);
        self.enum_cases.entry(key(enum_name)).or_default().push(Name::new(case.as_bytes()));
        self
    }

    /// Declare a `@type` alias on `class`: `Class::alias = body`.
    pub fn with_alias(&mut self, class: &str, alias: &str, body: Type<'arena>) -> &mut Self {
        self.declare(class);
        self.aliases.insert((key(class), key(alias)), body);
        self
    }

    /// Declare a class constant: `Class::CONST: type`, in declaration order.
    /// Enumerated directly by [`World::class_constants`]; the inheritance walk
    /// for a single `Class::CONST` lookup lives in
    /// [`World::class_constant_type`].
    pub fn with_class_constant(&mut self, class: &str, name: &'static str, r#type: Type<'arena>) -> &mut Self {
        self.declare(class);
        self.class_constants
            .entry(key(class))
            .or_default()
            .push(ClassConstant { name: Name::new(name.as_bytes()), r#type });
        self
    }

    /// Declare a global constant: `define('NAME', value)`.
    pub fn with_global_constant(&mut self, name: &str, r#type: Type<'arena>) -> &mut Self {
        self.global_constants.insert(key(name), r#type);
        self
    }

    /// Mark `class` as sealed with the given direct inheritors. Atomically
    /// registers `descends_from` edges so the world stays consistent.
    pub fn with_sealed(&mut self, class: &'static str, inheritors: &[&'static str]) -> &mut Self {
        let class_key = key(class);
        for inheritor in inheritors {
            self.ancestors.entry(key(inheritor)).or_default().insert(class_key.clone());
            self.sealed_parent.insert(key(inheritor), Name::new(class.as_bytes()));
        }

        self.sealed_inheritors
            .insert(class_key, inheritors.iter().map(|inheritor| Name::new(inheritor.as_bytes())).collect());
        self
    }

    fn collect_visible_properties(&self, class: &[u8]) -> Vec<ClassProperty<'arena>> {
        let Some(ancestors) = self.ancestors.get(class) else {
            return Vec::new();
        };

        let mut chain: Vec<&[u8]> = vec![class];
        for ancestor in ancestors {
            if ancestor.as_slice() != class {
                chain.push(ancestor);
            }
        }

        let mut seen: HashSet<&[u8]> = HashSet::new();
        let mut collected: Vec<ClassProperty<'arena>> = Vec::new();
        for link in chain {
            if let Some(properties) = self.properties.get(link) {
                for property in properties {
                    if seen.insert(property.name.as_bytes()) {
                        collected.push(*property);
                    }
                }
            }
        }

        collected
    }

    fn recompute_closure(&mut self) {
        loop {
            let mut changed = false;
            let names: Vec<Vec<u8>> = self.ancestors.keys().cloned().collect();
            for name in &names {
                let Some(direct) = self.ancestors.get(name).cloned() else {
                    continue;
                };
                for parent in direct {
                    let Some(ancestors_of_parent) = self.ancestors.get(&parent).cloned() else {
                        continue;
                    };
                    let Some(entry) = self.ancestors.get_mut(name) else {
                        continue;
                    };
                    for ancestor in ancestors_of_parent {
                        if entry.insert(ancestor) {
                            changed = true;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }
    }
}

impl Default for MockWorld<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'arena> World<'arena> for MockWorld<'arena> {
    fn descends_from(&self, child: Name<'_>, ancestor: Name<'_>) -> bool {
        if child.as_bytes() == ancestor.as_bytes() {
            return true;
        }

        self.ancestors.get(child.as_bytes()).is_some_and(|set| set.contains(ancestor.as_bytes()))
    }

    fn uses_trait(&self, class: Name<'_>, trait_name: Name<'_>) -> bool {
        self.traits_used.get(class.as_bytes()).is_some_and(|set| set.contains(trait_name.as_bytes()))
    }

    fn template_parameter_arity(&self, class: Name<'_>) -> usize {
        self.templates.get(class.as_bytes()).map_or(0, Vec::len)
    }

    fn template_parameter_at(&self, class: Name<'_>, position: usize) -> Option<TemplateParameter<'arena>> {
        self.templates.get(class.as_bytes())?.get(position).copied()
    }

    fn template_parameter_index(&self, class: Name<'_>, name: Name<'_>) -> Option<usize> {
        self.templates.get(class.as_bytes())?.iter().position(|parameter| parameter.name.as_bytes() == name.as_bytes())
    }

    fn inherited_template_argument(
        &self,
        child: Name<'_>,
        ancestor: Name<'_>,
        position: usize,
    ) -> Option<Type<'arena>> {
        if !self.descends_from(child, ancestor) {
            return None;
        }

        if let Some(arguments) = self.extended.get(&(child.as_bytes().to_vec(), ancestor.as_bytes().to_vec()))
            && let Some(argument) = arguments.get(position).copied()
        {
            return Some(argument);
        }

        for (parent_child, parent_ancestor) in self.extended.keys() {
            if parent_child.as_slice() != child.as_bytes() {
                continue;
            }

            let parent_ancestor_name = Name::new(parent_ancestor);
            if self.descends_from(parent_ancestor_name, ancestor)
                && let Some(argument) = self.inherited_template_argument(parent_ancestor_name, ancestor, position)
            {
                return Some(argument);
            }
        }

        None
    }

    fn template_parameter_forwards_to(
        &self,
        from_class: Name<'_>,
        from_parameter: Name<'_>,
        to_class: Name<'_>,
        to_parameter: Name<'_>,
    ) -> bool {
        let target = (to_class.as_bytes().to_vec(), to_parameter.as_bytes().to_vec());
        let mut queue: Vec<(Vec<u8>, Vec<u8>)> =
            vec![(from_class.as_bytes().to_vec(), from_parameter.as_bytes().to_vec())];
        let mut visited: HashSet<(Vec<u8>, Vec<u8>)> = HashSet::new();
        while let Some(node) = queue.pop() {
            if node == target {
                return true;
            }
            if !visited.insert(node.clone()) {
                continue;
            }

            let (class, parameter) = node;
            for ((child, parent), arguments) in &self.extended {
                if child.as_slice() != class.as_slice() {
                    continue;
                }

                for (slot, argument) in arguments.iter().enumerate() {
                    let [Atom::GenericParameter(generic)] = argument.atoms else {
                        continue;
                    };
                    let DefiningEntity::ClassLike(generic_class) = generic.defining_entity else {
                        continue;
                    };
                    if generic_class.as_bytes() != class.as_slice() || generic.name.as_bytes() != parameter.as_slice() {
                        continue;
                    }

                    if let Some(parent_parameters) = self.templates.get(parent)
                        && let Some(parent_parameter) = parent_parameters.get(slot)
                    {
                        queue.push((parent.clone(), parent_parameter.name.as_bytes().to_vec()));
                    }
                }
            }
        }

        false
    }

    fn class_has_method(&self, class: Name<'_>, method: Name<'_>) -> bool {
        let Some(ancestors) = self.ancestors.get(class.as_bytes()) else {
            return false;
        };

        ancestors.iter().any(|ancestor| self.methods.get(ancestor).is_some_and(|set| set.contains(method.as_bytes())))
    }

    fn class_property_type(&self, class: Name<'_>, property: Name<'_>) -> Option<Type<'arena>> {
        self.collect_visible_properties(class.as_bytes())
            .into_iter()
            .find(|entry| entry.name.as_bytes() == property.as_bytes())
            .map(|entry| entry.r#type)
    }

    fn class_has_property(&self, class: Name<'_>, property: Name<'_>) -> bool {
        let Some(ancestors) = self.ancestors.get(class.as_bytes()) else {
            return false;
        };

        ancestors.iter().any(|ancestor| {
            self.properties
                .get(ancestor)
                .is_some_and(|entries| entries.iter().any(|entry| entry.name.as_bytes() == property.as_bytes()))
        })
    }

    fn enum_backing(&self, enum_name: Name<'_>) -> Option<EnumBacking<'arena>> {
        self.enums.get(enum_name.as_bytes()).copied()
    }

    fn class_like_kind(&self, name: Name<'_>) -> Option<ClassLikeKind> {
        if let Some(kind) = self.class_like_kinds.get(name.as_bytes()) {
            return Some(*kind);
        }

        if self.enums.contains_key(name.as_bytes()) {
            return Some(ClassLikeKind::Enum);
        }

        if self.ancestors.contains_key(name.as_bytes()) {
            return Some(ClassLikeKind::Class);
        }

        None
    }

    fn is_final(&self, name: Name<'_>) -> bool {
        self.enums.contains_key(name.as_bytes()) || self.final_classes.contains(name.as_bytes())
    }

    fn alias_body(&self, class: Name<'_>, alias: Name<'_>) -> Option<Type<'arena>> {
        self.aliases.get(&(class.as_bytes().to_vec(), alias.as_bytes().to_vec())).copied()
    }

    fn class_constant_type(&self, class: Name<'_>, constant: Name<'_>) -> Option<Type<'arena>> {
        let ancestors = self.ancestors.get(class.as_bytes())?;

        ancestors.iter().find_map(|ancestor| {
            self.class_constants
                .get(ancestor)?
                .iter()
                .find(|entry| entry.name.as_bytes() == constant.as_bytes())
                .map(|entry| entry.r#type)
        })
    }

    fn class_constants(&self, class: Name<'_>) -> &[ClassConstant<'arena>] {
        self.class_constants.get(class.as_bytes()).map_or(&[], Vec::as_slice)
    }

    fn enum_cases(&self, enum_name: Name<'_>) -> &[Name<'arena>] {
        self.enum_cases.get(enum_name.as_bytes()).map_or(&[], Vec::as_slice)
    }

    fn global_constant_type(&self, name: Name<'_>) -> Option<Type<'arena>> {
        self.global_constants.get(name.as_bytes()).copied()
    }

    fn class_property_count(&self, class: Name<'_>) -> usize {
        self.collect_visible_properties(class.as_bytes()).len()
    }

    fn class_property_at(&self, class: Name<'_>, position: usize) -> Option<ClassProperty<'arena>> {
        self.collect_visible_properties(class.as_bytes()).into_iter().nth(position)
    }

    fn sealed_direct_inheritors(&self, class_like: Name<'_>) -> Option<&[Name<'arena>]> {
        self.sealed_inheritors.get(class_like.as_bytes()).map(Vec::as_slice)
    }

    fn sealed_parent_of(&self, child: Name<'_>) -> Option<Name<'arena>> {
        self.sealed_parent.get(child.as_bytes()).copied()
    }
}

pub fn is_contained<'arena, W>(
    f: &mut Fixture<'_, 'arena>,
    input: Type<'arena>,
    container: Type<'arena>,
    world: &W,
) -> bool
where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();
    let options = LatticeOptions::default().with_template_default_coercion();

    lattice::refines(input, container, world, options, &mut report, &mut f.builder)
}

pub fn is_contained_capturing<'arena, W>(
    f: &mut Fixture<'_, 'arena>,
    input: Type<'arena>,
    container: Type<'arena>,
    world: &W,
) -> (bool, LatticeReport<'arena>)
where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();
    let options = LatticeOptions::default().with_template_default_coercion();
    let verdict = lattice::refines(input, container, world, options, &mut report, &mut f.builder);

    (verdict, report)
}

pub fn is_contained_with<'arena, W>(
    f: &mut Fixture<'_, 'arena>,
    input: Type<'arena>,
    container: Type<'arena>,
    world: &W,
    ignore_null: bool,
    ignore_false: bool,
    inside_assertion: bool,
) -> bool
where
    W: World<'arena>,
{
    let options = LatticeOptions { ignore_null, ignore_false, inside_assertion, ..LatticeOptions::default() }
        .with_template_default_coercion();
    let mut report = LatticeReport::new();

    lattice::refines(input, container, world, options, &mut report, &mut f.builder)
}

pub fn atomic_is_contained<'arena, W>(
    f: &mut Fixture<'_, 'arena>,
    input: Atom<'arena>,
    container: Atom<'arena>,
    world: &W,
) -> bool
where
    W: World<'arena>,
{
    let input_type = f.builder.union_of(&[input]);
    let container_type = f.builder.union_of(&[container]);

    is_contained(f, input_type, container_type, world)
}

pub fn atomic_is_contained_capturing<'arena, W>(
    f: &mut Fixture<'_, 'arena>,
    input: Atom<'arena>,
    container: Atom<'arena>,
    world: &W,
) -> (bool, LatticeReport<'arena>)
where
    W: World<'arena>,
{
    let input_type = f.builder.union_of(&[input]);
    let container_type = f.builder.union_of(&[container]);

    is_contained_capturing(f, input_type, container_type, world)
}

pub fn overlaps<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> bool
where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();

    lattice::overlaps(a, b, world, LatticeOptions::default(), &mut report, &mut f.builder)
}

pub fn atomic_overlaps<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Atom<'arena>, b: Atom<'arena>, world: &W) -> bool
where
    W: World<'arena>,
{
    let a_type = f.builder.union_of(&[a]);
    let b_type = f.builder.union_of(&[b]);

    overlaps(f, a_type, b_type, world)
}

#[track_caller]
pub fn assert_subtype<'arena>(f: &mut Fixture<'_, 'arena>, input: Type<'arena>, container: Type<'arena>) {
    let world = empty_world();
    assert!(is_contained(f, input, container, &world), "expected {input} <: {container} but it is not");
}

#[track_caller]
pub fn assert_not_subtype<'arena>(f: &mut Fixture<'_, 'arena>, input: Type<'arena>, container: Type<'arena>) {
    let world = empty_world();
    assert!(!is_contained(f, input, container, &world), "expected NOT ({input} <: {container}) but it is");
}

#[track_caller]
pub fn assert_atomic_subtype<'arena>(f: &mut Fixture<'_, 'arena>, input: Atom<'arena>, container: Atom<'arena>) {
    let world = empty_world();
    assert!(atomic_is_contained(f, input, container, &world), "expected atomic {input} <: {container}");
}

#[track_caller]
pub fn assert_atomic_not_subtype<'arena>(f: &mut Fixture<'_, 'arena>, input: Atom<'arena>, container: Atom<'arena>) {
    let world = empty_world();
    assert!(!atomic_is_contained(f, input, container, &world), "expected NOT (atomic {input} <: {container})");
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
