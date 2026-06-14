mod common;

use common::MockWorld;

use proptest::prelude::*;

use mago_allocator::LocalArena;
use mago_allocator::copy::CopyInto;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::kind::AtomKind;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::join;
use mago_oracle::ty::join::JoinOptions;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::overlaps;
use mago_oracle::ty::lattice::refines;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
use mago_oracle::ty::well_known;
use mago_oracle::world::Variance;
use mago_oracle::world::World;

const CLASSES: &[&str] = &["A", "B", "C", "D", "E"];
const ENUMS: &[&str] = &["Color"];
const METHODS: &[&str] = &["doFoo", "getBar"];
const PROPERTIES: &[&str] = &["id", "name"];
const TEMPLATES: &[&str] = &["T", "U"];

#[derive(Debug, Clone)]
struct WorldRecipe {
    templates: Vec<Vec<Variance>>,
    edges: Vec<bool>,
    methods: Vec<bool>,
    properties: Vec<bool>,
    finals: Vec<bool>,
    sealed_markers: Vec<Option<u8>>,
}

#[derive(Debug, Clone, Copy)]
enum PrimitiveRecipe {
    Int,
    String,
    Float,
    Bool,
    Null,
    Void,
    Mixed,
    Never,
    Object,
    Scalar,
    Numeric,
    ArrayKey,
}

#[derive(Debug, Clone, Copy)]
enum RefinedIntRecipe {
    Positive,
    Negative,
    NonNegative,
    NonPositive,
    Range(i64, i64),
    From(i64),
    To(i64),
}

#[derive(Debug, Clone, Copy)]
enum RefinedStringRecipe {
    NonEmpty,
    Numeric,
    Lower,
    Upper,
    Truthy,
    ClassString,
    InterfaceString,
    EnumString,
}

#[derive(Debug, Clone, Copy)]
enum ConstraintRecipe {
    Mixed,
    Int,
    String,
    Numeric,
    ArrayKey,
    Scalar,
    IntOrString,
    IntOrFloat,
}

#[derive(Debug, Clone, Copy)]
enum ConjunctRecipe {
    Named(&'static str),
    HasMethod(&'static str),
    HasProperty(&'static str),
}

#[derive(Debug, Clone, Copy)]
enum ClassStringRecipe {
    Class,
    Interface,
    Enum,
    Trait,
}

#[derive(Debug, Clone)]
enum TypeRecipe {
    Primitive(PrimitiveRecipe),
    LiteralInt(i64),
    RefinedInt(RefinedIntRecipe),
    LiteralString(String),
    RefinedString(RefinedStringRecipe),
    LiteralFloat(f64),
    UnspecifiedLiteralFloat,
    ClassObject(&'static str),
    Enumeration(&'static str),
    HasMethod(&'static str),
    HasProperty(&'static str),
    IntersectionObject(&'static str, ConjunctRecipe),
    Template(&'static str, &'static str, ConstraintRecipe),
    ClassString(ClassStringRecipe),
    List(Box<TypeRecipe>, bool),
    KeyedUnsealed(Box<TypeRecipe>, Box<TypeRecipe>),
    Iterable(Box<TypeRecipe>, Box<TypeRecipe>),
    Generic(&'static str, Box<TypeRecipe>),
    ObjectShape(Box<TypeRecipe>, Box<TypeRecipe>, bool, bool),
    Callable1(Box<TypeRecipe>, Box<TypeRecipe>),
    Callable2(Box<TypeRecipe>, Box<TypeRecipe>, Box<TypeRecipe>),
    Union(Vec<TypeRecipe>),
}

fn arb_variance() -> impl Strategy<Value = Variance> {
    prop_oneof![Just(Variance::Invariant), Just(Variance::Covariant), Just(Variance::Contravariant)]
}

fn arb_world() -> impl Strategy<Value = WorldRecipe> {
    let class_templates = CLASSES.iter().map(|_| proptest::collection::vec(arb_variance(), 1)).collect::<Vec<_>>();
    let edge_count = CLASSES.len() * (CLASSES.len() - 1) / 2;
    let edges = proptest::collection::vec(any::<bool>(), edge_count);
    let methods = proptest::collection::vec(any::<bool>(), CLASSES.len() * METHODS.len());
    let properties = proptest::collection::vec(any::<bool>(), CLASSES.len() * PROPERTIES.len());
    let finals = proptest::collection::vec(any::<bool>(), CLASSES.len());
    let sealed = proptest::collection::vec(proptest::option::weighted(0.999, any::<u8>()), CLASSES.len());

    (class_templates, edges, methods, properties, finals, sealed).prop_map(
        |(templates, edges, methods, properties, finals, sealed_markers)| WorldRecipe {
            templates,
            edges,
            methods,
            properties,
            finals,
            sealed_markers,
        },
    )
}

fn build_world<'arena>(recipe: &WorldRecipe) -> MockWorld<'arena> {
    let mut world = MockWorld::new();

    for (index, class) in CLASSES.iter().enumerate() {
        let variances = &recipe.templates[index];
        if variances.is_empty() {
            world.declare(class);
        } else {
            let templates: Vec<(&'static str, Variance)> =
                variances.iter().enumerate().map(|(position, variance)| (TEMPLATES[position], *variance)).collect();
            world.with_templates(class, &templates);
        }
    }

    // PHP classes are single-inheritance: a class extends at most one class.
    // The recipe's edge bits are an arbitrary relation, so honour only the
    // first parent chosen for each child and skip the rest. This keeps the
    // generated hierarchies to valid single-inheritance forests, where two
    // unrelated classes genuinely share no descendant.
    let mut edge_index = 0;
    for (i, child) in CLASSES.iter().enumerate() {
        let mut child_has_parent = false;
        for (j, parent) in CLASSES.iter().enumerate().skip(i + 1) {
            if recipe.edges[edge_index] && !child_has_parent {
                let parent_arity = recipe.templates[j].len();
                let parent_arguments: Vec<Type<'arena>> =
                    std::iter::repeat_n(well_known::TYPE_MIXED, parent_arity).collect();
                world.with_extended(child, parent, parent_arguments);
                child_has_parent = true;
            }

            edge_index += 1;
        }
    }

    for (i, class) in CLASSES.iter().enumerate() {
        for (j, method) in METHODS.iter().enumerate() {
            if recipe.methods[i * METHODS.len() + j] {
                world.with_method(class, method);
            }
        }
    }

    for (i, class) in CLASSES.iter().enumerate() {
        for (j, property) in PROPERTIES.iter().enumerate() {
            if recipe.properties[i * PROPERTIES.len() + j] {
                world.with_property(class, property, well_known::TYPE_MIXED);
            }
        }
    }

    for enumeration in ENUMS {
        world.with_pure_enum(enumeration);
    }

    for (i, class) in CLASSES.iter().enumerate() {
        if !recipe.finals[i] {
            continue;
        }

        let class_name = mago_oracle::name::Name::new(class.as_bytes());
        let has_descendants = CLASSES.iter().any(|other| {
            *other != *class && world.descends_from(mago_oracle::name::Name::new(other.as_bytes()), class_name)
        });
        if !has_descendants {
            world.with_final(class);
        }
    }

    for (i, class) in CLASSES.iter().enumerate() {
        let marker = match recipe.sealed_markers[i] {
            Some(value) if value < 64 => value,
            _ => continue,
        };

        let class_name = mago_oracle::name::Name::new(class.as_bytes());
        let direct_children: Vec<&'static str> = CLASSES
            .iter()
            .filter(|&&candidate| {
                if candidate == *class {
                    return false;
                }

                let candidate_name = mago_oracle::name::Name::new(candidate.as_bytes());
                if !world.descends_from(candidate_name, class_name) {
                    return false;
                }

                !CLASSES.iter().any(|&intermediate| {
                    intermediate != *class
                        && intermediate != candidate
                        && world.descends_from(candidate_name, mago_oracle::name::Name::new(intermediate.as_bytes()))
                        && world.descends_from(mago_oracle::name::Name::new(intermediate.as_bytes()), class_name)
                })
            })
            .copied()
            .collect();
        if direct_children.len() < 2 {
            continue;
        }

        let count = (marker as usize % 3) + 1;
        let inheritors: Vec<&'static str> =
            direct_children.iter().take(count.min(direct_children.len())).copied().collect();
        if inheritors.len() >= 2 {
            world.with_sealed(class, &inheritors);
        }
    }

    world
}

fn primitive_recipe() -> impl Strategy<Value = TypeRecipe> {
    prop_oneof![
        Just(PrimitiveRecipe::Int),
        Just(PrimitiveRecipe::String),
        Just(PrimitiveRecipe::Float),
        Just(PrimitiveRecipe::Bool),
        Just(PrimitiveRecipe::Null),
        Just(PrimitiveRecipe::Void),
        Just(PrimitiveRecipe::Mixed),
        Just(PrimitiveRecipe::Never),
        Just(PrimitiveRecipe::Object),
        Just(PrimitiveRecipe::Scalar),
        Just(PrimitiveRecipe::Numeric),
        Just(PrimitiveRecipe::ArrayKey),
    ]
    .prop_map(TypeRecipe::Primitive)
}

fn literal_int_recipe() -> impl Strategy<Value = TypeRecipe> {
    prop_oneof![Just(0i64), Just(1), Just(-1), Just(42), Just(-42)].prop_map(TypeRecipe::LiteralInt)
}

fn refined_int_recipe() -> impl Strategy<Value = TypeRecipe> {
    prop_oneof![
        Just(RefinedIntRecipe::Positive),
        Just(RefinedIntRecipe::Negative),
        Just(RefinedIntRecipe::NonNegative),
        Just(RefinedIntRecipe::NonPositive),
        Just(RefinedIntRecipe::Range(-10, 10)),
        Just(RefinedIntRecipe::Range(0, 100)),
        Just(RefinedIntRecipe::From(0)),
        Just(RefinedIntRecipe::To(0)),
    ]
    .prop_map(TypeRecipe::RefinedInt)
}

fn literal_string_recipe() -> impl Strategy<Value = TypeRecipe> {
    prop_oneof![
        Just("foo".to_owned()),
        Just("bar".to_owned()),
        Just(String::new()),
        Just("0".to_owned()),
        Just("hello world".to_owned()),
    ]
    .prop_map(TypeRecipe::LiteralString)
}

fn refined_string_recipe() -> impl Strategy<Value = TypeRecipe> {
    prop_oneof![
        Just(RefinedStringRecipe::NonEmpty),
        Just(RefinedStringRecipe::Numeric),
        Just(RefinedStringRecipe::Lower),
        Just(RefinedStringRecipe::Upper),
        Just(RefinedStringRecipe::Truthy),
        Just(RefinedStringRecipe::ClassString),
        Just(RefinedStringRecipe::InterfaceString),
        Just(RefinedStringRecipe::EnumString),
    ]
    .prop_map(TypeRecipe::RefinedString)
}

fn literal_float_recipe() -> impl Strategy<Value = TypeRecipe> {
    prop_oneof![
        Just(TypeRecipe::LiteralFloat(0.0)),
        Just(TypeRecipe::LiteralFloat(1.5)),
        Just(TypeRecipe::LiteralFloat(-1.5)),
        Just(TypeRecipe::LiteralFloat(42.0)),
        Just(TypeRecipe::UnspecifiedLiteralFloat),
    ]
}

fn intersection_object_recipe() -> impl Strategy<Value = TypeRecipe> {
    let heads = ["A", "B", "C"];
    let conjuncts = [
        ConjunctRecipe::Named("D"),
        ConjunctRecipe::Named("E"),
        ConjunctRecipe::HasMethod("doFoo"),
        ConjunctRecipe::HasProperty("id"),
    ];
    (proptest::sample::select(heads.to_vec()), proptest::sample::select(conjuncts.to_vec()))
        .prop_map(|(head, conjunct)| TypeRecipe::IntersectionObject(head, conjunct))
}

fn class_object_recipe() -> impl Strategy<Value = TypeRecipe> {
    proptest::sample::select(CLASSES.to_vec()).prop_map(TypeRecipe::ClassObject)
}

fn enum_recipe() -> impl Strategy<Value = TypeRecipe> {
    proptest::sample::select(ENUMS.to_vec()).prop_map(TypeRecipe::Enumeration)
}

fn has_method_recipe() -> impl Strategy<Value = TypeRecipe> {
    proptest::sample::select(METHODS.to_vec()).prop_map(TypeRecipe::HasMethod)
}

fn has_property_recipe() -> impl Strategy<Value = TypeRecipe> {
    proptest::sample::select(PROPERTIES.to_vec()).prop_map(TypeRecipe::HasProperty)
}

fn constraint_recipe() -> impl Strategy<Value = ConstraintRecipe> {
    prop_oneof![
        Just(ConstraintRecipe::Mixed),
        Just(ConstraintRecipe::Int),
        Just(ConstraintRecipe::String),
        Just(ConstraintRecipe::Numeric),
        Just(ConstraintRecipe::ArrayKey),
        Just(ConstraintRecipe::Scalar),
        Just(ConstraintRecipe::IntOrString),
        Just(ConstraintRecipe::IntOrFloat),
    ]
}

fn template_recipe() -> impl Strategy<Value = TypeRecipe> {
    let scopes = ["A", "B", "C"];
    (proptest::sample::select(scopes.to_vec()), proptest::sample::select(TEMPLATES.to_vec()), constraint_recipe())
        .prop_map(|(scope, name, constraint)| TypeRecipe::Template(scope, name, constraint))
}

fn class_string_recipe() -> impl Strategy<Value = TypeRecipe> {
    prop_oneof![
        Just(ClassStringRecipe::Class),
        Just(ClassStringRecipe::Interface),
        Just(ClassStringRecipe::Enum),
        Just(ClassStringRecipe::Trait),
    ]
    .prop_map(TypeRecipe::ClassString)
}

/// Valid PHP array-key types: keys of a real array are always `int|string`
/// (or literals / `array-key` thereof). The generator must not produce
/// object- or float-keyed arrays - those are not constructible in PHP, and
/// holding the lattice laws to such nonsensical shapes is meaningless.
fn arb_array_key() -> impl Strategy<Value = TypeRecipe> {
    prop_oneof![
        literal_int_recipe(),
        literal_string_recipe(),
        Just(TypeRecipe::Primitive(PrimitiveRecipe::Int)),
        Just(TypeRecipe::Primitive(PrimitiveRecipe::String)),
        Just(TypeRecipe::Primitive(PrimitiveRecipe::ArrayKey)),
    ]
}

fn arb_type() -> impl Strategy<Value = TypeRecipe> {
    let leaf = prop_oneof![
        4 => primitive_recipe(),
        2 => literal_int_recipe(),
        2 => refined_int_recipe(),
        2 => literal_string_recipe(),
        2 => refined_string_recipe(),
        1 => literal_float_recipe(),
        2 => class_object_recipe(),
        1 => enum_recipe(),
        1 => has_method_recipe(),
        1 => has_property_recipe(),
        2 => intersection_object_recipe(),
        2 => template_recipe(),
        1 => class_string_recipe(),
    ];

    leaf.prop_recursive(5, 64, 6, |inner| {
        prop_oneof![
            inner.clone().prop_map(|recipe| TypeRecipe::List(Box::new(recipe), false)),
            inner.clone().prop_map(|recipe| TypeRecipe::List(Box::new(recipe), true)),
            (arb_array_key(), inner.clone())
                .prop_map(|(key, value)| TypeRecipe::KeyedUnsealed(Box::new(key), Box::new(value))),
            (inner.clone(), inner.clone())
                .prop_map(|(key, value)| TypeRecipe::Iterable(Box::new(key), Box::new(value))),
            inner.clone().prop_map(|recipe| TypeRecipe::Generic("A", Box::new(recipe))),
            inner.clone().prop_map(|recipe| TypeRecipe::Generic("B", Box::new(recipe))),
            inner.clone().prop_map(|recipe| TypeRecipe::Generic("C", Box::new(recipe))),
            (inner.clone(), inner.clone(), any::<bool>(), any::<bool>()).prop_map(|(a, b, optional, sealed)| {
                TypeRecipe::ObjectShape(Box::new(a), Box::new(b), optional, sealed)
            }),
            (inner.clone(), inner.clone())
                .prop_map(|(parameter, result)| TypeRecipe::Callable1(Box::new(parameter), Box::new(result))),
            (inner.clone(), inner.clone(), inner.clone()).prop_map(|(first, second, result)| {
                TypeRecipe::Callable2(Box::new(first), Box::new(second), Box::new(result))
            }),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| TypeRecipe::Union(vec![a, b])),
            (inner.clone(), inner.clone(), inner.clone()).prop_map(|(a, b, c)| TypeRecipe::Union(vec![a, b, c])),
            (inner.clone(), inner.clone(), inner.clone(), inner)
                .prop_map(|(a, b, c, d)| TypeRecipe::Union(vec![a, b, c, d])),
        ]
    })
}

fn arb_world_and_type() -> impl Strategy<Value = (WorldRecipe, TypeRecipe)> {
    (arb_world(), arb_type())
}

fn arb_world_and_pair() -> impl Strategy<Value = (WorldRecipe, TypeRecipe, TypeRecipe)> {
    (arb_world(), arb_type(), arb_type())
}

fn arb_world_and_triple() -> impl Strategy<Value = (WorldRecipe, TypeRecipe, TypeRecipe, TypeRecipe)> {
    (arb_world(), arb_type(), arb_type(), arb_type())
}

struct Probe<'scratch, 'arena> {
    builder: TypeBuilder<'scratch, 'arena, LocalArena, LocalArena>,
    world: MockWorld<'arena>,
}

impl<'scratch, 'arena> Probe<'scratch, 'arena> {
    fn new(
        output: &'arena LocalArena,
        scratch: &'scratch LocalArena,
        world_recipe: &WorldRecipe,
    ) -> Probe<'scratch, 'arena> {
        Probe { builder: TypeBuilder::new(output, scratch), world: build_world(world_recipe) }
    }

    fn build(&mut self, recipe: &TypeRecipe) -> Type<'arena> {
        let atoms = self.atoms_of(recipe);

        self.builder.union_of(&atoms)
    }

    fn atoms_of(&mut self, recipe: &TypeRecipe) -> Vec<Atom<'arena>> {
        match recipe {
            TypeRecipe::Primitive(primitive) => vec![primitive_atom(*primitive)],
            TypeRecipe::LiteralInt(value) => vec![Atom::int_literal(*value)],
            TypeRecipe::RefinedInt(refined) => vec![self.refined_int_atom(*refined)],
            TypeRecipe::LiteralString(value) => vec![self.builder.string_literal(value.as_bytes())],
            TypeRecipe::RefinedString(refined) => vec![self.refined_string_atom(*refined)],
            TypeRecipe::LiteralFloat(value) => vec![Atom::float_literal(*value)],
            TypeRecipe::UnspecifiedLiteralFloat => vec![well_known::LITERAL_FLOAT],
            TypeRecipe::ClassObject(name) => vec![self.builder.object_named(name.as_bytes())],
            TypeRecipe::Enumeration(name) => vec![self.builder.enum_any(name.as_bytes())],
            TypeRecipe::HasMethod(name) => {
                let method_name = self.builder.name(name.as_bytes());

                vec![Atom::HasMethod(mago_oracle::ty::atom::payload::object::has_method::HasMethodAtom { method_name })]
            }
            TypeRecipe::HasProperty(name) => {
                let property_name = self.builder.name(name.as_bytes());

                vec![Atom::HasProperty(mago_oracle::ty::atom::payload::object::has_property::HasPropertyAtom {
                    property_name,
                })]
            }
            TypeRecipe::IntersectionObject(head, conjunct) => {
                let head_atom = self.builder.object_named(head.as_bytes());
                let conjunct_atom = self.conjunct_atom(*conjunct);

                vec![self.builder.intersected(head_atom, &[conjunct_atom])]
            }
            TypeRecipe::Template(scope, name, constraint) => {
                let constraint_type = self.constraint_type(*constraint);
                let parameter_name = self.builder.name(name.as_bytes());
                let class = self.builder.name(scope.as_bytes());

                vec![self.builder.generic_parameter(
                    mago_oracle::ty::atom::payload::generic_parameter::GenericParameterAtom {
                        name: parameter_name,
                        defining_entity: mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity::ClassLike(
                            class,
                        ),
                        constraint: constraint_type,
                    },
                )]
            }
            TypeRecipe::ClassString(kind) => vec![class_string_atom(*kind)],
            TypeRecipe::List(element, non_empty) => {
                let element_type = self.build(element);

                vec![self.builder.list_of(element_type, *non_empty)]
            }
            TypeRecipe::KeyedUnsealed(key, value) => {
                let key_type = self.build(key);
                let value_type = self.build(value);

                vec![self.builder.keyed_unsealed(key_type, value_type, false)]
            }
            TypeRecipe::Iterable(key, value) => {
                let key_type = self.build(key);
                let value_type = self.build(value);

                vec![
                    self.builder
                        .iterable(mago_oracle::ty::atom::payload::iterable::IterableAtom { key_type, value_type }),
                ]
            }
            TypeRecipe::Generic(name, argument) => {
                let argument_type = self.build(argument);
                let class_name = self.builder.name(name.as_bytes());
                let arguments = self.builder.types(&[argument_type]);

                vec![self.builder.object(mago_oracle::ty::atom::payload::object::named::ObjectAtom {
                    name: class_name,
                    type_arguments: Some(arguments),
                    flags: mago_flags::U8Flags::empty(),
                })]
            }
            TypeRecipe::ObjectShape(first, second, optional, sealed) => {
                let first_type = self.build(first);
                let second_type = self.build(second);
                let x = self.builder.name(b"x");
                let y = self.builder.name(b"y");
                let properties = self.builder.known_properties(&[
                    mago_oracle::ty::atom::payload::object::shape::KnownProperty {
                        name: x,
                        value: first_type,
                        optional: *optional,
                    },
                    mago_oracle::ty::atom::payload::object::shape::KnownProperty {
                        name: y,
                        value: second_type,
                        optional: false,
                    },
                ]);
                let mut flags = mago_flags::U8Flags::empty();
                flags.set_value(mago_oracle::ty::atom::payload::object::shape::ObjectShapeFlag::Sealed, *sealed);

                vec![self.builder.object_shape(mago_oracle::ty::atom::payload::object::shape::ObjectShapeAtom {
                    known_properties: Some(properties),
                    flags,
                })]
            }
            TypeRecipe::Callable1(parameter, result) => {
                let parameter_type = self.build(parameter);
                let return_type = self.build(result);

                vec![self.callable(&[parameter_type], return_type)]
            }
            TypeRecipe::Callable2(first, second, result) => {
                let first_type = self.build(first);
                let second_type = self.build(second);
                let return_type = self.build(result);

                vec![self.callable(&[first_type, second_type], return_type)]
            }
            TypeRecipe::Union(members) => {
                let mut atoms = Vec::new();
                for member in members {
                    atoms.extend(self.atoms_of(member));
                }

                atoms
            }
        }
    }

    fn callable(&mut self, parameters: &[Type<'arena>], return_type: Type<'arena>) -> Atom<'arena> {
        let entries: Vec<mago_oracle::ty::atom::payload::callable::Parameter<'arena>> = parameters
            .iter()
            .enumerate()
            .map(|(index, parameter_type)| {
                let name = self.builder.name(format!("p{index}").as_bytes());

                mago_oracle::ty::atom::payload::callable::Parameter {
                    name,
                    r#type: *parameter_type,
                    flags: mago_flags::U8Flags::empty(),
                }
            })
            .collect();
        let signature_parameters = self.builder.parameters(&entries);
        let signature = self.builder.signature(mago_oracle::ty::atom::payload::callable::Signature {
            parameters: Some(signature_parameters),
            return_type,
            throws: None,
            flags: mago_flags::U8Flags::empty(),
        });

        Atom::Callable(mago_oracle::ty::atom::payload::callable::CallableAtom::Signature(signature))
    }

    fn refined_int_atom(&mut self, refined: RefinedIntRecipe) -> Atom<'arena> {
        match refined {
            RefinedIntRecipe::Positive => well_known::POSITIVE_INT,
            RefinedIntRecipe::Negative => well_known::NEGATIVE_INT,
            RefinedIntRecipe::NonNegative => well_known::NON_NEGATIVE_INT,
            RefinedIntRecipe::NonPositive => well_known::NON_POSITIVE_INT,
            RefinedIntRecipe::Range(lower, upper) => self.builder.int_range(Some(lower), Some(upper)),
            RefinedIntRecipe::From(lower) => self.builder.int_range(Some(lower), None),
            RefinedIntRecipe::To(upper) => self.builder.int_range(None, Some(upper)),
        }
    }

    fn refined_string_atom(&self, refined: RefinedStringRecipe) -> Atom<'arena> {
        match refined {
            RefinedStringRecipe::NonEmpty => well_known::NON_EMPTY_STRING,
            RefinedStringRecipe::Numeric => well_known::NUMERIC_STRING,
            RefinedStringRecipe::Lower => well_known::LOWERCASE_STRING,
            RefinedStringRecipe::Upper => well_known::UPPERCASE_STRING,
            RefinedStringRecipe::Truthy => well_known::TRUTHY_STRING,
            RefinedStringRecipe::ClassString => well_known::CLASS_STRING,
            RefinedStringRecipe::InterfaceString => well_known::INTERFACE_STRING,
            RefinedStringRecipe::EnumString => well_known::ENUM_STRING,
        }
    }

    fn conjunct_atom(&mut self, conjunct: ConjunctRecipe) -> Atom<'arena> {
        match conjunct {
            ConjunctRecipe::Named(name) => self.builder.object_named(name.as_bytes()),
            ConjunctRecipe::HasMethod(name) => {
                let method_name = self.builder.name(name.as_bytes());

                Atom::HasMethod(mago_oracle::ty::atom::payload::object::has_method::HasMethodAtom { method_name })
            }
            ConjunctRecipe::HasProperty(name) => {
                let property_name = self.builder.name(name.as_bytes());

                Atom::HasProperty(mago_oracle::ty::atom::payload::object::has_property::HasPropertyAtom {
                    property_name,
                })
            }
        }
    }

    fn constraint_type(&mut self, constraint: ConstraintRecipe) -> Type<'arena> {
        match constraint {
            ConstraintRecipe::Mixed => well_known::TYPE_MIXED,
            ConstraintRecipe::Int => well_known::TYPE_INT,
            ConstraintRecipe::String => well_known::TYPE_STRING,
            ConstraintRecipe::Numeric => well_known::TYPE_NUMERIC,
            ConstraintRecipe::ArrayKey => well_known::TYPE_ARRAY_KEY,
            ConstraintRecipe::Scalar => well_known::TYPE_SCALAR,
            ConstraintRecipe::IntOrString => self.builder.union_of(&[well_known::INT, well_known::STRING]),
            ConstraintRecipe::IntOrFloat => self.builder.union_of(&[well_known::INT, well_known::FLOAT]),
        }
    }

    fn refines(&mut self, input: Type<'arena>, container: Type<'arena>) -> bool {
        let mut report = LatticeReport::new();

        refines(input, container, &self.world, LatticeOptions::default(), &mut report, &mut self.builder)
    }

    fn overlaps(&mut self, a: Type<'arena>, b: Type<'arena>) -> bool {
        let mut report = LatticeReport::new();

        overlaps(a, b, &self.world, LatticeOptions::default(), &mut report, &mut self.builder)
    }

    fn meet(&mut self, a: Type<'arena>, b: Type<'arena>) -> Type<'arena> {
        let mut report = LatticeReport::new();

        meet::compute(a, b, &self.world, LatticeOptions::default(), &mut report, &mut self.builder)
    }

    fn subtract(&mut self, a: Type<'arena>, b: Type<'arena>) -> Type<'arena> {
        let mut report = LatticeReport::new();

        subtract::compute(a, b, &self.world, LatticeOptions::default(), &mut report, &mut self.builder)
    }

    fn join(&mut self, a: Type<'arena>, b: Type<'arena>) -> Type<'arena> {
        let mut atoms: Vec<Atom<'arena>> = a.atoms.to_vec();
        atoms.extend_from_slice(b.atoms);
        let canonical = join::compute(&atoms, &mut self.builder);

        self.builder.union_of(&canonical)
    }

    fn negate(&mut self, ty: Type<'arena>) -> Type<'arena> {
        let negated = self.builder.negated(ty);

        self.builder.union_of(&[negated])
    }

    fn equivalent(&mut self, a: Type<'arena>, b: Type<'arena>) -> bool {
        self.refines(a, b) && self.refines(b, a)
    }

    fn type_is_value_never(&mut self, ty: Type<'arena>) -> bool {
        if ty.is_never() {
            return true;
        }

        if ty.atoms.is_empty() {
            return true;
        }

        let atoms = ty.atoms.to_vec();
        atoms.iter().all(|atom| {
            if self.atom_is_value_never(*atom) {
                return true;
            }

            let singleton = self.builder.union_of(&[*atom]);
            !self.overlaps(singleton, singleton)
        })
    }

    fn atom_is_value_never(&self, atom: Atom<'arena>) -> bool {
        let Atom::Intersected(payload) = atom else {
            return false;
        };
        let Atom::Object(head) = payload.head else {
            return false;
        };

        let mut classes: Vec<mago_oracle::name::Name<'arena>> = vec![head.name];
        for conjunct in payload.conjuncts {
            if let Atom::Object(object) = conjunct {
                classes.push(object.name);
            }
        }

        for (index, left) in classes.iter().enumerate() {
            for right in &classes[index + 1..] {
                if left.as_bytes() == right.as_bytes() {
                    continue;
                }

                if !self.world.descends_from(*left, *right) && !self.world.descends_from(*right, *left) {
                    return true;
                }
            }
        }

        false
    }

    fn type_has_imprecise_atom(&self, ty: Type<'arena>) -> bool {
        ty.atoms.iter().any(|atom| atom_is_imprecise(*atom))
    }
}

fn primitive_atom(primitive: PrimitiveRecipe) -> Atom<'static> {
    match primitive {
        PrimitiveRecipe::Int => well_known::INT,
        PrimitiveRecipe::String => well_known::STRING,
        PrimitiveRecipe::Float => well_known::FLOAT,
        PrimitiveRecipe::Bool => well_known::BOOL,
        PrimitiveRecipe::Null => well_known::NULL,
        PrimitiveRecipe::Void => well_known::VOID,
        PrimitiveRecipe::Mixed => well_known::MIXED,
        PrimitiveRecipe::Never => well_known::NEVER,
        PrimitiveRecipe::Object => well_known::OBJECT,
        PrimitiveRecipe::Scalar => well_known::SCALAR,
        PrimitiveRecipe::Numeric => well_known::NUMERIC,
        PrimitiveRecipe::ArrayKey => well_known::ARRAY_KEY,
    }
}

fn class_string_atom(kind: ClassStringRecipe) -> Atom<'static> {
    match kind {
        ClassStringRecipe::Class => well_known::CLASS_STRING,
        ClassStringRecipe::Interface => well_known::INTERFACE_STRING,
        ClassStringRecipe::Enum => well_known::ENUM_STRING,
        ClassStringRecipe::Trait => well_known::TRAIT_STRING,
    }
}

fn atom_is_imprecise(atom: Atom<'_>) -> bool {
    if matches!(
        atom.kind(),
        AtomKind::GenericParameter
            | AtomKind::Object
            | AtomKind::HasMethod
            | AtomKind::HasProperty
            | AtomKind::ObjectShape
            | AtomKind::Callable
            | AtomKind::Iterable
            | AtomKind::ClassLikeString
            | AtomKind::Scalar
            | AtomKind::Numeric
            | AtomKind::ArrayKey
            | AtomKind::Mixed
            | AtomKind::ObjectAny
    ) || atom_is_refined_string(atom)
        || atom_is_empty_array_singleton(atom)
    {
        return true;
    }

    match atom {
        Atom::List(payload) => type_has_imprecise_atom_free(payload.element_type),
        Atom::Array(payload) => {
            payload.key_param.is_some_and(type_has_imprecise_atom_free)
                || payload.value_param.is_some_and(type_has_imprecise_atom_free)
        }
        _ => false,
    }
}

fn type_has_imprecise_atom_free(ty: Type<'_>) -> bool {
    ty.atoms.iter().any(|atom| atom_is_imprecise(*atom))
}

fn atom_is_refined_string(atom: Atom<'_>) -> bool {
    let Atom::String(payload) = atom else {
        return false;
    };

    if matches!(payload.literal, StringLiteral::Value(_)) {
        return false;
    }

    !payload.flags.is_empty() || !matches!(payload.casing, StringCasing::Unspecified)
}

fn atom_is_empty_array_singleton(atom: Atom<'_>) -> bool {
    match atom {
        Atom::List(payload) => {
            !payload.flags.contains(mago_oracle::ty::atom::payload::array::ListFlag::NonEmpty)
                && payload.element_type.is_never()
                && payload.known_elements.is_none()
        }
        Atom::Array(payload) => {
            if payload.flags.contains(mago_oracle::ty::atom::payload::array::ArrayFlag::NonEmpty) {
                return false;
            }

            let value_is_never = match payload.value_param {
                Some(value) => value.is_never(),
                None => true,
            };
            if !value_is_never {
                return false;
            }

            match payload.known_items {
                None => true,
                Some(items) => items.iter().all(|item| item.optional),
            }
        }
        _ => false,
    }
}

fn env_cases() -> u32 {
    std::env::var("MAGO_PROPTEST_CASES").ok().and_then(|value| value.parse().ok()).unwrap_or(256)
}

fn env_max_shrink_iters() -> u32 {
    std::env::var("MAGO_PROPTEST_MAX_SHRINK_ITERS").ok().and_then(|value| value.parse().ok()).unwrap_or(1000)
}

fn env_max_global_rejects(cases: u32) -> u32 {
    std::env::var("MAGO_PROPTEST_MAX_GLOBAL_REJECTS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| 1024.max(cases.saturating_mul(4)))
}

fn with_probe<F>(world_recipe: &WorldRecipe, run: F) -> Result<(), TestCaseError>
where
    F: for<'scratch, 'arena> FnOnce(&mut Probe<'scratch, 'arena>) -> Result<(), TestCaseError>,
{
    let output = LocalArena::new();
    let scratch = LocalArena::new();
    let mut probe = Probe::new(&output, &scratch, world_recipe);

    run(&mut probe)
}

proptest! {
    #![proptest_config({
        let cases = env_cases();
        ProptestConfig {
            cases,
            max_shrink_iters: env_max_shrink_iters(),
            max_global_rejects: env_max_global_rejects(cases),
            failure_persistence: None,
            ..ProptestConfig::default()
        }
    })]

    #[test]
    fn refines_is_reflexive((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            prop_assert!(probe.refines(a, a), "refines(a, a) should be true");
            Ok(())
        })?;
    }

    #[test]
    fn refines_bottom_axiom((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            prop_assert!(probe.refines(well_known::TYPE_NEVER, a), "NEVER must refine a");
            Ok(())
        })?;
    }

    #[test]
    fn refines_top_axiom((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            prop_assert!(probe.refines(a, well_known::TYPE_MIXED), "a must refine MIXED");
            Ok(())
        })?;
    }

    #[test]
    fn refines_is_transitive((world, a, b, c) in arb_world_and_triple()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let c = probe.build(&c);
            if probe.refines(a, b) && probe.refines(b, c) {
                prop_assert!(probe.refines(a, c), "transitivity");
            }
            Ok(())
        })?;
    }

    #[test]
    fn overlaps_is_symmetric((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            prop_assert_eq!(probe.overlaps(a, b), probe.overlaps(b, a), "overlaps should be symmetric");
            Ok(())
        })?;
    }

    #[test]
    fn overlaps_is_reflexive_for_non_bottom((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            if !a.is_never() {
                let atoms = a.atoms.to_vec();
                let all_uninhabited = atoms.iter().all(|atom| {
                    let singleton = probe.builder.union_of(&[*atom]);
                    !probe.overlaps(singleton, singleton)
                });
                if all_uninhabited {
                    return Ok(());
                }

                prop_assert!(probe.overlaps(a, a), "non-bottom a must overlap itself");
            }
            Ok(())
        })?;
    }

    #[test]
    fn refines_implies_overlaps((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_is_value_never(a) {
                return Ok(());
            }
            if probe.refines(a, b) {
                prop_assert!(probe.overlaps(a, b), "refines implies overlaps for non-bottom");
            }
            Ok(())
        })?;
    }

    #[test]
    fn meet_is_idempotent((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let m = probe.meet(a, a);
            prop_assert!(probe.refines(m, a), "meet(a, a) should refine a");
            prop_assert!(probe.refines(a, m), "a should refine meet(a, a)");
            Ok(())
        })?;
    }

    #[test]
    fn meet_is_commutative((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let ab = probe.meet(a, b);
            let ba = probe.meet(b, a);
            prop_assert!(probe.refines(ab, ba), "meet(a, b) <: meet(b, a)");
            prop_assert!(probe.refines(ba, ab), "meet(b, a) <: meet(a, b)");
            Ok(())
        })?;
    }

    #[test]
    fn meet_is_lower_bound((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let m = probe.meet(a, b);
            prop_assert!(probe.refines(m, a), "meet(a, b) should refine a");
            prop_assert!(probe.refines(m, b), "meet(a, b) should refine b");
            Ok(())
        })?;
    }

    #[test]
    fn meet_with_mixed_is_identity((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            if probe.type_is_value_never(a) {
                return Ok(());
            }
            let m = probe.meet(a, well_known::TYPE_MIXED);
            prop_assert!(probe.refines(m, a));
            prop_assert!(probe.refines(a, m));
            Ok(())
        })?;
    }

    #[test]
    fn subtract_with_never_is_identity((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let s = probe.subtract(a, well_known::TYPE_NEVER);
            prop_assert!(probe.refines(s, a));
            prop_assert!(probe.refines(a, s));
            Ok(())
        })?;
    }

    #[test]
    fn subtract_self_is_never((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let s = probe.subtract(a, a);
            prop_assert!(probe.refines(s, well_known::TYPE_NEVER));
            Ok(())
        })?;
    }

    #[test]
    fn subtract_is_sound((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let s = probe.subtract(a, b);
            prop_assert!(probe.refines(s, a), "subtract(a, b) must refine a");
            Ok(())
        })?;
    }

    #[test]
    fn meet_when_overlapping_is_non_empty((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if !probe.overlaps(a, b) {
                return Ok(());
            }
            let has_generic = a.atoms.iter().chain(b.atoms.iter()).any(|atom| atom.kind() == AtomKind::GenericParameter);
            prop_assume!(!has_generic);
            let has_same_kind = a.atoms.iter().any(|left| b.atoms.iter().any(|right| left.kind() == right.kind()));
            prop_assume!(has_same_kind);
            let m = probe.meet(a, b);
            prop_assert!(!m.is_never(), "meet returned NEVER despite overlap");
            Ok(())
        })?;
    }

    #[test]
    fn meet_subtract_partition((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let m = probe.meet(a, b);
            let s = probe.subtract(a, b);
            let mut atoms: Vec<Atom<'_>> = m.atoms.to_vec();
            atoms.extend_from_slice(s.atoms);
            let union = probe.builder.union_of(&atoms);
            prop_assert!(probe.refines(a, union), "A should refine meet(A, B) union subtract(A, B)");
            Ok(())
        })?;
    }

    #[test]
    fn subtract_disjoint_is_identity((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if !a.is_never() && !probe.overlaps(a, a) {
                return Ok(());
            }
            if probe.overlaps(a, b) {
                return Ok(());
            }
            let s = probe.subtract(a, b);
            prop_assert!(probe.refines(s, a) && probe.refines(a, s), "disjoint subtract should be identity");
            Ok(())
        })?;
    }

    #[test]
    fn subtract_when_subset_is_empty((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if a.is_never() || !probe.refines(a, b) {
                return Ok(());
            }
            let s = probe.subtract(a, b);
            prop_assert!(probe.refines(s, well_known::TYPE_NEVER), "subtract(A, B) should be never when A <: B");
            Ok(())
        })?;
    }

    #[test]
    fn meet_then_subtract_same_is_empty((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let m = probe.meet(a, b);
            if m.is_never() {
                return Ok(());
            }
            let s = probe.subtract(m, b);
            prop_assert!(probe.refines(s, well_known::TYPE_NEVER), "subtract(meet(A, B), B) should be never");
            Ok(())
        })?;
    }

    #[test]
    fn structural_join_is_idempotent_at_atom_level((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let atoms = a.atoms.to_vec();
            let options = JoinOptions::structural();
            let canonical = join::compute_with(&atoms, &options, &mut probe.builder);
            let rebuilt = probe.builder.union_of(&canonical);
            prop_assert!(probe.refines(rebuilt, a), "rebuilt should refine original");
            prop_assert!(probe.refines(a, rebuilt), "original should refine rebuilt");
            Ok(())
        })?;
    }

    #[test]
    fn canonical_join_widens_or_preserves((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let atoms = a.atoms.to_vec();
            let canonical = join::compute(&atoms, &mut probe.builder);
            let rebuilt = probe.builder.union_of(&canonical);
            prop_assert!(probe.refines(a, rebuilt), "original should refine canonical");
            Ok(())
        })?;
    }

    #[test]
    fn join_with_mixed_absorbs((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let mut atoms: Vec<Atom<'_>> = a.atoms.to_vec();
            atoms.push(well_known::MIXED);
            let canonical = join::compute(&atoms, &mut probe.builder);
            prop_assert_eq!(canonical.as_slice(), [well_known::MIXED].as_slice());
            Ok(())
        })?;
    }

    #[test]
    fn meet_with_never_is_never((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let left = probe.meet(a, well_known::TYPE_NEVER);
            prop_assert!(left.is_never(), "meet(a, NEVER) should be NEVER");
            let right = probe.meet(well_known::TYPE_NEVER, a);
            prop_assert!(right.is_never(), "meet(NEVER, a) should be NEVER");
            Ok(())
        })?;
    }

    #[test]
    fn meet_is_associative_lower_bound((world, a, b, c) in arb_world_and_triple()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let c = probe.build(&c);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) || probe.type_has_imprecise_atom(c) {
                return Ok(());
            }
            let ab = probe.meet(a, b);
            let left = probe.meet(ab, c);
            let bc = probe.meet(b, c);
            let right = probe.meet(a, bc);
            prop_assert!(probe.refines(left, a));
            prop_assert!(probe.refines(left, b));
            prop_assert!(probe.refines(left, c));
            prop_assert!(probe.refines(right, a));
            prop_assert!(probe.refines(right, b));
            prop_assert!(probe.refines(right, c));
            Ok(())
        })?;
    }

    #[test]
    fn meet_monotonic_in_rhs((world, a, b, c) in arb_world_and_triple()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let c = probe.build(&c);
            if !probe.refines(b, c) {
                return Ok(());
            }
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) || probe.type_has_imprecise_atom(c) {
                return Ok(());
            }
            let ab = probe.meet(a, b);
            let ac = probe.meet(a, c);
            prop_assert!(probe.refines(ab, ac), "monotonicity: b<:c implies meet(a,b)<:meet(a,c)");
            Ok(())
        })?;
    }

    #[test]
    fn subtract_monotonic_in_rhs((world, a, b, c) in arb_world_and_triple()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let c = probe.build(&c);
            if !probe.refines(b, c) {
                return Ok(());
            }
            let ac = probe.subtract(a, c);
            let ab = probe.subtract(a, b);
            prop_assert!(probe.refines(ac, ab), "anti-monotonicity: b<:c implies (a\\c)<:(a\\b)");
            Ok(())
        })?;
    }

    #[test]
    fn subtract_is_idempotent((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let s1 = probe.subtract(a, b);
            let s2 = probe.subtract(s1, b);
            prop_assert!(probe.refines(s2, s1), "(a\\b)\\b should refine a\\b");
            prop_assert!(probe.refines(s1, s2), "a\\b should refine (a\\b)\\b");
            Ok(())
        })?;
    }

    #[test]
    fn meet_idempotent_left((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let m1 = probe.meet(a, b);
            let m2 = probe.meet(m1, b);
            prop_assert!(probe.refines(m2, m1));
            prop_assert!(probe.refines(m1, m2));
            Ok(())
        })?;
    }

    #[test]
    fn meet_implies_refines_both((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let m = probe.meet(a, b);
            if !m.is_never() {
                prop_assert!(probe.refines(m, a), "meet result must refine a");
                prop_assert!(probe.refines(m, b), "meet result must refine b");
            }
            Ok(())
        })?;
    }

    #[test]
    fn meet_overlaps_iff_non_never((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if !a.is_never() && !probe.overlaps(a, a) {
                return Ok(());
            }
            if !b.is_never() && !probe.overlaps(b, b) {
                return Ok(());
            }
            let m = probe.meet(a, b);
            if !m.is_never() {
                prop_assert!(probe.overlaps(a, b), "non-never meet should imply overlap");
            }
            Ok(())
        })?;
    }

    #[test]
    fn refines_implies_meet_is_input((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if !probe.refines(a, b) {
                return Ok(());
            }
            let m = probe.meet(a, b);
            prop_assert!(probe.refines(m, a) && probe.refines(a, m), "a<:b implies meet(a,b) equivalent a");
            Ok(())
        })?;
    }

    #[test]
    fn join_is_upper_bound((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let joined = probe.join(a, b);
            prop_assert!(probe.refines(a, joined), "a should refine join(a,b)");
            prop_assert!(probe.refines(b, joined), "b should refine join(a,b)");
            Ok(())
        })?;
    }

    #[test]
    fn join_with_never_is_identity((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let mut atoms: Vec<Atom<'_>> = a.atoms.to_vec();
            atoms.push(well_known::NEVER);
            let options = JoinOptions::structural();
            let canonical = join::compute_with(&atoms, &options, &mut probe.builder);
            let rebuilt = probe.builder.union_of(&canonical);
            prop_assert!(probe.refines(rebuilt, a));
            prop_assert!(probe.refines(a, rebuilt));
            Ok(())
        })?;
    }

    #[test]
    fn refines_is_antisymmetric_modulo_equivalence((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.refines(a, b) && probe.refines(b, a) {
                let mab = probe.meet(a, b);
                prop_assert!(probe.refines(mab, a));
                prop_assert!(probe.refines(a, mab));
                prop_assert!(probe.refines(mab, b));
                prop_assert!(probe.refines(b, mab));
            }
            Ok(())
        })?;
    }

    #[test]
    fn overlaps_with_never_is_false((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            prop_assert!(!probe.overlaps(a, well_known::TYPE_NEVER), "nothing overlaps NEVER");
            prop_assert!(!probe.overlaps(well_known::TYPE_NEVER, a), "NEVER overlaps nothing");
            Ok(())
        })?;
    }

    #[test]
    fn disjoint_implies_meet_never((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_is_value_never(a) || probe.type_is_value_never(b) {
                return Ok(());
            }
            if probe.overlaps(a, b) {
                return Ok(());
            }
            let m = probe.meet(a, b);
            prop_assert!(m.is_never(), "disjoint inputs must meet to NEVER");
            Ok(())
        })?;
    }

    #[test]
    fn double_subtract_with_swapped_args_equivalent((world, a, b, c) in arb_world_and_triple()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let c = probe.build(&c);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) || probe.type_has_imprecise_atom(c) {
                return Ok(());
            }
            let ab = probe.subtract(a, b);
            let bc = probe.subtract(ab, c);
            let ac = probe.subtract(a, c);
            let cb = probe.subtract(ac, b);
            prop_assert!(probe.refines(bc, cb), "(a\\b)\\c should refine (a\\c)\\b");
            prop_assert!(probe.refines(cb, bc), "(a\\c)\\b should refine (a\\b)\\c");
            Ok(())
        })?;
    }

    #[test]
    fn meet_refines_join((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let m = probe.meet(a, b);
            let joined = probe.join(a, b);
            prop_assert!(probe.refines(m, joined), "meet refines join");
            Ok(())
        })?;
    }

    #[test]
    fn subtract_anti_monotonic_in_lhs((world, a, b, c) in arb_world_and_triple()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let c = probe.build(&c);
            if !probe.refines(a, b) {
                return Ok(());
            }
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) || probe.type_has_imprecise_atom(c) {
                return Ok(());
            }
            let ac = probe.subtract(a, c);
            let bc = probe.subtract(b, c);
            prop_assert!(probe.refines(ac, bc), "a<:b implies (a\\c)<:(b\\c)");
            Ok(())
        })?;
    }

    #[test]
    fn join_with_self_widens_to_at_least_a((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let joined = probe.join(a, a);
            prop_assert!(probe.refines(a, joined), "a <: join(a,a)");
            Ok(())
        })?;
    }

    #[test]
    fn join_is_commutative((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let ab = probe.join(a, b);
            let ba = probe.join(b, a);
            prop_assert!(probe.refines(ab, ba), "join(a,b) <: join(b,a)");
            prop_assert!(probe.refines(ba, ab), "join(b,a) <: join(a,b)");
            Ok(())
        })?;
    }

    #[test]
    fn subtract_with_mixed_is_never((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let r = probe.subtract(a, well_known::TYPE_MIXED);
            prop_assert!(r.is_never(), "a \\ mixed should be NEVER");
            Ok(())
        })?;
    }

    #[test]
    fn refines_implies_meet_value_equivalent((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if !probe.refines(a, b) {
                return Ok(());
            }
            if a.is_never() || probe.type_is_value_never(a) {
                return Ok(());
            }
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let m = probe.meet(a, b);
            prop_assert!(probe.refines(a, m) && probe.refines(m, a), "a<:b implies meet(a,b) equivalent a");
            Ok(())
        })?;
    }

    #[test]
    fn refines_implies_subtract_outcome_impossible((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if !probe.refines(a, b) {
                return Ok(());
            }
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let mut report = LatticeReport::new();
            let outcome = subtract::narrow(a, b, &probe.world, LatticeOptions::default(), &mut report, &mut probe.builder);
            prop_assert!(
                matches!(outcome, subtract::SubtractOutcome::Impossible),
                "a<:b implies subtract::narrow Impossible"
            );
            Ok(())
        })?;
    }

    #[test]
    fn disjoint_implies_subtract_value_equivalent((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if a.is_never() || probe.type_is_value_never(a) {
                return Ok(());
            }
            if probe.type_is_value_never(b) {
                return Ok(());
            }
            if probe.overlaps(a, b) {
                return Ok(());
            }
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let s = probe.subtract(a, b);
            prop_assert!(probe.refines(a, s) && probe.refines(s, a), "a # b implies (a \\ b) equivalent a");
            Ok(())
        })?;
    }

    #[test]
    fn negated_meet_with_self_is_never((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            if probe.type_has_imprecise_atom(a) || probe.type_is_value_never(a) {
                return Ok(());
            }
            let negated = probe.negate(a);
            let m = probe.meet(a, negated);
            prop_assert!(m.is_never(), "meet(T, !T) should be NEVER");
            Ok(())
        })?;
    }

    #[test]
    fn negated_meet_equals_subtract((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let negated = probe.negate(b);
            let lhs = probe.meet(a, negated);
            let rhs = probe.subtract(a, b);
            prop_assert!(probe.refines(lhs, rhs) && probe.refines(rhs, lhs), "meet(a, !b) equivalent subtract(a, b)");
            Ok(())
        })?;
    }

    #[test]
    fn negated_subtract_equals_meet((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) {
                return Ok(());
            }
            let negated = probe.negate(b);
            let lhs = probe.subtract(a, negated);
            let rhs = probe.meet(a, b);
            prop_assert!(probe.refines(lhs, rhs) && probe.refines(rhs, lhs), "subtract(a, !b) equivalent meet(a, b)");
            Ok(())
        })?;
    }

    #[test]
    fn double_negation_value_equal((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            if probe.type_has_imprecise_atom(a) {
                return Ok(());
            }
            let once = probe.negate(a);
            let twice = probe.negate(once);
            prop_assert!(probe.refines(a, twice) && probe.refines(twice, a), "!!T equivalent T value-wise");
            Ok(())
        })?;
    }

    #[test]
    fn negated_refines_iff_no_overlap((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if probe.type_has_imprecise_atom(a) || probe.type_has_imprecise_atom(b) || probe.type_is_value_never(a) {
                return Ok(());
            }
            let negated = probe.negate(b);
            let refines_negated = probe.refines(a, negated);
            let no_overlap = !probe.overlaps(a, b);
            prop_assert_eq!(refines_negated, no_overlap, "X <: !T iff !overlaps(X, T)");
            Ok(())
        })?;
    }

    #[test]
    fn copy_into_preserves_refines_verdict((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let original = probe.refines(a, b);

            let second_arena = LocalArena::new();
            let copied_a = a.copy_into(&second_arena);
            let copied_b = b.copy_into(&second_arena);
            prop_assert_eq!(copied_a, a, "CopyInto must preserve structural equality");
            prop_assert_eq!(copied_b, b, "CopyInto must preserve structural equality");

            let reimported_a = probe.builder.import(copied_a);
            let reimported_b = probe.builder.import(copied_b);
            prop_assert!(reimported_a.ptr_eq(&a), "import after copy must round-trip to the original allocation");
            prop_assert!(reimported_b.ptr_eq(&b), "import after copy must round-trip to the original allocation");

            prop_assert_eq!(probe.refines(reimported_a, reimported_b), original, "refines verdict must survive a copy round-trip");
            Ok(())
        })?;
    }

    #[test]
    fn import_is_idempotent((world, a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);

            let second_arena = LocalArena::new();
            let copied = a.copy_into(&second_arena);

            let first = probe.builder.import(copied);
            let second = probe.builder.import(copied);
            prop_assert!(first.ptr_eq(&second), "import(x) must be pointer-idempotent");
            prop_assert!(first.ptr_eq(&a), "importing a copy must return the original allocation");
            Ok(())
        })?;
    }

    #[test]
    fn lattice_pair_laws_hold((world, a, b) in arb_world_and_pair()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            if let Err(violation) = check_lattice_pair_laws(probe, a, b) {
                prop_assert!(false, "{}", violation);
            }
            Ok(())
        })?;
    }

    #[test]
    fn lattice_triple_laws_hold((world, a, b, c) in arb_world_and_triple()) {
        with_probe(&world, |probe| {
            let a = probe.build(&a);
            let b = probe.build(&b);
            let c = probe.build(&c);
            if let Err(violation) = check_lattice_triple_laws(probe, a, b, c) {
                prop_assert!(false, "{}", violation);
            }
            Ok(())
        })?;
    }

    #[test]
    fn sealed_full_cover_subtract_is_never((world, _a) in arb_world_and_type()) {
        with_probe(&world, |probe| {
            for &class_name in CLASSES {
                let name = mago_oracle::name::Name::new(class_name.as_bytes());
                let inheritors = match probe.world.sealed_direct_inheritors(name) {
                    Some(list) => list.iter().map(|inheritor| inheritor.as_bytes().to_vec()).collect::<Vec<_>>(),
                    None => continue,
                };
                if inheritors.is_empty() {
                    continue;
                }

                let head = probe.builder.object_named(class_name.as_bytes());
                let mut negations: Vec<Atom<'_>> = Vec::with_capacity(inheritors.len());
                for inheritor in &inheritors {
                    let inheritor_atom = probe.builder.object_named(inheritor);
                    let inheritor_type = probe.builder.union_of(&[inheritor_atom]);
                    negations.push(probe.builder.negated(inheritor_type));
                }

                let sealed_no_inheritors = probe.builder.intersected(head, &negations);
                let sealed_type = probe.builder.union_of(&[sealed_no_inheritors]);
                let m = probe.meet(sealed_type, well_known::TYPE_MIXED);
                prop_assert!(m.is_never(), "sealed full cover must be never for {}", class_name);
            }
            Ok(())
        })?;
    }
}

fn check_lattice_pair_laws<'arena>(
    probe: &mut Probe<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
) -> Result<(), String> {
    if !probe.refines(a, a) {
        return Err(format!("refl: a !<: a; a={a}"));
    }
    if !probe.refines(b, b) {
        return Err(format!("refl: b !<: b; b={b}"));
    }
    if !probe.refines(well_known::TYPE_NEVER, a) {
        return Err(format!("bottom: NEVER !<: a={a}"));
    }
    if !probe.refines(a, well_known::TYPE_MIXED) {
        return Err(format!("top: a={a} !<: MIXED"));
    }

    let ab = probe.meet(a, b);
    let ba = probe.meet(b, a);
    if !probe.equivalent(ab, ba) {
        return Err(format!("meet not commutative: meet(a,b)={ab}, meet(b,a)={ba}"));
    }
    if !probe.refines(ab, a) {
        return Err(format!("GLB: meet(a,b)={ab} !<: a={a}"));
    }
    if !probe.refines(ab, b) {
        return Err(format!("GLB: meet(a,b)={ab} !<: b={b}"));
    }

    let aub = probe.join(a, b);
    let bua = probe.join(b, a);
    if !probe.refines(a, aub) {
        return Err(format!("LUB: a={a} !<: join(a,b)={aub}"));
    }
    if !probe.refines(b, aub) {
        return Err(format!("LUB: b={b} !<: join(a,b)={aub}"));
    }
    if !probe.refines(a, bua) {
        return Err(format!("LUB symmetric: a={a} !<: join(b,a)={bua}"));
    }
    if !probe.refines(b, bua) {
        return Err(format!("LUB symmetric: b={b} !<: join(b,a)={bua}"));
    }

    let a_minus_b = probe.subtract(a, b);
    if !probe.refines(a_minus_b, a) {
        return Err(format!("subtract bound: a\\b={a_minus_b} !<: a={a}"));
    }

    let aa = probe.meet(a, a);
    if !probe.equivalent(aa, a) {
        return Err(format!("meet idempotence: meet(a,a)={aa}, expected a={a}"));
    }

    let aja = probe.join(a, a);
    if !probe.refines(a, aja) {
        return Err(format!("join LUB self: a={a} !<: join(a,a)={aja}"));
    }

    let a_mix = probe.meet(a, well_known::TYPE_MIXED);
    if !probe.equivalent(a_mix, a) {
        return Err(format!("meet identity: meet(a, MIXED)={a_mix}, expected a={a}"));
    }

    let a_nev = probe.meet(a, well_known::TYPE_NEVER);
    if !a_nev.is_never() {
        return Err(format!("meet absorb: meet(a, NEVER)={a_nev}, expected NEVER; a={a}"));
    }

    let a_join_nev = probe.join(a, well_known::TYPE_NEVER);
    if !probe.refines(a, a_join_nev) {
        return Err(format!("join identity bound: a={a} !<: join(a, NEVER)={a_join_nev}"));
    }

    let a_join_mix = probe.join(a, well_known::TYPE_MIXED);
    if !probe.equivalent(a_join_mix, well_known::TYPE_MIXED) {
        return Err(format!("join absorb: join(a, MIXED)={a_join_mix}, expected MIXED; a={a}"));
    }

    let a_minus_nev = probe.subtract(a, well_known::TYPE_NEVER);
    if !probe.equivalent(a_minus_nev, a) {
        return Err(format!("subtract identity: a\\NEVER={a_minus_nev}, expected a={a}"));
    }

    let a_minus_mix = probe.subtract(a, well_known::TYPE_MIXED);
    if !a_minus_mix.is_never() {
        return Err(format!("subtract absorb: a\\MIXED={a_minus_mix}, expected NEVER; a={a}"));
    }

    let a_minus_a = probe.subtract(a, a);
    if !a_minus_a.is_never() {
        return Err(format!("subtract self: a\\a={a_minus_a}, expected NEVER; a={a}"));
    }

    let overlap_ab = probe.overlaps(a, b);
    let overlap_ba = probe.overlaps(b, a);
    if overlap_ab != overlap_ba {
        return Err(format!("overlaps not symmetric: overlaps(a,b)={overlap_ab}, overlaps(b,a)={overlap_ba}"));
    }

    if probe.refines(a, b) && !probe.equivalent(ab, a) {
        return Err(format!("a<:b implies meet(a,b) equivalent a: meet={ab}, a={a}"));
    }

    Ok(())
}

fn check_lattice_triple_laws<'arena>(
    probe: &mut Probe<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    c: Type<'arena>,
) -> Result<(), String> {
    let ab = probe.meet(a, b);
    let left = probe.meet(ab, c);
    let bc = probe.meet(b, c);
    let right = probe.meet(a, bc);

    for (label, ty) in [("(a&b)&c", left), ("a&(b&c)", right)] {
        if !probe.refines(ty, a) {
            return Err(format!("meet GLB ternary: {label}={ty} !<: a={a}"));
        }
        if !probe.refines(ty, b) {
            return Err(format!("meet GLB ternary: {label}={ty} !<: b={b}"));
        }
        if !probe.refines(ty, c) {
            return Err(format!("meet GLB ternary: {label}={ty} !<: c={c}"));
        }
    }

    let join_ab = probe.join(a, b);
    let join_left = probe.join(join_ab, c);
    let join_bc = probe.join(b, c);
    let join_right = probe.join(a, join_bc);

    for (label, ty) in [("(a|b)|c", join_left), ("a|(b|c)", join_right)] {
        if !probe.refines(a, ty) {
            return Err(format!("join LUB ternary: a={a} !<: {label}={ty}"));
        }
        if !probe.refines(b, ty) {
            return Err(format!("join LUB ternary: b={b} !<: {label}={ty}"));
        }
        if !probe.refines(c, ty) {
            return Err(format!("join LUB ternary: c={c} !<: {label}={ty}"));
        }
    }

    if probe.refines(a, b) && probe.refines(b, c) && !probe.refines(a, c) {
        return Err(format!("refines transitivity: a<:b={a}<:{b}, b<:c, but a !<: c={c}"));
    }

    Ok(())
}
