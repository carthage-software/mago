use std::collections::BTreeMap;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use serde::Deserialize;
use serde::Serialize;

use crate::metadata::CodebaseMetadata;
use crate::symbol::SymbolKind;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::get_mixed;
use crate::ttype::get_string;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TPropertiesOf {
    pub visibility: Option<Visibility>,
    pub target_type: Box<TUnion>,
}

impl TPropertiesOf {
    #[inline]
    pub const fn new(target_type: Box<TUnion>) -> Self {
        TPropertiesOf { visibility: None, target_type }
    }

    #[inline]
    pub const fn public(target_type: Box<TUnion>) -> Self {
        TPropertiesOf { visibility: Some(Visibility::Public), target_type }
    }

    #[inline]
    pub const fn protected(target_type: Box<TUnion>) -> Self {
        TPropertiesOf { visibility: Some(Visibility::Protected), target_type }
    }

    #[inline]
    pub const fn private(target_type: Box<TUnion>) -> Self {
        TPropertiesOf { visibility: Some(Visibility::Private), target_type }
    }

    #[inline]
    pub const fn visibility(&self) -> Option<Visibility> {
        self.visibility
    }

    #[inline]
    pub const fn get_target_type(&self) -> &TUnion {
        &self.target_type
    }

    #[inline]
    pub const fn get_target_type_mut(&mut self) -> &mut TUnion {
        &mut self.target_type
    }

    /// Extracts properties from the given target types as a keyed array shape,
    /// optionally filtering by visibility.
    ///
    /// Returns `None` if no properties could be extracted.
    ///
    /// For a class like `class User { public string $name; public int $age; }`,
    /// this returns `array{name: string, age: int}`.
    ///
    /// For an enum like `enum Color: string { case Red = 'red'; ... }`,
    /// this returns `array{name: 'Red'|'Green'|'Blue', value: 'red'|'green'|'blue'}`.
    ///
    /// For non-final classes, the result is an unsealed array (with `parameters = (string, mixed)`)
    /// to indicate that subclasses may have additional properties.
    #[inline]
    pub fn get_properties_of_targets(
        target_types: &[TAtomic],
        codebase: &CodebaseMetadata,
        visibility_filter: Option<Visibility>,
        _retain_generics: bool,
    ) -> Option<TAtomic> {
        let mut known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();
        // Track whether we need an unsealed array (for non-final classes)
        let mut needs_unsealed = false;

        for target in target_types {
            match target {
                TAtomic::Object(TObject::Named(named)) => {
                    let Some(class_like_metadata) = codebase.get_class_like(&named.name) else {
                        continue;
                    };

                    // Non-final classes (not enums, interfaces, or traits) need unsealed arrays
                    // since subclasses may have additional properties
                    if class_like_metadata.kind == SymbolKind::Class && !class_like_metadata.flags.is_final() {
                        needs_unsealed = true;
                    }

                    for (prop_name, property_metadata) in class_like_metadata.properties.iter() {
                        // Filter by visibility if specified
                        if let Some(required_visibility) = visibility_filter
                            && property_metadata.read_visibility != required_visibility
                        {
                            continue;
                        }

                        // Get the property type
                        if let Some(type_metadata) = &property_metadata.type_metadata {
                            // Property name without the leading '$'
                            let name_str = prop_name.as_str();
                            let stripped_name = name_str.strip_prefix('$').unwrap_or(name_str);
                            let key = ArrayKey::String(Atom::from(stripped_name));
                            let is_optional = false; // Class properties are not optional in the shape

                            // Merge with existing entry if present (for union types)
                            if let Some((_, existing_union)) = known_items.get_mut(&key) {
                                for atomic in type_metadata.type_union.types.iter() {
                                    if !existing_union.types.contains(atomic) {
                                        let mut types = existing_union.types.clone().into_owned();
                                        types.push(atomic.clone());
                                        *existing_union = TUnion::from_vec(types);
                                    }
                                }
                            } else {
                                known_items.insert(key, (is_optional, type_metadata.type_union.clone()));
                            }
                        }
                    }
                }
                TAtomic::Object(TObject::Enum(TEnum { name: enum_name, case: specific_case })) => {
                    // Enums have built-in properties:
                    // - `name`: always present (the case name as a literal string)
                    // - `value`: only for backed enums (the backing value)
                    // Note: Enums are always final in PHP, so we don't need to make the array unsealed
                    let Some(class_like_metadata) = codebase.get_class_like(enum_name) else {
                        continue;
                    };

                    // Collect case names (and values for backed enums)
                    let mut name_types = vec![];
                    let mut value_types = vec![];

                    let cases_to_process: Vec<_> = if let Some(case_name) = specific_case {
                        // Specific case: only process that case
                        class_like_metadata.enum_cases.get(case_name).into_iter().collect()
                    } else {
                        // All cases
                        class_like_metadata.enum_cases.values().collect()
                    };

                    for case_metadata in cases_to_process {
                        // `name` property: literal string of the case name
                        name_types.push(TAtomic::Scalar(TScalar::literal_string(case_metadata.name)));

                        // `value` property: only for backed enums
                        if let Some(value_type) = &case_metadata.value_type {
                            value_types.push(value_type.clone());
                        }
                    }

                    // Add `name` property (all enums have this)
                    if !name_types.is_empty() {
                        let name_key = ArrayKey::String(atom("name"));
                        if let Some((_, existing_union)) = known_items.get_mut(&name_key) {
                            let mut types = existing_union.types.clone().into_owned();
                            types.extend(name_types);
                            *existing_union = TUnion::from_vec(types);
                        } else {
                            known_items.insert(name_key, (false, TUnion::from_vec(name_types)));
                        }
                    }

                    // Add `value` property (only backed enums have this)
                    if !value_types.is_empty() {
                        let value_key = ArrayKey::String(atom("value"));
                        if let Some((_, existing_union)) = known_items.get_mut(&value_key) {
                            let mut types = existing_union.types.clone().into_owned();
                            types.extend(value_types);
                            *existing_union = TUnion::from_vec(types);
                        } else {
                            known_items.insert(value_key, (false, TUnion::from_vec(value_types)));
                        }
                    }
                }
                TAtomic::GenericParameter(_parameter) => {
                    // For generic parameters, we can't expand at this point
                    // The caller should handle unexpanded types
                    continue;
                }
                _ => {
                    continue;
                }
            }
        }

        if known_items.is_empty() {
            None
        } else {
            let mut keyed_array = TKeyedArray::new().with_known_items(known_items).with_non_empty(true);

            // For non-final classes, make the array unsealed to indicate
            // that subclasses may have additional properties
            if needs_unsealed {
                keyed_array = keyed_array.with_parameters(Box::new(get_string()), Box::new(get_mixed()));
            }

            Some(TAtomic::Array(TArray::Keyed(keyed_array)))
        }
    }
}

impl TType for TPropertiesOf {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        vec![TypeRef::Union(&self.target_type)]
    }

    fn needs_population(&self) -> bool {
        self.target_type.needs_population()
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        if let Some(visibility) = &self.visibility {
            concat_atom!(visibility.as_str(), "-properties-of<", self.target_type.get_id().as_str(), ">")
        } else {
            concat_atom!("properties-of<", self.target_type.get_id().as_str(), ">")
        }
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Atom {
        self.get_id()
    }
}
