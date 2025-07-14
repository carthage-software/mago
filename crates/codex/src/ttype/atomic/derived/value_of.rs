use serde::Deserialize;
use serde::Serialize;

use mago_interner::ThreadedInterner;

use crate::get_enum;
use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::get_array_parameters;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TValueOf(Box<TAtomic>);

impl TValueOf {
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

    #[inline]
    pub fn get_value_of_targets(
        target_types: Vec<TAtomic>,
        codebase: &CodebaseMetadata,
        interner: &ThreadedInterner,
        retain_generics: bool,
    ) -> Option<TUnion> {
        let mut value_types = vec![];

        for target in target_types {
            match target {
                TAtomic::Array(array) => {
                    let (_, array_value_type) = get_array_parameters(&array, codebase, interner);

                    value_types.extend(array_value_type.types.iter().cloned());
                }
                TAtomic::Iterable(iterable) => {
                    value_types.extend(iterable.get_value_type().types.iter().cloned());
                }
                TAtomic::Object(TObject::Enum(TEnum { name: enum_name, case: Some(case_name) })) => {
                    let Some(metadata) = get_enum(codebase, interner, &enum_name) else {
                        continue;
                    };

                    let Some(case_metadata) = metadata.enum_cases.get(&case_name) else {
                        continue;
                    };

                    if let Some(case_value_type) = case_metadata.value_type.as_ref() {
                        value_types.push(case_value_type.clone());
                    }
                }
                TAtomic::Object(object) => {
                    let Some(name) = object.get_name() else {
                        continue;
                    };

                    let Some(metadata) = get_enum(codebase, interner, name) else {
                        continue;
                    };

                    for (_, case_metadata) in metadata.enum_cases.iter() {
                        if let Some(case_value_type) = case_metadata.value_type.as_ref() {
                            value_types.push(case_value_type.clone());
                        }
                    }
                }
                TAtomic::GenericParameter(parameter) => {
                    if retain_generics {
                        value_types.push(TAtomic::GenericParameter(parameter.clone()));
                    } else if let Some(generic_value_types) = Self::get_value_of_targets(
                        parameter.get_constraint().clone().types,
                        codebase,
                        interner,
                        retain_generics,
                    ) {
                        value_types.extend(generic_value_types.types);
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if value_types.is_empty() { None } else { Some(TUnion::new(value_types)) }
    }
}

impl TType for TValueOf {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        vec![TypeRef::Atomic(&self.0)]
    }

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        let mut id = String::new();
        id += "value-of<";
        id += &self.0.get_id(interner);
        id += ">";
        id
    }
}
