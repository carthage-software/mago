use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;

use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::union::TUnion;

fn class_name_of_constraint(constraint: &TAtomic) -> Option<Atom> {
    match constraint {
        TAtomic::Object(TObject::Named(named)) => Some(named.name),
        _ => None,
    }
}

/// The `new<X>` utility type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TNew(Arc<TUnion>);

impl TNew {
    #[must_use]
    pub fn new(target: Arc<TUnion>) -> Self {
        Self(target)
    }

    #[inline]
    #[must_use]
    pub fn get_target_type(&self) -> &TUnion {
        &self.0
    }

    #[inline]
    pub fn get_target_type_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.0)
    }

    /// Resolves each atomic in `target_types` to the object type it
    /// represents when treated as a class-string. Atomics that cannot be
    /// resolved to a single class name (for instance unresolved templates,
    /// generic `class-string`, or non-class-like unions) fall back to
    /// `TObject::Any`.
    #[must_use]
    pub fn get_new_targets(target_types: &[TAtomic], _codebase: &CodebaseMetadata) -> Option<TUnion> {
        let mut resulting_atomics: Vec<TAtomic> = Vec::new();

        for target in target_types {
            match target {
                TAtomic::Scalar(TScalar::String(TString { literal: Some(TStringLiteral::Value(name)), .. })) => {
                    resulting_atomics.push(TAtomic::Object(TObject::Named(TNamedObject::new(*name))));
                }
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Literal { value })) => {
                    resulting_atomics.push(TAtomic::Object(TObject::Named(TNamedObject::new(*value))));
                }
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. })) => {
                    match class_name_of_constraint(constraint.as_ref()) {
                        Some(name) => {
                            resulting_atomics.push(TAtomic::Object(TObject::Named(TNamedObject::new(name))));
                        }
                        None => resulting_atomics.push(TAtomic::Object(TObject::Any)),
                    }
                }
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic { constraint, .. })) => {
                    match class_name_of_constraint(constraint.as_ref()) {
                        Some(name) => {
                            resulting_atomics.push(TAtomic::Object(TObject::Named(TNamedObject::new(name))));
                        }
                        None => resulting_atomics.push(TAtomic::Object(TObject::Any)),
                    }
                }
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Any { .. })) => {
                    resulting_atomics.push(TAtomic::Object(TObject::Any));
                }
                _ => return None,
            }
        }

        if resulting_atomics.is_empty() { None } else { Some(TUnion::from_vec(resulting_atomics)) }
    }
}

impl TType for TNew {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        vec![TypeRef::Union(&self.0)]
    }

    fn needs_population(&self) -> bool {
        self.0.needs_population()
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        concat_atom!("new<", self.0.get_id().as_str(), ">")
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Atom {
        atom(self.get_id().as_str())
    }
}
