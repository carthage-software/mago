use foldhash::fast::RandomState;
use indexmap::IndexMap;
use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Issue;
use mago_span::Span;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;

use crate::flags::attribute::AttributeFlags;
use crate::identifier::method::MethodIdentifier;
use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::enum_case::EnumCaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::metadata::version_constraint::VersionConstraint;
use crate::symbol::SymbolKind;
use crate::ttype::atomic::TAtomic;
use crate::ttype::template::GenericTemplate;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

/// Type alias for template types stored in metadata.
/// Maps template parameter names to their defining entity and constraint type.
pub type TemplateTypes = IndexMap<Word, GenericTemplate, RandomState>;

/// Contains comprehensive metadata for a PHP class-like structure (class, interface, trait, enum).
///
/// Aggregates information about inheritance, traits, generics, methods, properties, constants,
/// attributes, docblock tags, analysis flags, and more.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ClassLikeMetadata {
    pub name: Word,
    pub original_name: Word,
    pub span: Span,
    pub direct_parent_interfaces: WordSet,
    pub all_parent_interfaces: WordSet,
    pub direct_parent_class: Option<Word>,
    pub require_extends: WordSet,
    pub require_implements: WordSet,
    pub all_parent_classes: WordSet,
    pub used_traits: WordSet,
    pub trait_alias_map: WordMap<Word>,
    pub trait_visibility_map: WordMap<Visibility>,
    pub trait_final_map: WordSet,
    pub child_class_likes: Option<WordSet>,
    pub name_span: Option<Span>,
    pub kind: SymbolKind,
    pub template_types: TemplateTypes,
    pub template_readonly: WordSet,
    pub template_variance: Vec<Variance>,
    pub template_extended_offsets: WordMap<Vec<TUnion>>,
    pub template_extended_parameters: WordMap<IndexMap<Word, TUnion, RandomState>>,
    pub template_extended_parameter_paths: WordMap<Vec<IndexMap<Word, TUnion, RandomState>>>,
    pub template_type_extends_count: WordMap<usize>,
    pub template_type_implements_count: WordMap<usize>,
    pub template_type_uses_count: WordMap<usize>,
    pub methods: WordSet,
    pub pseudo_methods: WordSet,
    pub static_pseudo_methods: WordSet,
    pub declaring_method_ids: WordMap<MethodIdentifier>,
    pub appearing_method_ids: WordMap<MethodIdentifier>,
    pub inheritable_method_ids: WordMap<MethodIdentifier>,
    pub overridden_method_ids: WordMap<IndexMap<Word, MethodIdentifier, RandomState>>,
    pub properties: WordMap<PropertyMetadata>,
    pub appearing_property_ids: WordMap<Word>,
    pub declaring_property_ids: WordMap<Word>,
    pub inheritable_property_ids: WordMap<Word>,
    pub overridden_property_ids: WordMap<WordSet>,
    pub initialized_properties: WordSet,
    pub constants: WordMap<ClassLikeConstantMetadata>,
    pub trait_constant_ids: WordMap<Word>,
    pub enum_cases: WordMap<EnumCaseMetadata>,
    pub invalid_dependencies: WordSet,
    pub attributes: Vec<AttributeMetadata>,
    pub enum_type: Option<TAtomic>,
    pub has_sealed_methods: Option<bool>,
    pub has_sealed_properties: Option<bool>,
    pub permitted_inheritors: Option<WordSet>,
    pub issues: Vec<Issue>,
    pub attribute_flags: Option<AttributeFlags>,
    pub flags: MetadataFlags,
    pub type_aliases: WordMap<TypeMetadata>,
    /// Imported type aliases in the form of (`from_fqcn`, `type_name`, span)
    pub imported_type_aliases: WordMap<(Word, Word, Span)>,
    /// Mixin types from @mixin annotations - these types' methods/properties
    /// can be accessed via magic methods (__call, __get, __set, __callStatic)
    pub mixins: Vec<TUnion>,
    pub version_constraint: VersionConstraint,
}

impl ClassLikeMetadata {
    #[must_use]
    pub fn new(
        name: Word,
        original_name: Word,
        span: Span,
        name_span: Option<Span>,
        flags: MetadataFlags,
    ) -> ClassLikeMetadata {
        ClassLikeMetadata {
            constants: WordMap::default(),
            trait_constant_ids: WordMap::default(),
            enum_cases: WordMap::default(),
            flags,
            kind: SymbolKind::Class,
            direct_parent_interfaces: WordSet::default(),
            all_parent_classes: WordSet::default(),
            appearing_method_ids: WordMap::default(),
            attributes: Vec::new(),
            all_parent_interfaces: WordSet::default(),
            declaring_method_ids: WordMap::default(),
            appearing_property_ids: WordMap::default(),
            declaring_property_ids: WordMap::default(),
            direct_parent_class: None,
            require_extends: WordSet::default(),
            require_implements: WordSet::default(),
            inheritable_method_ids: WordMap::default(),
            enum_type: None,
            inheritable_property_ids: WordMap::default(),
            initialized_properties: WordSet::default(),
            invalid_dependencies: WordSet::default(),
            span,
            name_span,
            methods: WordSet::default(),
            pseudo_methods: WordSet::default(),
            static_pseudo_methods: WordSet::default(),
            overridden_method_ids: WordMap::default(),
            overridden_property_ids: WordMap::default(),
            properties: WordMap::default(),
            template_variance: Vec::new(),
            template_type_extends_count: WordMap::default(),
            template_extended_parameters: WordMap::default(),
            template_extended_parameter_paths: WordMap::default(),
            template_extended_offsets: WordMap::default(),
            template_type_implements_count: WordMap::default(),
            template_type_uses_count: WordMap::default(),
            template_types: TemplateTypes::default(),
            used_traits: WordSet::default(),
            trait_alias_map: WordMap::default(),
            trait_visibility_map: WordMap::default(),
            trait_final_map: WordSet::default(),
            name,
            original_name,
            child_class_likes: None,
            template_readonly: WordSet::default(),
            has_sealed_methods: None,
            has_sealed_properties: None,
            permitted_inheritors: None,
            issues: vec![],
            attribute_flags: None,
            type_aliases: WordMap::default(),
            imported_type_aliases: WordMap::default(),
            mixins: Vec::default(),
            version_constraint: VersionConstraint::unconstrained(),
        }
    }

    /// Returns `true` when this class-like is available in the given PHP
    /// version.
    #[inline]
    #[must_use]
    pub fn is_available_in_version(&self, version: PHPVersion) -> bool {
        self.version_constraint.allows_version(version)
    }

    /// Returns `true` when this class-like is available across the entire
    /// supplied [`PHPVersionRange`].
    #[inline]
    #[must_use]
    pub fn is_available_in_version_range(&self, range: PHPVersionRange) -> bool {
        self.version_constraint.allows_version_range(range)
    }

    /// Returns a reference to the map of trait method aliases.
    #[inline]
    #[must_use]
    pub fn get_trait_alias_map(&self) -> &WordMap<Word> {
        &self.trait_alias_map
    }

    /// Returns a vector of the generic type parameter names.
    #[inline]
    #[must_use]
    pub fn get_template_type_names(&self) -> Vec<Word> {
        self.template_types.keys().copied().collect()
    }

    /// Returns type parameters for a specific generic parameter name.
    #[inline]
    #[must_use]
    pub fn get_template_type(&self, name: Word) -> Option<&GenericTemplate> {
        self.template_types.get(&name)
    }

    /// Returns type parameters for a specific generic parameter name with its index.
    #[inline]
    #[must_use]
    pub fn get_template_type_with_index(&self, name: Word) -> Option<(usize, &GenericTemplate)> {
        self.template_types.get_full(&name).map(|(index, _, types)| (index, types))
    }

    #[must_use]
    pub fn get_template_for_index(&self, index: usize) -> Option<(Word, &GenericTemplate)> {
        self.template_types.get_index(index).map(|(name, types)| (*name, types))
    }

    #[must_use]
    pub fn get_template_name_for_index(&self, index: usize) -> Option<Word> {
        self.template_types.get_index(index).map(|(name, _)| *name)
    }

    #[must_use]
    pub fn get_template_index_for_name(&self, name: Word) -> Option<usize> {
        self.template_types.get_index_of(&name)
    }

    /// Checks if a specific parent is either a parent class or interface.
    #[inline]
    #[must_use]
    pub fn has_parent(&self, parent: Word) -> bool {
        self.all_parent_classes.contains(&parent) || self.all_parent_interfaces.contains(&parent)
    }

    /// Checks if a specific parent has template extended parameters.
    #[inline]
    #[must_use]
    pub fn has_template_extended_parameter(&self, parent: Word) -> bool {
        self.template_extended_parameters.contains_key(&parent)
    }

    /// Checks if a specific method appears in this class-like.
    #[inline]
    #[must_use]
    pub fn has_appearing_method(&self, method: Word) -> bool {
        self.appearing_method_ids.contains_key(&method)
    }

    /// Returns a vector of property names.
    #[inline]
    #[must_use]
    pub fn get_property_names(&self) -> WordSet {
        self.properties.keys().copied().collect()
    }

    /// Checks if a specific property appears in this class-like.
    #[inline]
    #[must_use]
    pub fn has_appearing_property(&self, name: Word) -> bool {
        self.appearing_property_ids.contains_key(&name)
    }

    /// Checks if a specific property is declared in this class-like.
    #[inline]
    #[must_use]
    pub fn has_declaring_property(&self, name: Word) -> bool {
        self.declaring_property_ids.contains_key(&name)
    }

    /// Takes ownership of the issues found for this class-like structure.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }

    /// Adds a single direct parent interface.
    #[inline]
    pub fn add_direct_parent_interface(&mut self, interface: Word) {
        self.direct_parent_interfaces.insert(interface);
        self.all_parent_interfaces.insert(interface);
    }

    /// Adds a single interface to the list of all parent interfaces. Use with caution, normally derived.
    #[inline]
    pub fn add_all_parent_interface(&mut self, interface: Word) {
        self.all_parent_interfaces.insert(interface);
    }

    /// Adds multiple interfaces to the list of all parent interfaces. Use with caution.
    #[inline]
    pub fn add_all_parent_interfaces(&mut self, interfaces: impl IntoIterator<Item = Word>) {
        self.all_parent_interfaces.extend(interfaces);
    }

    /// Adds multiple ancestor classes. Use with caution.
    #[inline]
    pub fn add_all_parent_classes(&mut self, classes: impl IntoIterator<Item = Word>) {
        self.all_parent_classes.extend(classes);
    }

    /// Adds a single used trait. Returns `true` if the trait was not already present.
    #[inline]
    pub fn add_used_trait(&mut self, trait_name: Word) -> bool {
        self.used_traits.insert(trait_name)
    }

    /// Adds multiple used traits.
    #[inline]
    pub fn add_used_traits(&mut self, traits: impl IntoIterator<Item = Word>) {
        self.used_traits.extend(traits);
    }

    /// Adds or updates a single trait alias. Returns the previous original name if one existed for the alias.
    #[inline]
    pub fn add_trait_alias(&mut self, method: Word, alias: Word) -> Option<Word> {
        self.trait_alias_map.insert(method, alias)
    }

    /// Adds or updates a single trait visibility override. Returns the previous visibility if one existed.
    #[inline]
    pub fn add_trait_visibility(&mut self, method: Word, visibility: Visibility) -> Option<Visibility> {
        self.trait_visibility_map.insert(method, visibility)
    }

    /// Adds a single template type definition.
    #[inline]
    pub fn add_template_type(&mut self, name: Word, constraint: GenericTemplate) {
        self.template_types.insert(name, constraint);
    }

    /// Set the variance for the template parameters
    #[inline]
    pub fn set_template_variance(&mut self, template_variance: Vec<Variance>) {
        self.template_variance = template_variance;
    }

    /// Adds or replaces the offset types for a specific template parameter name.
    #[inline]
    pub fn add_template_extended_offset(&mut self, name: Word, types: Vec<TUnion>) -> Option<Vec<TUnion>> {
        self.template_extended_offsets.insert(name, types)
    }

    /// Adds or replaces the resolved parameters for a specific parent FQCN.
    #[inline]
    pub fn extend_template_extended_parameters(
        &mut self,
        template_extended_parameters: WordMap<IndexMap<Word, TUnion, RandomState>>,
    ) {
        self.template_extended_parameters.extend(template_extended_parameters);
    }

    /// Adds or replaces a single resolved parameter for the parent FQCN.
    #[inline]
    pub fn add_template_extended_parameter(
        &mut self,
        parent_fqcn: Word,
        parameter_name: Word,
        parameter_type: TUnion,
    ) -> Option<TUnion> {
        self.template_extended_parameters.entry(parent_fqcn).or_default().insert(parameter_name, parameter_type)
    }

    /// Records one complete parameterization of `ancestor` (a single inheritance
    /// path), de-duplicating against parameterizations already recorded.
    #[inline]
    pub fn record_template_extended_path(&mut self, ancestor: Word, parameters: IndexMap<Word, TUnion, RandomState>) {
        if parameters.is_empty() {
            return;
        }

        let paths = self.template_extended_parameter_paths.entry(ancestor).or_default();
        if !paths.contains(&parameters) {
            paths.push(parameters);
        }
    }

    /// Adds or updates the declaring method identifier for a method name.
    #[inline]
    pub fn add_declaring_method_id(
        &mut self,
        method: Word,
        declaring_method_id: MethodIdentifier,
    ) -> Option<MethodIdentifier> {
        self.add_appearing_method_id(method, declaring_method_id);
        self.declaring_method_ids.insert(method, declaring_method_id)
    }

    /// Adds or updates the appearing method identifier for a method name.
    #[inline]
    pub fn add_appearing_method_id(
        &mut self,
        method: Word,
        appearing_method_id: MethodIdentifier,
    ) -> Option<MethodIdentifier> {
        self.appearing_method_ids.insert(method, appearing_method_id)
    }

    /// Adds a parent method identifier to the map for an overridden method. Initializes map if needed. Returns the previous value if one existed.
    #[inline]
    pub fn add_overridden_method_parent(
        &mut self,
        method: Word,
        parent_method_id: MethodIdentifier,
    ) -> Option<MethodIdentifier> {
        self.overridden_method_ids
            .entry(method)
            .or_default()
            .insert(parent_method_id.get_class_name(), parent_method_id)
    }

    /// Adds or updates a property's metadata. Returns the previous metadata if the property existed.
    #[inline]
    pub fn add_property(&mut self, name: Word, property_metadata: PropertyMetadata) -> Option<PropertyMetadata> {
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
    pub fn add_declaring_property_id(&mut self, prop: Word, declaring_fqcn: Word) -> Option<Word> {
        self.appearing_property_ids.insert(prop, declaring_fqcn);
        self.declaring_property_ids.insert(prop, declaring_fqcn)
    }

    #[must_use]
    pub fn get_missing_required_interface<'meta>(&self, other: &'meta ClassLikeMetadata) -> Option<&'meta Word> {
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

    #[must_use]
    pub fn get_missing_required_extends<'meta>(&self, other: &'meta ClassLikeMetadata) -> Option<&'meta Word> {
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

    #[must_use]
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
        self.attributes.shrink_to_fit();
        self.constants.shrink_to_fit();
        self.enum_cases.shrink_to_fit();
        self.type_aliases.shrink_to_fit();
    }
}
