use ahash::HashMap;
use ahash::HashSet;
use ahash::RandomState;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_reporting::Issue;
use mago_span::Span;

use crate::flags::attribute::AttributeFlags;
use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::enum_case::EnumCaseMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::misc::GenericParent;
use crate::symbol::SymbolKind;
use crate::ttype::atomic::TAtomic;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

type TemplateTuple = (StringIdentifier, Vec<(GenericParent, TUnion)>);

/// Contains comprehensive metadata for a PHP class-like structure (class, interface, trait, enum).
///
/// Aggregates information about inheritance, traits, generics, methods, properties, constants,
/// attributes, docblock tags, analysis flags, and more.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassLikeMetadata {
    pub name: StringIdentifier,
    pub original_name: StringIdentifier,
    pub span: Span,
    pub direct_parent_interfaces: Vec<StringIdentifier>,
    pub all_parent_interfaces: Vec<StringIdentifier>,
    pub direct_parent_class: Option<StringIdentifier>,
    pub require_extends: Vec<StringIdentifier>,
    pub require_implements: Vec<StringIdentifier>,
    pub all_parent_classes: Vec<StringIdentifier>,
    pub used_traits: HashSet<StringIdentifier>,
    pub trait_alias_map: HashMap<StringIdentifier, StringIdentifier>,
    pub trait_visibility_map: HashMap<StringIdentifier, Visibility>,
    pub trait_final_map: HashSet<StringIdentifier>,
    pub child_class_likes: Option<HashSet<StringIdentifier>>,
    pub name_span: Option<Span>,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_immutable: bool,
    pub is_readonly: bool,
    pub is_deprecated: bool,
    pub is_enum_interface: bool,
    pub specialized_instance: bool,
    pub is_populated: bool,
    pub is_internal: bool,
    pub is_mutation_free: bool,
    pub is_external_mutation_free: bool,
    pub allows_private_mutation: bool,
    pub has_consistent_constructor: bool,
    pub has_consistent_templates: bool,
    pub kind: SymbolKind,
    pub template_types: Vec<TemplateTuple>,
    pub template_readonly: HashSet<StringIdentifier>,
    pub template_variance: HashMap<usize, Variance>,
    pub template_extended_offsets: HashMap<StringIdentifier, Vec<TUnion>>,
    pub template_extended_parameters: HashMap<StringIdentifier, IndexMap<StringIdentifier, TUnion, RandomState>>,
    pub template_type_extends_count: HashMap<StringIdentifier, usize>,
    pub template_type_implements_count: HashMap<StringIdentifier, usize>,
    pub template_type_uses_count: HashMap<StringIdentifier, usize>,
    pub methods: Vec<StringIdentifier>,
    pub pseudo_methods: Vec<StringIdentifier>,
    pub static_pseudo_methods: Vec<StringIdentifier>,
    pub declaring_method_ids: HashMap<StringIdentifier, StringIdentifier>,
    pub appearing_method_ids: HashMap<StringIdentifier, StringIdentifier>,
    pub overridden_method_ids: HashMap<StringIdentifier, HashSet<StringIdentifier>>,
    pub inheritable_method_ids: HashMap<StringIdentifier, StringIdentifier>,
    pub potential_declaring_method_ids: HashMap<StringIdentifier, HashSet<StringIdentifier>>,
    pub properties: HashMap<StringIdentifier, PropertyMetadata>,
    pub appearing_property_ids: HashMap<StringIdentifier, StringIdentifier>,
    pub declaring_property_ids: HashMap<StringIdentifier, StringIdentifier>,
    pub inheritable_property_ids: HashMap<StringIdentifier, StringIdentifier>,
    pub overridden_property_ids: HashMap<StringIdentifier, Vec<StringIdentifier>>,
    pub initialized_properties: Vec<StringIdentifier>,
    pub constants: IndexMap<StringIdentifier, ClassLikeConstantMetadata, RandomState>,
    pub enum_cases: IndexMap<StringIdentifier, EnumCaseMetadata, RandomState>,
    pub invalid_dependencies: Vec<StringIdentifier>,
    pub attributes: Vec<AttributeMetadata>,
    pub enum_type: Option<TAtomic>,
    pub has_sealed_methods: Option<bool>,
    pub has_sealed_properties: Option<bool>,
    pub permitted_inheritors: Option<HashSet<StringIdentifier>>,
    pub issues: Vec<Issue>,
    pub attribute_flags: Option<AttributeFlags>,
    pub unchecked: bool,
}

impl ClassLikeMetadata {
    pub fn new(
        name: StringIdentifier,
        original_name: StringIdentifier,
        span: Span,
        name_span: Option<Span>,
    ) -> ClassLikeMetadata {
        ClassLikeMetadata {
            constants: IndexMap::with_hasher(RandomState::new()),
            enum_cases: IndexMap::with_hasher(RandomState::new()),
            specialized_instance: false,
            is_populated: false,
            is_deprecated: false,
            is_abstract: false,
            is_final: false,
            is_readonly: false,
            is_immutable: false,
            is_internal: false,
            is_mutation_free: false,
            is_external_mutation_free: false,
            allows_private_mutation: false,
            has_consistent_constructor: false,
            has_consistent_templates: false,
            is_enum_interface: false,
            kind: SymbolKind::Class,
            direct_parent_interfaces: vec![],
            all_parent_classes: vec![],
            appearing_method_ids: HashMap::default(),
            attributes: Vec::new(),
            all_parent_interfaces: vec![],
            declaring_method_ids: HashMap::default(),
            appearing_property_ids: HashMap::default(),
            declaring_property_ids: HashMap::default(),
            direct_parent_class: None,
            require_extends: vec![],
            require_implements: vec![],
            inheritable_method_ids: HashMap::default(),
            enum_type: None,
            inheritable_property_ids: HashMap::default(),
            initialized_properties: vec![],
            invalid_dependencies: Vec::new(),
            span,
            name_span,
            methods: vec![],
            pseudo_methods: vec![],
            static_pseudo_methods: vec![],
            overridden_method_ids: HashMap::default(),
            overridden_property_ids: HashMap::default(),
            potential_declaring_method_ids: HashMap::default(),
            properties: HashMap::default(),
            template_variance: HashMap::default(),
            template_type_extends_count: HashMap::default(),
            template_extended_parameters: HashMap::default(),
            template_extended_offsets: HashMap::default(),
            template_type_implements_count: HashMap::default(),
            template_type_uses_count: HashMap::default(),
            template_types: vec![],
            used_traits: HashSet::default(),
            trait_alias_map: HashMap::default(),
            trait_visibility_map: HashMap::default(),
            trait_final_map: HashSet::default(),
            name,
            original_name,
            child_class_likes: None,
            template_readonly: HashSet::default(),
            has_sealed_methods: None,
            has_sealed_properties: None,
            permitted_inheritors: None,
            issues: vec![],
            attribute_flags: None,
            unchecked: false,
        }
    }

    /// Returns the source code location (span) covering the entire definition.
    #[inline]
    pub fn get_span(&self) -> Span {
        self.span
    }

    /// Checks if this class-like is user-defined.
    #[inline]
    pub fn is_user_defined(&self) -> bool {
        self.span.start.source.category().is_user_defined()
    }

    /// Returns the specific source code location (span) of the class-like name identifier.
    #[inline]
    pub fn get_name_span(&self) -> Option<Span> {
        self.name_span
    }

    /// Checks if this class-like is a backed enum.
    #[inline]
    pub const fn is_backed_enum(&self) -> bool {
        self.kind.is_enum() && self.enum_type.is_some()
    }

    /// Returns a reference to the list of all parent interfaces.
    #[inline]
    pub fn get_all_parent_interfaces(&self) -> &[StringIdentifier] {
        &self.all_parent_interfaces
    }

    /// Returns a reference to the list of all parent classes.
    #[inline]
    pub fn get_all_parent_classes(&self) -> &[StringIdentifier] {
        &self.all_parent_classes
    }

    /// Returns a slice of direct parent interfaces.
    #[inline]
    pub fn get_direct_parent_interfaces(&self) -> Vec<StringIdentifier> {
        self.direct_parent_interfaces.to_vec()
    }

    /// Returns the direct parent class, if one exists.
    #[inline]
    pub fn get_direct_parent_class(&self) -> Option<StringIdentifier> {
        self.direct_parent_class
    }

    /// Returns a reference to the set of used traits.
    #[inline]
    pub fn get_used_traits(&self) -> Vec<StringIdentifier> {
        self.used_traits.iter().copied().collect()
    }

    /// Returns a reference to the map of trait method aliases.
    #[inline]
    pub fn get_trait_alias_map(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.trait_alias_map
    }

    /// Returns a vector of the generic type parameter names.
    #[inline]
    pub fn get_template_type_names(&self) -> Vec<StringIdentifier> {
        self.template_types.iter().map(|(name, _)| *name).collect()
    }

    /// Returns type parameters for a specific generic parameter name.
    #[inline]
    pub fn get_template_type(&self, name: &StringIdentifier) -> Option<&Vec<(GenericParent, TUnion)>> {
        self.template_types.iter().find_map(|(param_name, types)| if param_name == name { Some(types) } else { None })
    }

    /// Returns type parameters for a specific generic parameter name with its index.
    #[inline]
    pub fn get_template_type_with_index(
        &self,
        name: &StringIdentifier,
    ) -> Option<(usize, &Vec<(GenericParent, TUnion)>)> {
        self.template_types
            .iter()
            .enumerate()
            .find_map(|(index, (param_name, types))| if param_name == name { Some((index, types)) } else { None })
    }

    pub fn get_template_for_index(&self, index: usize) -> Option<(StringIdentifier, &Vec<(GenericParent, TUnion)>)> {
        self.template_types.get(index).map(|(name, types)| (*name, types))
    }

    pub fn get_template_name_for_index(&self, index: usize) -> Option<StringIdentifier> {
        self.template_types.get(index).map(|(name, _)| *name)
    }

    pub fn get_template_index_for_name(&self, name: &StringIdentifier) -> Option<usize> {
        self.template_types.iter().position(|(param_name, _)| param_name == name)
    }

    /// Checks if a specific parent is either a parent class or interface.
    #[inline]
    pub fn has_parent(&self, parent: &StringIdentifier) -> bool {
        self.all_parent_classes.contains(parent) || self.all_parent_interfaces.contains(parent)
    }

    /// Checks if a specific parent has template extended parameters.
    #[inline]
    pub fn has_template_extended_parameter(&self, parent: &StringIdentifier) -> bool {
        self.template_extended_parameters.contains_key(parent)
    }

    /// Returns a slice of methods defined directly in this class-like.
    #[inline]
    pub fn get_methods(&self) -> &[StringIdentifier] {
        &self.methods
    }

    /// Checks if a specific method is defined in this class-like.
    #[inline]
    pub fn has_method(&self, method: &StringIdentifier) -> bool {
        self.methods.contains(method)
    }

    /// Returns a reference to the map of method name to its declaring class/trait FQCN.
    #[inline]
    pub fn get_declaring_method_ids(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.declaring_method_ids
    }

    /// Returns a reference to the map of method name to its appearing class/trait FQCN in this context.
    #[inline]
    pub fn get_appearing_method_ids(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.appearing_method_ids
    }

    /// Checks if a specific method appears in this class-like.
    #[inline]
    pub fn has_appearing_method(&self, method: &StringIdentifier) -> bool {
        self.appearing_method_ids.contains_key(method)
    }

    /// Returns a reference to the map of overridden method name to the set of parent FQCNs.
    #[inline]
    pub fn get_overridden_method_ids(&self) -> &HashMap<StringIdentifier, HashSet<StringIdentifier>> {
        &self.overridden_method_ids
    }

    /// Returns a reference to a specific method's overridden parent FQCNs.
    #[inline]
    pub fn get_overridden_method_id(&self, method: &StringIdentifier) -> Option<&HashSet<StringIdentifier>> {
        self.overridden_method_ids.get(method)
    }

    /// Returns a mutable reference to a specific method's overridden parent FQCNs.
    #[inline]
    pub fn get_overridden_method_id_mut(
        &mut self,
        method: &StringIdentifier,
    ) -> Option<&mut HashSet<StringIdentifier>> {
        self.overridden_method_ids.get_mut(method)
    }

    /// Returns a reference to the map of method name to the FQCN it's inherited from.
    #[inline]
    pub fn get_inheritable_method_ids(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.inheritable_method_ids
    }

    /// Returns a reference to a specific method's potential declaring classes/traits.
    #[inline]
    pub fn get_potential_declaring_method_id(&self, method: &StringIdentifier) -> Option<&HashSet<StringIdentifier>> {
        self.potential_declaring_method_ids.get(method)
    }

    /// Returns a reference to the map of property name to its metadata.
    #[inline]
    pub fn get_properties(&self) -> &HashMap<StringIdentifier, PropertyMetadata> {
        &self.properties
    }

    /// Returns a mutable reference to the map of property name to its metadata.
    #[inline]
    pub fn get_properties_mut(&mut self) -> &mut HashMap<StringIdentifier, PropertyMetadata> {
        &mut self.properties
    }

    /// Returns a vector of property names.
    #[inline]
    pub fn get_property_names(&self) -> Vec<StringIdentifier> {
        self.properties.keys().copied().collect()
    }

    /// Returns a reference to a specific property by its name.
    #[inline]
    pub fn get_property(&self, name: &StringIdentifier) -> Option<&PropertyMetadata> {
        self.properties.get(name)
    }

    /// Returns a reference to the map of property name to its appearing class/trait FQCN.
    #[inline]
    pub fn get_appearing_property_ids(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.appearing_property_ids
    }

    /// Checks if a specific property appears in this class-like.
    #[inline]
    pub fn has_appearing_property(&self, name: &StringIdentifier) -> bool {
        self.appearing_property_ids.contains_key(name)
    }

    /// Returns a reference to the map of property name to its declaring class/trait FQCN.
    #[inline]
    pub fn get_declaring_property_ids(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.declaring_property_ids
    }

    /// Checks if a specific property is declared in this class-like.
    #[inline]
    pub fn has_declaring_property(&self, name: &StringIdentifier) -> bool {
        self.declaring_property_ids.contains_key(name)
    }

    /// Returns a reference to the map of property name to the FQCN it's inherited from.
    #[inline]
    pub fn get_inheritable_property_ids(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.inheritable_property_ids
    }

    /// Returns a slice of invalid dependencies (unresolved parent classes/interfaces).
    #[inline]
    pub fn get_invalid_dependencies(&self) -> &[StringIdentifier] {
        &self.invalid_dependencies
    }

    /// Returns a slice of attributes attached to the class-like definition.
    #[inline]
    pub fn get_attributes(&self) -> &[AttributeMetadata] {
        &self.attributes
    }

    /// Takes ownership of the issues found for this class-like structure.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }

    /// Adds a single direct parent interface.
    #[inline]
    pub fn add_direct_parent_interface(&mut self, interface: StringIdentifier) {
        self.direct_parent_interfaces.push(interface);
        self.all_parent_interfaces.push(interface);
    }

    /// Adds a single interface to the list of all parent interfaces. Use with caution, normally derived.
    #[inline]
    pub fn add_all_parent_interface(&mut self, interface: StringIdentifier) {
        self.all_parent_interfaces.push(interface);
    }

    /// Adds multiple interfaces to the list of all parent interfaces. Use with caution.
    #[inline]
    pub fn add_all_parent_interfaces(&mut self, interfaces: impl IntoIterator<Item = StringIdentifier>) {
        self.all_parent_interfaces.extend(interfaces);
    }

    /// Sets the direct parent class.
    #[inline]
    pub fn set_direct_parent_class(&mut self, parent: Option<StringIdentifier>) {
        self.direct_parent_class = parent;
        if let Some(parent) = &self.direct_parent_class {
            self.all_parent_classes.push(*parent);
        }
    }

    /// Returns a new instance with the direct parent class set.
    #[inline]
    pub fn with_direct_parent_class(mut self, parent: Option<StringIdentifier>) -> Self {
        self.set_direct_parent_class(parent);
        self
    }

    /// Adds a single required extend entry.
    #[inline]
    pub fn add_require_extend(&mut self, require: StringIdentifier) {
        self.set_direct_parent_class(Some(require));
        self.require_extends.push(require);
    }

    /// Adds a single required implement entry.
    #[inline]
    pub fn add_require_implement(&mut self, require: StringIdentifier) {
        self.add_all_parent_interface(require);
        self.require_implements.push(require);
    }

    /// Adds multiple ancestor classes. Use with caution.
    #[inline]
    pub fn add_all_parent_classes(&mut self, classes: impl IntoIterator<Item = StringIdentifier>) {
        self.all_parent_classes.extend(classes);
    }

    /// Adds a single used trait. Returns `true` if the trait was not already present.
    #[inline]
    pub fn add_used_trait(&mut self, trait_name: StringIdentifier) -> bool {
        self.used_traits.insert(trait_name)
    }

    /// Adds multiple used traits.
    #[inline]
    pub fn add_used_traits(&mut self, traits: impl IntoIterator<Item = StringIdentifier>) {
        self.used_traits.extend(traits);
    }

    /// Adds or updates a single trait alias. Returns the previous original name if one existed for the alias.
    #[inline]
    pub fn add_trait_alias(&mut self, method: StringIdentifier, alias: StringIdentifier) -> Option<StringIdentifier> {
        self.trait_alias_map.insert(method, alias)
    }

    /// Adds or updates a single trait visibility override. Returns the previous visibility if one existed.
    #[inline]
    pub fn add_trait_visibility(&mut self, method: StringIdentifier, visibility: Visibility) -> Option<Visibility> {
        self.trait_visibility_map.insert(method, visibility)
    }

    /// Adds a single final trait method. Returns `true` if the method was not already present.
    #[inline]
    pub fn add_trait_final(&mut self, method: StringIdentifier) -> bool {
        self.trait_final_map.insert(method)
    }

    /// Adds a single template type definition.
    #[inline]
    pub fn add_template_type(&mut self, template: TemplateTuple) {
        self.template_types.push(template);
    }
    /// Adds a single readonly template parameter. Returns `true` if the parameter was not already present.
    #[inline]
    pub fn add_template_readonly(&mut self, template_name: StringIdentifier) -> bool {
        self.template_readonly.insert(template_name)
    }

    /// Adds or updates the variance for a specific parameter index. Returns the previous variance if one existed.
    #[inline]
    pub fn add_template_variance_parameter(&mut self, index: usize, variance: Variance) -> Option<Variance> {
        self.template_variance.insert(index, variance)
    }

    /// Adds or replaces the offset types for a specific template parameter name.
    #[inline]
    pub fn add_template_extended_offset(&mut self, name: StringIdentifier, types: Vec<TUnion>) -> Option<Vec<TUnion>> {
        self.template_extended_offsets.insert(name, types)
    }

    /// Adds or replaces the resolved parameters for a specific parent FQCN.
    #[inline]
    pub fn extend_template_extended_parameters(
        &mut self,
        template_extended_parameters: HashMap<StringIdentifier, IndexMap<StringIdentifier, TUnion, RandomState>>,
    ) {
        self.template_extended_parameters.extend(template_extended_parameters);
    }

    /// Adds or replaces a single resolved parameter for the parent FQCN.
    #[inline]
    pub fn add_template_extended_parameter(
        &mut self,
        parent_fqcn: StringIdentifier,
        parameter_name: StringIdentifier,
        parameter_type: TUnion,
    ) -> Option<TUnion> {
        self.template_extended_parameters.entry(parent_fqcn).or_default().insert(parameter_name, parameter_type)
    }

    /// Adds a single method name.
    #[inline]
    pub fn add_method(&mut self, method: StringIdentifier) {
        self.methods.push(method);
    }

    /// Adds or updates the declaring class FQCN for a method name.
    #[inline]
    pub fn add_declaring_method_id(
        &mut self,
        method: StringIdentifier,
        declaring_fqcn: StringIdentifier,
    ) -> Option<StringIdentifier> {
        self.add_appearing_method_id(method, declaring_fqcn);
        self.declaring_method_ids.insert(method, declaring_fqcn)
    }

    /// Adds or updates the appearing class FQCN for a method name.
    #[inline]
    pub fn add_appearing_method_id(
        &mut self,
        method: StringIdentifier,
        appearing_fqcn: StringIdentifier,
    ) -> Option<StringIdentifier> {
        self.appearing_method_ids.insert(method, appearing_fqcn)
    }

    /// Adds a parent FQCN to the set for an overridden method. Initializes set if needed. Returns `true` if added.
    #[inline]
    pub fn add_overridden_method_parent(&mut self, method: StringIdentifier, parent_fqcn: StringIdentifier) -> bool {
        self.overridden_method_ids.entry(method).or_default().insert(parent_fqcn)
    }

    /// Adds or updates the inheriting source FQCN for a method name.
    #[inline]
    pub fn add_inheritable_method_id(
        &mut self,
        method: StringIdentifier,
        source_fqcn: StringIdentifier,
    ) -> Option<StringIdentifier> {
        self.inheritable_method_ids.insert(method, source_fqcn)
    }

    /// Sets the potential declaring method class names map, replacing the existing one.
    #[inline]
    pub fn set_potential_declaring_method_class_names(
        &mut self,
        method: StringIdentifier,
        potentially_declaring_fqcns: HashSet<StringIdentifier>,
    ) {
        self.potential_declaring_method_ids.insert(method, potentially_declaring_fqcns);
    }

    /// Adds a potential declaring FQCN to the set for a method. Initializes set if needed. Returns `true` if added.
    #[inline]
    pub fn add_potential_declaring_method(
        &mut self,
        method: StringIdentifier,
        potential_fqcn: StringIdentifier,
    ) -> bool {
        self.potential_declaring_method_ids.entry(method).or_default().insert(potential_fqcn)
    }

    /// Adds or updates a property's metadata. Returns the previous metadata if the property existed.
    #[inline]
    pub fn add_property(
        &mut self,
        name: StringIdentifier,
        property_metadata: PropertyMetadata,
    ) -> Option<PropertyMetadata> {
        let class_name = self.name;

        self.add_declaring_property_id(name, class_name);
        if property_metadata.has_default() {
            self.initialized_properties.push(name);
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

    /// Adds or updates the appearing class FQCN for a property name.
    #[inline]
    pub fn add_appearing_property_id(
        &mut self,
        prop: StringIdentifier,
        appearing_fqcn: StringIdentifier,
    ) -> Option<StringIdentifier> {
        self.appearing_property_ids.insert(prop, appearing_fqcn)
    }

    /// Adds or updates the declaring class FQCN for a property name.
    #[inline]
    pub fn add_declaring_property_id(
        &mut self,
        prop: StringIdentifier,
        declaring_fqcn: StringIdentifier,
    ) -> Option<StringIdentifier> {
        self.add_appearing_property_id(prop, declaring_fqcn);
        self.declaring_property_ids.insert(prop, declaring_fqcn)
    }

    #[inline]
    pub fn mark_as_populated(&mut self) {
        self.is_populated = true;
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
