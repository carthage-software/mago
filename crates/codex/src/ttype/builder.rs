use std::borrow::Cow;
use std::collections::BTreeMap;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::i64_atom;
use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_span::Span;
use mago_type_syntax;
use mago_type_syntax::ast::object::ObjectType;
use mago_type_syntax::ast::*;

use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::callable::TCallableSignature;
use crate::ttype::atomic::callable::parameter::TCallableParameter;
use crate::ttype::atomic::conditional::TConditional;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::properties_of::TPropertiesOf;
use crate::ttype::atomic::derived::value_of::TValueOf;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::iterable::TIterable;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::error::TypeError;
use crate::ttype::get_arraykey;
use crate::ttype::get_bool;
use crate::ttype::get_closed_resource;
use crate::ttype::get_false;
use crate::ttype::get_float;
use crate::ttype::get_int;
use crate::ttype::get_literal_float;
use crate::ttype::get_literal_int;
use crate::ttype::get_literal_string;
use crate::ttype::get_lowercase_string;
use crate::ttype::get_mixed;
use crate::ttype::get_negative_int;
use crate::ttype::get_never;
use crate::ttype::get_non_empty_lowercase_string;
use crate::ttype::get_non_empty_string;
use crate::ttype::get_non_empty_unspecified_literal_string;
use crate::ttype::get_non_negative_int;
use crate::ttype::get_null;
use crate::ttype::get_nullable_float;
use crate::ttype::get_nullable_int;
use crate::ttype::get_nullable_object;
use crate::ttype::get_nullable_scalar;
use crate::ttype::get_nullable_string;
use crate::ttype::get_numeric;
use crate::ttype::get_numeric_string;
use crate::ttype::get_open_resource;
use crate::ttype::get_positive_int;
use crate::ttype::get_resource;
use crate::ttype::get_scalar;
use crate::ttype::get_string;
use crate::ttype::get_true;
use crate::ttype::get_truthy_string;
use crate::ttype::get_unspecified_literal_string;
use crate::ttype::get_void;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;

/// Parses a type string (typically from a PHPDoc comment) and resolves it
/// into a semantic `TUnion` type representation.
///
/// This function orchestrates the two main phases:
///
/// 1. Parsing the raw string into an Abstract Syntax Tree (AST) using the `mago_type_syntax` crate.
/// 2. Converting the AST into a `TUnion`, resolving names, templates, and
///    keywords into their semantic counterparts.
///
/// # Arguments
///
/// * `type_string` - The raw string slice containing the type to parse (e.g., `"int|string"`).
/// * `span` - The original `Span` of the `type_string` within its source file.
///   This is crucial for accurate error reporting and position tracking.
/// * `scope` - The `NamespaceScope` active at the location of the type string.
///   Used during conversion to resolve unqualified names, aliases (`use` statements),
///   and namespace-relative names.
/// * `type_context` - The context providing information about currently defined
///   template parameters (e.g., from `@template` tags). Needed
///   during conversion to resolve template parameter references.
/// * `classname` - An optional `Atom` representing the fully qualified name
///   of the current class context. Used during conversion to resolve
///   `self` type references. Should be `None` if not in a class context.
///
/// # Returns
///
/// * `Ok(TUnion)`: The resolved semantic type representation on success.
/// * `Err(TypeError)`: If any parsing or (future) conversion error occurs.
///   The `TypeError` can encapsulate errors originating from the
///   syntax parsing phase.
pub fn get_type_from_string(
    type_string: &str,
    span: Span,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Atom>,
) -> Result<TUnion, TypeError> {
    let ast = mago_type_syntax::parse_str(span, type_string)?;

    get_union_from_type_ast(&ast, scope, type_context, classname)
}

#[inline]
pub fn get_union_from_type_ast<'i>(
    ttype: &Type<'i>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Atom>,
) -> Result<TUnion, TypeError> {
    Ok(match ttype {
        Type::Parenthesized(parenthesized_type) => {
            get_union_from_type_ast(&parenthesized_type.inner, scope, type_context, classname)?
        }
        Type::Nullable(nullable_type) => match nullable_type.inner.as_ref() {
            Type::Null(_) => get_null(),
            Type::String(_) => get_nullable_string(),
            Type::Int(_) => get_nullable_int(),
            Type::Float(_) => get_nullable_float(),
            Type::Object(_) => get_nullable_object(),
            Type::Scalar(_) => get_nullable_scalar(),
            _ => get_union_from_type_ast(&nullable_type.inner, scope, type_context, classname)?.as_nullable(),
        },
        Type::Union(UnionType { left, right, .. }) if matches!(left.as_ref(), Type::Null(_)) => match right.as_ref() {
            Type::Null(_) => get_null(),
            Type::String(_) => get_nullable_string(),
            Type::Int(_) => get_nullable_int(),
            Type::Float(_) => get_nullable_float(),
            Type::Object(_) => get_nullable_object(),
            Type::Scalar(_) => get_nullable_scalar(),
            _ => get_union_from_type_ast(right, scope, type_context, classname)?.as_nullable(),
        },
        Type::Union(UnionType { left, right, .. }) if matches!(right.as_ref(), Type::Null(_)) => match left.as_ref() {
            Type::Null(_) => get_null(),
            Type::String(_) => get_nullable_string(),
            Type::Int(_) => get_nullable_int(),
            Type::Float(_) => get_nullable_float(),
            Type::Object(_) => get_nullable_object(),
            Type::Scalar(_) => get_nullable_scalar(),
            _ => get_union_from_type_ast(left, scope, type_context, classname)?.as_nullable(),
        },
        Type::Union(union_type) => {
            let left = get_union_from_type_ast(&union_type.left, scope, type_context, classname)?;
            let right = get_union_from_type_ast(&union_type.right, scope, type_context, classname)?;

            let combined_types: Vec<TAtomic> = left.types.iter().chain(right.types.iter()).cloned().collect();

            TUnion::from_vec(combined_types)
        }
        Type::Intersection(intersection) => {
            let left = get_union_from_type_ast(&intersection.left, scope, type_context, classname)?;
            let right = get_union_from_type_ast(&intersection.right, scope, type_context, classname)?;

            let left_str = left.get_id();
            let right_str = right.get_id();

            let left_types = left.types.into_owned();
            let right_types = right.types.into_owned();
            let mut intersection_types = vec![];
            for left_type in left_types {
                if !left_type.can_be_intersected() {
                    return Err(TypeError::InvalidType(
                        ttype.to_string(),
                        format!(
                            "Type `{}` used in intersection cannot be intersected with another type ( `{}` )",
                            left_type.get_id(),
                            right_str,
                        ),
                        ttype.span(),
                    ));
                }

                for right_type in &right_types {
                    let mut intersection = left_type.clone();

                    if !intersection.add_intersection_type(right_type.clone()) {
                        return Err(TypeError::InvalidType(
                            ttype.to_string(),
                            format!(
                                "Type `{}` used in intersection cannot be intersected with another type ( `{}` )",
                                right_type.get_id(),
                                left_str,
                            ),
                            ttype.span(),
                        ));
                    }

                    intersection_types.push(intersection);
                }
            }

            TUnion::from_vec(intersection_types)
        }
        Type::Slice(slice) => wrap_atomic(get_array_type_from_ast(
            None,
            Some(slice.inner.as_ref()),
            false,
            scope,
            type_context,
            classname,
        )?),
        Type::Array(ArrayType { parameters, .. }) | Type::AssociativeArray(AssociativeArrayType { parameters, .. }) => {
            let (key, value) = match parameters {
                Some(parameters) => {
                    let key = parameters.entries.first().map(|g| &g.inner);
                    let value = parameters.entries.get(1).map(|g| &g.inner);

                    (key, value)
                }
                None => (None, None),
            };

            wrap_atomic(get_array_type_from_ast(key, value, false, scope, type_context, classname)?)
        }
        Type::NonEmptyArray(non_empty_array) => {
            let (key, value) = match &non_empty_array.parameters {
                Some(parameters) => {
                    let key = parameters.entries.first().map(|g| &g.inner);
                    let value = parameters.entries.get(1).map(|g| &g.inner);

                    (key, value)
                }
                None => (None, None),
            };

            wrap_atomic(get_array_type_from_ast(key, value, true, scope, type_context, classname)?)
        }
        Type::List(list_type) => {
            let value = list_type.parameters.as_ref().and_then(|p| p.entries.first().map(|g| &g.inner));

            wrap_atomic(get_list_type_from_ast(value, false, scope, type_context, classname)?)
        }
        Type::NonEmptyList(non_empty_list_type) => {
            let value = non_empty_list_type.parameters.as_ref().and_then(|p| p.entries.first().map(|g| &g.inner));

            wrap_atomic(get_list_type_from_ast(value, true, scope, type_context, classname)?)
        }
        Type::ClassString(class_string_type) => get_class_string_type_from_ast(
            class_string_type.span(),
            TClassLikeStringKind::Class,
            &class_string_type.parameter,
            scope,
            type_context,
            classname,
        )?,
        Type::InterfaceString(interface_string_type) => get_class_string_type_from_ast(
            interface_string_type.span(),
            TClassLikeStringKind::Interface,
            &interface_string_type.parameter,
            scope,
            type_context,
            classname,
        )?,
        Type::EnumString(enum_string_type) => get_class_string_type_from_ast(
            enum_string_type.span(),
            TClassLikeStringKind::Enum,
            &enum_string_type.parameter,
            scope,
            type_context,
            classname,
        )?,
        Type::TraitString(trait_string_type) => get_class_string_type_from_ast(
            trait_string_type.span(),
            TClassLikeStringKind::Trait,
            &trait_string_type.parameter,
            scope,
            type_context,
            classname,
        )?,
        Type::MemberReference(member_reference) => {
            let class_like_name = if member_reference.class.value.eq_ignore_ascii_case("self")
                || member_reference.class.value.eq_ignore_ascii_case("static")
                || member_reference.class.value.eq("this")
                || member_reference.class.value.eq("$this")
            {
                let Some(classname) = classname else {
                    return Err(TypeError::InvalidType(
                        ttype.to_string(),
                        "Cannot resolve `self` type reference outside of a class context".to_string(),
                        member_reference.span(),
                    ));
                };

                classname
            } else {
                let (class_like_name, _) = scope.resolve(NameKind::Default, member_reference.class.value);

                atom(&class_like_name)
            };

            let member_selector = match member_reference.member {
                MemberReferenceSelector::Wildcard(_) => TReferenceMemberSelector::Wildcard,
                MemberReferenceSelector::Identifier(identifier) => {
                    TReferenceMemberSelector::Identifier(atom(identifier.value))
                }
                MemberReferenceSelector::StartsWith(identifier, _) => {
                    TReferenceMemberSelector::StartsWith(atom(identifier.value))
                }
                MemberReferenceSelector::EndsWith(_, identifier) => {
                    TReferenceMemberSelector::EndsWith(atom(identifier.value))
                }
            };

            wrap_atomic(TAtomic::Reference(TReference::Member { class_like_name, member_selector }))
        }
        Type::Object(object_type) => wrap_atomic(get_object_from_ast(object_type, scope, type_context, classname)?),
        Type::Shape(shape_type) => wrap_atomic(get_shape_from_ast(shape_type, scope, type_context, classname)?),
        Type::Callable(callable_type) => {
            wrap_atomic(get_callable_from_ast(callable_type, scope, type_context, classname)?)
        }
        Type::Reference(reference_type) => wrap_atomic(get_reference_from_ast(
            &reference_type.identifier,
            reference_type.parameters.as_ref(),
            scope,
            type_context,
            classname,
        )?),
        Type::Mixed(_) => get_mixed(),
        Type::Null(_) => get_null(),
        Type::Void(_) => get_void(),
        Type::Never(_) => get_never(),
        Type::Resource(_) => get_resource(),
        Type::ClosedResource(_) => get_closed_resource(),
        Type::OpenResource(_) => get_open_resource(),
        Type::True(_) => get_true(),
        Type::False(_) => get_false(),
        Type::Bool(_) => get_bool(),
        Type::Float(_) => get_float(),
        Type::Int(_) => get_int(),
        Type::String(_) => get_string(),
        Type::ArrayKey(_) => get_arraykey(),
        Type::Numeric(_) => get_numeric(),
        Type::Scalar(_) => get_scalar(),
        Type::NumericString(_) => get_numeric_string(),
        Type::NonEmptyString(_) => get_non_empty_string(),
        Type::TruthyString(_) | Type::NonFalsyString(_) => get_truthy_string(),
        Type::UnspecifiedLiteralString(_) => get_unspecified_literal_string(),
        Type::NonEmptyUnspecifiedLiteralString(_) => get_non_empty_unspecified_literal_string(),
        Type::NonEmptyLowercaseString(_) => get_non_empty_lowercase_string(),
        Type::LowercaseString(_) => get_lowercase_string(),
        Type::LiteralFloat(lit) => get_literal_float(*lit.value),
        Type::LiteralInt(lit) => get_literal_int(lit.value as i64),
        Type::LiteralString(lit) => get_literal_string(atom(lit.value)),
        Type::Negated(negated) => match negated.number {
            LiteralIntOrFloatType::Int(lit) => get_literal_int(-(lit.value as i64)),
            LiteralIntOrFloatType::Float(lit) => get_literal_float(-(*lit.value)),
        },
        Type::Posited(posited) => match posited.number {
            LiteralIntOrFloatType::Int(lit) => get_literal_int(lit.value as i64),
            LiteralIntOrFloatType::Float(lit) => get_literal_float(*lit.value),
        },
        Type::Iterable(iterable) => match iterable.parameters.as_ref() {
            Some(parameters) => match parameters.entries.len() {
                0 => wrap_atomic(TAtomic::Iterable(TIterable::mixed())),
                1 => {
                    let value_type =
                        get_union_from_type_ast(&parameters.entries[0].inner, scope, type_context, classname)?;

                    wrap_atomic(TAtomic::Iterable(TIterable::of_value(Box::new(value_type))))
                }
                _ => {
                    let key_type =
                        get_union_from_type_ast(&parameters.entries[0].inner, scope, type_context, classname)?;

                    let value_type =
                        get_union_from_type_ast(&parameters.entries[1].inner, scope, type_context, classname)?;

                    wrap_atomic(TAtomic::Iterable(TIterable::new(Box::new(key_type), Box::new(value_type))))
                }
            },
            None => wrap_atomic(TAtomic::Iterable(TIterable::mixed())),
        },
        Type::PositiveInt(_) => get_positive_int(),
        Type::NegativeInt(_) => get_negative_int(),
        Type::NonPositiveInt(_) => get_positive_int(),
        Type::NonNegativeInt(_) => get_non_negative_int(),
        Type::IntRange(range) => {
            let min = match range.min {
                IntOrKeyword::NegativeInt { int, .. } => Some(-(int.value as i64)),
                IntOrKeyword::Int(literal_int_type) => Some(literal_int_type.value as i64),
                IntOrKeyword::Keyword(_) => None,
            };

            let max = match range.max {
                IntOrKeyword::NegativeInt { int, .. } => Some(-(int.value as i64)),
                IntOrKeyword::Int(literal_int_type) => Some(literal_int_type.value as i64),
                IntOrKeyword::Keyword(_) => None,
            };

            if let (Some(min_value), Some(max_value)) = (min, max)
                && min_value > max_value
            {
                return Err(TypeError::InvalidType(
                    ttype.to_string(),
                    "Minimum value of an int range cannot be greater than maximum value".to_string(),
                    ttype.span(),
                ));
            }

            TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::Integer(TInteger::from_bounds(min, max)))))
        }
        Type::Conditional(conditional) => TUnion::from_single(Cow::Owned(TAtomic::Conditional(TConditional::new(
            Box::new(get_union_from_type_ast(&conditional.subject, scope, type_context, classname)?),
            Box::new(get_union_from_type_ast(&conditional.target, scope, type_context, classname)?),
            Box::new(get_union_from_type_ast(&conditional.then, scope, type_context, classname)?),
            Box::new(get_union_from_type_ast(&conditional.otherwise, scope, type_context, classname)?),
            conditional.is_negated(),
        )))),
        Type::Variable(variable_type) => TUnion::from_single(Cow::Owned(TAtomic::Variable(atom(variable_type.value)))),
        Type::KeyOf(key_of_type) => {
            let target = get_union_from_type_ast(&key_of_type.parameter.entry.inner, scope, type_context, classname)?;

            let mut atomics = vec![];
            for target_type in target.types.into_owned() {
                atomics.push(TAtomic::Derived(TDerived::KeyOf(TKeyOf::new(Box::new(target_type)))));
            }

            TUnion::from_vec(atomics)
        }
        Type::ValueOf(value_of_type) => {
            let target = get_union_from_type_ast(&value_of_type.parameter.entry.inner, scope, type_context, classname)?;

            let mut atomics = vec![];
            for target_type in target.types.into_owned() {
                atomics.push(TAtomic::Derived(TDerived::ValueOf(TValueOf::new(Box::new(target_type)))));
            }

            TUnion::from_vec(atomics)
        }
        Type::PropertiesOf(properties_of_type) => {
            let target =
                get_union_from_type_ast(&properties_of_type.parameter.entry.inner, scope, type_context, classname)?;

            let mut atomics = vec![];
            for target_type in target.types.into_owned() {
                atomics.push(TAtomic::Derived(TDerived::PropertiesOf(match properties_of_type.filter {
                    PropertiesOfFilter::All => TPropertiesOf::new(Box::new(target_type)),
                    PropertiesOfFilter::Public => TPropertiesOf::public(Box::new(target_type)),
                    PropertiesOfFilter::Protected => TPropertiesOf::protected(Box::new(target_type)),
                    PropertiesOfFilter::Private => TPropertiesOf::private(Box::new(target_type)),
                })));
            }

            TUnion::from_vec(atomics)
        }
        _ => {
            return Err(TypeError::UnsupportedType(ttype.to_string(), ttype.span()));
        }
    })
}

#[inline]
fn get_object_from_ast(
    object: &ObjectType<'_>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Atom>,
) -> Result<TAtomic, TypeError> {
    let Some(properties) = object.properties.as_ref() else {
        return Ok(TAtomic::Object(TObject::Any));
    };

    let mut known_properties = BTreeMap::new();
    for property in &properties.fields {
        let property_is_optional = property.is_optional();

        let Some(field_key) = property.key.as_ref() else {
            continue;
        };

        let key = match field_key.key {
            ShapeKey::String { value, .. } => atom(value),
            ShapeKey::Integer { value, .. } => i64_atom(value),
        };

        let property_type = get_union_from_type_ast(&property.value, scope, type_context, classname)?;

        known_properties.insert(key, (property_is_optional, property_type));
    }

    Ok(TAtomic::Object(TObject::new_with_properties(properties.ellipsis.is_none(), known_properties)))
}

#[inline]
fn get_shape_from_ast(
    shape: &ShapeType<'_>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Atom>,
) -> Result<TAtomic, TypeError> {
    if shape.kind.is_list() {
        let mut list = TList::new(match &shape.additional_fields {
            Some(additional_fields) => match &additional_fields.parameters {
                Some(parameters) => Box::new(if let Some(k) = parameters.entries.first().map(|g| &g.inner) {
                    get_union_from_type_ast(k, scope, type_context, classname)?
                } else {
                    get_mixed()
                }),
                None => Box::new(get_mixed()),
            },
            None => Box::new(get_never()),
        });

        list.known_elements = Some({
            let mut tree = BTreeMap::new();
            let mut next_offset: usize = 0;

            for field in &shape.fields {
                let field_is_optional = field.is_optional();

                let offset = match field.key.as_ref() {
                    Some(field_key) => {
                        let array_key = match field_key.key {
                            ShapeKey::String { value, .. } => ArrayKey::String(atom(value)),
                            ShapeKey::Integer { value, .. } => ArrayKey::Integer(value),
                        };

                        if let ArrayKey::Integer(offset) = array_key {
                            if offset > 0 && (offset as usize) == next_offset {
                                next_offset += 1;

                                offset as usize
                            } else {
                                return Err(TypeError::InvalidType(
                                    shape.to_string(),
                                    "List shape keys must be sequential".to_string(),
                                    field_key.span(),
                                ));
                            }
                        } else {
                            return Err(TypeError::InvalidType(
                                shape.to_string(),
                                "List shape keys are expected to be integers".to_string(),
                                field_key.span(),
                            ));
                        }
                    }
                    None => {
                        let offset = next_offset;

                        next_offset += 1;

                        offset
                    }
                };

                let mut field_value_type = get_union_from_type_ast(&field.value, scope, type_context, classname)?;
                if field_is_optional {
                    field_value_type.possibly_undefined = true;
                }

                tree.insert(offset, (field_is_optional, field_value_type));
            }

            tree
        });

        list.non_empty = shape.has_non_optional_fields() || shape.kind.is_non_empty();

        Ok(TAtomic::Array(TArray::List(list)))
    } else {
        let mut keyed_array = TKeyedArray::new();

        keyed_array.parameters = match &shape.additional_fields {
            Some(additional_fields) => Some(match &additional_fields.parameters {
                Some(parameters) => (
                    Box::new(if let Some(k) = parameters.entries.first().map(|g| &g.inner) {
                        get_union_from_type_ast(k, scope, type_context, classname)?
                    } else {
                        get_mixed()
                    }),
                    Box::new(if let Some(v) = parameters.entries.get(1).map(|g| &g.inner) {
                        get_union_from_type_ast(v, scope, type_context, classname)?
                    } else {
                        get_mixed()
                    }),
                ),
                None => (Box::new(get_arraykey()), Box::new(get_mixed())),
            }),
            None => None,
        };

        keyed_array.known_items = Some({
            let mut tree = BTreeMap::new();
            let mut next_offset = 0;

            for field in &shape.fields {
                let field_is_optional = field.is_optional();

                let array_key = match field.key.as_ref() {
                    Some(field_key) => {
                        let array_key = match field_key.key {
                            ShapeKey::String { value, .. } => ArrayKey::String(atom(value)),
                            ShapeKey::Integer { value, .. } => ArrayKey::Integer(value),
                        };

                        if let ArrayKey::Integer(offset) = array_key
                            && offset >= next_offset
                        {
                            next_offset = offset + 1;
                        }

                        array_key
                    }
                    None => {
                        let array_key = ArrayKey::Integer(next_offset);

                        next_offset += 1;

                        array_key
                    }
                };

                let mut field_value_type = get_union_from_type_ast(&field.value, scope, type_context, classname)?;
                if field_is_optional {
                    field_value_type.possibly_undefined = true;
                }

                tree.insert(array_key, (field_is_optional, field_value_type));
            }

            tree
        });

        keyed_array.non_empty = shape.has_non_optional_fields() || shape.kind.is_non_empty();

        Ok(TAtomic::Array(TArray::Keyed(keyed_array)))
    }
}

#[inline]
fn get_callable_from_ast(
    callable: &CallableType<'_>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Atom>,
) -> Result<TAtomic, TypeError> {
    let mut parameters = vec![];
    let mut return_type = None;

    if let Some(specification) = &callable.specification {
        for parameter_ast in specification.parameters.entries.iter() {
            let parameter_type = if let Some(parameter_type) = &parameter_ast.parameter_type {
                get_union_from_type_ast(parameter_type, scope, type_context, classname)?
            } else {
                get_mixed()
            };

            parameters.push(TCallableParameter::new(
                Some(Box::new(parameter_type)),
                false,
                parameter_ast.is_variadic(),
                parameter_ast.is_optional(),
            ));
        }

        if let Some(ret) = specification.return_type.as_ref() {
            return_type = Some(get_union_from_type_ast(&ret.return_type, scope, type_context, classname)?);
        }
    } else {
        // `callable` without a specification should be treated the same as
        // `callable(mixed...): mixed`
        parameters.push(TCallableParameter::new(Some(Box::new(get_mixed())), false, true, false));
        return_type = Some(get_mixed());
    }

    Ok(TAtomic::Callable(TCallable::Signature(
        TCallableSignature::new(callable.kind.is_pure(), callable.kind.is_closure())
            .with_parameters(parameters)
            .with_return_type(return_type.map(Box::new)),
    )))
}

#[inline]
fn get_reference_from_ast<'i>(
    reference_identifier: &Identifier<'i>,
    generics: Option<&GenericParameters<'i>>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Atom>,
) -> Result<TAtomic, TypeError> {
    let reference_name = reference_identifier.value;

    let mut is_this = false;
    let mut is_named_object = false;
    let fq_reference_name_id = if reference_name == "this" || reference_name == "static" || reference_name == "self" {
        is_named_object = true;
        is_this = reference_name != "self";

        classname.unwrap_or_else(|| atom("static"))
    } else {
        if let Some(defining_entities) = type_context.get_template_definition(reference_name)
            && generics.is_none()
        {
            return Ok(get_template_atomic(defining_entities, atom(reference_name)));
        }

        let (fq_reference_name, _) = scope.resolve(NameKind::Default, reference_name);

        // `Closure` -> `Closure(mixed...): mixed`
        if fq_reference_name.eq_ignore_ascii_case("Closure") && generics.is_none() {
            return Ok(TAtomic::Callable(TCallable::Signature(
                TCallableSignature::new(false, true)
                    .with_parameters(vec![TCallableParameter::new(Some(Box::new(get_mixed())), false, true, false)])
                    .with_return_type(Some(Box::new(get_mixed()))),
            )));
        }

        atom(&fq_reference_name)
    };

    let mut type_parameters = None;
    if let Some(generics) = generics {
        let mut parameters = vec![];
        for generic in &generics.entries {
            let generic_type = get_union_from_type_ast(&generic.inner, scope, type_context, classname)?;

            parameters.push(generic_type);
        }

        type_parameters = Some(parameters);
    }

    let is_generator = fq_reference_name_id.eq_ignore_ascii_case("Generator");

    let is_iterator = is_generator
        || fq_reference_name_id.eq_ignore_ascii_case("Iterator")
        || fq_reference_name_id.eq_ignore_ascii_case("IteratorAggregate")
        || fq_reference_name_id.eq_ignore_ascii_case("Traversable");

    'iterator: {
        if !is_iterator {
            break 'iterator;
        }

        let Some(type_parameters) = &mut type_parameters else {
            type_parameters = Some(vec![get_mixed(), get_mixed()]);

            break 'iterator;
        };

        if type_parameters.len() == 1 {
            type_parameters.insert(0, get_mixed());
        } else if type_parameters.is_empty() {
            type_parameters.push(get_mixed());
            type_parameters.push(get_mixed());
        }

        if !is_generator {
            break 'iterator;
        }

        while type_parameters.len() < 4 {
            type_parameters.push(get_mixed());
        }
    }

    if is_named_object {
        Ok(TAtomic::Object(TObject::Named(TNamedObject {
            name: fq_reference_name_id,
            type_parameters,
            intersection_types: None,
            is_this,
            remapped_parameters: false,
        })))
    } else {
        Ok(TAtomic::Reference(TReference::Symbol {
            name: fq_reference_name_id,
            parameters: type_parameters,
            intersection_types: None,
        }))
    }
}

#[inline]
fn get_array_type_from_ast<'i, 'p>(
    mut key: Option<&'p Type<'i>>,
    mut value: Option<&'p Type<'i>>,
    non_empty: bool,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Atom>,
) -> Result<TAtomic, TypeError> {
    if key.is_some() && value.is_none() {
        std::mem::swap(&mut key, &mut value);
    }

    let mut array = TKeyedArray::new_with_parameters(
        Box::new(if let Some(k) = key {
            get_union_from_type_ast(k, scope, type_context, classname)?
        } else {
            get_arraykey()
        }),
        Box::new(if let Some(v) = value {
            get_union_from_type_ast(v, scope, type_context, classname)?
        } else {
            get_mixed()
        }),
    );

    array.non_empty = non_empty;

    Ok(TAtomic::Array(TArray::Keyed(array)))
}

#[inline]
fn get_list_type_from_ast(
    value: Option<&Type<'_>>,
    non_empty: bool,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Atom>,
) -> Result<TAtomic, TypeError> {
    Ok(TAtomic::Array(TArray::List(TList {
        element_type: Box::new(if let Some(v) = value {
            get_union_from_type_ast(v, scope, type_context, classname)?
        } else {
            get_mixed()
        }),
        known_count: None,
        known_elements: None,
        non_empty,
    })))
}

#[inline]
fn get_class_string_type_from_ast(
    span: Span,
    kind: TClassLikeStringKind,
    parameter: &Option<SingleGenericParameter<'_>>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Atom>,
) -> Result<TUnion, TypeError> {
    Ok(match parameter {
        Some(parameter) => {
            let constraint_union = get_union_from_type_ast(&parameter.entry.inner, scope, type_context, classname)?;

            let mut class_strings = vec![];
            for constraint in constraint_union.types.into_owned() {
                match constraint {
                    TAtomic::Object(TObject::Named(_))
                    | TAtomic::Object(TObject::Enum(_))
                    | TAtomic::Reference(TReference::Symbol { .. }) => class_strings
                        .push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::of_type(kind, constraint)))),
                    TAtomic::GenericParameter(TGenericParameter {
                        parameter_name,
                        defining_entity,
                        constraint,
                        ..
                    }) => {
                        for constraint_atomic in constraint.types.into_owned() {
                            class_strings.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::generic(
                                kind,
                                parameter_name,
                                defining_entity,
                                constraint_atomic,
                            ))));
                        }
                    }
                    _ => {
                        return Err(TypeError::InvalidType(
                            kind.to_string(),
                            format!(
                                "class string parameter must target an object type, found `{}`.",
                                constraint.get_id()
                            ),
                            span,
                        ));
                    }
                }
            }

            TUnion::from_vec(class_strings)
        }
        None => wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::any(kind)))),
    })
}

#[inline]
fn get_template_atomic(defining_entities: &[(GenericParent, TUnion)], parameter_name: Atom) -> TAtomic {
    let (defining_entity, constraint) = &defining_entities[0];

    TAtomic::GenericParameter(TGenericParameter {
        parameter_name,
        constraint: Box::new(constraint.clone()),
        defining_entity: *defining_entity,
        intersection_types: None,
    })
}
