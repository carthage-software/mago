use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::concat_atom;

use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::get_array_parameters;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TKeyOf(Box<TAtomic>);

impl TKeyOf {
    pub fn new(object: Box<TAtomic>) -> Self {
        Self(object)
    }

    #[inline]
    pub const fn get_target_type(&self) -> &TAtomic {
        &self.0
    }

    #[inline]
    pub const fn get_target_type_mut(&mut self) -> &mut TAtomic {
        &mut self.0
    }

    pub fn get_key_of_targets(
        target_types: &[TAtomic],
        codebase: &CodebaseMetadata,
        retain_generics: bool,
    ) -> Option<TUnion> {
        let mut key_types = vec![];

        for target in target_types {
            match target {
                TAtomic::Array(array) => {
                    let (array_key_type, _) = get_array_parameters(array, codebase);

                    key_types.extend(array_key_type.types.iter().cloned());
                }
                TAtomic::Iterable(iterable) => {
                    key_types.extend(iterable.get_key_type().types.iter().cloned());
                }
                TAtomic::GenericParameter(parameter) => {
                    if retain_generics {
                        key_types.push(TAtomic::GenericParameter(parameter.clone()));
                    } else if let Some(generic_key_types) =
                        Self::get_key_of_targets(parameter.get_constraint().types.as_ref(), codebase, retain_generics)
                    {
                        key_types.extend(generic_key_types.types.into_owned());
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if key_types.is_empty() { None } else { Some(TUnion::from_vec(key_types)) }
    }
}

impl TType for TKeyOf {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        vec![TypeRef::Atomic(&self.0)]
    }

    fn needs_population(&self) -> bool {
        self.0.needs_population()
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn get_id(&self) -> Atom {
        concat_atom!("key-of<", self.0.get_id().as_str(), ">")
    }
}
