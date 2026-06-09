use std::sync::Arc;

use mago_word::Word;
use mago_word::concat_word;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::union::TUnion;

/// Represents an indexed access type `T[K]`.
///
/// This type resolves to the type of elements in `T` at index `K`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TIndexAccess {
    target_type: Arc<TUnion>,
    index_type: Arc<TUnion>,
}

impl TIndexAccess {
    #[must_use]
    pub fn new(target: TUnion, index: TUnion) -> Self {
        Self { target_type: Arc::new(target), index_type: Arc::new(index) }
    }

    #[inline]
    #[must_use]
    pub fn get_target_type(&self) -> &TUnion {
        &self.target_type
    }

    #[inline]
    pub fn get_target_type_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.target_type)
    }

    #[inline]
    #[must_use]
    pub fn get_index_type(&self) -> &TUnion {
        &self.index_type
    }

    #[inline]
    pub fn get_index_type_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.index_type)
    }

    #[must_use]
    pub fn get_indexed_access_result(
        target_types: &[TAtomic],
        index_types: &[TAtomic],
        retain_generics: bool,
    ) -> Option<TUnion> {
        let mut indexed_values = vec![];
        for target_type in target_types {
            'indices: for index_type in index_types {
                if let TAtomic::GenericParameter(index_parameter) = index_type
                    && !retain_generics
                {
                    if let Some(generic_indexed_values) = Self::get_indexed_access_result(
                        std::slice::from_ref(target_type),
                        index_parameter.get_constraint().types.as_ref(),
                        retain_generics,
                    ) {
                        indexed_values.extend(generic_indexed_values.types.into_owned());
                    }

                    continue 'indices;
                }

                match target_type {
                    TAtomic::Array(target_array) => {
                        let Some(array_key) = index_type.to_array_key() else {
                            continue 'indices;
                        };

                        match target_array {
                            TArray::List(list_array) => {
                                let ArrayKey::Integer(target_index) = array_key else {
                                    continue 'indices;
                                };

                                if target_index < 0 {
                                    continue 'indices;
                                }

                                let Some(known_elements) = list_array.known_elements.as_ref() else {
                                    continue 'indices;
                                };

                                let Some((_, known_element_type)) = known_elements.get(&(target_index as usize)) else {
                                    continue 'indices;
                                };

                                indexed_values.extend(known_element_type.types.iter().cloned());
                            }
                            TArray::Keyed(keyed_array) => {
                                let Some(known_items) = keyed_array.known_items.as_ref() else {
                                    continue 'indices;
                                };

                                let Some((_, known_item_type)) = known_items.get(&array_key) else {
                                    continue 'indices;
                                };

                                indexed_values.extend(known_item_type.types.iter().cloned());
                            }
                        }
                    }
                    TAtomic::GenericParameter(parameter) => {
                        if retain_generics {
                            indexed_values.push(TAtomic::GenericParameter(parameter.clone()));
                        } else if let Some(generic_indexed_values) = Self::get_indexed_access_result(
                            parameter.get_constraint().types.as_ref(),
                            index_types,
                            retain_generics,
                        ) {
                            indexed_values.extend(generic_indexed_values.types.into_owned());
                        }
                    }
                    _ => {
                        // Continue to next target type
                    }
                }
            }
        }

        if indexed_values.is_empty() { None } else { Some(TUnion::from_vec(indexed_values)) }
    }
}

impl TType for TIndexAccess {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        vec![TypeRef::Union(&self.target_type), TypeRef::Union(&self.index_type)]
    }

    fn needs_population(&self) -> bool {
        self.target_type.needs_population() || self.index_type.needs_population()
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Word {
        concat_word!(self.target_type.get_id(), b"[", self.index_type.get_id(), b"]")
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Word {
        self.get_id()
    }
}
