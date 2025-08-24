use ahash::HashMap;
use ahash::RandomState;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;
use mago_reporting::Issue;
use mago_span::Span;

use crate::flags::attribute::AttributeFlags;
use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::enum_case::EnumCaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::property::PropertyMetadata;
use crate::misc::GenericParent;
use crate::symbol::SymbolKind;
use crate::ttype::atomic::TAtomic;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

type TemplateTuple = (Atom, Vec<(GenericParent, TUnion)>);

/// Contains comprehensive metadata for a PHP class-like structure (class, interface, trait, enum).
///
/// Aggregates information about inheritance, traits, generics, methods, properties, constants,
/// attributes, docblock tags, analysis flags, and more.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassLikeMetadata {
    pub name: Atom,
    pub original_name: Atom,
    pub span: Span,
    pub direct_parent_interfaces: AtomSet,
    pub all_parent_interfaces: AtomSet,
    pub direct_parent_class: Option<Atom>,
    pub require_extends: AtomSet,
    pub require_implements: AtomSet,
    pub all_parent_classes: AtomSet,
    pub used_traits: AtomSet,
    pub trait_alias_map: AtomMap<Atom>,
    pub trait_visibility_map: AtomMap<Visibility>,
    pub trait_final_map: AtomSet,
    pub child_class_likes: Option<AtomSet>,
    pub name_span: Option<Span>,
    pub kind: SymbolKind,
    pub template_types: Vec<TemplateTuple>,
    pub template_readonly: AtomSet,
    pub template_variance: HashMap<usize, Variance>,
    pub template_extended_offsets: AtomMap<Vec<TUnion>>,
    pub template_extended_parameters: AtomMap<IndexMap<Atom, TUnion, RandomState>>,
    pub template_type_extends_count: AtomMap<usize>,
    pub template_type_implements_count: AtomMap<usize>,
    pub template_type_uses_count: AtomMap<usize>,
    pub methods: AtomSet,
    pub pseudo_methods: AtomSet,
    pub static_pseudo_methods: AtomSet,
    pub declaring_method_ids: AtomMap<Atom>,
    pub appearing_method_ids: AtomMap<Atom>,
    pub overridden_method_ids: AtomMap<AtomSet>,
    pub inheritable_method_ids: AtomMap<Atom>,
    pub potential_declaring_method_ids: AtomMap<AtomSet>,
    pub properties: AtomMap<PropertyMetadata>,
    pub appearing_property_ids: AtomMap<Atom>,
    pub declaring_property_ids: AtomMap<Atom>,
    pub inheritable_property_ids: AtomMap<Atom>,
    pub overridden_property_ids: AtomMap<AtomSet>,
    pub initialized_properties: AtomSet,
    pub constants: IndexMap<Atom, ClassLikeConstantMetadata, RandomState>,
    pub enum_cases: IndexMap<Atom, EnumCaseMetadata, RandomState>,
    pub invalid_dependencies: AtomSet,
    pub attributes: Vec<AttributeMetadata>,
    pub enum_type: Option<TAtomic>,
    pub has_sealed_methods: Option<bool>,
    pub has_sealed_properties: Option<bool>,
    pub permitted_inheritors: Option<AtomSet>,
    pub issues: Vec<Issue>,
    pub attribute_flags: Option<AttributeFlags>,
    pub flags: MetadataFlags,
}

impl ClassLikeMetadata {
    pub fn new(
        name: Atom,
        original_name: Atom,
        span: Span,
        name_span: Option<Span>,
        flags: MetadataFlags,
    ) -> ClassLikeMetadata {
        ClassLikeMetadata {
            constants: IndexMap::with_hasher(RandomState::new()),
            enum_cases: IndexMap::with_hasher(RandomState::new()),
            flags,
            kind: SymbolKind::Class,
            direct_parent_interfaces: AtomSet::default(),
            all_parent_classes: AtomSet::default(),
            appearing_method_ids: AtomMap::default(),
            attributes: Vec::new(),
            all_parent_interfaces: AtomSet::default(),
            declaring_method_ids: AtomMap::default(),
            appearing_property_ids: AtomMap::default(),
            declaring_property_ids: AtomMap::default(),
            direct_parent_class: None,
            require_extends: AtomSet::default(),
            require_implements: AtomSet::default(),
            inheritable_method_ids: AtomMap::default(),
            enum_type: None,
            inheritable_property_ids: AtomMap::default(),
            initialized_properties: AtomSet::default(),
            invalid_dependencies: AtomSet::default(),
            span,
            name_span,
            methods: AtomSet::default(),
            pseudo_methods: AtomSet::default(),
            static_pseudo_methods: AtomSet::default(),
            overridden_method_ids: AtomMap::default(),
            overridden_property_ids: AtomMap::default(),
            potential_declaring_method_ids: AtomMap::default(),
            properties: AtomMap::default(),
            template_variance: HashMap::default(),
            template_type_extends_count: AtomMap::default(),
            template_extended_parameters: AtomMap::default(),
            template_extended_offsets: AtomMap::default(),
            template_type_implements_count: AtomMap::default(),
            template_type_uses_count: AtomMap::default(),
            template_types: Vec::default(),
            used_traits: AtomSet::default(),
            trait_alias_map: AtomMap::default(),
            trait_visibility_map: AtomMap::default(),
            trait_final_map: AtomSet::default(),
            name,
            original_name,
            child_class_likes: None,
            template_readonly: AtomSet::default(),
            has_sealed_methods: None,
            has_sealed_properties: None,
            permitted_inheritors: None,
            issues: vec![],
            attribute_flags: None,
        }
    }

    /// Returns a reference to the map of trait method aliases.
    #[inline]
    pub fn get_trait_alias_map(&self) -> &AtomMap<Atom> {
        &self.trait_alias_map
    }

    /// Returns a vector of the generic type parameter names.
    #[inline]
    pub fn get_template_type_names(&self) -> Vec<Atom> {
        self.template_types.iter().map(|(name, _)| *name).collect()
    }

    /// Returns type parameters for a specific generic parameter name.
    #[inline]
    pub fn get_template_type(&self, name: &Atom) -> Option<&Vec<(GenericParent, TUnion)>> {
        self.template_types.iter().find_map(|(param_name, types)| if param_name == name { Some(types) } else { None })
    }

    /// Returns type parameters for a specific generic parameter name with its index.
    #[inline]
    pub fn get_template_type_with_index(&self, name: &Atom) -> Option<(usize, &Vec<(GenericParent, TUnion)>)> {
        self.template_types
            .iter()
            .enumerate()
            .find_map(|(index, (param_name, types))| if param_name == name { Some((index, types)) } else { None })
    }

    pub fn get_template_for_index(&self, index: usize) -> Option<(Atom, &Vec<(GenericParent, TUnion)>)> {
        self.template_types.get(index).map(|(name, types)| (*name, types))
    }

    pub fn get_template_name_for_index(&self, index: usize) -> Option<Atom> {
        self.template_types.get(index).map(|(name, _)| *name)
    }

    pub fn get_template_index_for_name(&self, name: &Atom) -> Option<usize> {
        self.template_types.iter().position(|(param_name, _)| param_name == name)
    }

    /// Checks if a specific parent is either a parent class or interface.
    #[inline]
    pub fn has_parent(&self, parent: &Atom) -> bool {
        self.all_parent_classes.contains(parent) || self.all_parent_interfaces.contains(parent)
    }

    /// Checks if a specific parent has template extended parameters.
    #[inline]
    pub fn has_template_extended_parameter(&self, parent: &Atom) -> bool {
        self.template_extended_parameters.contains_key(parent)
    }

    /// Checks if a specific method appears in this class-like.
    #[inline]
    pub fn has_appearing_method(&self, method: &Atom) -> bool {
        self.appearing_method_ids.contains_key(method)
    }

    /// Returns a reference to a specific method's potential declaring classes/traits.
    #[inline]
    pub fn get_potential_declaring_method_id(&self, method: &Atom) -> Option<&AtomSet> {
        self.potential_declaring_method_ids.get(method)
    }

    /// Returns a vector of property names.
    #[inline]
    pub fn get_property_names(&self) -> AtomSet {
        self.properties.keys().copied().collect()
    }

    /// Checks if a specific property appears in this class-like.
    #[inline]
    pub fn has_appearing_property(&self, name: &Atom) -> bool {
        self.appearing_property_ids.contains_key(name)
    }

    /// Checks if a specific property is declared in this class-like.
    #[inline]
    pub fn has_declaring_property(&self, name: &Atom) -> bool {
        self.declaring_property_ids.contains_key(name)
    }

    /// Takes ownership of the issues found for this class-like structure.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }

    /// Adds a single direct parent interface.
    #[inline]
    pub fn add_direct_parent_interface(&mut self, interface: Atom) {
        self.direct_parent_interfaces.insert(interface);
        self.all_parent_interfaces.insert(interface);
    }

    /// Adds a single interface to the list of all parent interfaces. Use with caution, normally derived.
    #[inline]
    pub fn add_all_parent_interface(&mut self, interface: Atom) {
        self.all_parent_interfaces.insert(interface);
    }

    /// Adds multiple interfaces to the list of all parent interfaces. Use with caution.
    #[inline]
    pub fn add_all_parent_interfaces(&mut self, interfaces: impl IntoIterator<Item = Atom>) {
        self.all_parent_interfaces.extend(interfaces);
    }

    /// Adds multiple ancestor classes. Use with caution.
    #[inline]
    pub fn add_all_parent_classes(&mut self, classes: impl IntoIterator<Item = Atom>) {
        self.all_parent_classes.extend(classes);
    }

    /// Adds a single used trait. Returns `true` if the trait was not already present.
    #[inline]
    pub fn add_used_trait(&mut self, trait_name: Atom) -> bool {
        self.used_traits.insert(trait_name)
    }

    /// Adds multiple used traits.
    #[inline]
    pub fn add_used_traits(&mut self, traits: impl IntoIterator<Item = Atom>) {
        self.used_traits.extend(traits);
    }

    /// Adds or updates a single trait alias. Returns the previous original name if one existed for the alias.
    #[inline]
    pub fn add_trait_alias(&mut self, method: Atom, alias: Atom) -> Option<Atom> {
        self.trait_alias_map.insert(method, alias)
    }

    /// Adds or updates a single trait visibility override. Returns the previous visibility if one existed.
    #[inline]
    pub fn add_trait_visibility(&mut self, method: Atom, visibility: Visibility) -> Option<Visibility> {
        self.trait_visibility_map.insert(method, visibility)
    }

    /// Adds a single template type definition.
    #[inline]
    pub fn add_template_type(&mut self, template: TemplateTuple) {
        self.template_types.push(template);
    }

    /// Adds or updates the variance for a specific parameter index. Returns the previous variance if one existed.
    #[inline]
    pub fn add_template_variance_parameter(&mut self, index: usize, variance: Variance) -> Option<Variance> {
        self.template_variance.insert(index, variance)
    }

    /// Adds or replaces the offset types for a specific template parameter name.
    #[inline]
    pub fn add_template_extended_offset(&mut self, name: Atom, types: Vec<TUnion>) -> Option<Vec<TUnion>> {
        self.template_extended_offsets.insert(name, types)
    }

    /// Adds or replaces the resolved parameters for a specific parent FQCN.
    #[inline]
    pub fn extend_template_extended_parameters(
        &mut self,
        template_extended_parameters: AtomMap<IndexMap<Atom, TUnion, RandomState>>,
    ) {
        self.template_extended_parameters.extend(template_extended_parameters);
    }

    /// Adds or replaces a single resolved parameter for the parent FQCN.
    #[inline]
    pub fn add_template_extended_parameter(
        &mut self,
        parent_fqcn: Atom,
        parameter_name: Atom,
        parameter_type: TUnion,
    ) -> Option<TUnion> {
        self.template_extended_parameters.entry(parent_fqcn).or_default().insert(parameter_name, parameter_type)
    }

    /// Adds or updates the declaring class FQCN for a method name.
    #[inline]
    pub fn add_declaring_method_id(&mut self, method: Atom, declaring_fqcn: Atom) -> Option<Atom> {
        self.add_appearing_method_id(method, declaring_fqcn);
        self.declaring_method_ids.insert(method, declaring_fqcn)
    }

    /// Adds or updates the appearing class FQCN for a method name.
    #[inline]
    pub fn add_appearing_method_id(&mut self, method: Atom, appearing_fqcn: Atom) -> Option<Atom> {
        self.appearing_method_ids.insert(method, appearing_fqcn)
    }

    /// Adds a parent FQCN to the set for an overridden method. Initializes set if needed. Returns `true` if added.
    #[inline]
    pub fn add_overridden_method_parent(&mut self, method: Atom, parent_fqcn: Atom) -> bool {
        self.overridden_method_ids.entry(method).or_default().insert(parent_fqcn)
    }

    /// Adds a potential declaring FQCN to the set for a method. Initializes set if needed. Returns `true` if added.
    #[inline]
    pub fn add_potential_declaring_method(&mut self, method: Atom, potential_fqcn: Atom) -> bool {
        self.potential_declaring_method_ids.entry(method).or_default().insert(potential_fqcn)
    }

    /// Adds or updates a property's metadata. Returns the previous metadata if the property existed.
    #[inline]
    pub fn add_property(&mut self, name: Atom, property_metadata: PropertyMetadata) -> Option<PropertyMetadata> {
        let class_name = self.name;

        self.add_declaring_property_id(name, class_name);
        if property_metadata.flags.has_default() {
            self.initialized_properties.insert(name);
        }

        if !property_metadata.is_final() {
            self.inheritable_property_ids.insert(name, class_name);
        }

        self.properties.insert(name, property_metadata)
    }

    /// Adds or updates a property's metadata using just the property metadata. Returns the previous metadata if the property existed.
    #[inline]
    pub fn add_property_metadata(&mut self, property_metadata: PropertyMetadata) -> Option<PropertyMetadata> {
        let name = property_metadata.get_name().0;

        self.add_property(name, property_metadata)
    }

    /// Adds or updates the declaring class FQCN for a property name.
    #[inline]
    pub fn add_declaring_property_id(&mut self, prop: Atom, declaring_fqcn: Atom) -> Option<Atom> {
        self.appearing_property_ids.insert(prop, declaring_fqcn);
        self.declaring_property_ids.insert(prop, declaring_fqcn)
    }

    pub fn get_missing_required_interface<'a>(&self, other: &'a ClassLikeMetadata) -> Option<&'a Atom> {
        for required_interface in &other.require_implements {
            if self.all_parent_interfaces.contains(required_interface) {
                continue;
            }

            if (self.flags.is_abstract() || self.kind.is_trait())
                && self.require_implements.contains(required_interface)
            {
                continue; // Abstract classes and traits can require interfaces they implement
            }

            return Some(required_interface);
        }

        None
    }

    pub fn get_missing_required_extends<'a>(&self, other: &'a ClassLikeMetadata) -> Option<&'a Atom> {
        for required_extend in &other.require_extends {
            if self.all_parent_classes.contains(required_extend) {
                continue;
            }

            if self.kind.is_interface() && self.all_parent_interfaces.contains(required_extend) {
                continue;
            }

            if (self.flags.is_abstract() || self.kind.is_trait()) && self.require_extends.contains(required_extend) {
                continue; // Abstract classes and traits can require classes they extend
            }

            return Some(required_extend);
        }

        None
    }

    pub fn is_permitted_to_inherit(&self, other: &ClassLikeMetadata) -> bool {
        if self.kind.is_trait() || self.flags.is_abstract() {
            return true; // Traits and abstract classes can always inherit
        }

        let Some(permitted_inheritors) = &other.permitted_inheritors else {
            return true; // No restrictions, inheriting is allowed
        };

        if permitted_inheritors.contains(&self.name) {
            return true; // This class-like is explicitly permitted to inherit
        }

        self.all_parent_interfaces.iter().any(|parent_interface| permitted_inheritors.contains(parent_interface))
            || self.all_parent_classes.iter().any(|parent_class| permitted_inheritors.contains(parent_class))
            || self.used_traits.iter().any(|used_trait| permitted_inheritors.contains(used_trait))
    }

    #[inline]
    pub fn mark_as_populated(&mut self) {
        self.flags |= MetadataFlags::POPULATED;
        self.shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.properties.shrink_to_fit();
        self.initialized_properties.shrink_to_fit();
        self.appearing_property_ids.shrink_to_fit();
        self.declaring_property_ids.shrink_to_fit();
        self.inheritable_property_ids.shrink_to_fit();
        self.overridden_property_ids.shrink_to_fit();
        self.appearing_method_ids.shrink_to_fit();
        self.declaring_method_ids.shrink_to_fit();
        self.inheritable_method_ids.shrink_to_fit();
        self.overridden_method_ids.shrink_to_fit();
        self.potential_declaring_method_ids.shrink_to_fit();
        self.attributes.shrink_to_fit();
        self.constants.shrink_to_fit();
        self.enum_cases.shrink_to_fit();
    }
}
