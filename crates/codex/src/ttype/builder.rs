use std::collections::BTreeMap;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_span::Span;
use mago_type_syntax;
use mago_type_syntax::ast::*;

use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::callable::TCallableSignature;
use crate::ttype::atomic::callable::parameter::TCallableParameter;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::properties_of::TPropertiesOf;
use crate::ttype::atomic::derived::value_of::TValueOf;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
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
use crate::ttype::get_mixed;
use crate::ttype::get_never;
use crate::ttype::get_non_empty_string;
use crate::ttype::get_non_empty_unspecified_literal_string;
use crate::ttype::get_null;
use crate::ttype::get_numeric;
use crate::ttype::get_numeric_string;
use crate::ttype::get_object;
use crate::ttype::get_open_resource;
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

use super::atomic::array::key::ArrayKey;
use super::atomic::array::list::TList;
use super::atomic::conditional::TConditional;
use super::atomic::generic::TGenericParameter;
use super::atomic::iterable::TIterable;
use super::atomic::reference::TReference;
use super::atomic::scalar::int::TInteger;

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
/// * `classname` - An optional `StringIdentifier` representing the fully qualified name
///   of the current class context. Used during conversion to resolve
///   `self` type references. Should be `None` if not in a class context.
/// * `interner` - The shared `ThreadedInterner` instance for string interning and comparison.
///
/// # Returns
///
/// * `Ok(TUnion)`: The resolved semantic type representation on success.
/// * `Err(TypeError)`: If any parsing or (future) conversion error occurs.
///   The `TypeError` can encapsulate errors originating from the
///   syntax parsing phase.
pub fn get_type_from_string<'i>(
    type_string: &'i str,
    span: Span,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<&StringIdentifier>,
    interner: &'i ThreadedInterner,
) -> Result<TUnion, TypeError> {
    let ast = mago_type_syntax::parse_str(span, type_string)?;

    get_union_from_type_ast(&ast, scope, type_context, classname, interner)
}

#[inline]
pub fn get_union_from_type_ast<'i>(
    ttype: &Type<'i>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<&StringIdentifier>,
    interner: &'i ThreadedInterner,
) -> Result<TUnion, TypeError> {
    Ok(match ttype {
        Type::Parenthesized(parenthesized_type) => {
            get_union_from_type_ast(&parenthesized_type.inner, scope, type_context, classname, interner)?
        }
        Type::Union(union_type) => {
            let left = get_union_from_type_ast(&union_type.left, scope, type_context, classname, interner)?;
            let right = get_union_from_type_ast(&union_type.right, scope, type_context, classname, interner)?;

            TUnion::new(left.types.into_iter().chain(right.types).collect())
        }
        Type::Nullable(nullable_type) => {
            let mut inner = get_union_from_type_ast(&nullable_type.inner, scope, type_context, classname, interner)?;
            inner.types.push(TAtomic::Null);
            inner.types.shrink_to_fit();

            inner
        }
        Type::Intersection(intersection) => {
            let left = get_union_from_type_ast(&intersection.left, scope, type_context, classname, interner)?;
            let right = get_union_from_type_ast(&intersection.right, scope, type_context, classname, interner)?;

            let left_str = left.get_id(Some(interner));
            let right_str = right.get_id(Some(interner));

            let left_types = left.types;
            let right_types = right.types;
            let mut intersection_types = vec![];
            for left_type in left_types {
                if !left_type.can_be_intersected() {
                    return Err(TypeError::InvalidType(
                        format!(
                            "Type `{}` used in intersection cannot be intersected with another type ( `{}` )",
                            left_type.get_id(Some(interner)),
                            right_str,
                        ),
                        ttype.span(),
                    ));
                }

                for right_type in &right_types {
                    let mut intersection = left_type.clone();

                    if !intersection.add_intersection_type(right_type.clone()) {
                        return Err(TypeError::InvalidType(
                            format!(
                                "Type `{}` used in intersection cannot be intersected with another type ( `{}` )",
                                right_type.get_id(Some(interner)),
                                left_str,
                            ),
                            ttype.span(),
                        ));
                    }

                    intersection_types.push(intersection);
                }
            }

            TUnion::new(intersection_types)
        }
        Type::Slice(slice) => wrap_atomic(get_array_type_from_ast(
            None,
            Some(slice.inner.as_ref()),
            false,
            scope,
            type_context,
            classname,
            interner,
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

            wrap_atomic(get_array_type_from_ast(key, value, false, scope, type_context, classname, interner)?)
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

            wrap_atomic(get_array_type_from_ast(key, value, true, scope, type_context, classname, interner)?)
        }
        Type::List(list_type) => {
            let value = list_type.parameters.as_ref().and_then(|p| p.entries.first().map(|g| &g.inner));

            wrap_atomic(get_list_type_from_ast(value, false, scope, type_context, classname, interner)?)
        }
        Type::NonEmptyList(non_empty_list_type) => {
            let value = non_empty_list_type.parameters.as_ref().and_then(|p| p.entries.first().map(|g| &g.inner));

            wrap_atomic(get_list_type_from_ast(value, true, scope, type_context, classname, interner)?)
        }
        Type::ClassString(class_string_type) => get_class_string_type_from_ast(
            class_string_type.span(),
            TClassLikeStringKind::Class,
            &class_string_type.parameter,
            scope,
            type_context,
            classname,
            interner,
        )?,
        Type::InterfaceString(interface_string_type) => get_class_string_type_from_ast(
            interface_string_type.span(),
            TClassLikeStringKind::Interface,
            &interface_string_type.parameter,
            scope,
            type_context,
            classname,
            interner,
        )?,
        Type::EnumString(enum_string_type) => get_class_string_type_from_ast(
            enum_string_type.span(),
            TClassLikeStringKind::Enum,
            &enum_string_type.parameter,
            scope,
            type_context,
            classname,
            interner,
        )?,
        Type::TraitString(trait_string_type) => get_class_string_type_from_ast(
            trait_string_type.span(),
            TClassLikeStringKind::Trait,
            &trait_string_type.parameter,
            scope,
            type_context,
            classname,
            interner,
        )?,
        Type::MemberReference(member_reference) => {
            let class_like_name = if member_reference.class.value.eq_ignore_ascii_case("self")
                || member_reference.class.value.eq_ignore_ascii_case("static")
                || member_reference.class.value.eq("this")
                || member_reference.class.value.eq("$this")
            {
                let Some(classname) = classname else {
                    return Err(TypeError::InvalidType(
                        "Cannot resolve `self` type reference outside of a class context".to_string(),
                        member_reference.span(),
                    ));
                };

                *classname
            } else {
                let (class_like_name, _) = scope.resolve(NameKind::Default, member_reference.class.value);

                interner.intern(&class_like_name)
            };

            let member_selector = match member_reference.member {
                MemberReferenceSelector::Wildcard(_) => TReferenceMemberSelector::Wildcard,
                MemberReferenceSelector::Identifier(identifier) => {
                    TReferenceMemberSelector::Identifier(interner.intern(identifier.value))
                }
                MemberReferenceSelector::StartsWith(identifier, _) => {
                    TReferenceMemberSelector::StartsWith(interner.intern(identifier.value))
                }
                MemberReferenceSelector::EndsWith(_, identifier) => {
                    TReferenceMemberSelector::EndsWith(interner.intern(identifier.value))
                }
            };

            wrap_atomic(TAtomic::Reference(TReference::Member { class_like_name, member_selector }))
        }
        Type::Shape(shape_type) => {
            wrap_atomic(get_shape_from_ast(shape_type, scope, type_context, classname, interner)?)
        }
        Type::Callable(callable_type) => {
            wrap_atomic(get_callable_from_ast(callable_type, scope, type_context, classname, interner)?)
        }
        Type::Reference(reference_type) => wrap_atomic(get_reference_from_ast(
            &reference_type.identifier,
            reference_type.parameters.as_ref(),
            scope,
            type_context,
            classname,
            interner,
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
        Type::Object(_) => get_object(),
        Type::Numeric(_) => get_numeric(),
        Type::Scalar(_) => get_scalar(),
        Type::NumericString(_) => get_numeric_string(),
        Type::NonEmptyString(_) => get_non_empty_string(),
        Type::TruthyString(_) => get_truthy_string(),
        Type::UnspecifiedLiteralString(_) => get_unspecified_literal_string(),
        Type::NonEmptyUnspecifiedLiteralString(_) => get_non_empty_unspecified_literal_string(),
        Type::LiteralFloat(lit) => get_literal_float(*lit.value),
        Type::LiteralInt(lit) => get_literal_int(lit.value as i64),
        Type::LiteralString(lit) => get_literal_string(lit.value.to_owned()),
        Type::Negated(negated) => match negated.inner.as_ref() {
            Type::LiteralInt(lit) => get_literal_int(-(lit.value as i64)),
            Type::LiteralFloat(lit) => get_literal_float(-(*lit.value)),
            inner => return Err(TypeError::InvalidType(format!("Type `{inner}` cannot be negated"), ttype.span())),
        },
        Type::Posited(posited) => match posited.inner.as_ref() {
            Type::LiteralInt(lit) => get_literal_int(lit.value as i64),
            Type::LiteralFloat(lit) => get_literal_float(*lit.value),
            inner => return Err(TypeError::InvalidType(format!("Type `{inner}` cannot be posited"), ttype.span())),
        },
        Type::Iterable(iterable) => match iterable.parameters.as_ref() {
            Some(parameters) => match parameters.entries.len() {
                0 => wrap_atomic(TAtomic::Iterable(TIterable::mixed())),
                1 => {
                    let value_type = get_union_from_type_ast(
                        &parameters.entries[0].inner,
                        scope,
                        type_context,
                        classname,
                        interner,
                    )?;

                    wrap_atomic(TAtomic::Iterable(TIterable::of_value(Box::new(value_type))))
                }
                _ => {
                    let key_type = get_union_from_type_ast(
                        &parameters.entries[0].inner,
                        scope,
                        type_context,
                        classname,
                        interner,
                    )?;

                    let value_type = get_union_from_type_ast(
                        &parameters.entries[1].inner,
                        scope,
                        type_context,
                        classname,
                        interner,
                    )?;

                    wrap_atomic(TAtomic::Iterable(TIterable::new(Box::new(key_type), Box::new(value_type))))
                }
            },
            None => wrap_atomic(TAtomic::Iterable(TIterable::mixed())),
        },
        Type::PositiveInt(_) => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::positive()))]),
        Type::NegativeInt(_) => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::negative()))]),
        Type::NonPositiveInt(_) => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::non_positive()))]),
        Type::NonNegativeInt(_) => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::non_negative()))]),
        Type::IntRange(range) => {
            match (&range.min, &range.max) {
                (IntOrKeyword::Int(min), IntOrKeyword::Int(max)) => {
                    if min.value > max.value {
                        return Err(TypeError::InvalidType(
                            "Minimum value of an int range cannot be greater than maximum value".to_string(),
                            ttype.span(),
                        ));
                    }

                    TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::from_bounds(
                        Some(min.value as i64),
                        Some(max.value as i64),
                    )))])
                }
                (IntOrKeyword::Int(min), _) => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(
                    TInteger::from_bounds(Some(min.value as i64), None),
                ))]),
                (_, IntOrKeyword::Int(max)) => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(
                    TInteger::from_bounds(None, Some(max.value as i64)),
                ))]),
                (_, _) => get_int(),
            }
        }
        Type::Conditional(conditional) => TUnion::new(vec![TAtomic::Conditional(TConditional::new(
            Box::new(get_union_from_type_ast(&conditional.subject, scope, type_context, classname, interner)?),
            Box::new(get_union_from_type_ast(&conditional.target, scope, type_context, classname, interner)?),
            Box::new(get_union_from_type_ast(&conditional.then, scope, type_context, classname, interner)?),
            Box::new(get_union_from_type_ast(&conditional.otherwise, scope, type_context, classname, interner)?),
            conditional.is_negated(),
        ))]),
        Type::Variable(variable_type) => TUnion::new(vec![TAtomic::Variable(interner.intern(variable_type.value))]),
        Type::KeyOf(key_of_type) => {
            let target =
                get_union_from_type_ast(&key_of_type.parameter.entry.inner, scope, type_context, classname, interner)?;

            let mut atomics = vec![];
            for target_type in target.types {
                atomics.push(TAtomic::Derived(TDerived::KeyOf(TKeyOf::new(Box::new(target_type)))));
            }

            TUnion::new(atomics)
        }
        Type::ValueOf(value_of_type) => {
            let target = get_union_from_type_ast(
                &value_of_type.parameter.entry.inner,
                scope,
                type_context,
                classname,
                interner,
            )?;

            let mut atomics = vec![];
            for target_type in target.types {
                atomics.push(TAtomic::Derived(TDerived::ValueOf(TValueOf::new(Box::new(target_type)))));
            }

            TUnion::new(atomics)
        }
        Type::PropertiesOf(properties_of_type) => {
            let target = get_union_from_type_ast(
                &properties_of_type.parameter.entry.inner,
                scope,
                type_context,
                classname,
                interner,
            )?;

            let mut atomics = vec![];
            for target_type in target.types {
                atomics.push(TAtomic::Derived(TDerived::PropertiesOf(match properties_of_type.filter {
                    PropertiesOfFilter::All => TPropertiesOf::new(Box::new(target_type)),
                    PropertiesOfFilter::Public => TPropertiesOf::public(Box::new(target_type)),
                    PropertiesOfFilter::Protected => TPropertiesOf::protected(Box::new(target_type)),
                    PropertiesOfFilter::Private => TPropertiesOf::private(Box::new(target_type)),
                })));
            }

            TUnion::new(atomics)
        }
        _ => {
            return Err(TypeError::UnsupportedType(ttype.to_string(), ttype.span()));
        }
    })
}

#[inline]
fn get_shape_from_ast(
    shape: &ShapeType<'_>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<&StringIdentifier>,
    interner: &ThreadedInterner,
) -> Result<TAtomic, TypeError> {
    if shape.kind.is_list() {
        let mut list = TList::new(match &shape.additional_fields {
            Some(additional_fields) => match &additional_fields.parameters {
                Some(parameters) => Box::new(if let Some(k) = parameters.entries.first().map(|g| &g.inner) {
                    get_union_from_type_ast(k, scope, type_context, classname, interner)?
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
                        let field_key_type =
                            get_union_from_type_ast(&field_key.name, scope, type_context, classname, interner)?;
                        let single_key_type = field_key_type.get_single_owned();

                        let array_key = match single_key_type.to_array_key() {
                            Some(array_key) => array_key,
                            None => match single_key_type {
                                TAtomic::Reference(TReference::Symbol {
                                    name,
                                    parameters,
                                    intersection_types: None,
                                }) if parameters.is_none() => {
                                    let name_str = interner.lookup(&name);
                                    let last_part = name_str.split("\\").last().unwrap_or(name_str);

                                    ArrayKey::String(last_part.to_string())
                                }
                                _ => {
                                    return Err(TypeError::InvalidType(
                                        format!(
                                            "Shape key must be a literal string or int, found `{}`",
                                            single_key_type.get_id(Some(interner))
                                        ),
                                        field_key.span(),
                                    ));
                                }
                            },
                        };

                        if let ArrayKey::Integer(offset) = array_key {
                            if offset > 0 && (offset as usize) == next_offset {
                                next_offset += 1;

                                offset as usize
                            } else {
                                return Err(TypeError::InvalidType(
                                    "List shape keys must be sequential".to_string(),
                                    field_key.span(),
                                ));
                            }
                        } else {
                            return Err(TypeError::InvalidType(
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

                let field_value_type = get_union_from_type_ast(&field.value, scope, type_context, classname, interner)?;

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
                        get_union_from_type_ast(k, scope, type_context, classname, interner)?
                    } else {
                        get_mixed()
                    }),
                    Box::new(if let Some(v) = parameters.entries.get(1).map(|g| &g.inner) {
                        get_union_from_type_ast(v, scope, type_context, classname, interner)?
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
                        let field_key_type =
                            get_union_from_type_ast(&field_key.name, scope, type_context, classname, interner)?;

                        let single_key_type = field_key_type.get_single_owned();
                        let array_key = match single_key_type.to_array_key() {
                            Some(array_key) => array_key,
                            None => match single_key_type {
                                TAtomic::Reference(TReference::Symbol {
                                    name,
                                    parameters,
                                    intersection_types: None,
                                }) if parameters.is_none() => {
                                    let name_str = interner.lookup(&name);
                                    let last_part = name_str.split("\\").last().unwrap_or(name_str);

                                    ArrayKey::String(last_part.to_string())
                                }
                                _ => {
                                    return Err(TypeError::InvalidType(
                                        format!(
                                            "Shape key must be a literal string or int, found `{}`",
                                            single_key_type.get_id(Some(interner))
                                        ),
                                        field_key.span(),
                                    ));
                                }
                            },
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

                let field_value_type = get_union_from_type_ast(&field.value, scope, type_context, classname, interner)?;

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
    classname: Option<&StringIdentifier>,
    interner: &ThreadedInterner,
) -> Result<TAtomic, TypeError> {
    let mut parameters = vec![];
    let mut return_type = None;

    if let Some(specification) = &callable.specification {
        for parameter_ast in specification.parameters.entries.iter() {
            let parameter_type =
                get_union_from_type_ast(&parameter_ast.parameter_type, scope, type_context, classname, interner)?;

            parameters.push(TCallableParameter::new(
                Some(Box::new(parameter_type)),
                false,
                parameter_ast.is_variadic(),
                parameter_ast.is_optional(),
            ));
        }

        if let Some(ret) = specification.return_type.as_ref() {
            return_type = Some(get_union_from_type_ast(&ret.return_type, scope, type_context, classname, interner)?);
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
    classname: Option<&StringIdentifier>,
    interner: &ThreadedInterner,
) -> Result<TAtomic, TypeError> {
    let reference_name = reference_identifier.value;

    if reference_name == "this" || reference_name == "static" || reference_name == "self" {
        let class_name = if let Some(classname) = classname { *classname } else { interner.intern("$this") };

        return Ok(TAtomic::Object(TObject::Named(
            TNamedObject::new(class_name).with_is_this(reference_name != "self"),
        )));
    }

    // `Closure` -> `Closure(mixed...): mixed`
    if reference_name.eq_ignore_ascii_case("Closure") {
        return Ok(TAtomic::Callable(TCallable::Signature(
            TCallableSignature::new(false, true)
                .with_parameters(vec![TCallableParameter::new(Some(Box::new(get_mixed())), false, true, false)])
                .with_return_type(Some(Box::new(get_mixed()))),
        )));
    }

    let mut type_parameters = vec![];
    if let Some(generics) = generics {
        for generic in &generics.entries {
            let generic_type = get_union_from_type_ast(&generic.inner, scope, type_context, classname, interner)?;

            type_parameters.push(generic_type);
        }
    } else if let Some(defining_entities) = type_context.get_template_definition(reference_name) {
        return Ok(get_template_atomic(defining_entities, interner.intern(reference_name)));
    }

    let (fq_reference_name, _) = scope.resolve(NameKind::Default, reference_name);
    let fq_reference_name_id = interner.intern(&fq_reference_name);

    Ok(TAtomic::Reference(TReference::Symbol {
        name: fq_reference_name_id,
        parameters: if type_parameters.is_empty() { None } else { Some(type_parameters) },
        intersection_types: None,
    }))
}

#[inline]
fn get_array_type_from_ast<'i, 'p>(
    mut key: Option<&'p Type<'i>>,
    mut value: Option<&'p Type<'i>>,
    non_empty: bool,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<&StringIdentifier>,
    interner: &ThreadedInterner,
) -> Result<TAtomic, TypeError> {
    if key.is_some() && value.is_none() {
        std::mem::swap(&mut key, &mut value);
    }

    let mut array = TKeyedArray::new_with_parameters(
        Box::new(if let Some(k) = key {
            get_union_from_type_ast(k, scope, type_context, classname, interner)?
        } else {
            get_arraykey()
        }),
        Box::new(if let Some(v) = value {
            get_union_from_type_ast(v, scope, type_context, classname, interner)?
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
    classname: Option<&StringIdentifier>,
    interner: &ThreadedInterner,
) -> Result<TAtomic, TypeError> {
    Ok(TAtomic::Array(TArray::List(TList {
        element_type: Box::new(if let Some(v) = value {
            get_union_from_type_ast(v, scope, type_context, classname, interner)?
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
    classname: Option<&StringIdentifier>,
    interner: &ThreadedInterner,
) -> Result<TUnion, TypeError> {
    Ok(match parameter {
        Some(parameter) => {
            let constraint_union =
                get_union_from_type_ast(&parameter.entry.inner, scope, type_context, classname, interner)?;

            let mut class_strings = vec![];
            for constraint in constraint_union.types {
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
                        for constraint_atomic in constraint.types {
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
                            format!(
                                "class string parameter must target an object type, found `{}`.",
                                constraint.get_id(Some(interner))
                            ),
                            span,
                        ));
                    }
                }
            }

            TUnion::new(class_strings)
        }
        None => wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::any(kind)))),
    })
}

#[inline]
fn get_template_atomic(defining_entities: &[(GenericParent, TUnion)], parameter_name: StringIdentifier) -> TAtomic {
    let (defining_entity, constraint) = &defining_entities[0];

    TAtomic::GenericParameter(TGenericParameter {
        parameter_name,
        constraint: Box::new(constraint.clone()),
        defining_entity: *defining_entity,
        intersection_types: None,
    })
}
