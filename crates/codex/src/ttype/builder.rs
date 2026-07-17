use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
use mago_phpdoc_syntax::cst::r#type::*;
use mago_span::*;
use mago_word::*;

use crate::misc::VariableIdentifier;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
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
use crate::ttype::atomic::reference::*;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::*;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TStringCasing;
use crate::ttype::error::TypeError;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::template::GenericTemplate;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;
use crate::ttype::*;

/// Converts a parsed `PHPDoc` type node into a semantic `TUnion` type representation,
/// resolving names, templates, and keywords into their semantic counterparts.
///
/// # Arguments
///
/// * `ttype` - The type node, as parsed by the `mago_phpdoc_syntax` crate.
/// * `scope` - The `NamespaceScope` active at the location of the type.
///   Used during conversion to resolve unqualified names, aliases (`use` statements),
///   and namespace-relative names.
/// * `type_context` - The context providing information about currently defined
///   template parameters (e.g., from `@template` tags). Needed
///   during conversion to resolve template parameter references.
/// * `classname` - An optional `Word` representing the fully qualified name
///   of the current class context. Used during conversion to resolve
///   `self` type references. Should be `None` if not in a class context.
///
/// # Errors
///
/// Returns a [`TypeError`] if:
/// - An unsupported type construct is encountered
/// - Type references cannot be resolved (e.g., `self` outside a class context)
/// - Invalid type combinations are used (e.g., incompatible intersection types)
/// - Int range has minimum greater than maximum
#[inline]
pub fn get_union_from_type(
    ttype: &Type<'_>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Word>,
) -> Result<TUnion, TypeError> {
    Ok(match ttype {
        Type::Parenthesized(parenthesized_type) => {
            get_union_from_type(parenthesized_type.inner, scope, type_context, classname)?
        }
        Type::Nullable(nullable_type) => match nullable_type.inner {
            Type::Null(_) => get_null(),
            Type::String(_) => get_nullable_string(),
            Type::Int(_) => get_nullable_int(),
            Type::Float(_) => get_nullable_float(),
            Type::Object(_) => get_nullable_object(),
            Type::Scalar(_) => get_nullable_scalar(),
            _ => get_union_from_type(nullable_type.inner, scope, type_context, classname)?.as_nullable(),
        },
        Type::Union(UnionType { left, right, .. }) if matches!(**left, Type::Null(_)) => match **right {
            Type::Null(_) => get_null(),
            Type::String(_) => get_nullable_string(),
            Type::Int(_) => get_nullable_int(),
            Type::Float(_) => get_nullable_float(),
            Type::Object(_) => get_nullable_object(),
            Type::Scalar(_) => get_nullable_scalar(),
            _ => get_union_from_type(right, scope, type_context, classname)?.as_nullable(),
        },
        Type::Union(UnionType { left, right, .. }) if matches!(**right, Type::Null(_)) => match **left {
            Type::Null(_) => get_null(),
            Type::String(_) => get_nullable_string(),
            Type::Int(_) => get_nullable_int(),
            Type::Float(_) => get_nullable_float(),
            Type::Object(_) => get_nullable_object(),
            Type::Scalar(_) => get_nullable_scalar(),
            _ => get_union_from_type(left, scope, type_context, classname)?.as_nullable(),
        },
        Type::Union(union_type) => {
            let left = get_union_from_type(union_type.left, scope, type_context, classname)?;
            let right = get_union_from_type(union_type.right, scope, type_context, classname)?;

            let combined_types: Vec<TAtomic> = left.types.iter().chain(right.types.iter()).cloned().collect();

            TUnion::from_vec(combined_types)
        }
        Type::Intersection(intersection) => {
            if matches!(intersection.left, Type::NonEmptyString(_)) {
                match intersection.right {
                    Type::String(_) => return Ok(get_non_empty_string()),
                    Type::NonEmptyString(_) => return Ok(get_non_empty_string()),
                    Type::LowercaseString(_) => return Ok(get_non_empty_lowercase_string()),
                    Type::NonEmptyLowercaseString(_) => return Ok(get_non_empty_lowercase_string()),
                    Type::UppercaseString(_) => return Ok(get_non_empty_uppercase_string()),
                    Type::NonEmptyUppercaseString(_) => return Ok(get_non_empty_uppercase_string()),
                    _ => {}
                }
            }

            if matches!(intersection.right, Type::NonEmptyString(_)) {
                match intersection.left {
                    Type::String(_) => return Ok(get_non_empty_string()),
                    Type::NonEmptyString(_) => return Ok(get_non_empty_string()),
                    Type::LowercaseString(_) => return Ok(get_non_empty_lowercase_string()),
                    Type::NonEmptyLowercaseString(_) => return Ok(get_non_empty_lowercase_string()),
                    Type::UppercaseString(_) => return Ok(get_non_empty_uppercase_string()),
                    Type::NonEmptyUppercaseString(_) => return Ok(get_non_empty_uppercase_string()),
                    _ => {}
                }
            }

            let object_and_callable = (matches!(intersection.left, Type::Object(_))
                && matches!(intersection.right, Type::Callable(_)))
                || (matches!(intersection.left, Type::Callable(_)) && matches!(intersection.right, Type::Object(_)));
            if object_and_callable {
                return Ok(wrap_atomic(TAtomic::Object(TObject::new_has_method(word("__invoke")))));
            }

            let left = get_union_from_type(intersection.left, scope, type_context, classname)?;
            let right = get_union_from_type(intersection.right, scope, type_context, classname)?;

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
        Type::Slice(slice) => {
            wrap_atomic(get_array_type(None, Some(slice.inner), false, scope, type_context, classname)?)
        }
        Type::Array(ArrayType { parameters, .. }) | Type::AssociativeArray(AssociativeArrayType { parameters, .. }) => {
            let (key, value) = match parameters {
                Some(parameters) => {
                    let key = parameters.entries.first().map(|g| &g.inner);
                    let value = parameters.entries.get(1).map(|g| &g.inner);

                    (key, value)
                }
                None => (None, None),
            };

            wrap_atomic(get_array_type(key, value, false, scope, type_context, classname)?)
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

            wrap_atomic(get_array_type(key, value, true, scope, type_context, classname)?)
        }
        Type::List(list_type) => {
            let value = list_type.parameters.as_ref().and_then(|p| p.entries.first().map(|g| &g.inner));

            wrap_atomic(get_list_type(value, false, scope, type_context, classname)?)
        }
        Type::NonEmptyList(non_empty_list_type) => {
            let value = non_empty_list_type.parameters.as_ref().and_then(|p| p.entries.first().map(|g| &g.inner));

            wrap_atomic(get_list_type(value, true, scope, type_context, classname)?)
        }
        Type::ClassString(class_string_type) => get_class_string_type(
            class_string_type.span(),
            TClassLikeStringKind::Class,
            class_string_type.parameter.as_ref(),
            scope,
            type_context,
            classname,
        )?,
        Type::InterfaceString(interface_string_type) => get_class_string_type(
            interface_string_type.span(),
            TClassLikeStringKind::Interface,
            interface_string_type.parameter.as_ref(),
            scope,
            type_context,
            classname,
        )?,
        Type::EnumString(enum_string_type) => get_class_string_type(
            enum_string_type.span(),
            TClassLikeStringKind::Enum,
            enum_string_type.parameter.as_ref(),
            scope,
            type_context,
            classname,
        )?,
        Type::TraitString(trait_string_type) => get_class_string_type(
            trait_string_type.span(),
            TClassLikeStringKind::Trait,
            trait_string_type.parameter.as_ref(),
            scope,
            type_context,
            classname,
        )?,
        Type::ClassLikeString(_) => {
            return Err(TypeError::UnsupportedType(ttype.to_string(), ttype.span()));
        }
        Type::MemberReference(member_reference) => {
            let class_like_name = match &member_reference.kind {
                ReferenceKind::Self_(_) | ReferenceKind::Static(_) => {
                    let Some(classname) = classname else {
                        return Err(TypeError::InvalidType(
                            ttype.to_string(),
                            "Cannot resolve `self` type reference outside of a class context".to_string(),
                            member_reference.span(),
                        ));
                    };

                    classname
                }
                ReferenceKind::Identifier(identifier) if identifier.value.eq(b"this") => {
                    let Some(classname) = classname else {
                        return Err(TypeError::InvalidType(
                            ttype.to_string(),
                            "Cannot resolve `self` type reference outside of a class context".to_string(),
                            member_reference.span(),
                        ));
                    };

                    classname
                }
                ReferenceKind::Parent(_) => word("parent"),
                ReferenceKind::Identifier(identifier) => {
                    let (class_like_name, _) = scope.resolve(NameKind::Default, identifier.value);

                    word(&class_like_name)
                }
            };

            let member_selector = match member_reference.member {
                MemberReferenceSelector::Wildcard(_) => TReferenceMemberSelector::Wildcard,
                MemberReferenceSelector::Identifier(identifier) => {
                    TReferenceMemberSelector::Identifier(word(identifier.value))
                }
                MemberReferenceSelector::StartsWith(identifier, _) => {
                    TReferenceMemberSelector::StartsWith(word(identifier.value))
                }
                MemberReferenceSelector::EndsWith(_, identifier) => {
                    TReferenceMemberSelector::EndsWith(word(identifier.value))
                }
            };

            wrap_atomic(TAtomic::Reference(TReference::Member { class_like_name, member_selector }))
        }
        Type::GlobalWildcardReference(global_wildcard) => {
            let selector = match global_wildcard.selector {
                GlobalWildcardSelector::StartsWith(identifier, _) => {
                    TGlobalReferenceSelector::StartsWith(word(identifier.value))
                }
                GlobalWildcardSelector::EndsWith(_, identifier) => {
                    TGlobalReferenceSelector::EndsWith(word(identifier.value))
                }
            };

            wrap_atomic(TAtomic::Reference(TReference::Global { selector }))
        }
        Type::AliasReference(alias_reference) => {
            let class_like_name = match &alias_reference.class {
                ReferenceKind::Self_(_) | ReferenceKind::Static(_) => {
                    let Some(classname) = classname else {
                        return Err(TypeError::InvalidType(
                            ttype.to_string(),
                            "Cannot resolve `self` type reference outside of a class context".to_string(),
                            alias_reference.span(),
                        ));
                    };

                    classname
                }
                ReferenceKind::Identifier(identifier) if identifier.value.eq(b"this") => {
                    let Some(classname) = classname else {
                        return Err(TypeError::InvalidType(
                            ttype.to_string(),
                            "Cannot resolve `self` type reference outside of a class context".to_string(),
                            alias_reference.span(),
                        ));
                    };

                    classname
                }
                ReferenceKind::Parent(_) => word("parent"),
                ReferenceKind::Identifier(identifier) => {
                    let (class_like_name, _) = scope.resolve(NameKind::Default, identifier.value);

                    ascii_lowercase_word(&class_like_name)
                }
            };

            let alias_name = match alias_reference.alias {
                AliasName::Identifier(identifier) => word(identifier.value),
                AliasName::Keyword(keyword) => word(keyword.value),
            };

            wrap_atomic(TAtomic::Alias(TAlias::new(class_like_name, alias_name)))
        }
        Type::Object(object_type) => wrap_atomic(get_object_from_type(object_type, scope, type_context, classname)?),
        Type::Shape(shape_type) => wrap_atomic(get_shape_from_type(shape_type, scope, type_context, classname)?),
        Type::Callable(callable_type) => {
            wrap_atomic(get_callable_from_type(callable_type, scope, type_context, classname)?)
        }
        Type::Reference(reference_type) => {
            if let ReferenceKind::Identifier(identifier) = &reference_type.kind {
                let reference_name_atom = word(identifier.value);

                if let Some((source_class, original_name)) = type_context.get_imported_type_alias(reference_name_atom) {
                    return Ok(wrap_atomic(TAtomic::Alias(TAlias::new(*source_class, *original_name))));
                }

                if type_context.has_type_alias(reference_name_atom)
                    && let Some(class_name) = classname
                {
                    return Ok(wrap_atomic(TAtomic::Alias(TAlias::new(class_name, reference_name_atom))));
                }
            }

            wrap_atomic(get_reference_from_kind(
                &reference_type.kind,
                reference_type.parameters.as_ref(),
                scope,
                type_context,
                classname,
            )?)
        }
        Type::Mixed(_) | Type::Wildcard(_) => get_mixed(),
        Type::NonEmptyMixed(_) => get_truthy_mixed(),
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
        Type::Empty(_) => get_empty(),
        Type::EmptyScalar(_) => get_empty_scalar(),
        Type::CallableString(_) => get_callable_string(),
        Type::LowercaseCallableString(_) => get_string_with_props(false, false, false, true, TStringCasing::Lowercase),
        Type::UppercaseCallableString(_) => get_string_with_props(false, false, false, true, TStringCasing::Uppercase),
        Type::NumericString(_) => get_numeric_string(),
        Type::NonEmptyString(_) => get_non_empty_string(),
        Type::TruthyString(_) | Type::NonFalsyString(_) => get_truthy_string(),
        Type::UnspecifiedLiteralString(_) => get_unspecified_literal_string(),
        Type::NonEmptyUnspecifiedLiteralString(_) => get_non_empty_unspecified_literal_string(),
        Type::NonEmptyLowercaseString(_) => get_non_empty_lowercase_string(),
        Type::LowercaseString(_) => get_lowercase_string(),
        Type::NonEmptyUppercaseString(_) => get_non_empty_uppercase_string(),
        Type::UppercaseString(_) => get_uppercase_string(),
        Type::UnspecifiedLiteralInt(_) => get_unspecified_literal_int(),
        Type::UnspecifiedLiteralFloat(_) => get_unspecified_literal_float(),
        Type::LiteralFloat(lit) => get_literal_float(*lit.value),
        Type::LiteralInt(lit) => get_literal_int(lit.value as i64),
        Type::LiteralString(lit) => get_literal_string(word(lit.value)),
        Type::Negated(negated) => match negated.operand {
            Type::LiteralInt(lit) => get_literal_int(-(lit.value as i64)),
            Type::LiteralFloat(lit) => get_literal_float(-(*lit.value)),
            _ => return Err(TypeError::UnsupportedType(ttype.to_string(), ttype.span())),
        },
        Type::Posited(posited) => match posited.operand {
            Type::LiteralInt(lit) => get_literal_int(lit.value as i64),
            Type::LiteralFloat(lit) => get_literal_float(*lit.value),
            _ => return Err(TypeError::UnsupportedType(ttype.to_string(), ttype.span())),
        },
        Type::Iterable(iterable) => match iterable.parameters.as_ref() {
            Some(parameters) => match parameters.entries.len() {
                0 => wrap_atomic(TAtomic::Iterable(TIterable::mixed())),
                1 => {
                    let value_type = get_union_from_type(&parameters.entries[0].inner, scope, type_context, classname)?;

                    wrap_atomic(TAtomic::Iterable(TIterable::of_value(Arc::new(value_type))))
                }
                _ => {
                    let key_type = get_union_from_type(&parameters.entries[0].inner, scope, type_context, classname)?;

                    let value_type = get_union_from_type(&parameters.entries[1].inner, scope, type_context, classname)?;

                    wrap_atomic(TAtomic::Iterable(TIterable::new(Arc::new(key_type), Arc::new(value_type))))
                }
            },
            None => wrap_atomic(TAtomic::Iterable(TIterable::mixed())),
        },
        Type::PositiveInt(_) => get_positive_int(),
        Type::NegativeInt(_) => get_negative_int(),
        Type::NonPositiveInt(_) => get_non_positive_int(),
        Type::NonNegativeInt(_) => get_non_negative_int(),
        Type::NonZeroInt(_) => get_non_zero_int(),
        Type::TrailingPipe(trailing) => get_union_from_type(trailing.inner, scope, type_context, classname)?,
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
            Arc::new(get_union_from_type(conditional.subject, scope, type_context, classname)?),
            Arc::new(get_union_from_type(conditional.target, scope, type_context, classname)?),
            Arc::new(get_union_from_type(conditional.then, scope, type_context, classname)?),
            Arc::new(get_union_from_type(conditional.r#else, scope, type_context, classname)?),
            conditional.is_negated(),
        )))),
        Type::ThisVariable(_) => {
            TUnion::from_single(Cow::Owned(TAtomic::Object(TObject::Named(TNamedObject::new_this(word("$this"))))))
        }
        Type::Variable(variable) => {
            if variable.value == b"$this" {
                TUnion::from_single(Cow::Owned(TAtomic::Object(TObject::Named(TNamedObject::new_this(word("$this"))))))
            } else {
                TUnion::from_single(Cow::Owned(TAtomic::Variable(word(variable.value))))
            }
        }
        Type::KeyOf(key_of_type) => TUnion::from_atomic(TAtomic::Derived(TDerived::KeyOf(TKeyOf::new(Arc::new(
            get_union_from_type(&key_of_type.parameter.entry.inner, scope, type_context, classname)?,
        ))))),
        Type::ValueOf(value_of_type) => TUnion::from_atomic(TAtomic::Derived(TDerived::ValueOf(TValueOf::new(
            Arc::new(get_union_from_type(&value_of_type.parameter.entry.inner, scope, type_context, classname)?),
        )))),
        Type::IntMask(int_mask_type) => {
            let mut values = Vec::new();
            for entry in &int_mask_type.parameters.entries {
                values.push(get_union_from_type(&entry.inner, scope, type_context, classname)?);
            }
            TUnion::from_atomic(TAtomic::Derived(TDerived::IntMask(TIntMask::new(values))))
        }
        Type::IntMaskOf(int_mask_of_type) => {
            TUnion::from_atomic(TAtomic::Derived(TDerived::IntMaskOf(TIntMaskOf::new(Arc::new(get_union_from_type(
                &int_mask_of_type.parameter.entry.inner,
                scope,
                type_context,
                classname,
            )?)))))
        }
        Type::New(new_type) => TUnion::from_atomic(TAtomic::Derived(TDerived::New(TNew::new(Arc::new(
            get_union_from_type(&new_type.parameter.entry.inner, scope, type_context, classname)?,
        ))))),
        Type::TemplateType(template_type_type) => {
            let entries = &template_type_type.parameters.entries;
            if entries.len() != 3 {
                return Err(TypeError::InvalidType(
                    template_type_type.to_string(),
                    format!(
                        "`template-type<O, C, T>` expects exactly 3 parameters (object, class-name, template-name), got {}",
                        entries.len()
                    ),
                    template_type_type.span(),
                ));
            }

            let object = Arc::new(get_union_from_type(&entries[0].inner, scope, type_context, classname)?);
            let class_arg = Arc::new(get_union_from_type(&entries[1].inner, scope, type_context, classname)?);
            let template_name = Arc::new(get_union_from_type(&entries[2].inner, scope, type_context, classname)?);

            TUnion::from_atomic(TAtomic::Derived(TDerived::TemplateType(TTemplateType::new(
                object,
                class_arg,
                template_name,
            ))))
        }
        Type::PropertiesOf(properties_of_type) => {
            TUnion::from_atomic(TAtomic::Derived(TDerived::PropertiesOf(match properties_of_type.filter {
                PropertiesOfFilter::All => TPropertiesOf::new(Arc::new(get_union_from_type(
                    &properties_of_type.parameter.entry.inner,
                    scope,
                    type_context,
                    classname,
                )?)),
                PropertiesOfFilter::Public => TPropertiesOf::public(Arc::new(get_union_from_type(
                    &properties_of_type.parameter.entry.inner,
                    scope,
                    type_context,
                    classname,
                )?)),
                PropertiesOfFilter::Protected => TPropertiesOf::protected(Arc::new(get_union_from_type(
                    &properties_of_type.parameter.entry.inner,
                    scope,
                    type_context,
                    classname,
                )?)),
                PropertiesOfFilter::Private => TPropertiesOf::private(Arc::new(get_union_from_type(
                    &properties_of_type.parameter.entry.inner,
                    scope,
                    type_context,
                    classname,
                )?)),
            })))
        }
        Type::IndexAccess(index_access_type) => {
            TUnion::from_atomic(TAtomic::Derived(TDerived::IndexAccess(TIndexAccess::new(
                get_union_from_type(index_access_type.target, scope, type_context, classname)?,
                get_union_from_type(index_access_type.index, scope, type_context, classname)?,
            ))))
        }
        _ => {
            return Err(TypeError::UnsupportedType(ttype.to_string(), ttype.span()));
        }
    })
}

#[inline]
fn get_object_from_type(
    object: &ObjectType<'_>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Word>,
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
            ShapeKey::String { value, .. } => word(value),
            ShapeKey::Integer { value, .. } => i64_word(value),
            ShapeKey::ClassLikeConstant { class_name, constant_name, .. } => {
                concat_word!(class_name.value, b"::", constant_name.value)
            }
        };

        let property_type = get_union_from_type(property.value, scope, type_context, classname)?;

        known_properties.insert(key, (property_is_optional, property_type));
    }

    Ok(TAtomic::Object(TObject::new_with_properties(properties.ellipsis.is_none(), known_properties)))
}

#[inline]
fn get_shape_from_type(
    shape: &ShapeType<'_>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Word>,
) -> Result<TAtomic, TypeError> {
    if shape.kind.is_list() {
        let mut list = TList::new(match &shape.additional_fields {
            Some(additional_fields) => match &additional_fields.parameters {
                Some(parameters) => Arc::new(if let Some(k) = parameters.entries.first().map(|g| &g.inner) {
                    get_union_from_type(k, scope, type_context, classname)?
                } else {
                    get_mixed()
                }),
                None => Arc::new(get_mixed()),
            },
            None => Arc::new(get_never()),
        });

        list.known_elements = Some({
            let mut tree = BTreeMap::new();
            let mut next_offset: usize = 0;

            for field in &shape.fields {
                let field_is_optional = field.is_optional();

                let offset = if let Some(field_key) = field.key.as_ref() {
                    let array_key = match field_key.key {
                        ShapeKey::String { value, .. } => ArrayKey::String(word(value)),
                        ShapeKey::Integer { value, .. } => ArrayKey::Integer(value),
                        ShapeKey::ClassLikeConstant { class_name, constant_name, .. } => {
                            let class_like_name = if class_name.value.eq_ignore_ascii_case(b"self")
                                || class_name.value.eq_ignore_ascii_case(b"static")
                                || class_name.value.eq(b"this")
                                || class_name.value.eq(b"$this")
                            {
                                classname.unwrap_or_else(|| word(class_name.value))
                            } else if class_name.value.eq_ignore_ascii_case(b"parent") {
                                word("parent")
                            } else {
                                let (resolved, _) = scope.resolve(NameKind::Default, class_name.value);
                                word(&resolved)
                            };

                            ArrayKey::ClassLikeConstant { class_like_name, constant_name: word(constant_name.value) }
                        }
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
                } else {
                    let offset = next_offset;

                    next_offset += 1;

                    offset
                };

                let mut field_value_type = get_union_from_type(field.value, scope, type_context, classname)?;
                if field_is_optional {
                    field_value_type.set_possibly_undefined(true, None);
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
                    Arc::new(if let Some(k) = parameters.entries.first().map(|g| &g.inner) {
                        get_union_from_type(k, scope, type_context, classname)?
                    } else {
                        get_mixed()
                    }),
                    Arc::new(if let Some(v) = parameters.entries.get(1).map(|g| &g.inner) {
                        get_union_from_type(v, scope, type_context, classname)?
                    } else {
                        get_mixed()
                    }),
                ),
                None => (Arc::new(get_arraykey()), Arc::new(get_mixed())),
            }),
            None => None,
        };

        keyed_array.known_items = Some({
            let mut tree = BTreeMap::new();
            let mut next_offset = 0;

            for field in &shape.fields {
                let field_is_optional = field.is_optional();

                let array_key = if let Some(field_key) = field.key.as_ref() {
                    let array_key = match field_key.key {
                        ShapeKey::String { value, .. } => ArrayKey::String(word(value)),
                        ShapeKey::Integer { value, .. } => ArrayKey::Integer(value),
                        ShapeKey::ClassLikeConstant { class_name, constant_name, .. } => {
                            let class_like_name = if class_name.value.eq_ignore_ascii_case(b"self")
                                || class_name.value.eq_ignore_ascii_case(b"static")
                                || class_name.value.eq(b"this")
                                || class_name.value.eq(b"$this")
                            {
                                classname.unwrap_or_else(|| word(class_name.value))
                            } else if class_name.value.eq_ignore_ascii_case(b"parent") {
                                word("parent")
                            } else {
                                let (resolved, _) = scope.resolve(NameKind::Default, class_name.value);
                                word(&resolved)
                            };

                            ArrayKey::ClassLikeConstant { class_like_name, constant_name: word(constant_name.value) }
                        }
                    };

                    if let ArrayKey::Integer(offset) = array_key
                        && offset >= next_offset
                    {
                        next_offset = offset.saturating_add(1);
                    }

                    array_key
                } else {
                    let array_key = ArrayKey::Integer(next_offset);

                    next_offset = next_offset.saturating_add(1);

                    array_key
                };

                let mut field_value_type = get_union_from_type(field.value, scope, type_context, classname)?;
                if field_is_optional {
                    field_value_type.set_possibly_undefined(true, None);
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
fn get_callable_from_type(
    callable: &CallableType<'_>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Word>,
) -> Result<TAtomic, TypeError> {
    let mut parameters = vec![];
    let mut return_type = None;

    if let Some(specification) = &callable.specification {
        for parameter_ast in &specification.parameters.entries {
            let parameter_type = if let Some(parameter_type) = &parameter_ast.parameter_type {
                get_union_from_type(parameter_type, scope, type_context, classname)?
            } else {
                get_mixed()
            };

            parameters.push(
                TCallableParameter::new(
                    Some(Arc::new(parameter_type)),
                    parameter_ast.is_by_reference(),
                    parameter_ast.is_variadic(),
                    parameter_ast.is_optional(),
                )
                .with_name(parameter_ast.variable.map(|variable| VariableIdentifier(word(variable.value)))),
            );
        }

        if let Some(ret) = specification.return_type.as_ref() {
            return_type = Some(get_union_from_type(ret.return_type, scope, type_context, classname)?);
        }
    } else {
        // `callable` without a specification should be treated the same as
        // `callable(mixed...): mixed`
        parameters.push(TCallableParameter::new(Some(Arc::new(get_mixed())), false, true, false));
        return_type = Some(get_mixed());
    }

    Ok(TAtomic::Callable(TCallable::Signature(
        TCallableSignature::new(callable.kind.is_pure(), callable.kind.is_closure())
            .with_parameters(parameters)
            .with_return_type(return_type.map(Arc::new)),
    )))
}

#[inline]
fn get_reference_from_kind(
    kind: &ReferenceKind<'_>,
    generics: Option<&GenericParameters<'_>>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Word>,
) -> Result<TAtomic, TypeError> {
    let mut is_this = false;
    let mut is_static = false;
    let mut is_named_object = false;
    let fq_reference_name_id = match kind {
        ReferenceKind::Self_(_) => {
            is_named_object = true;

            classname.unwrap_or_else(|| word("static"))
        }
        ReferenceKind::Static(_) => {
            is_named_object = true;
            is_static = true;

            classname.unwrap_or_else(|| word("static"))
        }
        ReferenceKind::Identifier(identifier) if identifier.value == b"this" => {
            is_named_object = true;
            is_this = true;
            is_static = true;

            classname.unwrap_or_else(|| word("static"))
        }
        ReferenceKind::Parent(_) => {
            is_named_object = true;

            word("parent")
        }
        ReferenceKind::Identifier(identifier) => {
            let reference_name = identifier.value;
            let reference_name_atom = word(reference_name);
            if let Some(defining_entities) = type_context.get_template_definition(reference_name_atom)
                && generics.is_none()
            {
                return Ok(get_template_atomic(defining_entities, reference_name_atom));
            }

            let (fq_reference_name, _) = scope.resolve(NameKind::Default, reference_name);

            // `Closure` -> `Closure(mixed...): mixed`
            if fq_reference_name.eq_ignore_ascii_case(b"Closure") && generics.is_none() {
                return Ok(TAtomic::Callable(TCallable::Signature(
                    TCallableSignature::new(false, true)
                        .with_parameters(vec![TCallableParameter::new(Some(Arc::new(get_mixed())), false, true, false)])
                        .with_return_type(Some(Arc::new(get_mixed()))),
                )));
            }

            word(&fq_reference_name)
        }
    };

    let mut type_parameters = None;
    let mut type_parameter_variances = None;
    if let Some(generics) = generics {
        let mut parameters = vec![];
        let mut variances = vec![];
        for generic in &generics.entries {
            let mut generic_type = get_union_from_type(&generic.inner, scope, type_context, classname)?;

            for atomic in generic_type.types.to_mut() {
                if let TAtomic::Object(TObject::Named(named)) = atomic
                    && named.is_this
                {
                    named.name = classname.unwrap_or_else(|| word("static"));
                    named.is_this = false;
                }
            }

            parameters.push(generic_type);

            let variance = match generic.variance {
                Some(variance) => Variance::from(variance),
                None if matches!(generic.inner, Type::Wildcard(_)) => Variance::Bivariant,
                None => Variance::Invariant,
            };

            variances.push(variance);
        }

        type_parameters = Some(parameters);
        if variances.iter().any(|variance| !variance.is_invariant()) {
            type_parameter_variances = Some(variances);
        }
    }

    let is_generator = fq_reference_name_id.as_bytes().eq_ignore_ascii_case(b"Generator");

    let is_iterator = is_generator
        || fq_reference_name_id.as_bytes().eq_ignore_ascii_case(b"Iterator")
        || fq_reference_name_id.as_bytes().eq_ignore_ascii_case(b"IteratorAggregate")
        || fq_reference_name_id.as_bytes().eq_ignore_ascii_case(b"Traversable");

    let mixed_default = || {
        let mut union = get_mixed();
        union.set_from_template_default(true);
        union
    };

    'iterator: {
        if !is_iterator {
            break 'iterator;
        }

        let Some(type_parameters) = &mut type_parameters else {
            type_parameters = Some(vec![mixed_default(), mixed_default()]);

            break 'iterator;
        };

        if type_parameters.len() == 1 {
            type_parameters.insert(0, mixed_default());
        } else if type_parameters.is_empty() {
            type_parameters.push(mixed_default());
            type_parameters.push(mixed_default());
        }

        if !is_generator {
            break 'iterator;
        }

        while type_parameters.len() < 4 {
            type_parameters.push(mixed_default());
        }
    }

    if is_named_object {
        Ok(TAtomic::Object(TObject::Named(TNamedObject {
            name: fq_reference_name_id,
            type_parameters,
            variances: type_parameter_variances,
            intersection_types: None,
            is_static,
            is_this,
            remapped_parameters: false,
        })))
    } else {
        Ok(TAtomic::Reference(TReference::Symbol {
            name: fq_reference_name_id,
            parameters: type_parameters,
            variances: type_parameter_variances,
            intersection_types: None,
        }))
    }
}

#[inline]
fn get_array_type<'src>(
    mut key: Option<&'src Type<'src>>,
    mut value: Option<&'src Type<'src>>,
    non_empty: bool,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Word>,
) -> Result<TAtomic, TypeError> {
    if key.is_some() && value.is_none() {
        std::mem::swap(&mut key, &mut value);
    }

    let mut array = TKeyedArray::new_with_parameters(
        Arc::new(if let Some(k) = key {
            get_union_from_type(k, scope, type_context, classname)?
        } else {
            get_arraykey()
        }),
        Arc::new(if let Some(v) = value {
            get_union_from_type(v, scope, type_context, classname)?
        } else {
            get_mixed()
        }),
    );

    array.non_empty = non_empty;

    Ok(TAtomic::Array(TArray::Keyed(array)))
}

#[inline]
fn get_list_type(
    value: Option<&Type<'_>>,
    non_empty: bool,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Word>,
) -> Result<TAtomic, TypeError> {
    Ok(TAtomic::Array(TArray::List(TList {
        element_type: Arc::new(if let Some(v) = value {
            get_union_from_type(v, scope, type_context, classname)?
        } else {
            get_mixed()
        }),
        known_count: None,
        known_elements: None,
        non_empty,
    })))
}

#[inline]
fn get_class_string_type(
    span: Span,
    kind: TClassLikeStringKind,
    parameter: Option<&SingleGenericParameter<'_>>,
    scope: &NamespaceScope,
    type_context: &TypeResolutionContext,
    classname: Option<Word>,
) -> Result<TUnion, TypeError> {
    Ok(match parameter {
        Some(parameter) => {
            let constraint_union = get_union_from_type(&parameter.entry.inner, scope, type_context, classname)?;

            let mut class_strings = vec![];
            for constraint in constraint_union.types.into_owned() {
                match constraint {
                    TAtomic::Object(TObject::Named(_) | TObject::Enum(_) | TObject::HasMethod(_))
                    | TAtomic::Reference(TReference::Symbol { .. })
                    | TAtomic::Alias(_)
                    | TAtomic::Variable(_)
                    | TAtomic::Conditional(_)
                    | TAtomic::Derived(
                        TDerived::IndexAccess(_) | TDerived::New(_) | TDerived::TemplateType(_) | TDerived::ValueOf(_),
                    ) => class_strings
                        .push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::of_type(kind, constraint)))),
                    TAtomic::GenericParameter(TGenericParameter {
                        parameter_name,
                        defining_entity,
                        constraint: nested_constraint,
                        ..
                    }) => {
                        for constraint_atomic in Arc::unwrap_or_clone(nested_constraint).types.into_owned() {
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
fn get_template_atomic(defining_entities: &[GenericTemplate], parameter_name: Word) -> TAtomic {
    let GenericTemplate { defining_entity: template_source, constraint: template_type, .. } = &defining_entities[0];

    TAtomic::GenericParameter(TGenericParameter {
        parameter_name,
        constraint: Arc::new(template_type.clone()),
        defining_entity: *template_source,
        intersection_types: None,
    })
}
