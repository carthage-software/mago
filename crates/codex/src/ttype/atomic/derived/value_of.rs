use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use serde::Deserialize;
use serde::Serialize;

use crate::get_class_like;
use crate::get_enum;
use crate::is_instance_of;
use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::atomic::scalar::TScalar;
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
        target_types: &[TAtomic],
        codebase: &CodebaseMetadata,
        retain_generics: bool,
    ) -> Option<TUnion> {
        let mut value_types = vec![];

        for target in target_types {
            match target {
                TAtomic::Array(array) => {
                    let (_, array_value_type) = get_array_parameters(array, codebase);

                    value_types.extend(array_value_type.types.iter().cloned());
                }
                TAtomic::Iterable(iterable) => {
                    value_types.extend(iterable.get_value_type().types.iter().cloned());
                }
                TAtomic::Object(TObject::Enum(TEnum { name: enum_name, case: Some(case_name) })) => {
                    let Some(metadata) = get_enum(codebase, enum_name) else {
                        continue;
                    };

                    let Some(case_metadata) = metadata.enum_cases.get(case_name) else {
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

                    let Some(class_like_metadata) = get_class_like(codebase, name) else {
                        continue;
                    };

                    if class_like_metadata.kind.is_enum() {
                        for (_, case_metadata) in class_like_metadata.enum_cases.iter() {
                            if let Some(case_value_type) = case_metadata.value_type.as_ref() {
                                value_types.push(case_value_type.clone());
                            }
                        }

                        continue;
                    }

                    if !class_like_metadata.kind.is_interface() {
                        continue;
                    }

                    let is_enum_interface = class_like_metadata.flags.is_enum_interface()
                        || is_instance_of(codebase, &class_like_metadata.name, &atom("unitenum"));

                    if !is_enum_interface {
                        continue;
                    }

                    if is_instance_of(codebase, &class_like_metadata.name, &atom("stringbackedenum")) {
                        value_types.push(TAtomic::Scalar(TScalar::string()));
                        continue;
                    }

                    if is_instance_of(codebase, &class_like_metadata.name, &atom("intbackedenum")) {
                        value_types.push(TAtomic::Scalar(TScalar::int()));
                        continue;
                    }

                    value_types.push(TAtomic::Scalar(TScalar::int()));
                    value_types.push(TAtomic::Scalar(TScalar::string()));
                }
                TAtomic::GenericParameter(parameter) => {
                    if retain_generics {
                        value_types.push(TAtomic::GenericParameter(parameter.clone()));
                    } else if let Some(generic_value_types) =
                        Self::get_value_of_targets(parameter.get_constraint().types.as_ref(), codebase, retain_generics)
                    {
                        value_types.extend(generic_value_types.types.into_owned());
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if value_types.is_empty() { None } else { Some(TUnion::from_vec(value_types)) }
    }
}

impl TType for TValueOf {
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
        concat_atom!("value-of<", self.0.get_id().as_str(), ">")
    }
}
