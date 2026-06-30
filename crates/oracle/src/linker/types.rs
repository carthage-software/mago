use std::num::NonZeroU32;

use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_flags::U8Flags;
use mago_hir::ir::item::annotation::generics::TypeParameterDefiningEntity;
use mago_hir::ir::r#type::Type as HirType;
use mago_hir::ir::r#type::TypeKind as HirTypeKind;
use mago_hir::ir::r#type::annotation::CallableTypeAnnotation;
use mago_hir::ir::r#type::annotation::CallableTypeKind;
use mago_hir::ir::r#type::annotation::FloatLiteral;
use mago_hir::ir::r#type::annotation::GlobalSelector;
use mago_hir::ir::r#type::annotation::IntLiteral;
use mago_hir::ir::r#type::annotation::MemberReferenceSelector;
use mago_hir::ir::r#type::annotation::PropertiesOfFilter;
use mago_hir::ir::r#type::annotation::ReferenceKind;
use mago_hir::ir::r#type::annotation::ShapeTypeAnnotation;
use mago_hir::ir::r#type::annotation::ShapeTypeAnnotationKey;
use mago_hir::ir::r#type::annotation::StringCasing as HirStringCasing;
use mago_hir::ir::r#type::annotation::StringLiteral as HirStringLiteral;
use mago_hir::ir::r#type::annotation::StringTypeAnnotation;
use mago_hir::ir::r#type::annotation::TypeAnnotation;
use mago_hir::ir::r#type::annotation::TypeAnnotationKind;

use crate::symbol::class_like::ClassLikeKind;
use crate::symbol::class_like::part::visibility::Visibility;
use crate::ty::Atom;
use crate::ty::Type;
use crate::ty::atom::payload::alias::AliasAtom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ArrayKey;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::array::KnownItem;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::callable::Parameter;
use crate::ty::atom::payload::callable::ParameterFlag;
use crate::ty::atom::payload::callable::Signature;
use crate::ty::atom::payload::callable::SignatureFlag;
use crate::ty::atom::payload::conditional::ConditionalAtom;
use crate::ty::atom::payload::derived::DerivedAtom;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::iterable::IterableAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::object::named::ObjectFlag;
use crate::ty::atom::payload::object::shape::KnownProperty;
use crate::ty::atom::payload::object::shape::ObjectShapeAtom;
use crate::ty::atom::payload::object::shape::ObjectShapeFlag;
use crate::ty::atom::payload::reference::GlobalReferenceAtom;
use crate::ty::atom::payload::reference::MemberReferenceAtom;
use crate::ty::atom::payload::reference::NameSelector;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::float::LiteralFloat;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::atom::payload::variable::VariableAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::well_known;
use crate::var::Var;

/// Bridges a HIR native type into an oracle [`Type`], interning every atom
/// through the builder. The atom list is gathered on the builder's scratch
/// arena. Returns `None` only for the empty union (no atoms).
pub(crate) fn lower_hir_type<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    hir: &HirType<'arena>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    let mut atoms = builder.scratch_vec();
    lower_kind(builder, &hir.kind, &mut atoms);
    if atoms.is_empty() {
        return None;
    }

    Some(builder.union_of(&atoms))
}

/// Bridges a phpdoc type annotation into an oracle [`Type`], interning every
/// atom through the builder on its scratch arena.
pub fn lower_type_annotation<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    annotation: &TypeAnnotation<'arena>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    let mut atoms = builder.scratch_vec();
    lower_annotation_kind(builder, &annotation.kind, &mut atoms);
    if atoms.is_empty() {
        return None;
    }

    Some(builder.union_of(&atoms))
}

/// Lowers a nested annotation to a full [`Type`], defaulting to `mixed` for the
/// empty union so a structural payload always has a concrete component type.
fn lower_subtype<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    annotation: &TypeAnnotation<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    lower_type_annotation(builder, annotation).unwrap_or(well_known::TYPE_MIXED)
}

/// The class-like name a reference points at.
pub(crate) fn reference_name<'arena>(kind: &ReferenceKind<'arena>) -> &'arena [u8] {
    match kind {
        ReferenceKind::Identifier(identifier)
        | ReferenceKind::Self_(identifier)
        | ReferenceKind::Static(identifier)
        | ReferenceKind::Parent(identifier) => identifier.value,
    }
}

/// Lowers a phpdoc type-annotation kind into `out`, building the faithful oracle
/// atom for every kind that has one (arrays, lists, shapes, class-strings,
/// callables, generics, derived/conditional types, references, ...). Only the
/// kinds with no dedicated atom (`$this`, wildcards, slices) fall back to a
/// sound supertype, never structural kinds that the lattice can represent.
fn lower_annotation_kind<'scratch, 'arena, S, A>(
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    kind: &TypeAnnotationKind<'arena>,
    out: &mut Vec<'scratch, Atom<'arena>, S>,
) where
    S: Arena,
    A: Arena,
{
    match kind {
        TypeAnnotationKind::Named(named) => {
            let name = builder.intern_class_like_path(reference_name(&named.kind));
            let type_arguments = named.type_arguments.map(|delimited| {
                let mut arguments = builder.scratch_vec();
                for argument in delimited.as_slice() {
                    let argument = lower_subtype(builder, argument);
                    arguments.push(argument);
                }

                builder.types(&arguments)
            });

            let mut flags = U8Flags::empty();
            if matches!(named.kind, ReferenceKind::Static(_)) {
                flags = flags.with(ObjectFlag::IsStatic);
            }

            out.push(builder.object(ObjectAtom { name, type_arguments, flags }));
        }
        TypeAnnotationKind::GenericParameter(parameter) => {
            let constraint = parameter.bound.map_or(well_known::TYPE_MIXED, |bound| lower_subtype(builder, bound));
            let defining_entity = defining_entity(builder, &parameter.defining_entity);
            let name = builder.intern(parameter.name.value);

            out.push(builder.generic_parameter(GenericParameterAtom { name, defining_entity, constraint }));
        }
        TypeAnnotationKind::Union(members) => {
            for member in *members {
                lower_annotation_kind(builder, &member.kind, out);
            }
        }
        TypeAnnotationKind::Intersection(members) => {
            let mut conjuncts = builder.scratch_vec();
            for member in *members {
                lower_annotation_kind(builder, &member.kind, &mut conjuncts);
            }
            if let Some((head, rest)) = conjuncts.split_first() {
                out.push(builder.intersected(*head, rest));
            }
        }
        TypeAnnotationKind::Array(non_empty, key, value) => {
            let key_type = lower_subtype(builder, key);
            let value_type = lower_subtype(builder, value);
            out.push(builder.unsealed_keyed_array_atom(key_type, value_type, *non_empty));
        }
        TypeAnnotationKind::List(non_empty, value) => {
            let element_type = lower_subtype(builder, value);
            out.push(builder.list_of_atom(element_type, *non_empty));
        }
        TypeAnnotationKind::Iterable(key, value) => {
            let key_type = lower_subtype(builder, key);
            let value_type = lower_subtype(builder, value);
            out.push(builder.iterable(IterableAtom { key_type, value_type }));
        }
        TypeAnnotationKind::ClassLikeString(constraint) => {
            out.push(class_like_string(builder, ClassLikeKind::Class, constraint))
        }
        TypeAnnotationKind::ClassString(constraint) => {
            out.push(class_like_string(builder, ClassLikeKind::Class, constraint))
        }
        TypeAnnotationKind::InterfaceString(constraint) => {
            out.push(class_like_string(builder, ClassLikeKind::Interface, constraint))
        }
        TypeAnnotationKind::EnumString(constraint) => {
            out.push(class_like_string(builder, ClassLikeKind::Enum, constraint))
        }
        TypeAnnotationKind::TraitString(constraint) => {
            out.push(class_like_string(builder, ClassLikeKind::Trait, constraint))
        }
        TypeAnnotationKind::String(string) => out.push(string_atom(builder, string)),
        TypeAnnotationKind::Int(None) => out.push(well_known::INT),
        TypeAnnotationKind::Int(Some(IntLiteral::Specific(value))) => out.push(Atom::int_literal(*value)),
        TypeAnnotationKind::Int(Some(IntLiteral::Unspecified)) => out.push(well_known::LITERAL_INT),
        TypeAnnotationKind::IntRange(lower, upper) => out.push(builder.int_range_atom(*lower, *upper)),
        TypeAnnotationKind::Float(None) => out.push(well_known::FLOAT),
        TypeAnnotationKind::Float(Some(FloatLiteral::Specific(value))) => {
            out.push(Atom::Float(FloatAtom::Literal(LiteralFloat::new(value.into_inner()))))
        }
        TypeAnnotationKind::Float(Some(FloatLiteral::Unspecified)) => out.push(well_known::LITERAL_FLOAT),
        TypeAnnotationKind::Bool(None) => out.push(well_known::BOOL),
        TypeAnnotationKind::Bool(Some(true)) => out.push(well_known::TRUE),
        TypeAnnotationKind::Bool(Some(false)) => out.push(well_known::FALSE),
        TypeAnnotationKind::Null => out.push(well_known::NULL),
        TypeAnnotationKind::Void => out.push(well_known::VOID),
        TypeAnnotationKind::Never | TypeAnnotationKind::Empty => out.push(well_known::NEVER),
        TypeAnnotationKind::Numeric => out.push(well_known::NUMERIC),
        TypeAnnotationKind::ArrayKey => out.push(well_known::ARRAY_KEY),
        TypeAnnotationKind::Scalar | TypeAnnotationKind::EmptyScalar => out.push(well_known::SCALAR),
        TypeAnnotationKind::Object => out.push(well_known::OBJECT),
        TypeAnnotationKind::StringableObject => out.push(builder.named_object_atom(b"Stringable")),
        TypeAnnotationKind::ObjectShape(shape) => {
            let known_properties = if shape.fields.as_slice().is_empty() {
                None
            } else {
                let mut properties = builder.scratch_vec();
                for field in shape.fields.as_slice() {
                    let name = shape_property_name(builder, &field.key);
                    let value = lower_subtype(builder, &field.value);
                    properties.push(KnownProperty { name, value, optional: field.optional });
                }

                Some(builder.known_properties(&properties))
            };

            let mut flags = U8Flags::empty();
            if shape.sealed {
                flags = flags.with(ObjectShapeFlag::Sealed);
            }

            out.push(builder.object_shape(ObjectShapeAtom { known_properties, flags }));
        }
        TypeAnnotationKind::Shape(shape) => out.push(shape_atom(builder, shape)),
        TypeAnnotationKind::Callable(callable) => out.push(callable_atom(builder, callable)),
        TypeAnnotationKind::Resource(None) => out.push(well_known::RESOURCE),
        TypeAnnotationKind::Resource(Some(true)) => out.push(well_known::OPEN_RESOURCE),
        TypeAnnotationKind::Resource(Some(false)) => out.push(well_known::CLOSED_RESOURCE),
        TypeAnnotationKind::Mixed(_) => out.push(well_known::MIXED),
        TypeAnnotationKind::Conditional(conditional) => {
            let subject = lower_subtype(builder, conditional.subject);
            let target = lower_subtype(builder, conditional.target);
            let then = lower_subtype(builder, conditional.then);
            let otherwise = lower_subtype(builder, conditional.r#else);
            out.push(builder.conditional(ConditionalAtom {
                subject,
                target,
                then,
                otherwise,
                negated: conditional.is_negated,
            }));
        }
        TypeAnnotationKind::KeyOf(inner) => {
            let target = lower_subtype(builder, inner);
            out.push(builder.derived(DerivedAtom::KeyOf(target)));
        }
        TypeAnnotationKind::ValueOf(inner) => {
            let target = lower_subtype(builder, inner);
            out.push(builder.derived(DerivedAtom::ValueOf(target)));
        }
        TypeAnnotationKind::IntMask(members) => {
            let mut types = builder.scratch_vec();
            for member in *members {
                let member = lower_subtype(builder, member);
                types.push(member);
            }
            let members = builder.types(&types);
            out.push(builder.derived(DerivedAtom::IntMask(members)));
        }
        TypeAnnotationKind::IntMaskOf(inner) => {
            let target = lower_subtype(builder, inner);
            out.push(builder.derived(DerivedAtom::IntMaskOf(target)));
        }
        TypeAnnotationKind::New(inner) => {
            let target = lower_subtype(builder, inner);
            out.push(builder.derived(DerivedAtom::New(target)));
        }
        TypeAnnotationKind::IndexAccess(target, index) => {
            let target = lower_subtype(builder, target);
            let index = lower_subtype(builder, index);
            out.push(builder.derived(DerivedAtom::IndexAccess { target, index }));
        }
        TypeAnnotationKind::PropertiesOf(filter, inner) => {
            let target = lower_subtype(builder, inner);
            out.push(builder.derived(DerivedAtom::PropertiesOf { target, visibility: properties_visibility(*filter) }));
        }
        TypeAnnotationKind::TemplateType(members) => {
            let object = members.first().map_or(well_known::TYPE_MIXED, |member| lower_subtype(builder, member));
            let class_name = members.get(1).map_or(well_known::TYPE_MIXED, |member| lower_subtype(builder, member));
            let template_name = members.get(2).map_or(well_known::TYPE_MIXED, |member| lower_subtype(builder, member));
            out.push(builder.derived(DerivedAtom::TemplateType { object, class_name, template_name }));
        }
        TypeAnnotationKind::Negated(inner) => {
            let inner = lower_subtype(builder, inner);
            out.push(builder.negated(inner));
        }
        TypeAnnotationKind::Posited(inner) => lower_annotation_kind(builder, &inner.kind, out),
        TypeAnnotationKind::AliasReference(kind, name) => {
            let class_name = builder.intern_class_like_path(reference_name(kind));
            let alias_name = builder.intern(name.value);
            out.push(builder.alias(AliasAtom { class_name, alias_name }));
        }
        TypeAnnotationKind::MemberReference(identifier, selector) => {
            let class_like_name = builder.intern_class_like_path(identifier.value);
            let selector = member_selector(builder, selector);
            out.push(builder.member_reference(MemberReferenceAtom { class_like_name, selector }));
        }
        TypeAnnotationKind::GlobalSelector(selector) => {
            let selector = global_selector(builder, selector);
            out.push(builder.global_reference(GlobalReferenceAtom { selector }));
        }
        TypeAnnotationKind::Variable(variable) => {
            let name = Var::new(builder.intern(variable.name));
            out.push(Atom::Variable(VariableAtom { name }));
        }
        TypeAnnotationKind::Slice(inner) => {
            let value = lower_subtype(builder, inner);
            out.push(builder.unsealed_keyed_array_atom(well_known::TYPE_ARRAY_KEY, value, false));
        }
        TypeAnnotationKind::Wildcard => out.push(well_known::PLACEHOLDER),
        TypeAnnotationKind::ThisVariable => out.push(well_known::OBJECT),
    }
}

/// Builds a `class-string`/`interface-string`/… atom, deriving its specifier
/// from the constraint: an unconstrained `mixed` becomes `Any`, a template
/// parameter becomes `Generic`, and anything else becomes `OfType`.
fn class_like_string<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    kind: ClassLikeKind,
    constraint: &TypeAnnotation<'arena>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    let specifier = if matches!(constraint.kind, TypeAnnotationKind::Mixed(_)) {
        ClassLikeStringSpecifier::Any
    } else if matches!(constraint.kind, TypeAnnotationKind::GenericParameter(_)) {
        ClassLikeStringSpecifier::Generic { constraint: lower_subtype(builder, constraint) }
    } else {
        ClassLikeStringSpecifier::OfType { constraint: lower_subtype(builder, constraint) }
    };

    builder.class_like_string(ClassLikeStringAtom { kind, specifier })
}

/// Builds a refined `string` atom from a phpdoc string annotation, carrying its
/// literal value, casing, and refinement flags.
fn string_atom<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    annotation: &StringTypeAnnotation<'arena>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    let literal = match annotation.literal {
        Some(HirStringLiteral::Specific(value)) => StringLiteral::Value(builder.intern(value)),
        Some(HirStringLiteral::Unspecified) => StringLiteral::Unspecified,
        None => StringLiteral::None,
    };

    let casing = match annotation.casing {
        Some(HirStringCasing::Lowercase) => StringCasing::Lowercase,
        Some(HirStringCasing::Uppercase) => StringCasing::Uppercase,
        None => StringCasing::Unspecified,
    };

    let mut flags = U8Flags::empty();
    if annotation.numeric {
        flags = flags.with(StringRefinementFlag::Numeric);
    }
    if annotation.truthy {
        flags = flags.with(StringRefinementFlag::Truthy);
    }
    if annotation.non_empty || annotation.truthy {
        flags = flags.with(StringRefinementFlag::NonEmpty);
    }
    if annotation.callable {
        flags = flags.with(StringRefinementFlag::Callable);
    }

    builder.string(StringAtom { literal, casing, flags })
}

/// Builds an array-shape atom: a list shape becomes a sealed (or rest-typed)
/// `list{…}`, a keyed shape a sealed (or rest-typed) `array{…}`.
fn shape_atom<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    shape: &ShapeTypeAnnotation<'arena>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    if shape.is_list {
        let mut elements = builder.scratch_vec();
        for (index, field) in shape.fields.as_slice().iter().enumerate() {
            let value = lower_subtype(builder, &field.value);
            elements.push(KnownElement { index: index as u32, value, optional: field.optional });
        }

        return match shape.additional_fields {
            None => builder.sealed_list_atom(&elements, shape.non_empty),
            Some(additional) => {
                let element_type = lower_subtype(builder, additional.value_type);
                let known_count = NonZeroU32::new(elements.len() as u32);
                let known_elements = Some(builder.known_elements(&elements));
                let mut flags = U8Flags::empty();
                if shape.non_empty {
                    flags = flags.with(ListFlag::NonEmpty);
                }

                builder.list(ListAtom { element_type, known_elements, known_count, flags })
            }
        };
    }

    let mut items = builder.scratch_vec();
    for field in shape.fields.as_slice() {
        let key = array_key(builder, &field.key);
        let value = lower_subtype(builder, &field.value);
        items.push(KnownItem { key, value, optional: field.optional });
    }

    match shape.additional_fields {
        None => builder.sealed_keyed_array_atom(&items, shape.non_empty),
        Some(additional) => {
            let key_param = Some(lower_subtype(builder, additional.key_type));
            let value_param = Some(lower_subtype(builder, additional.value_type));
            let known_items = Some(builder.known_items(&items));
            let mut flags = U8Flags::empty();
            if shape.non_empty {
                flags = flags.with(ArrayFlag::NonEmpty);
            }

            builder.array(ArrayAtom { key_param, value_param, known_items, flags })
        }
    }
}

/// Builds a callable/closure atom from a phpdoc callable annotation, lowering
/// each parameter and the return type into a signature.
fn callable_atom<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    callable: &CallableTypeAnnotation<'arena>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut variadic = false;
    let parameters = callable.parameters.map(|delimited| {
        let mut parameters = builder.scratch_vec();
        for parameter in delimited.as_slice() {
            let r#type = parameter.r#type.map_or(well_known::TYPE_MIXED, |r#type| lower_subtype(builder, r#type));
            let name = parameter.variable.map_or(&b""[..], |variable| variable.name);
            let name = builder.intern(name);

            let mut flags = U8Flags::empty();
            if parameter.has_default {
                flags = flags.with(ParameterFlag::HasDefault);
            }
            if parameter.by_reference {
                flags = flags.with(ParameterFlag::ByReference);
            }
            if parameter.variadic {
                flags = flags.with(ParameterFlag::Variadic);
                variadic = true;
            }

            parameters.push(Parameter { name, r#type, flags });
        }

        builder.parameters(&parameters)
    });

    let return_type = callable.r#return.map_or(well_known::TYPE_MIXED, |r#return| lower_subtype(builder, r#return));

    let mut flags = U8Flags::empty();
    if variadic {
        flags = flags.with(SignatureFlag::IsVariadic);
    }
    if matches!(callable.kind, CallableTypeKind::PureCallable | CallableTypeKind::PureClosure) {
        flags = flags.with(SignatureFlag::IsPure);
    }

    let signature = builder.signature(Signature { parameters, return_type, throws: None, flags });
    match callable.kind {
        CallableTypeKind::Closure | CallableTypeKind::PureClosure => Atom::Callable(CallableAtom::Closure(signature)),
        CallableTypeKind::Callable | CallableTypeKind::PureCallable => {
            Atom::Callable(CallableAtom::Signature(signature))
        }
    }
}

/// Maps a phpdoc generic-parameter defining entity to its oracle counterpart.
/// A closure-defined parameter has no dedicated entity, so it is attributed to a
/// synthetic `{closure}` function.
fn defining_entity<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    entity: &TypeParameterDefiningEntity<'arena>,
) -> DefiningEntity<'arena>
where
    S: Arena,
    A: Arena,
{
    match entity {
        TypeParameterDefiningEntity::ClassLike(identifier) => {
            DefiningEntity::ClassLike(builder.intern_class_like_path(identifier.value))
        }
        TypeParameterDefiningEntity::Function(identifier) => {
            DefiningEntity::Function(builder.intern_function_like_path(identifier.value))
        }
        TypeParameterDefiningEntity::Method(identifier, name) => DefiningEntity::Method {
            class: builder.intern_class_like_path(identifier.value),
            method: builder.intern(name.value),
        },
        TypeParameterDefiningEntity::Closure(_) => {
            DefiningEntity::Function(builder.intern_function_like_path(b"{closure}"))
        }
    }
}

/// The oracle array key for a phpdoc shape key.
fn array_key<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    key: &ShapeTypeAnnotationKey<'arena>,
) -> ArrayKey<'arena>
where
    S: Arena,
    A: Arena,
{
    match key {
        ShapeTypeAnnotationKey::String(value) => ArrayKey::String(builder.intern(value)),
        ShapeTypeAnnotationKey::Integer(value) => ArrayKey::Int(*value),
        ShapeTypeAnnotationKey::ClassLikeConstant(identifier, name) => ArrayKey::Const {
            class: builder.intern_class_like_path(identifier.value),
            name: builder.intern(name.value),
        },
    }
}

/// The property name an object-shape field declares. Non-string keys are
/// rendered to their textual form, since object properties are named by string.
fn shape_property_name<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    key: &ShapeTypeAnnotationKey<'arena>,
) -> &'arena [u8]
where
    S: Arena,
    A: Arena,
{
    match key {
        ShapeTypeAnnotationKey::String(value) => builder.intern(value),
        ShapeTypeAnnotationKey::Integer(value) => builder.intern(value.to_string().as_bytes()),
        ShapeTypeAnnotationKey::ClassLikeConstant(_, name) => builder.intern(name.value),
    }
}

/// The oracle name selector for a phpdoc member-reference selector.
fn member_selector<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    selector: &MemberReferenceSelector<'arena>,
) -> NameSelector<'arena>
where
    S: Arena,
    A: Arena,
{
    match selector {
        MemberReferenceSelector::Wildcard => NameSelector::Wildcard,
        MemberReferenceSelector::Exact(name) => NameSelector::Identifier(builder.intern(name.value)),
        MemberReferenceSelector::StartsWith(name) => NameSelector::StartsWith(builder.intern(name.value)),
        MemberReferenceSelector::EndsWith(name) => NameSelector::EndsWith(builder.intern(name.value)),
    }
}

/// The oracle name selector for a phpdoc global selector.
fn global_selector<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    selector: &GlobalSelector<'arena>,
) -> NameSelector<'arena>
where
    S: Arena,
    A: Arena,
{
    match selector {
        GlobalSelector::StartsWith(identifier) => NameSelector::StartsWith(builder.intern(identifier.value)),
        GlobalSelector::EndsWith(identifier) => NameSelector::EndsWith(builder.intern(identifier.value)),
    }
}

/// The property visibility a `properties-of` filter selects.
fn properties_visibility(filter: PropertiesOfFilter) -> Option<Visibility> {
    match filter {
        PropertiesOfFilter::All => None,
        PropertiesOfFilter::Public => Some(Visibility::Public),
        PropertiesOfFilter::Protected => Some(Visibility::Protected),
        PropertiesOfFilter::Private => Some(Visibility::Private),
    }
}

/// Lowers a HIR type kind into `out`. Every HIR native type kind is handled.
fn lower_kind<'scratch, 'arena, S, A>(
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    kind: &HirTypeKind<'arena>,
    out: &mut Vec<'scratch, Atom<'arena>, S>,
) where
    S: Arena,
    A: Arena,
{
    match kind {
        HirTypeKind::Parenthesized(inner) => lower_kind(builder, &inner.kind, out),
        HirTypeKind::Nullable(inner) => {
            lower_kind(builder, &inner.kind, out);
            out.push(well_known::NULL);
        }
        HirTypeKind::Named(identifier) => out.push(builder.named_object_atom(identifier.value)),
        HirTypeKind::Union(members) => {
            for member in *members {
                lower_kind(builder, &member.kind, out);
            }
        }
        HirTypeKind::Intersection(members) => {
            let mut conjuncts = builder.scratch_vec();
            for member in *members {
                lower_kind(builder, &member.kind, &mut conjuncts);
            }
            if let Some((head, rest)) = conjuncts.split_first() {
                out.push(builder.intersected(*head, rest));
            }
        }
        HirTypeKind::Null => out.push(well_known::NULL),
        HirTypeKind::Array => out.push(well_known::ARRAY_KEY_MIXED),
        HirTypeKind::Callable => out.push(well_known::CALLABLE),
        HirTypeKind::Static(identifier) => {
            let name = builder.intern_class_like_path(identifier.value);
            out.push(builder.object(ObjectAtom {
                name,
                type_arguments: None,
                flags: U8Flags::empty().with(ObjectFlag::IsStatic),
            }));
        }
        HirTypeKind::Self_(identifier) | HirTypeKind::Parent(identifier) => {
            out.push(builder.named_object_atom(identifier.value))
        }
        HirTypeKind::Void => out.push(well_known::VOID),
        HirTypeKind::Never => out.push(well_known::NEVER),
        HirTypeKind::Float => out.push(well_known::FLOAT),
        HirTypeKind::Bool(None) => out.push(well_known::BOOL),
        HirTypeKind::Bool(Some(true)) => out.push(well_known::TRUE),
        HirTypeKind::Bool(Some(false)) => out.push(well_known::FALSE),
        HirTypeKind::Integer => out.push(well_known::INT),
        HirTypeKind::String => out.push(well_known::STRING),
        HirTypeKind::Object => out.push(well_known::OBJECT),
        HirTypeKind::Mixed => out.push(well_known::MIXED),
        HirTypeKind::Iterable => out.push(well_known::ITERABLE_MIXED_MIXED),
    }
}
