use serde::Deserialize;
use serde::Serialize;

use mago_interner::ThreadedInterner;

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
        target_types: Vec<TAtomic>,
        codebase: &CodebaseMetadata,
        interner: &ThreadedInterner,
        retain_generics: bool,
    ) -> Option<TUnion> {
        let mut key_types = vec![];

        for target in target_types {
            match target {
                TAtomic::Array(array) => {
                    let (array_key_type, _) = get_array_parameters(&array, codebase, interner);

                    key_types.extend(array_key_type.types.iter().cloned());
                }
                TAtomic::Iterable(iterable) => {
                    key_types.extend(iterable.get_key_type().types.iter().cloned());
                }
                TAtomic::GenericParameter(parameter) => {
                    if retain_generics {
                        key_types.push(TAtomic::GenericParameter(parameter.clone()));
                    } else if let Some(generic_key_types) = Self::get_key_of_targets(
                        parameter.get_constraint().clone().types,
                        codebase,
                        interner,
                        retain_generics,
                    ) {
                        key_types.extend(generic_key_types.types);
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if key_types.is_empty() { None } else { Some(TUnion::new(key_types)) }
    }
}

impl TType for TKeyOf {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        vec![TypeRef::Atomic(&self.0)]
    }

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        let mut id = String::new();
        id += "key-of<";
        id += &self.0.get_id(interner);
        id += ">";
        id
    }
}
