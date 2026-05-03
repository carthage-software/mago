use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::concat_atom;

use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::add_optional_union_type;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::get_specialized_template_type;
use crate::ttype::union::TUnion;

/// The `template-type<Object, ClassName, TemplateName>` utility type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TTemplateType {
    object: Arc<TUnion>,
    class_name: Arc<TUnion>,
    template_name: Arc<TUnion>,
}

impl TTemplateType {
    #[must_use]
    pub fn new(object: Arc<TUnion>, class_name: Arc<TUnion>, template_name: Arc<TUnion>) -> Self {
        Self { object, class_name, template_name }
    }

    #[inline]
    #[must_use]
    pub fn get_object(&self) -> &TUnion {
        &self.object
    }

    #[inline]
    #[must_use]
    pub fn get_class_name(&self) -> &TUnion {
        &self.class_name
    }

    #[inline]
    #[must_use]
    pub fn get_template_name(&self) -> &TUnion {
        &self.template_name
    }

    #[inline]
    pub fn get_object_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.object)
    }

    #[inline]
    pub fn get_class_name_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.class_name)
    }

    #[inline]
    pub fn get_template_name_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.template_name)
    }

    /// Extracts a class name from a `TUnion` used as the `ClassName`
    /// parameter of `template-type<>`.
    #[must_use]
    pub fn extract_class_name(class_name_type: &TUnion) -> Option<Atom> {
        for atomic in class_name_type.types.as_ref() {
            match atomic {
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Literal { value })) => return Some(*value),
                TAtomic::Scalar(TScalar::String(TString { literal: Some(TStringLiteral::Value(name)), .. })) => {
                    return Some(*name);
                }
                TAtomic::Object(TObject::Named(named)) => return Some(named.name),
                _ => {}
            }
        }

        None
    }

    /// Extracts the template parameter name from a `TUnion` expected to
    /// hold a single literal string atomic.
    #[must_use]
    pub fn extract_template_name(template_name_type: &TUnion) -> Option<Atom> {
        for atomic in template_name_type.types.as_ref() {
            if let TAtomic::Scalar(TScalar::String(TString { literal: Some(TStringLiteral::Value(name)), .. })) = atomic
            {
                return Some(*name);
            }
        }

        None
    }

    /// Resolves the template type against the recorded object, class and
    /// template name. Returns `None` if the lookup cannot be completed
    /// statically (unknown class, missing template, non-object atomics).
    #[must_use]
    pub fn resolve(&self, codebase: &CodebaseMetadata) -> Option<TUnion> {
        let class_name = Self::extract_class_name(&self.class_name)?;
        let template_name = Self::extract_template_name(&self.template_name)?;

        let mut merged: Option<TUnion> = None;
        for atomic in self.object.types.as_ref() {
            if let Some(resolved) = resolve_template_from_atomic(atomic, class_name, template_name, codebase) {
                merged = Some(add_optional_union_type(resolved, merged.as_ref(), codebase));
            }
        }

        merged
    }
}

fn resolve_template_from_atomic(
    atomic: &TAtomic,
    class_name: Atom,
    template_name: Atom,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    match atomic {
        TAtomic::Object(TObject::Enum(named)) => {
            let instantiated_class_metadata = codebase.get_class_like(&named.name)?;

            get_specialized_template_type(codebase, template_name, class_name, instantiated_class_metadata, None)
        }
        TAtomic::Object(TObject::Named(named)) => {
            let instantiated_class_metadata = codebase.get_class_like(&named.name)?;

            let mut merged: Option<TUnion> = get_specialized_template_type(
                codebase,
                template_name,
                class_name,
                instantiated_class_metadata,
                named.get_type_parameters(),
            );

            if let Some(intersections) = named.intersection_types.as_ref() {
                for intersection in intersections {
                    if let Some(resolved) =
                        resolve_template_from_atomic(intersection, class_name, template_name, codebase)
                    {
                        merged = Some(add_optional_union_type(resolved, merged.as_ref(), codebase));
                    }
                }
            }

            merged
        }
        TAtomic::GenericParameter(param) => {
            let mut merged: Option<TUnion> = None;

            for constraint_atomic in param.constraint.types.as_ref() {
                if let Some(resolved) =
                    resolve_template_from_atomic(constraint_atomic, class_name, template_name, codebase)
                {
                    merged = Some(add_optional_union_type(resolved, merged.as_ref(), codebase));
                }
            }

            merged
        }
        _ => None,
    }
}

impl TType for TTemplateType {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        vec![TypeRef::Union(&self.object), TypeRef::Union(&self.class_name), TypeRef::Union(&self.template_name)]
    }

    fn needs_population(&self) -> bool {
        self.object.needs_population() || self.class_name.needs_population() || self.template_name.needs_population()
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        concat_atom!(
            "template-type<",
            self.object.get_id().as_str(),
            ", ",
            self.class_name.get_id().as_str(),
            ", ",
            self.template_name.get_id().as_str(),
            ">"
        )
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Atom {
        self.get_id()
    }
}
