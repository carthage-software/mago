//! Collect PHPDoc names from parsed types, leaving prose and literal values intact.

use mago_phpdoc_syntax::cst::r#type::Type;
use mago_phpdoc_syntax::cst::{AssertPattern, Document, Identifier, TagValue, TemplateTagValue};

mod bindings;
mod edits;
mod types;

pub(super) use edits::{alias_locations, class_locations};

#[derive(Clone, Copy)]
pub(super) struct Name<'arena> {
    pub identifier: Identifier<'arena>,
    pub kind: NameKind,
}

#[derive(Clone, Copy)]
pub(super) enum NameKind {
    Type { generic: bool },
    Class,
    Variable,
}

impl<'arena> Name<'arena> {
    fn class(identifier: Identifier<'arena>) -> Self {
        Self { identifier, kind: NameKind::Class }
    }
}

pub(super) fn names<'arena>(document: &Document<'arena>) -> Option<Vec<Name<'arena>>> {
    collect(document, true)
}

pub(super) fn parameter_names<'arena>(document: &Document<'arena>) -> Option<Vec<Name<'arena>>> {
    collect(document, false)
}

pub(super) fn type_names<'arena>(ty: &Type<'arena>) -> Option<Vec<Name<'arena>>> {
    let mut names = Vec::new();
    types::collect(ty, &mut names)?;
    Some(names)
}

fn collect<'arena>(document: &Document<'arena>, strict: bool) -> Option<Vec<Name<'arena>>> {
    if document.has_errors() {
        return None;
    }
    let mut names = Vec::new();
    for tag in document.tags() {
        match tag.value {
            TagValue::Param(value) => types::collect(value.r#type, &mut names)?,
            TagValue::ParamOut(value) => types::collect(value.r#type, &mut names)?,
            TagValue::ParamClosureThis(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Return(value) | TagValue::RealReturn(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Var(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Throws(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Mixin(value) => types::collect(value.r#type, &mut names)?,
            TagValue::SelfOut(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Template(value) => template(&value, &mut names)?,
            TagValue::Extends(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Implements(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Use(value) => types::collect(value.r#type, &mut names)?,
            TagValue::RequireExtends(value) => types::collect(value.r#type, &mut names)?,
            TagValue::RequireImplements(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Sealed(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Inheritors(value) => types::collect(value.r#type, &mut names)?,
            TagValue::Property(value) | TagValue::PropertyRead(value) | TagValue::PropertyWrite(value) => {
                if let Some(ty) = value.r#type {
                    types::collect(ty, &mut names)?;
                }
            }
            TagValue::Assert(value) | TagValue::AssertIfTrue(value) | TagValue::AssertIfFalse(value) => {
                if let AssertPattern::Type(ty) = value.pattern {
                    types::collect(ty, &mut names)?;
                }
            }
            TagValue::Where(value) => types::collect(value.r#type, &mut names)?,
            TagValue::TypeAlias(value) => types::collect(value.r#type, &mut names)?,
            TagValue::TypeAliasImport(value) => names.push(Name::class(value.imported_from)),
            TagValue::Method(value) => {
                if let Some(ty) = value.return_type {
                    types::collect(ty, &mut names)?;
                }
                if let Some(templates) = value.templates {
                    for parameter in templates.entries.iter() {
                        template(&parameter.template, &mut names)?;
                    }
                }
                for parameter in value.parameters.entries.iter() {
                    if let Some(ty) = parameter.r#type {
                        types::collect(ty, &mut names)?;
                    }
                    if let Some(default) = parameter.default {
                        types::constant(default.value, &mut names);
                    }
                }
            }
            TagValue::Generic(_) | TagValue::Invalid(_) if strict => return None,
            _ => {}
        }
    }
    Some(names)
}

fn template<'arena>(value: &TemplateTagValue<'arena>, names: &mut Vec<Name<'arena>>) -> Option<()> {
    if let Some(bound) = value.bound {
        types::collect(bound.r#type, names)?;
    }
    if let Some(default) = value.default {
        types::collect(default.r#type, names)?;
    }
    Some(())
}
