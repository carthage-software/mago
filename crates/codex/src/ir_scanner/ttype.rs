use std::collections::BTreeMap;
use std::sync::Arc;

use mago_hir::ir::generics::TypeParameterDefiningEntity;
use mago_hir::ir::r#type::Type;
use mago_hir::ir::r#type::TypeKind;
use mago_hir::ir::r#type::annotation::CallableTypeAnnotation;
use mago_hir::ir::r#type::annotation::CallableTypeKind;
use mago_hir::ir::r#type::annotation::FloatLiteral;
use mago_hir::ir::r#type::annotation::GenericParameterAnnotation;
use mago_hir::ir::r#type::annotation::GlobalSelector;
use mago_hir::ir::r#type::annotation::IntLiteral;
use mago_hir::ir::r#type::annotation::MemberReferenceSelector;
use mago_hir::ir::r#type::annotation::NamedTypeAnnotation;
use mago_hir::ir::r#type::annotation::ObjectShapeTypeAnnotation;
use mago_hir::ir::r#type::annotation::PropertiesOfFilter;
use mago_hir::ir::r#type::annotation::ReferenceKind;
use mago_hir::ir::r#type::annotation::ShapeTypeAnnotation;
use mago_hir::ir::r#type::annotation::ShapeTypeAnnotationKey;
use mago_hir::ir::r#type::annotation::StringCasing;
use mago_hir::ir::r#type::annotation::StringLiteral;
use mago_hir::ir::r#type::annotation::StringTypeAnnotation;
use mago_hir::ir::r#type::annotation::TypeAnnotation;
use mago_hir::ir::r#type::annotation::TypeAnnotationKind;
use mago_word::Word;
use mago_word::ascii_lowercase_word;
use mago_word::concat_word;
use mago_word::empty_word;
use mago_word::i64_word;
use mago_word::word;

use crate::metadata::ttype::TypeMetadata;
use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::alias::ReferenceKind as AliasReferenceKind;
use crate::ttype::atomic::alias::TAlias;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::callable::TCallableSignature;
use crate::ttype::atomic::callable::parameter::TCallableParameter;
use crate::ttype::atomic::conditional::TConditional;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::derived::index_access::TIndexAccess;
use crate::ttype::atomic::derived::int_mask::TIntMask;
use crate::ttype::atomic::derived::int_mask_of::TIntMaskOf;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::new::TNew;
use crate::ttype::atomic::derived::properties_of::TPropertiesOf;
use crate::ttype::atomic::derived::template_type::TTemplateType;
use crate::ttype::atomic::derived::value_of::TValueOf;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::iterable::TIterable;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::reference::TGlobalReferenceSelector;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::get_arraykey;
use crate::ttype::get_bool;
use crate::ttype::get_false;
use crate::ttype::get_float;
use crate::ttype::get_int;
use crate::ttype::get_keyed_array;
use crate::ttype::get_list;
use crate::ttype::get_literal_float;
use crate::ttype::get_literal_int;
use crate::ttype::get_literal_string;
use crate::ttype::get_mixed;
use crate::ttype::get_mixed_callable;
use crate::ttype::get_mixed_iterable;
use crate::ttype::get_mixed_keyed_array;
use crate::ttype::get_never;
use crate::ttype::get_non_empty_list;
use crate::ttype::get_non_empty_unspecified_literal_string;
use crate::ttype::get_null;
use crate::ttype::get_nullable_float;
use crate::ttype::get_nullable_int;
use crate::ttype::get_nullable_object;
use crate::ttype::get_nullable_string;
use crate::ttype::get_numeric;
use crate::ttype::get_object;
use crate::ttype::get_scalar;
use crate::ttype::get_string;
use crate::ttype::get_string_with_props;
use crate::ttype::get_true;
use crate::ttype::get_truthy_mixed;
use crate::ttype::get_unspecified_literal_float;
use crate::ttype::get_unspecified_literal_int;
use crate::ttype::get_unspecified_literal_string;
use crate::ttype::get_void;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;

pub fn type_metadata_from_type(ty: &Type<'_>, classname: Option<Word>) -> TypeMetadata {
    TypeMetadata::new(union_from_type(&ty.kind, classname), ty.span)
}

#[must_use]
pub fn type_metadata_from_annotation(annotation: &TypeAnnotation<'_>, classname: Option<Word>) -> TypeMetadata {
    TypeMetadata::from_docblock(union_from_annotation(&annotation.kind, classname), annotation.span)
}

#[must_use]
pub fn merge_type_preserving_nullability(docblock: TypeMetadata, declaration: Option<&TypeMetadata>) -> TypeMetadata {
    if docblock.type_union.types.iter().any(TAtomic::is_conditional) {
        return docblock;
    }

    if declaration.is_some_and(|declaration| declaration.type_union.is_nullable())
        && !docblock.type_union.accepts_null()
    {
        docblock.map_type_union(TUnion::as_nullable)
    } else {
        docblock
    }
}

pub(super) fn generic_parent(entity: &TypeParameterDefiningEntity<'_>) -> GenericParent {
    match entity {
        TypeParameterDefiningEntity::ClassLike(identifier) => {
            GenericParent::ClassLike(ascii_lowercase_word(identifier.value))
        }
        TypeParameterDefiningEntity::Function(identifier) => {
            GenericParent::FunctionLike((empty_word(), ascii_lowercase_word(identifier.value)))
        }
        TypeParameterDefiningEntity::Method(identifier, name) => {
            GenericParent::FunctionLike((ascii_lowercase_word(identifier.value), ascii_lowercase_word(name.value)))
        }
        TypeParameterDefiningEntity::Closure(_) => GenericParent::FunctionLike((empty_word(), empty_word())),
    }
}

fn generic_parameter(annotation: &GenericParameterAnnotation<'_>, classname: Option<Word>) -> TUnion {
    let constraint = annotation.bound.map_or_else(get_mixed, |bound| union_from_annotation(&bound.kind, classname));

    wrap_atomic(TAtomic::GenericParameter(TGenericParameter {
        parameter_name: word(annotation.name.value),
        constraint: Arc::new(constraint),
        defining_entity: generic_parent(&annotation.defining_entity),
        intersection_types: None,
    }))
}

fn class_string(kind: TClassLikeStringKind, inner: &TypeAnnotationKind<'_>, classname: Option<Word>) -> TUnion {
    let constraint_union = union_from_annotation(inner, classname);

    let mut atomics = Vec::new();
    for constraint in constraint_union.types.into_owned() {
        match constraint {
            TAtomic::Object(TObject::Named(_) | TObject::Enum(_) | TObject::HasMethod(_))
            | TAtomic::Reference(TReference::Symbol { .. })
            | TAtomic::Alias(_) => {
                atomics.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::of_type(kind, constraint))));
            }
            TAtomic::GenericParameter(TGenericParameter { parameter_name, defining_entity, constraint, .. }) => {
                for nested in Arc::unwrap_or_clone(constraint).types.into_owned() {
                    atomics.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::generic(
                        kind,
                        parameter_name,
                        defining_entity,
                        nested,
                    ))));
                }
            }
            _ => atomics.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::any(kind)))),
        }
    }

    if atomics.is_empty() {
        return wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::any(kind))));
    }

    TUnion::from_vec(atomics)
}

fn string_annotation(string: &StringTypeAnnotation<'_>) -> TUnion {
    if let Some(StringLiteral::Specific(value)) = string.literal {
        return get_literal_string(word(value));
    }

    let casing = match string.casing {
        Some(StringCasing::Lowercase) => crate::ttype::atomic::scalar::string::TStringCasing::Lowercase,
        Some(StringCasing::Uppercase) => crate::ttype::atomic::scalar::string::TStringCasing::Uppercase,
        None => crate::ttype::atomic::scalar::string::TStringCasing::Unspecified,
    };

    if string.literal == Some(StringLiteral::Unspecified) {
        if string.non_empty {
            return get_non_empty_unspecified_literal_string();
        }

        return get_unspecified_literal_string();
    }

    if !string.non_empty && !string.numeric && !string.truthy && !string.callable && string.casing.is_none() {
        return get_string();
    }

    get_string_with_props(string.numeric, string.truthy, string.non_empty, string.callable, casing)
}

#[must_use]
pub fn union_from_annotation(kind: &TypeAnnotationKind<'_>, classname: Option<Word>) -> TUnion {
    match kind {
        TypeAnnotationKind::Named(named) => annotation_named(named, classname),
        TypeAnnotationKind::GenericParameter(annotation) => generic_parameter(annotation, classname),
        TypeAnnotationKind::Union(kinds) => annotation_union(kinds, classname),
        TypeAnnotationKind::Intersection(kinds) => annotation_intersection(kinds, classname),
        TypeAnnotationKind::Array(non_empty, key, value) => {
            let key_type = union_from_annotation(key, classname);
            let value_type = union_from_annotation(value, classname);
            if *non_empty {
                let mut keyed = TKeyedArray::new_with_parameters(Arc::new(key_type), Arc::new(value_type));
                keyed.non_empty = true;
                wrap_atomic(TAtomic::Array(TArray::Keyed(keyed)))
            } else {
                get_keyed_array(key_type, value_type)
            }
        }
        TypeAnnotationKind::List(non_empty, value) => {
            let value_type = union_from_annotation(value, classname);
            if *non_empty { get_non_empty_list(value_type) } else { get_list(value_type) }
        }
        TypeAnnotationKind::Iterable(key, value) => wrap_atomic(TAtomic::Iterable(TIterable::new(
            Arc::new(union_from_annotation(key, classname)),
            Arc::new(union_from_annotation(value, classname)),
        ))),
        TypeAnnotationKind::ClassLikeString(inner) | TypeAnnotationKind::ClassString(inner) => {
            class_string(TClassLikeStringKind::Class, inner, classname)
        }
        TypeAnnotationKind::InterfaceString(inner) => class_string(TClassLikeStringKind::Interface, inner, classname),
        TypeAnnotationKind::EnumString(inner) => class_string(TClassLikeStringKind::Enum, inner, classname),
        TypeAnnotationKind::TraitString(inner) => class_string(TClassLikeStringKind::Trait, inner, classname),
        TypeAnnotationKind::Mixed(non_empty) => {
            if *non_empty {
                get_truthy_mixed()
            } else {
                get_mixed()
            }
        }
        TypeAnnotationKind::Null => get_null(),
        TypeAnnotationKind::Void => get_void(),
        TypeAnnotationKind::Never => get_never(),
        TypeAnnotationKind::Resource(None) => crate::ttype::get_resource(),
        TypeAnnotationKind::Resource(Some(true)) => crate::ttype::get_closed_resource(),
        TypeAnnotationKind::Resource(Some(false)) => crate::ttype::get_open_resource(),
        TypeAnnotationKind::Bool(Some(true)) => get_true(),
        TypeAnnotationKind::Bool(Some(false)) => get_false(),
        TypeAnnotationKind::Bool(None) => get_bool(),
        TypeAnnotationKind::Float(Some(FloatLiteral::Specific(value))) => get_literal_float(value.into_inner()),
        TypeAnnotationKind::Float(Some(FloatLiteral::Unspecified)) => get_unspecified_literal_float(),
        TypeAnnotationKind::Float(None) => get_float(),
        TypeAnnotationKind::Int(Some(IntLiteral::Specific(value))) => get_literal_int(*value),
        TypeAnnotationKind::Int(Some(IntLiteral::Unspecified)) => get_unspecified_literal_int(),
        TypeAnnotationKind::Int(None) => get_int(),
        TypeAnnotationKind::String(string) => string_annotation(string),
        TypeAnnotationKind::Object => get_object(),
        TypeAnnotationKind::Numeric => get_numeric(),
        TypeAnnotationKind::ArrayKey => get_arraykey(),
        TypeAnnotationKind::Scalar => get_scalar(),
        TypeAnnotationKind::IntRange(min, max) => {
            wrap_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::from_bounds(*min, *max))))
        }
        TypeAnnotationKind::MemberReference(identifier, selector) => {
            let member_selector = match selector {
                MemberReferenceSelector::Wildcard => TReferenceMemberSelector::Wildcard,
                MemberReferenceSelector::Exact(name) => TReferenceMemberSelector::Identifier(word(name.value)),
                MemberReferenceSelector::StartsWith(name) => TReferenceMemberSelector::StartsWith(word(name.value)),
                MemberReferenceSelector::EndsWith(name) => TReferenceMemberSelector::EndsWith(word(name.value)),
            };

            wrap_atomic(TAtomic::Reference(TReference::Member {
                class_like_name: word(identifier.value),
                member_selector,
            }))
        }
        TypeAnnotationKind::GlobalSelector(selector) => {
            let selector = match selector {
                GlobalSelector::StartsWith(identifier) => TGlobalReferenceSelector::StartsWith(word(identifier.value)),
                GlobalSelector::EndsWith(identifier) => TGlobalReferenceSelector::EndsWith(word(identifier.value)),
            };

            wrap_atomic(TAtomic::Reference(TReference::Global { selector }))
        }
        TypeAnnotationKind::AliasReference(reference, name) => {
            let reference = match reference {
                ReferenceKind::Self_(identifier) => AliasReferenceKind::Self_(ascii_lowercase_word(identifier.value)),
                ReferenceKind::Static(identifier) => AliasReferenceKind::Static(ascii_lowercase_word(identifier.value)),
                ReferenceKind::Parent(identifier) => AliasReferenceKind::Parent(ascii_lowercase_word(identifier.value)),
                ReferenceKind::Identifier(identifier) => {
                    AliasReferenceKind::Identifier(ascii_lowercase_word(identifier.value))
                }
            };

            wrap_atomic(TAtomic::Alias(TAlias::new(reference, word(name.value))))
        }
        TypeAnnotationKind::Variable(variable) => wrap_atomic(TAtomic::Variable(word(variable.name))),
        TypeAnnotationKind::ThisVariable => {
            wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_this(word("$this")))))
        }
        TypeAnnotationKind::KeyOf(inner) => wrap_atomic(TAtomic::Derived(TDerived::KeyOf(TKeyOf::new(Arc::new(
            union_from_annotation(inner, classname),
        ))))),
        TypeAnnotationKind::ValueOf(inner) => wrap_atomic(TAtomic::Derived(TDerived::ValueOf(TValueOf::new(
            Arc::new(union_from_annotation(inner, classname)),
        )))),
        TypeAnnotationKind::IntMaskOf(inner) => wrap_atomic(TAtomic::Derived(TDerived::IntMaskOf(TIntMaskOf::new(
            Arc::new(union_from_annotation(inner, classname)),
        )))),
        TypeAnnotationKind::IntMask(kinds) => wrap_atomic(TAtomic::Derived(TDerived::IntMask(TIntMask::new(
            kinds.iter().map(|kind| union_from_annotation(kind, classname)).collect(),
        )))),
        TypeAnnotationKind::IndexAccess(target, index) => wrap_atomic(TAtomic::Derived(TDerived::IndexAccess(
            TIndexAccess::new(union_from_annotation(target, classname), union_from_annotation(index, classname)),
        ))),
        TypeAnnotationKind::New(inner) => {
            wrap_atomic(TAtomic::Derived(TDerived::New(TNew::new(Arc::new(union_from_annotation(inner, classname))))))
        }
        TypeAnnotationKind::Negated(inner) => match negated_literal(inner) {
            Some(union) => union,
            None => union_from_annotation(inner, classname),
        },
        TypeAnnotationKind::Posited(inner) => union_from_annotation(inner, classname),
        TypeAnnotationKind::Shape(shape) => shape_annotation(shape, classname),
        TypeAnnotationKind::ObjectShape(shape) => object_shape_annotation(shape, classname),
        TypeAnnotationKind::Slice(inner) => get_keyed_array(get_arraykey(), union_from_annotation(inner, classname)),
        TypeAnnotationKind::TemplateType([object, class_argument, template_name]) => {
            wrap_atomic(TAtomic::Derived(TDerived::TemplateType(TTemplateType::new(
                Arc::new(union_from_annotation(object, classname)),
                Arc::new(union_from_annotation(class_argument, classname)),
                Arc::new(union_from_annotation(template_name, classname)),
            ))))
        }
        TypeAnnotationKind::PropertiesOf(filter, inner) => {
            let target = Arc::new(union_from_annotation(inner, classname));
            let properties_of = match filter {
                PropertiesOfFilter::All => TPropertiesOf::new(target),
                PropertiesOfFilter::Public => TPropertiesOf::public(target),
                PropertiesOfFilter::Protected => TPropertiesOf::protected(target),
                PropertiesOfFilter::Private => TPropertiesOf::private(target),
            };

            wrap_atomic(TAtomic::Derived(TDerived::PropertiesOf(properties_of)))
        }
        TypeAnnotationKind::Callable(callable) => callable_annotation(callable, classname),
        TypeAnnotationKind::Conditional(conditional) => wrap_atomic(TAtomic::Conditional(TConditional::new(
            Arc::new(union_from_annotation(conditional.subject, classname)),
            Arc::new(union_from_annotation(conditional.target, classname)),
            Arc::new(union_from_annotation(conditional.then, classname)),
            Arc::new(union_from_annotation(conditional.r#else, classname)),
            conditional.is_negated,
        ))),
        _ => get_mixed(),
    }
}

fn callable_annotation(callable: &CallableTypeAnnotation<'_>, classname: Option<Word>) -> TUnion {
    let is_pure = matches!(callable.kind, CallableTypeKind::PureCallable | CallableTypeKind::PureClosure);
    let is_closure = matches!(callable.kind, CallableTypeKind::Closure | CallableTypeKind::PureClosure);

    if callable.parameters.is_empty() && callable.r#return.is_none() {
        return wrap_atomic(TAtomic::Callable(TCallable::Signature(
            TCallableSignature::new(is_pure, is_closure)
                .with_parameters(vec![TCallableParameter::new(Some(Arc::new(get_mixed())), false, true, false)])
                .with_return_type(Some(Arc::new(get_mixed()))),
        )));
    }

    let parameters = callable
        .parameters
        .iter()
        .map(|parameter| {
            let parameter_type = parameter
                .r#type
                .map_or_else(get_mixed, |annotation| union_from_annotation(&annotation.kind, classname));
            TCallableParameter::new(
                Some(Arc::new(parameter_type)),
                parameter.by_reference,
                parameter.variadic,
                parameter.has_default,
            )
        })
        .collect();

    let return_type = callable.r#return.map(|annotation| Arc::new(union_from_annotation(annotation, classname)));

    wrap_atomic(TAtomic::Callable(TCallable::Signature(
        TCallableSignature::new(is_pure, is_closure).with_parameters(parameters).with_return_type(return_type),
    )))
}

fn shape_annotation(shape: &ShapeTypeAnnotation<'_>, classname: Option<Word>) -> TUnion {
    let non_empty = shape.non_empty || shape.fields.iter().any(|field| !field.optional);

    if shape.is_list {
        let element_type = match shape.additional_fields {
            Some(additional) => union_from_annotation(additional.value_type, classname),
            None => get_never(),
        };

        let mut list = TList::new(Arc::new(element_type));
        let mut known_elements = BTreeMap::new();
        let mut next_offset = 0usize;
        for field in shape.fields {
            let offset = match field.key {
                ShapeTypeAnnotationKey::Integer(value) if value >= 0 => value as usize,
                _ => next_offset,
            };
            next_offset = offset.saturating_add(1);

            let mut value = union_from_annotation(&field.value, classname);
            if field.optional {
                value.set_possibly_undefined(true, None);
            }
            known_elements.insert(offset, (field.optional, value));
        }

        list.known_elements = Some(known_elements);
        list.non_empty = non_empty;

        return wrap_atomic(TAtomic::Array(TArray::List(list)));
    }

    let mut keyed = TKeyedArray::new();

    keyed.parameters = shape.additional_fields.map(|additional| {
        (
            Arc::new(union_from_annotation(additional.key_type, classname)),
            Arc::new(union_from_annotation(additional.value_type, classname)),
        )
    });

    let mut known_items = BTreeMap::new();
    for field in shape.fields {
        let array_key = match field.key {
            ShapeTypeAnnotationKey::String(value) => ArrayKey::String(word(value)),
            ShapeTypeAnnotationKey::Integer(value) => ArrayKey::Integer(value),
            ShapeTypeAnnotationKey::ClassLikeConstant(identifier, name) => {
                ArrayKey::ClassLikeConstant { class_like_name: word(identifier.value), constant_name: word(name.value) }
            }
        };

        let mut value = union_from_annotation(&field.value, classname);
        if field.optional {
            value.set_possibly_undefined(true, None);
        }

        known_items.insert(array_key, (field.optional, value));
    }

    keyed.non_empty = non_empty;
    keyed.known_items = Some(known_items);

    wrap_atomic(TAtomic::Array(TArray::Keyed(keyed)))
}

fn object_shape_annotation(shape: &ObjectShapeTypeAnnotation<'_>, classname: Option<Word>) -> TUnion {
    let mut known_properties = BTreeMap::new();
    for field in shape.fields {
        let key = match field.key {
            ShapeTypeAnnotationKey::String(value) => word(value),
            ShapeTypeAnnotationKey::Integer(value) => i64_word(value),
            ShapeTypeAnnotationKey::ClassLikeConstant(identifier, name) => {
                concat_word!(identifier.value, b"::", name.value)
            }
        };

        known_properties.insert(key, (field.optional, union_from_annotation(&field.value, classname)));
    }

    wrap_atomic(TAtomic::Object(TObject::new_with_properties(shape.sealed, known_properties)))
}

fn negated_literal(kind: &TypeAnnotationKind<'_>) -> Option<TUnion> {
    match kind {
        TypeAnnotationKind::Int(Some(IntLiteral::Specific(value))) => Some(get_literal_int(-*value)),
        TypeAnnotationKind::Float(Some(FloatLiteral::Specific(value))) => Some(get_literal_float(-value.into_inner())),
        _ => None,
    }
}

fn annotation_named(named: &NamedTypeAnnotation<'_>, classname: Option<Word>) -> TUnion {
    let parameters: Option<Vec<TUnion>> = if named.type_arguments.is_empty() {
        None
    } else {
        Some(named.type_arguments.iter().map(|argument| union_from_annotation(argument, classname)).collect())
    };

    match named.kind {
        ReferenceKind::Self_(identifier) | ReferenceKind::Parent(identifier) => wrap_atomic(TAtomic::Object(
            TObject::Named(TNamedObject::new(ascii_lowercase_word(identifier.value)).with_type_parameters(parameters)),
        )),
        ReferenceKind::Static(identifier) => wrap_atomic(TAtomic::Object(TObject::Named(
            TNamedObject::new_static(ascii_lowercase_word(identifier.value)).with_type_parameters(parameters),
        ))),
        ReferenceKind::Identifier(identifier) => {
            let value = identifier.value;

            if value.eq_ignore_ascii_case(b"Closure") && parameters.is_none() {
                return wrap_atomic(TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(true))));
            }

            wrap_atomic(TAtomic::Reference(TReference::Symbol {
                name: word(value),
                parameters: pad_iterator_parameters(value, parameters),
                intersection_types: None,
            }))
        }
    }
}

fn pad_iterator_parameters(name: &[u8], mut parameters: Option<Vec<TUnion>>) -> Option<Vec<TUnion>> {
    let is_generator = name.eq_ignore_ascii_case(b"Generator");
    let is_iterator = is_generator
        || name.eq_ignore_ascii_case(b"Iterator")
        || name.eq_ignore_ascii_case(b"IteratorAggregate")
        || name.eq_ignore_ascii_case(b"Traversable");
    if !is_iterator {
        return parameters;
    }

    let mixed_default = || {
        let mut union = get_mixed();
        union.set_from_template_default(true);
        union
    };

    match &mut parameters {
        None => parameters = Some(vec![mixed_default(), mixed_default()]),
        Some(entries) if entries.len() == 1 => entries.insert(0, mixed_default()),
        Some(entries) if entries.is_empty() => {
            entries.push(mixed_default());
            entries.push(mixed_default());
        }
        Some(_) => {}
    }

    if is_generator {
        if let Some(entries) = &mut parameters {
            while entries.len() < 4 {
                entries.push(mixed_default());
            }
        }
    }

    parameters
}

fn annotation_union(kinds: &[TypeAnnotationKind<'_>], classname: Option<Word>) -> TUnion {
    if kinds.len() == 2 {
        if let Some(null_index) = kinds.iter().position(|kind| matches!(kind, TypeAnnotationKind::Null)) {
            let other = &kinds[1 - null_index];
            return match other {
                TypeAnnotationKind::String(string) if is_plain_string(string) => get_nullable_string(),
                TypeAnnotationKind::Int(None) => get_nullable_int(),
                TypeAnnotationKind::Float(None) => get_nullable_float(),
                TypeAnnotationKind::Object => get_nullable_object(),
                _ => union_from_annotation(other, classname).as_nullable(),
            };
        }
    }

    let mut atomics = Vec::new();
    for kind in kinds {
        atomics.extend(union_from_annotation(kind, classname).types.into_owned());
    }

    TUnion::from_vec(atomics)
}

fn is_plain_string(string: &StringTypeAnnotation<'_>) -> bool {
    string.literal.is_none()
        && string.casing.is_none()
        && !string.non_empty
        && !string.numeric
        && !string.truthy
        && !string.callable
}

fn annotation_intersection(kinds: &[TypeAnnotationKind<'_>], classname: Option<Word>) -> TUnion {
    if let [first, second] = kinds {
        let object_and_callable = (matches!(first, TypeAnnotationKind::Object)
            && matches!(second, TypeAnnotationKind::Callable(_)))
            || (matches!(first, TypeAnnotationKind::Callable(_)) && matches!(second, TypeAnnotationKind::Object));
        if object_and_callable {
            return wrap_atomic(TAtomic::Object(TObject::new_has_method(word("__invoke"))));
        }
    }

    if kinds.len() >= 2 && kinds.iter().all(|kind| matches!(kind, TypeAnnotationKind::String(_))) {
        let mut merged = StringTypeAnnotation {
            casing: None,
            literal: None,
            non_empty: false,
            numeric: false,
            truthy: false,
            callable: false,
        };
        for kind in kinds {
            if let TypeAnnotationKind::String(string) = kind {
                merged.casing = merged.casing.or(string.casing);
                merged.literal = merged.literal.or(string.literal);
                merged.non_empty |= string.non_empty;
                merged.numeric |= string.numeric;
                merged.truthy |= string.truthy;
                merged.callable |= string.callable;
            }
        }

        return string_annotation(&merged);
    }

    let mut members: Vec<TAtomic> = Vec::new();
    let mut started = false;
    for kind in kinds {
        let atomics: Vec<TAtomic> = union_from_annotation(kind, classname).types.into_owned();
        if atomics.is_empty() {
            continue;
        }

        if !started {
            members = atomics.into_iter().filter(|atomic| atomic.can_be_intersected()).collect();
            if members.is_empty() {
                continue;
            }

            started = true;
            continue;
        }

        let mut distributed = Vec::with_capacity(members.len() * atomics.len());
        for base in &members {
            for addition in &atomics {
                let mut combined = base.clone();
                combined.add_intersection_type(addition.clone_without_intersection_types());
                if let Some(nested) = addition.get_intersection_types() {
                    for nested_type in nested {
                        combined.add_intersection_type(nested_type.clone());
                    }
                }
                distributed.push(combined);
            }
        }
        members = distributed;
    }

    if members.is_empty() {
        return get_mixed();
    }

    TUnion::from_vec(members)
}

#[must_use]
pub fn union_from_type(kind: &TypeKind<'_>, classname: Option<Word>) -> TUnion {
    match *kind {
        TypeKind::Named(identifier) => named_reference(identifier.value),
        TypeKind::Union(kinds) => union_from_kinds(kinds, classname),
        TypeKind::Intersection(kinds) => intersection(kinds, classname),
        TypeKind::Null => get_null(),
        TypeKind::Array => get_mixed_keyed_array(),
        TypeKind::Callable => get_mixed_callable(),
        TypeKind::Static(identifier) => wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_static(
            ascii_lowercase_word(identifier.value),
        )))),
        TypeKind::Self_(identifier) => {
            wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_word(identifier.value)))))
        }
        TypeKind::Parent(identifier) => {
            wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_word(identifier.value)))))
        }
        TypeKind::Void => get_void(),
        TypeKind::Never => get_never(),
        TypeKind::Float => get_float(),
        TypeKind::Bool(Some(true)) => get_true(),
        TypeKind::Bool(Some(false)) => get_false(),
        TypeKind::Bool(None) => get_bool(),
        TypeKind::Integer => get_int(),
        TypeKind::String => get_string(),
        TypeKind::Object => get_object(),
        TypeKind::Mixed => get_mixed(),
        TypeKind::Iterable => get_mixed_iterable(),
    }
}

fn union_from_kinds(kinds: &[TypeKind<'_>], classname: Option<Word>) -> TUnion {
    if kinds.len() == 2 {
        if let Some(null_index) = kinds.iter().position(|kind| matches!(kind, TypeKind::Null)) {
            let other = &kinds[1 - null_index];

            return match other {
                TypeKind::String => get_nullable_string(),
                TypeKind::Integer => get_nullable_int(),
                TypeKind::Float => get_nullable_float(),
                TypeKind::Object => get_nullable_object(),
                _ => union_from_type(other, classname).as_nullable(),
            };
        }
    }

    let mut atomics = Vec::new();
    for kind in kinds {
        atomics.extend(union_from_type(kind, classname).types.into_owned());
    }

    TUnion::from_vec(atomics)
}

fn named_reference(value: &[u8]) -> TUnion {
    if value.eq_ignore_ascii_case(b"Generator") {
        let mixed_default = || {
            let mut union = get_mixed();
            union.set_from_template_default(true);
            union
        };

        return wrap_atomic(TAtomic::Object(TObject::Named(
            TNamedObject::new(word(value)).with_type_parameters(Some(vec![
                mixed_default(),
                mixed_default(),
                mixed_default(),
                mixed_default(),
            ])),
        )));
    }

    if value.eq_ignore_ascii_case(b"Closure") {
        return wrap_atomic(TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(true))));
    }

    wrap_atomic(TAtomic::Reference(TReference::Symbol {
        name: word(value),
        parameters: None,
        intersection_types: None,
    }))
}

fn intersection(kinds: &[TypeKind<'_>], classname: Option<Word>) -> TUnion {
    let mut atomics: Vec<TAtomic> = Vec::new();
    for kind in kinds {
        for atomic in union_from_type(kind, classname).types.into_owned() {
            if !atomic.can_be_intersected() {
                continue;
            }

            match atomics.last_mut() {
                Some(base) => {
                    base.add_intersection_type(atomic);
                }
                None => atomics.push(atomic),
            }
        }
    }

    if atomics.is_empty() {
        return get_mixed();
    }

    TUnion::from_vec(atomics)
}
