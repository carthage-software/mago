//! Traverse type operands and class constants without treating keys or strings as names.

use mago_phpdoc_syntax::cst::r#type::{GenericParameters, ReferenceKind, ShapeField, ShapeKey, Type};
use mago_phpdoc_syntax::cst::{ConstantExpression, Identifier};

use super::{Name, NameKind};

pub(super) fn collect<'arena>(ty: &Type<'arena>, names: &mut Vec<Name<'arena>>) -> Option<()> {
    match ty {
        Type::Parenthesized(value) => collect(value.inner, names)?,
        Type::Nullable(value) => collect(value.inner, names)?,
        Type::TrailingPipe(value) => collect(value.inner, names)?,
        Type::Slice(value) => collect(value.inner, names)?,
        Type::Negated(value) => collect(value.operand, names)?,
        Type::Posited(value) => collect(value.operand, names)?,
        Type::Union(value) => {
            collect(value.left, names)?;
            collect(value.right, names)?;
        }
        Type::Intersection(value) => {
            collect(value.left, names)?;
            collect(value.right, names)?;
        }
        Type::Conditional(value) => {
            for ty in [value.subject, value.target, value.then, value.r#else] {
                collect(ty, names)?;
            }
        }
        Type::IndexAccess(value) => {
            collect(value.target, names)?;
            collect(value.index, names)?;
        }
        Type::Reference(value) => {
            if let ReferenceKind::Identifier(identifier) = value.kind {
                names.push(Name { identifier, kind: NameKind::Type { generic: value.parameters.is_some() } });
            }
            generics(value.parameters.as_ref(), names)?;
        }
        Type::MemberReference(value) => reference(value.kind, names),
        Type::AliasReference(value) => reference(value.class, names),
        Type::Array(value) => generics(value.parameters.as_ref(), names)?,
        Type::NonEmptyArray(value) => generics(value.parameters.as_ref(), names)?,
        Type::AssociativeArray(value) => generics(value.parameters.as_ref(), names)?,
        Type::List(value) => generics(value.parameters.as_ref(), names)?,
        Type::NonEmptyList(value) => generics(value.parameters.as_ref(), names)?,
        Type::Iterable(value) => generics(value.parameters.as_ref(), names)?,
        Type::IntMask(value) => generics(Some(&value.parameters), names)?,
        Type::TemplateType(value) => generics(Some(&value.parameters), names)?,
        Type::ClassString(value) => {
            if let Some(parameter) = &value.parameter {
                collect(&parameter.entry.inner, names)?;
            }
        }
        Type::ClassLikeString(value) => {
            if let Some(parameter) = &value.parameter {
                collect(&parameter.entry.inner, names)?;
            }
        }
        Type::InterfaceString(value) => {
            if let Some(parameter) = &value.parameter {
                collect(&parameter.entry.inner, names)?;
            }
        }
        Type::EnumString(value) => {
            if let Some(parameter) = &value.parameter {
                collect(&parameter.entry.inner, names)?;
            }
        }
        Type::TraitString(value) => {
            if let Some(parameter) = &value.parameter {
                collect(&parameter.entry.inner, names)?;
            }
        }
        Type::KeyOf(value) => collect(&value.parameter.entry.inner, names)?,
        Type::ValueOf(value) => collect(&value.parameter.entry.inner, names)?,
        Type::IntMaskOf(value) => collect(&value.parameter.entry.inner, names)?,
        Type::New(value) => collect(&value.parameter.entry.inner, names)?,
        Type::PropertiesOf(value) => collect(&value.parameter.entry.inner, names)?,
        Type::Shape(value) => {
            for field in value.fields.iter() {
                shape_field(field, names)?;
            }
            if let Some(additional) = &value.additional_fields {
                generics(additional.parameters.as_ref(), names)?;
            }
        }
        Type::Object(value) => {
            if let Some(properties) = &value.properties {
                for field in properties.fields.iter() {
                    shape_field(field, names)?;
                }
            }
        }
        Type::Callable(value) => {
            if let Some(specification) = &value.specification {
                let start = names.len();
                for parameter in specification.parameters.entries.iter() {
                    if let Some(ty) = &parameter.parameter_type {
                        collect(ty, names)?;
                    }
                }
                if let Some(ty) = &specification.return_type {
                    collect(ty.return_type, names)?;
                }
                // A callable type's own parameters shadow outer bindings.
                let mut index = 0;
                names.retain(|name| {
                    let shadowed = index >= start
                        && matches!(name.kind, NameKind::Variable)
                        && specification.parameters.entries.iter().any(|parameter| {
                            parameter.variable.is_some_and(|variable| variable.value == name.identifier.value)
                        });
                    index += 1;
                    !shadowed
                });
            }
        }
        Type::Variable(variable) => names.push(Name {
            identifier: Identifier { span: variable.span, value: variable.value },
            kind: NameKind::Variable,
        }),
        Type::Mixed(_)
        | Type::NonEmptyMixed(_)
        | Type::Null(_)
        | Type::Void(_)
        | Type::Never(_)
        | Type::Resource(_)
        | Type::ClosedResource(_)
        | Type::OpenResource(_)
        | Type::True(_)
        | Type::False(_)
        | Type::Bool(_)
        | Type::Float(_)
        | Type::Int(_)
        | Type::PositiveInt(_)
        | Type::NegativeInt(_)
        | Type::NonPositiveInt(_)
        | Type::NonNegativeInt(_)
        | Type::NonZeroInt(_)
        | Type::String(_)
        | Type::StringableObject(_)
        | Type::ArrayKey(_)
        | Type::Numeric(_)
        | Type::Scalar(_)
        | Type::Empty(_)
        | Type::EmptyScalar(_)
        | Type::CallableString(_)
        | Type::LowercaseCallableString(_)
        | Type::UppercaseCallableString(_)
        | Type::NumericString(_)
        | Type::NonEmptyString(_)
        | Type::NonEmptyLowercaseString(_)
        | Type::LowercaseString(_)
        | Type::NonEmptyUppercaseString(_)
        | Type::UppercaseString(_)
        | Type::TruthyString(_)
        | Type::NonFalsyString(_)
        | Type::UnspecifiedLiteralInt(_)
        | Type::UnspecifiedLiteralString(_)
        | Type::UnspecifiedLiteralFloat(_)
        | Type::NonEmptyUnspecifiedLiteralString(_)
        | Type::LiteralFloat(_)
        | Type::LiteralInt(_)
        | Type::LiteralString(_)
        | Type::ThisVariable(_)
        | Type::IntRange(_)
        | Type::Wildcard(_)
        | Type::GlobalWildcardReference(_) => {}
        _ => return None,
    }
    Some(())
}

fn reference<'arena>(kind: ReferenceKind<'arena>, names: &mut Vec<Name<'arena>>) {
    if let ReferenceKind::Identifier(identifier) = kind {
        names.push(Name::class(identifier));
    }
}

fn generics<'arena>(parameters: Option<&GenericParameters<'arena>>, names: &mut Vec<Name<'arena>>) -> Option<()> {
    if let Some(parameters) = parameters {
        for parameter in parameters.entries.iter() {
            collect(&parameter.inner, names)?;
        }
    }
    Some(())
}

fn shape_field<'arena>(field: &ShapeField<'arena>, names: &mut Vec<Name<'arena>>) -> Option<()> {
    if let Some(key) = &field.key
        && let ShapeKey::ClassLikeConstant { class_name, .. } = key.key
    {
        names.push(Name::class(class_name));
    }
    collect(field.value, names)
}

pub(super) fn constant<'arena>(expression: &ConstantExpression<'arena>, names: &mut Vec<Name<'arena>>) {
    match expression {
        ConstantExpression::ClassLikeConstantAccess(value) => names.push(Name::class(value.class)),
        ConstantExpression::UnaryPrefix(value) => constant(value.operand, names),
        ConstantExpression::Array(value) => {
            for item in value.items.iter() {
                if let Some(key) = item.key {
                    constant(key, names);
                }
                constant(item.value, names);
            }
        }
        _ => {}
    }
}
