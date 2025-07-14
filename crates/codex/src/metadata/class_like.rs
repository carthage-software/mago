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

    /// Returns a reference the direct parent class, if one exists.
    #[inline]
    pub fn get_direct_parent_class_ref(&self) -> Option<&StringIdentifier> {
        self.direct_parent_class.as_ref()
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

    /// Returns a reference to the map of trait visibility overrides.
    #[inline]
    pub fn get_trait_visibility_map(&self) -> &HashMap<StringIdentifier, Visibility> {
        &self.trait_visibility_map
    }

    /// Returns a reference to the set of trait methods marked final.
    #[inline]
    pub fn get_trait_final_map(&self) -> &HashSet<StringIdentifier> {
        &self.trait_final_map
    }

    /// Returns a reference to the list of required interfaces/classes.
    #[inline]
    pub fn get_require_extends(&self) -> &[StringIdentifier] {
        &self.require_extends
    }

    /// Returns a reference to the list of required interfaces.
    #[inline]
    pub fn get_require_implements(&self) -> &[StringIdentifier] {
        &self.require_implements
    }

    /// Checks if this class-like requires extending a specific interface/class.
    #[inline]
    pub fn has_require_extends(&self, name: &StringIdentifier) -> bool {
        self.require_extends.contains(name)
    }

    /// Checks if this class-like requires implementing a specific interface.
    #[inline]
    pub fn has_require_implements(&self, name: &StringIdentifier) -> bool {
        self.require_implements.contains(name)
    }

    /// Checks if this class-like requires implementing or extending a specific interface/class.
    #[inline]
    pub fn has_require(&self, name: &StringIdentifier) -> bool {
        self.require_extends.contains(name) || self.require_implements.contains(name)
    }

    /// Returns a reference to the set of direct child classlikes, if tracked.
    #[inline]
    pub fn get_child_class_likes(&self) -> Option<&HashSet<StringIdentifier>> {
        self.child_class_likes.as_ref()
    }

    /// Checks if this class-like has template types.
    #[inline]
    pub fn has_template_types(&self) -> bool {
        !self.template_types.is_empty()
    }

    /// Returns a slice of the generic type parameters (`@template T`).
    #[inline]
    pub fn get_template_types(&self) -> &[(StringIdentifier, Vec<(GenericParent, TUnion)>)] {
        &self.template_types
    }

    /// Returns a mutable reference to the generic type parameters (`@template T`).
    #[inline]
    pub fn get_template_types_mut(&mut self) -> &mut Vec<(StringIdentifier, Vec<(GenericParent, TUnion)>)> {
        &mut self.template_types
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

    /// Returns a reference to the set of `@readonly` template parameters.
    #[inline]
    pub fn get_template_readonly(&self) -> &HashSet<StringIdentifier> {
        &self.template_readonly
    }

    /// Returns a mutable reference to the set of `@readonly` template parameters.
    #[inline]
    pub fn has_readonly_template(&self, name: &StringIdentifier) -> bool {
        self.template_readonly.contains(name)
    }

    /// Returns a reference to the map of template parameter variances.
    #[inline]
    pub fn get_template_variance(&self) -> &HashMap<usize, Variance> {
        &self.template_variance
    }

    /// Returns the variance for a specific template parameter index.
    #[inline]
    pub fn get_template_variance_for_index(&self, index: usize) -> Option<&Variance> {
        self.template_variance.get(&index)
    }

    /// Checks if a specific parent is a parent interface.
    #[inline]
    pub fn has_parent_interface(&self, parent: &StringIdentifier) -> bool {
        self.all_parent_interfaces.contains(parent)
    }

    /// Checks if a specific parent is a direct parent interface.
    #[inline]
    pub fn has_direct_parent_interface(&self, parent: &StringIdentifier) -> bool {
        self.direct_parent_interfaces.contains(parent)
    }

    /// Checks if a specific parent is a parent class.
    #[inline]
    pub fn has_parent_class(&self, parent: &StringIdentifier) -> bool {
        self.all_parent_classes.contains(parent)
    }

    /// Checks if a specific parent is a direct parent class.
    #[inline]
    pub fn has_direct_parent_class(&self, parent: &StringIdentifier) -> bool {
        self.direct_parent_class.as_ref() == Some(parent)
    }

    /// Checks if a specific parent is either a parent class or interface.
    #[inline]
    pub fn has_parent(&self, parent: &StringIdentifier) -> bool {
        self.all_parent_classes.contains(parent) || self.all_parent_interfaces.contains(parent)
    }

    /// Checks if a specific parent is either a direct parent class or interface.
    #[inline]
    pub fn has_direct_parent(&self, parent: &StringIdentifier) -> bool {
        self.direct_parent_class.as_ref() == Some(parent) || self.direct_parent_interfaces.contains(parent)
    }

    /// Checks if a specific trait is used.
    #[inline]
    pub fn has_used_trait(&self, trait_name: &StringIdentifier) -> bool {
        self.used_traits.contains(trait_name)
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

    pub fn get_pseudo_methods(&self) -> &[StringIdentifier] {
        &self.pseudo_methods
    }

    pub fn get_static_pseudo_methods(&self) -> &[StringIdentifier] {
        &self.static_pseudo_methods
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

    /// Returns a reference to a specific method's declaring class/trait FQCN.
    #[inline]
    pub fn get_declaring_method_id(&self, method: &StringIdentifier) -> Option<&StringIdentifier> {
        self.declaring_method_ids.get(method)
    }

    /// Checks if a specific method is declared in this class-like.
    #[inline]
    pub fn has_declaring_method(&self, method: &StringIdentifier) -> bool {
        self.declaring_method_ids.contains_key(method)
    }

    /// Returns a reference to the map of method name to its appearing class/trait FQCN in this context.
    #[inline]
    pub fn get_appearing_method_ids(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.appearing_method_ids
    }

    /// Returns a reference to a specific method's appearing class/trait FQCN in this context.
    #[inline]
    pub fn get_appearing_method_id(&self, method: &StringIdentifier) -> Option<&StringIdentifier> {
        self.appearing_method_ids.get(method)
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

    /// Checks if a specific method is overridden in this class-like.
    #[inline]
    pub fn has_overridden_method(&self, method: &StringIdentifier) -> bool {
        self.overridden_method_ids.contains_key(method)
    }

    /// Returns a reference to the map of method name to the FQCN it's inherited from.
    #[inline]
    pub fn get_inheritable_method_ids(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.inheritable_method_ids
    }

    /// Returns a reference to a specific method's inherited FQCN.
    #[inline]
    pub fn get_inheritable_method_id(&self, method: &StringIdentifier) -> Option<&StringIdentifier> {
        self.inheritable_method_ids.get(method)
    }

    /// Checks if a specific method is inherited in this class-like.
    #[inline]
    pub fn has_inheritable_method(&self, method: &StringIdentifier) -> bool {
        self.inheritable_method_ids.contains_key(method)
    }

    /// Returns a reference to the map of method name to potential declaring classes/traits.
    #[inline]
    pub fn get_potential_declaring_method_ids(&self) -> &HashMap<StringIdentifier, HashSet<StringIdentifier>> {
        &self.potential_declaring_method_ids
    }

    /// Returns a reference to a specific method's potential declaring classes/traits.
    #[inline]
    pub fn get_potential_declaring_method_id(&self, method: &StringIdentifier) -> Option<&HashSet<StringIdentifier>> {
        self.potential_declaring_method_ids.get(method)
    }

    /// Checks if a specific method has potential declaring classes/traits.
    #[inline]
    pub fn has_potential_declaring_method(&self, method: &StringIdentifier) -> bool {
        self.potential_declaring_method_ids.contains_key(method)
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

    /// Checks if a specific property exists.
    #[inline]
    pub fn has_property(&self, name: &StringIdentifier) -> bool {
        self.properties.contains_key(name)
    }

    /// Returns a reference to the map of property name to its appearing class/trait FQCN.
    #[inline]
    pub fn get_appearing_property_ids(&self) -> &HashMap<StringIdentifier, StringIdentifier> {
        &self.appearing_property_ids
    }

    /// Returns a reference to a specific property name's appearing class/trait FQCN.
    #[inline]
    pub fn get_appearing_property_id(&self, name: &StringIdentifier) -> Option<&StringIdentifier> {
        self.appearing_property_ids.get(name)
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

    /// Returns a reference to a specific property name's declaring class/trait FQCN.
    #[inline]
    pub fn get_declaring_property_id(&self, name: &StringIdentifier) -> Option<&StringIdentifier> {
        self.declaring_property_ids.get(name)
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

    /// Returns a reference to a specific property name's inherited FQCN.
    #[inline]
    pub fn get_inheritable_property_id(&self, name: &StringIdentifier) -> Option<&StringIdentifier> {
        self.inheritable_property_ids.get(name)
    }

    /// Checks if a specific property is inherited in this class-like.
    #[inline]
    pub fn has_inheritable_property(&self, name: &StringIdentifier) -> bool {
        self.inheritable_property_ids.contains_key(name)
    }

    /// Returns a reference to the map of overridden property name to the list of parent FQCNs.
    #[inline]
    pub fn get_overridden_property_ids(&self) -> &HashMap<StringIdentifier, Vec<StringIdentifier>> {
        &self.overridden_property_ids
    }

    /// Returns a reference to a specific property name's overridden parent FQCNs.
    #[inline]
    pub fn get_overridden_property_id(&self, name: &StringIdentifier) -> Option<&Vec<StringIdentifier>> {
        self.overridden_property_ids.get(name)
    }

    /// Checks if a specific property is overridden in this class-like.
    #[inline]
    pub fn has_overridden_property(&self, name: &StringIdentifier) -> bool {
        self.overridden_property_ids.contains_key(name)
    }

    /// Returns a slice of properties initialized using a default value.
    #[inline]
    pub fn get_initialized_properties(&self) -> &[StringIdentifier] {
        &self.initialized_properties
    }

    /// Checks if a specific property is initialized using a default value.
    #[inline]
    pub fn has_initialized_property(&self, name: &StringIdentifier) -> bool {
        self.initialized_properties.contains(name)
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

    /// Returns the backing type (`int` or `string`) if this is a backed enum.
    #[inline]
    pub fn get_enum_type(&self) -> Option<&TAtomic> {
        self.enum_type.as_ref()
    }

    /// Returns a mutable reference to the backing type (`int` or `string`) if this is a backed enum.
    #[inline]
    pub fn get_enum_type_mut(&mut self) -> &mut Option<TAtomic> {
        &mut self.enum_type
    }

    /// Returns the `@sealed-methods` status (`Some(true)` or `Some(false)`) or `None` if unspecified.
    #[inline]
    pub fn has_sealed_methods(&self) -> Option<bool> {
        self.has_sealed_methods
    }

    /// Returns the `@sealed-properties` status (`Some(true)` or `Some(false)`) or `None` if unspecified.
    #[inline]
    pub fn get_has_sealed_properties(&self) -> Option<bool> {
        self.has_sealed_properties
    }

    /// Returns a slice of issues found for this class-like structure.
    #[inline]
    pub fn get_issues(&self) -> &[Issue] {
        &self.issues
    }

    /// Takes ownership of the issues found for this class-like structure.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }

    /// Sets the direct parent interfaces, replacing existing ones.
    #[inline]
    pub fn set_direct_parent_interfaces(&mut self, interfaces: impl IntoIterator<Item = StringIdentifier>) {
        self.direct_parent_interfaces = interfaces.into_iter().collect();
        self.all_parent_interfaces.extend(self.direct_parent_interfaces.iter().cloned());
    }

    /// Returns a new instance with the direct parent interfaces replaced.
    #[inline]
    pub fn with_direct_parent_interfaces(mut self, interfaces: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_direct_parent_interfaces(interfaces);
        self
    }

    /// Adds a single direct parent interface.
    #[inline]
    pub fn add_direct_parent_interface(&mut self, interface: StringIdentifier) {
        self.direct_parent_interfaces.push(interface);
        self.all_parent_interfaces.push(interface);
    }

    /// Returns a new instance with the direct parent interface added.
    #[inline]
    pub fn with_added_direct_parent_interface(mut self, interface: StringIdentifier) -> Self {
        self.add_direct_parent_interface(interface);
        self
    }

    /// Adds multiple direct parent interfaces.
    #[inline]
    pub fn add_direct_parent_interfaces(&mut self, interfaces: impl IntoIterator<Item = StringIdentifier>) {
        for interface in interfaces {
            self.add_direct_parent_interface(interface);
        }
    }

    /// Returns a new instance with the direct parent interfaces added.
    #[inline]
    pub fn with_added_direct_parent_interfaces(
        mut self,
        interfaces: impl IntoIterator<Item = StringIdentifier>,
    ) -> Self {
        self.add_direct_parent_interfaces(interfaces);
        self
    }

    /// Clears all direct parent interfaces.
    #[inline]
    pub fn unset_direct_parent_interfaces(&mut self) {
        for interface in &self.direct_parent_interfaces {
            self.all_parent_interfaces.retain(|i| i != interface);
        }

        self.direct_parent_interfaces.clear();
    }

    /// Returns a new instance with no direct parent interfaces.
    #[inline]
    pub fn without_direct_parent_interfaces(mut self) -> Self {
        self.unset_direct_parent_interfaces();
        self
    }

    /// Sets all parent interfaces (direct and indirect), replacing existing ones.
    #[inline]
    pub fn set_all_parent_interfaces(&mut self, interfaces: impl IntoIterator<Item = StringIdentifier>) {
        self.all_parent_interfaces = interfaces.into_iter().collect();
    }

    /// Returns a new instance with all parent interfaces replaced.
    #[inline]
    pub fn with_all_parent_interfaces(mut self, interfaces: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_all_parent_interfaces(interfaces);
        self
    }

    /// Adds a single interface to the list of all parent interfaces. Use with caution, normally derived.
    #[inline]
    pub fn add_all_parent_interface(&mut self, interface: StringIdentifier) {
        self.all_parent_interfaces.push(interface);
    }

    /// Returns a new instance with the interface added to the list of all parent interfaces.
    #[inline]
    pub fn with_added_all_parent_interface(mut self, interface: StringIdentifier) -> Self {
        self.add_all_parent_interface(interface);
        self
    }

    /// Adds multiple interfaces to the list of all parent interfaces. Use with caution.
    #[inline]
    pub fn add_all_parent_interfaces(&mut self, interfaces: impl IntoIterator<Item = StringIdentifier>) {
        self.all_parent_interfaces.extend(interfaces);
    }

    /// Returns a new instance with the interfaces added to the list of all parent interfaces.
    #[inline]
    pub fn with_added_all_parent_interfaces(mut self, interfaces: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_all_parent_interfaces(interfaces);
        self
    }

    /// Clears all parent interfaces. Use with caution.
    #[inline]
    pub fn unset_all_parent_interfaces(&mut self) {
        self.all_parent_interfaces.clear();
    }

    /// Returns a new instance with no parent interfaces. Use with caution.
    #[inline]
    pub fn without_all_parent_interfaces(mut self) -> Self {
        self.unset_all_parent_interfaces();
        self
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

    /// Sets the direct parent class to `None`.
    #[inline]
    pub fn unset_direct_parent_class(&mut self) {
        if let Some(parent) = &self.direct_parent_class {
            self.all_parent_classes.retain(|p| p != parent);
        }

        self.direct_parent_class = None;
    }

    /// Returns a new instance with the direct parent class set to `None`.
    #[inline]
    pub fn without_direct_parent_class(mut self) -> Self {
        self.unset_direct_parent_class();
        self
    }

    /// Sets the required extended classes/interfaces from traits, replacing existing ones.
    #[inline]
    pub fn set_require_extends(&mut self, requires: impl IntoIterator<Item = StringIdentifier>) {
        self.require_extends = requires.into_iter().collect();
    }

    /// Returns a new instance with the required extends replaced.
    #[inline]
    pub fn with_require_extends(mut self, requires: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_require_extends(requires);
        self
    }

    /// Adds a single required extend entry.
    #[inline]
    pub fn add_require_extend(&mut self, require: StringIdentifier) {
        self.set_direct_parent_class(Some(require));
        self.require_extends.push(require);
    }

    /// Returns a new instance with the required extend entry added.
    #[inline]
    pub fn with_added_require_extend(mut self, require: StringIdentifier) -> Self {
        self.add_require_extend(require);
        self
    }

    /// Adds multiple required extend entries.
    #[inline]
    pub fn add_require_extends(&mut self, requires: impl IntoIterator<Item = StringIdentifier>) {
        self.require_extends.extend(requires);
    }

    /// Returns a new instance with the required extend entries added.
    #[inline]
    pub fn with_added_require_extends(mut self, requires: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_require_extends(requires);
        self
    }

    /// Clears all required extend entries.
    #[inline]
    pub fn unset_require_extends(&mut self) {
        self.require_extends.clear();
    }

    /// Returns a new instance with no required extend entries.
    #[inline]
    pub fn without_require_extends(mut self) -> Self {
        self.unset_require_extends();
        self
    }

    /// Sets the required implemented interfaces from traits, replacing existing ones.
    #[inline]
    pub fn set_require_implements(&mut self, requires: impl IntoIterator<Item = StringIdentifier>) {
        self.require_implements = requires.into_iter().collect();
    }

    /// Returns a new instance with the required implements replaced.
    #[inline]
    pub fn with_require_implements(mut self, requires: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_require_implements(requires);
        self
    }

    /// Adds a single required implement entry.
    #[inline]
    pub fn add_require_implement(&mut self, require: StringIdentifier) {
        self.add_all_parent_interface(require);
        self.require_implements.push(require);
    }

    /// Returns a new instance with the required implement entry added.
    #[inline]
    pub fn with_added_require_implement(mut self, require: StringIdentifier) -> Self {
        self.add_require_implement(require);
        self
    }

    /// Adds multiple required implement entries.
    #[inline]
    pub fn add_require_implements(&mut self, requires: impl IntoIterator<Item = StringIdentifier>) {
        self.require_implements.extend(requires);
    }

    /// Returns a new instance with the required implement entries added.
    #[inline]
    pub fn with_added_require_implements(mut self, requires: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_require_implements(requires);
        self
    }

    /// Clears all required implement entries.
    #[inline]
    pub fn unset_require_implements(&mut self) {
        self.require_implements.clear();
    }

    /// Returns a new instance with no required implement entries.
    #[inline]
    pub fn without_require_implements(mut self) -> Self {
        self.unset_require_implements();
        self
    }

    /// Sets all ancestor classes, replacing existing ones. Use with caution, normally derived.
    #[inline]
    pub fn set_all_parent_classes(&mut self, classes: impl IntoIterator<Item = StringIdentifier>) {
        self.all_parent_classes = classes.into_iter().collect();
    }

    /// Returns a new instance with all ancestor classes replaced. Use with caution.
    #[inline]
    pub fn with_all_parent_classes(mut self, classes: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_all_parent_classes(classes);
        self
    }

    /// Adds a single ancestor class. Use with caution.
    #[inline]
    pub fn add_all_parent_class(&mut self, class: StringIdentifier) {
        self.all_parent_classes.push(class);
    }

    /// Returns a new instance with the ancestor class added. Use with caution.
    #[inline]
    pub fn with_added_all_parent_class(mut self, class: StringIdentifier) -> Self {
        self.add_all_parent_class(class);
        self
    }

    /// Adds multiple ancestor classes. Use with caution.
    #[inline]
    pub fn add_all_parent_classes(&mut self, classes: impl IntoIterator<Item = StringIdentifier>) {
        self.all_parent_classes.extend(classes);
    }

    /// Returns a new instance with the ancestor classes added. Use with caution.
    #[inline]
    pub fn with_added_all_parent_classes(mut self, classes: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_all_parent_classes(classes);
        self
    }

    /// Clears all ancestor classes. Use with caution.
    #[inline]
    pub fn unset_all_parent_classes(&mut self) {
        self.all_parent_classes.clear();
    }

    /// Returns a new instance with no ancestor classes. Use with caution.
    #[inline]
    pub fn without_all_parent_classes(mut self) -> Self {
        self.unset_all_parent_classes();
        self
    }

    /// Sets the used traits, replacing existing ones.
    #[inline]
    pub fn set_used_traits(&mut self, traits: impl IntoIterator<Item = StringIdentifier>) {
        self.used_traits = traits.into_iter().collect();
    }

    /// Returns a new instance with the used traits replaced.
    #[inline]
    pub fn with_used_traits(mut self, traits: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_used_traits(traits);
        self
    }

    /// Adds a single used trait. Returns `true` if the trait was not already present.
    #[inline]
    pub fn add_used_trait(&mut self, trait_name: StringIdentifier) -> bool {
        self.used_traits.insert(trait_name)
    }

    /// Returns a new instance with the used trait added.
    #[inline]
    pub fn with_added_used_trait(mut self, trait_name: StringIdentifier) -> Self {
        self.add_used_trait(trait_name);
        self
    }

    /// Adds multiple used traits.
    #[inline]
    pub fn add_used_traits(&mut self, traits: impl IntoIterator<Item = StringIdentifier>) {
        self.used_traits.extend(traits);
    }

    /// Returns a new instance with the used traits added.
    #[inline]
    pub fn with_added_used_traits(mut self, traits: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_used_traits(traits);
        self
    }

    /// Clears all used traits.
    #[inline]
    pub fn unset_used_traits(&mut self) {
        self.used_traits.clear();
    }

    /// Returns a new instance with no used traits.
    #[inline]
    pub fn without_used_traits(mut self) -> Self {
        self.unset_used_traits();
        self
    }

    /// Sets the trait alias map, replacing the existing one.
    #[inline]
    pub fn set_trait_alias_map(&mut self, map: HashMap<StringIdentifier, StringIdentifier>) {
        self.trait_alias_map = map;
    }

    /// Returns a new instance with the trait alias map replaced.
    #[inline]
    pub fn with_trait_alias_map(mut self, map: HashMap<StringIdentifier, StringIdentifier>) -> Self {
        self.set_trait_alias_map(map);
        self
    }

    /// Adds or updates a single trait alias. Returns the previous original name if one existed for the alias.
    #[inline]
    pub fn add_trait_alias(&mut self, method: StringIdentifier, alias: StringIdentifier) -> Option<StringIdentifier> {
        self.trait_alias_map.insert(method, alias)
    }

    /// Returns a new instance with the trait alias added or updated.
    #[inline]
    pub fn with_added_trait_alias(mut self, method: StringIdentifier, alias: StringIdentifier) -> Self {
        self.add_trait_alias(method, alias);
        self
    }

    /// Clears the trait alias map.
    #[inline]
    pub fn unset_trait_alias_map(&mut self) {
        self.trait_alias_map.clear();
    }

    /// Returns a new instance with an empty trait alias map.
    #[inline]
    pub fn without_trait_alias_map(mut self) -> Self {
        self.unset_trait_alias_map();
        self
    }

    /// Sets the trait visibility map, replacing the existing one.
    #[inline]
    pub fn set_trait_visibility_map(&mut self, map: HashMap<StringIdentifier, Visibility>) {
        self.trait_visibility_map = map;
    }

    /// Returns a new instance with the trait visibility map replaced.
    #[inline]
    pub fn with_trait_visibility_map(mut self, map: HashMap<StringIdentifier, Visibility>) -> Self {
        self.set_trait_visibility_map(map);
        self
    }

    /// Adds or updates a single trait visibility override. Returns the previous visibility if one existed.
    #[inline]
    pub fn add_trait_visibility(&mut self, method: StringIdentifier, visibility: Visibility) -> Option<Visibility> {
        self.trait_visibility_map.insert(method, visibility)
    }

    /// Returns a new instance with the trait visibility override added or updated.
    #[inline]
    pub fn with_added_trait_visibility(mut self, method: StringIdentifier, visibility: Visibility) -> Self {
        self.add_trait_visibility(method, visibility);
        self
    }

    /// Clears the trait visibility map.
    #[inline]
    pub fn unset_trait_visibility_map(&mut self) {
        self.trait_visibility_map.clear();
    }

    /// Returns a new instance with an empty trait visibility map.
    #[inline]
    pub fn without_trait_visibility_map(mut self) -> Self {
        self.unset_trait_visibility_map();
        self
    }

    /// Sets the final trait methods map, replacing existing ones.
    #[inline]
    pub fn set_trait_final_map(&mut self, map: impl IntoIterator<Item = StringIdentifier>) {
        self.trait_final_map = map.into_iter().collect();
    }

    /// Returns a new instance with the final trait methods map replaced.
    #[inline]
    pub fn with_trait_final_map(mut self, map: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_trait_final_map(map);
        self
    }

    /// Adds a single final trait method. Returns `true` if the method was not already present.
    #[inline]
    pub fn add_trait_final(&mut self, method: StringIdentifier) -> bool {
        self.trait_final_map.insert(method)
    }

    /// Returns a new instance with the final trait method added.
    #[inline]
    pub fn with_added_trait_final(mut self, method: StringIdentifier) -> Self {
        self.add_trait_final(method);
        self
    }

    /// Adds multiple final trait methods.
    #[inline]
    pub fn add_trait_finals(&mut self, methods: impl IntoIterator<Item = StringIdentifier>) {
        self.trait_final_map.extend(methods);
    }

    /// Returns a new instance with the final trait methods added.
    #[inline]
    pub fn with_added_trait_finals(mut self, methods: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_trait_finals(methods);
        self
    }

    /// Clears all final trait methods.
    #[inline]
    pub fn unset_trait_final_map(&mut self) {
        self.trait_final_map.clear();
    }

    /// Returns a new instance with no final trait methods.
    #[inline]
    pub fn without_trait_final_map(mut self) -> Self {
        self.unset_trait_final_map();
        self
    }

    /// Sets the set of child classlikes.
    #[inline]
    pub fn set_child_class_likes(&mut self, children: Option<HashSet<StringIdentifier>>) {
        self.child_class_likes = children;
    }

    /// Returns a new instance with the set of child classlikes set.
    #[inline]
    pub fn with_child_class_likes(mut self, children: Option<HashSet<StringIdentifier>>) -> Self {
        self.set_child_class_likes(children);
        self
    }

    /// Adds a single child class-like. Initializes the set if it was `None`. Returns `true` if added/initialized.
    #[inline]
    pub fn add_child_class_like(&mut self, child: StringIdentifier) -> bool {
        self.child_class_likes.get_or_insert_with(|| HashSet::with_hasher(RandomState::new())).insert(child)
    }

    /// Returns a new instance with the child classlike added. Initializes the set if it was `None`.
    #[inline]
    pub fn with_added_child_class_like(mut self, child: StringIdentifier) -> Self {
        self.add_child_class_like(child);
        self
    }

    /// Adds multiple child classlikes. Initializes the set if it was `None`.
    #[inline]
    pub fn add_child_class_likes(&mut self, children: impl IntoIterator<Item = StringIdentifier>) {
        self.child_class_likes.get_or_insert_with(|| HashSet::with_hasher(RandomState::new())).extend(children)
    }

    /// Returns a new instance with the child classlikes added. Initializes the set if it was `None`.
    #[inline]
    pub fn with_added_child_class_likes(mut self, children: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_child_class_likes(children);
        self
    }

    /// Sets the child classlikes to `None`.
    #[inline]
    pub fn unset_child_class_likes(&mut self) {
        self.child_class_likes = None;
    }

    /// Returns a new instance with the child classlikes set to `None`.
    #[inline]
    pub fn without_child_class_likes(mut self) -> Self {
        self.unset_child_class_likes();
        self
    }

    /// Sets the span for the class-like name identifier.
    #[inline]
    pub fn set_name_span(&mut self, name_span: Option<Span>) {
        self.name_span = name_span;
    }

    /// Returns a new instance with the name span set.
    #[inline]
    pub fn with_name_span(mut self, name_span: Option<Span>) -> Self {
        self.set_name_span(name_span);
        self
    }

    /// Sets the name span to `None`.
    #[inline]
    pub fn unset_name_span(&mut self) {
        self.name_span = None;
    }

    /// Returns a new instance with the name span set to `None`.
    #[inline]
    pub fn without_name_span(mut self) -> Self {
        self.unset_name_span();
        self
    }

    /// Sets the kind of class-like (Class, Interface, Trait, Enum). Use with caution after creation.
    #[inline]
    pub fn set_kind(&mut self, kind: SymbolKind) {
        self.kind = kind;
    }

    /// Returns a new instance with the kind set. Use with caution after creation.
    #[inline]
    pub fn with_kind(mut self, kind: SymbolKind) -> Self {
        self.set_kind(kind);
        self
    }

    /// Sets the template types, replacing existing ones.
    #[inline]
    pub fn set_template_types(&mut self, templates: impl IntoIterator<Item = TemplateTuple>) {
        self.template_types = templates.into_iter().collect();
    }

    /// Returns a new instance with the template types replaced.
    #[inline]
    pub fn with_template_types(mut self, templates: impl IntoIterator<Item = TemplateTuple>) -> Self {
        self.set_template_types(templates);
        self
    }

    /// Adds a single template type definition.
    #[inline]
    pub fn add_template_type(&mut self, template: TemplateTuple) {
        self.template_types.push(template);
    }

    /// Returns a new instance with the template type added.
    #[inline]
    pub fn with_added_template_type(mut self, template: TemplateTuple) -> Self {
        self.add_template_type(template);
        self
    }

    /// Adds multiple template type definitions.
    #[inline]
    pub fn add_template_types(&mut self, templates: impl IntoIterator<Item = TemplateTuple>) {
        self.template_types.extend(templates);
    }

    /// Returns a new instance with the template types added.
    #[inline]
    pub fn with_added_template_types(mut self, templates: impl IntoIterator<Item = TemplateTuple>) -> Self {
        self.add_template_types(templates);
        self
    }

    /// Clears all template types.
    #[inline]
    pub fn unset_template_types(&mut self) {
        self.template_types.clear();
    }

    /// Returns a new instance with no template types.
    #[inline]
    pub fn without_template_types(mut self) -> Self {
        self.unset_template_types();
        self
    }

    /// Sets the readonly template parameters, replacing existing ones.
    #[inline]
    pub fn set_template_readonly(&mut self, readonly_templates: impl IntoIterator<Item = StringIdentifier>) {
        self.template_readonly = readonly_templates.into_iter().collect();
    }

    /// Returns a new instance with the readonly template parameters replaced.
    #[inline]
    pub fn with_template_readonly(mut self, readonly_templates: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_template_readonly(readonly_templates);
        self
    }

    /// Adds a single readonly template parameter. Returns `true` if the parameter was not already present.
    #[inline]
    pub fn add_template_readonly(&mut self, template_name: StringIdentifier) -> bool {
        self.template_readonly.insert(template_name)
    }

    /// Returns a new instance with the readonly template parameter added.
    #[inline]
    pub fn with_added_template_readonly(mut self, template_name: StringIdentifier) -> Self {
        self.add_template_readonly(template_name);
        self
    }

    /// Adds multiple readonly template parameters.
    #[inline]
    pub fn add_template_readonlies(&mut self, template_names: impl IntoIterator<Item = StringIdentifier>) {
        self.template_readonly.extend(template_names);
    }

    /// Returns a new instance with the readonly template parameters added.
    #[inline]
    pub fn with_added_template_readonlies(
        mut self,
        template_names: impl IntoIterator<Item = StringIdentifier>,
    ) -> Self {
        self.add_template_readonlies(template_names);
        self
    }

    /// Clears all readonly template parameters.
    #[inline]
    pub fn unset_template_readonly(&mut self) {
        self.template_readonly.clear();
    }

    /// Returns a new instance with no readonly template parameters.
    #[inline]
    pub fn without_template_readonly(mut self) -> Self {
        self.unset_template_readonly();
        self
    }

    /// Sets the generic variance map, replacing the existing one.
    #[inline]
    pub fn set_generic_variance(&mut self, map: HashMap<usize, Variance>) {
        self.template_variance = map;
    }

    /// Returns a new instance with the generic variance map replaced.
    #[inline]
    pub fn with_generic_variance(mut self, map: HashMap<usize, Variance>) -> Self {
        self.set_generic_variance(map);
        self
    }

    /// Adds or updates the variance for a specific parameter index. Returns the previous variance if one existed.
    #[inline]
    pub fn add_template_variance_parameter(&mut self, index: usize, variance: Variance) -> Option<Variance> {
        self.template_variance.insert(index, variance)
    }

    /// Returns a new instance with the variance for the parameter index added or updated.
    #[inline]
    pub fn with_added_template_variance_parameter(mut self, index: usize, variance: Variance) -> Self {
        self.add_template_variance_parameter(index, variance);
        self
    }

    /// Sets the template extended offsets map, replacing the existing one.
    #[inline]
    pub fn set_template_extended_offsets(&mut self, map: HashMap<StringIdentifier, Vec<TUnion>>) {
        self.template_extended_offsets = map;
    }

    /// Returns a new instance with the template extended offsets map replaced.
    #[inline]
    pub fn with_template_extended_offsets(mut self, map: HashMap<StringIdentifier, Vec<TUnion>>) -> Self {
        self.set_template_extended_offsets(map);
        self
    }

    /// Adds or replaces the offset types for a specific template parameter name.
    #[inline]
    pub fn add_template_extended_offset(&mut self, name: StringIdentifier, types: Vec<TUnion>) -> Option<Vec<TUnion>> {
        self.template_extended_offsets.insert(name, types)
    }

    /// Returns a new instance with the offset types for the template parameter name added or updated.
    #[inline]
    pub fn with_added_template_extended_offset(mut self, name: StringIdentifier, types: Vec<TUnion>) -> Self {
        self.add_template_extended_offset(name, types);
        self
    }

    /// Clears the template extended offsets map.
    #[inline]
    pub fn unset_template_extended_offsets(&mut self) {
        self.template_extended_offsets.clear();
    }

    /// Returns a new instance with an empty template extended offsets map.
    #[inline]
    pub fn without_template_extended_offsets(mut self) -> Self {
        self.unset_template_extended_offsets();
        self
    }

    /// Sets the template extended parameters map, replacing the existing one.
    #[inline]
    pub fn set_template_extended_parameters(
        &mut self,
        map: HashMap<StringIdentifier, IndexMap<StringIdentifier, TUnion, RandomState>>,
    ) {
        self.template_extended_parameters = map;
    }

    /// Returns a new instance with the template extended parameters map replaced.
    #[inline]
    pub fn with_template_extended_parameters(
        mut self,
        map: HashMap<StringIdentifier, IndexMap<StringIdentifier, TUnion, RandomState>>,
    ) -> Self {
        self.set_template_extended_parameters(map);
        self
    }

    /// Adds or replaces the resolved parameters for a specific parent FQCN.
    #[inline]
    pub fn add_template_extended_parameter_map(
        &mut self,
        parent_fqcn: StringIdentifier,
        parameters: IndexMap<StringIdentifier, TUnion, RandomState>,
    ) -> Option<IndexMap<StringIdentifier, TUnion, RandomState>> {
        self.template_extended_parameters.insert(parent_fqcn, parameters)
    }

    /// Adds or replaces the resolved parameters for a specific parent FQCN.
    #[inline]
    pub fn extend_template_extended_parameters(
        &mut self,
        template_extended_parameters: HashMap<StringIdentifier, IndexMap<StringIdentifier, TUnion, RandomState>>,
    ) {
        self.template_extended_parameters.extend(template_extended_parameters);
    }

    /// Returns a new instance with the resolved parameters for the parent FQCN added or updated.
    #[inline]
    pub fn with_added_template_extended_parameter_map(
        mut self,
        parent_fqcn: StringIdentifier,
        parameters: IndexMap<StringIdentifier, TUnion, RandomState>,
    ) -> Self {
        self.add_template_extended_parameter_map(parent_fqcn, parameters);
        self
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

    /// Returns a new instance with a single resolved parameter added or updated for the parent FQCN.
    #[inline]
    pub fn with_added_template_extended_parameter(
        mut self,
        parent_fqcn: StringIdentifier,
        parameter_name: StringIdentifier,
        parameter_type: TUnion,
    ) -> Self {
        self.add_template_extended_parameter(parent_fqcn, parameter_name, parameter_type);
        self
    }

    /// Clears the template extended parameters map.
    #[inline]
    pub fn unset_template_extended_parameters(&mut self) {
        self.template_extended_parameters.clear();
    }

    /// Returns a new instance with an empty template extended parameters map.
    #[inline]
    pub fn without_template_extended_parameters(mut self) -> Self {
        self.unset_template_extended_parameters();
        self
    }

    /// Sets the directly defined methods, replacing existing ones.
    #[inline]
    pub fn set_methods(&mut self, methods: impl IntoIterator<Item = StringIdentifier>) {
        self.methods = methods.into_iter().collect();
    }

    /// Returns a new instance with the methods replaced.
    #[inline]
    pub fn with_methods(mut self, methods: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_methods(methods);
        self
    }

    /// Adds a single method name.
    #[inline]
    pub fn add_method(&mut self, method: StringIdentifier) {
        self.methods.push(method);
    }

    /// Returns a new instance with the method name added.
    #[inline]
    pub fn with_added_method(mut self, method: StringIdentifier) -> Self {
        self.add_method(method);
        self
    }

    /// Adds multiple method names.
    #[inline]
    pub fn add_methods(&mut self, methods: impl IntoIterator<Item = StringIdentifier>) {
        self.methods.extend(methods);
    }

    /// Returns a new instance with the method names added.
    #[inline]
    pub fn with_added_methods(mut self, methods: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_methods(methods);
        self
    }

    /// Clears all directly defined methods.
    #[inline]
    pub fn unset_methods(&mut self) {
        self.methods.clear();
    }

    /// Returns a new instance with no directly defined methods.
    #[inline]
    pub fn without_methods(mut self) -> Self {
        self.unset_methods();
        self
    }

    /// Sets the declaring method IDs map, replacing the existing one.
    #[inline]
    pub fn set_declaring_method_ids(&mut self, map: HashMap<StringIdentifier, StringIdentifier>) {
        self.unset_declaring_method_ids();
        for (method, declaring_fqcn) in map {
            self.add_declaring_method_id(method, declaring_fqcn);
        }
    }

    /// Returns a new instance with the declaring method IDs map replaced.
    #[inline]
    pub fn with_declaring_method_ids(mut self, map: HashMap<StringIdentifier, StringIdentifier>) -> Self {
        self.set_declaring_method_ids(map);
        self
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

    /// Returns a new instance with the declaring class FQCN for the method added or updated.
    #[inline]
    pub fn with_added_declaring_method_id(
        mut self,
        method: StringIdentifier,
        declaring_fqcn: StringIdentifier,
    ) -> Self {
        self.add_declaring_method_id(method, declaring_fqcn);
        self
    }

    /// Clears the declaring method IDs map.
    #[inline]
    pub fn unset_declaring_method_ids(&mut self) {
        for (metod, _) in self.declaring_method_ids.drain() {
            self.appearing_method_ids.remove(&metod);
        }
    }

    /// Returns a new instance with an empty declaring method IDs map.
    #[inline]
    pub fn without_declaring_method_ids(mut self) -> Self {
        self.unset_declaring_method_ids();
        self
    }

    /// Sets the appearing method IDs map, replacing the existing one.
    #[inline]
    pub fn set_appearing_method_ids(&mut self, map: HashMap<StringIdentifier, StringIdentifier>) {
        self.appearing_method_ids = map;
    }

    /// Returns a new instance with the appearing method IDs map replaced.
    #[inline]
    pub fn with_appearing_method_ids(mut self, map: HashMap<StringIdentifier, StringIdentifier>) -> Self {
        self.set_appearing_method_ids(map);
        self
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

    /// Returns a new instance with the appearing class FQCN for the method added or updated.
    #[inline]
    pub fn with_added_appearing_method_id(
        mut self,
        method: StringIdentifier,
        appearing_fqcn: StringIdentifier,
    ) -> Self {
        self.add_appearing_method_id(method, appearing_fqcn);
        self
    }

    /// Clears the appearing method IDs map.
    #[inline]
    pub fn unset_appearing_method_ids(&mut self) {
        self.appearing_method_ids.clear();
    }

    /// Returns a new instance with an empty appearing method IDs map.
    #[inline]
    pub fn without_appearing_method_ids(mut self) -> Self {
        self.unset_appearing_method_ids();
        self
    }

    /// Sets the overridden method IDs map, replacing the existing one.
    #[inline]
    pub fn set_overridden_method_ids(&mut self, map: HashMap<StringIdentifier, HashSet<StringIdentifier>>) {
        self.overridden_method_ids = map;
    }

    /// Returns a new instance with the overridden method IDs map replaced.
    #[inline]
    pub fn with_overridden_method_ids(mut self, map: HashMap<StringIdentifier, HashSet<StringIdentifier>>) -> Self {
        self.set_overridden_method_ids(map);
        self
    }

    /// Adds a parent FQCN to the set for an overridden method. Initializes set if needed. Returns `true` if added.
    #[inline]
    pub fn add_overridden_method_parent(&mut self, method: StringIdentifier, parent_fqcn: StringIdentifier) -> bool {
        self.overridden_method_ids.entry(method).or_default().insert(parent_fqcn)
    }

    /// Returns a new instance with the parent FQCN added for the overridden method.
    #[inline]
    pub fn with_added_overridden_method_parent(
        mut self,
        method: StringIdentifier,
        parent_fqcn: StringIdentifier,
    ) -> Self {
        self.add_overridden_method_parent(method, parent_fqcn);
        self
    }

    /// Adds multiple parent FQCNs for an overridden method.
    #[inline]
    pub fn add_overridden_method_parents(
        &mut self,
        method: StringIdentifier,
        parents: impl IntoIterator<Item = StringIdentifier>,
    ) {
        self.overridden_method_ids.entry(method).or_default().extend(parents);
    }

    /// Returns a new instance with multiple parent FQCNs added for the overridden method.
    #[inline]
    pub fn with_added_overridden_method_parents(
        mut self,
        method: StringIdentifier,
        parents: impl IntoIterator<Item = StringIdentifier>,
    ) -> Self {
        self.add_overridden_method_parents(method, parents);
        self
    }

    /// Clears the overridden method IDs map.
    #[inline]
    pub fn unset_overridden_method_ids(&mut self) {
        self.overridden_method_ids.clear();
    }

    /// Returns a new instance with an empty overridden method IDs map.
    #[inline]
    pub fn without_overridden_method_ids(mut self) -> Self {
        self.unset_overridden_method_ids();
        self
    }

    /// Sets the inheritable method IDs map, replacing the existing one.
    #[inline]
    pub fn set_inheritable_method_ids(&mut self, map: HashMap<StringIdentifier, StringIdentifier>) {
        self.inheritable_method_ids = map;
    }

    /// Returns a new instance with the inheritable method IDs map replaced.
    #[inline]
    pub fn with_inheritable_method_ids(mut self, map: HashMap<StringIdentifier, StringIdentifier>) -> Self {
        self.set_inheritable_method_ids(map);
        self
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

    /// Returns a new instance with the inheriting source FQCN for the method added or updated.
    #[inline]
    pub fn with_added_inheritable_method_id(mut self, method: StringIdentifier, source_fqcn: StringIdentifier) -> Self {
        self.add_inheritable_method_id(method, source_fqcn);
        self
    }

    /// Clears the inheritable method IDs map.
    #[inline]
    pub fn unset_inheritable_method_ids(&mut self) {
        self.inheritable_method_ids.clear();
    }

    /// Returns a new instance with an empty inheritable method IDs map.
    #[inline]
    pub fn without_inheritable_method_ids(mut self) -> Self {
        self.unset_inheritable_method_ids();
        self
    }

    /// Sets the potential declaring method IDs map, replacing the existing one.
    #[inline]
    pub fn set_potential_declaring_method_ids(&mut self, map: HashMap<StringIdentifier, HashSet<StringIdentifier>>) {
        self.potential_declaring_method_ids = map;
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
    pub fn add_potential_declaring_method_class_name(
        &mut self,
        method: StringIdentifier,
        potentially_declaring_fqcn: StringIdentifier,
    ) {
        self.potential_declaring_method_ids.entry(method).or_default().insert(potentially_declaring_fqcn);
    }

    /// Returns a new instance with the potential declaring method IDs map replaced.
    #[inline]
    pub fn with_potential_declaring_method_ids(
        mut self,
        map: HashMap<StringIdentifier, HashSet<StringIdentifier>>,
    ) -> Self {
        self.set_potential_declaring_method_ids(map);
        self
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

    /// Returns a new instance with the potential declaring FQCN added for the method.
    #[inline]
    pub fn with_added_potential_declaring_method(
        mut self,
        method: StringIdentifier,
        potential_fqcn: StringIdentifier,
    ) -> Self {
        self.add_potential_declaring_method(method, potential_fqcn);
        self
    }

    /// Adds multiple potential declaring FQCNs for a method.
    #[inline]
    pub fn add_potential_declaring_methods(
        &mut self,
        method: StringIdentifier,
        potentials: impl IntoIterator<Item = StringIdentifier>,
    ) {
        self.potential_declaring_method_ids.entry(method).or_default().extend(potentials);
    }

    /// Returns a new instance with multiple potential declaring FQCNs added for the method.
    #[inline]
    pub fn with_added_potential_declaring_methods(
        mut self,
        method: StringIdentifier,
        potentials: impl IntoIterator<Item = StringIdentifier>,
    ) -> Self {
        self.add_potential_declaring_methods(method, potentials);
        self
    }

    /// Clears the potential declaring method IDs map.
    #[inline]
    pub fn unset_potential_declaring_method_ids(&mut self) {
        self.potential_declaring_method_ids.clear();
    }

    /// Returns a new instance with an empty potential declaring method IDs map.
    #[inline]
    pub fn without_potential_declaring_method_ids(mut self) -> Self {
        self.unset_potential_declaring_method_ids();
        self
    }

    /// Sets the properties map, replacing the existing one.
    #[inline]
    pub fn set_properties(&mut self, map: HashMap<StringIdentifier, PropertyMetadata>) {
        self.unset_properties();
        for (name, prop) in map {
            self.add_property(name, prop);
        }
    }

    /// Returns a new instance with the properties map replaced.
    #[inline]
    pub fn with_properties(mut self, map: HashMap<StringIdentifier, PropertyMetadata>) -> Self {
        self.set_properties(map);
        self
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
            self.add_initialized_property(name);
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

    /// Returns a new instance with the property added or updated.
    #[inline]
    pub fn with_added_property(mut self, name: StringIdentifier, property_meta: PropertyMetadata) -> Self {
        self.add_property(name, property_meta);
        self
    }

    /// Clears the properties map.
    #[inline]
    pub fn unset_properties(&mut self) {
        for (prop_name, prop) in self.properties.drain() {
            if prop.has_default() {
                self.initialized_properties.retain(|p| p != &prop_name);
            }
        }
    }

    /// Returns a new instance with an empty properties map.
    #[inline]
    pub fn without_properties(mut self) -> Self {
        self.unset_properties();
        self
    }

    /// Sets the appearing property IDs map, replacing the existing one.
    #[inline]
    pub fn set_appearing_property_ids(&mut self, map: HashMap<StringIdentifier, StringIdentifier>) {
        self.appearing_property_ids = map;
    }

    /// Returns a new instance with the appearing property IDs map replaced.
    #[inline]
    pub fn with_appearing_property_ids(mut self, map: HashMap<StringIdentifier, StringIdentifier>) -> Self {
        self.set_appearing_property_ids(map);
        self
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

    /// Returns a new instance with the appearing class FQCN for the property added or updated.
    #[inline]
    pub fn with_added_appearing_property_id(
        mut self,
        prop: StringIdentifier,
        appearing_fqcn: StringIdentifier,
    ) -> Self {
        self.add_appearing_property_id(prop, appearing_fqcn);
        self
    }

    /// Clears the appearing property IDs map.
    #[inline]
    pub fn unset_appearing_property_ids(&mut self) {
        self.appearing_property_ids.clear();
    }

    /// Returns a new instance with an empty appearing property IDs map.
    #[inline]
    pub fn without_appearing_property_ids(mut self) -> Self {
        self.unset_appearing_property_ids();
        self
    }

    /// Sets the declaring property IDs map, replacing the existing one.
    #[inline]
    pub fn set_declaring_property_ids(&mut self, map: HashMap<StringIdentifier, StringIdentifier>) {
        self.unset_declaring_property_ids();
        for (prop, declaring_fqcn) in map {
            self.add_declaring_property_id(prop, declaring_fqcn);
        }
    }

    /// Returns a new instance with the declaring property IDs map replaced.
    #[inline]
    pub fn with_declaring_property_ids(mut self, map: HashMap<StringIdentifier, StringIdentifier>) -> Self {
        self.set_declaring_property_ids(map);
        self
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

    /// Returns a new instance with the declaring class FQCN for the property added or updated.
    #[inline]
    pub fn with_added_declaring_property_id(
        mut self,
        prop: StringIdentifier,
        declaring_fqcn: StringIdentifier,
    ) -> Self {
        self.add_declaring_property_id(prop, declaring_fqcn);
        self
    }

    /// Clears the declaring property IDs map.
    #[inline]
    pub fn unset_declaring_property_ids(&mut self) {
        for (metod, _) in self.declaring_property_ids.drain() {
            self.appearing_property_ids.remove(&metod);
        }
    }

    /// Returns a new instance with an empty declaring property IDs map.
    #[inline]
    pub fn without_declaring_property_ids(mut self) -> Self {
        self.unset_declaring_property_ids();
        self
    }

    /// Sets the inheritable property IDs map, replacing the existing one.
    #[inline]
    pub fn set_inheritable_property_ids(&mut self, map: HashMap<StringIdentifier, StringIdentifier>) {
        self.inheritable_property_ids = map;
    }

    /// Returns a new instance with the inheritable property IDs map replaced.
    #[inline]
    pub fn with_inheritable_property_ids(mut self, map: HashMap<StringIdentifier, StringIdentifier>) -> Self {
        self.set_inheritable_property_ids(map);
        self
    }

    /// Adds or updates the inheriting source FQCN for a property name.
    #[inline]
    pub fn add_inheritable_property_id(
        &mut self,
        prop: StringIdentifier,
        source_fqcn: StringIdentifier,
    ) -> Option<StringIdentifier> {
        self.inheritable_property_ids.insert(prop, source_fqcn)
    }

    /// Returns a new instance with the inheriting source FQCN for the property added or updated.
    #[inline]
    pub fn with_added_inheritable_property_id(mut self, prop: StringIdentifier, source_fqcn: StringIdentifier) -> Self {
        self.add_inheritable_property_id(prop, source_fqcn);
        self
    }

    /// Clears the inheritable property IDs map.
    #[inline]
    pub fn unset_inheritable_property_ids(&mut self) {
        self.inheritable_property_ids.clear();
    }

    /// Returns a new instance with an empty inheritable property IDs map.
    #[inline]
    pub fn without_inheritable_property_ids(mut self) -> Self {
        self.unset_inheritable_property_ids();
        self
    }

    /// Sets the overridden property IDs map, replacing the existing one.
    #[inline]
    pub fn set_overridden_property_ids(&mut self, map: HashMap<StringIdentifier, Vec<StringIdentifier>>) {
        self.overridden_property_ids = map;
    }

    /// Returns a new instance with the overridden property IDs map replaced.
    #[inline]
    pub fn with_overridden_property_ids(mut self, map: HashMap<StringIdentifier, Vec<StringIdentifier>>) -> Self {
        self.set_overridden_property_ids(map);
        self
    }

    /// Adds a parent FQCN to the list for an overridden property. Initializes list if needed.
    #[inline]
    pub fn add_overridden_property_parent(&mut self, prop: StringIdentifier, parent_fqcn: StringIdentifier) {
        self.overridden_property_ids.entry(prop).or_default().push(parent_fqcn)
    }

    /// Returns a new instance with the parent FQCN added for the overridden property.
    #[inline]
    pub fn with_added_overridden_property_parent(
        mut self,
        prop: StringIdentifier,
        parent_fqcn: StringIdentifier,
    ) -> Self {
        self.add_overridden_property_parent(prop, parent_fqcn);
        self
    }

    /// Adds multiple parent FQCNs for an overridden property.
    #[inline]
    pub fn add_overridden_property_parents(
        &mut self,
        prop: StringIdentifier,
        parents: impl IntoIterator<Item = StringIdentifier>,
    ) {
        self.overridden_property_ids.entry(prop).or_default().extend(parents);
    }

    /// Returns a new instance with multiple parent FQCNs added for the overridden property.
    #[inline]
    pub fn with_added_overridden_property_parents(
        mut self,
        prop: StringIdentifier,
        parents: impl IntoIterator<Item = StringIdentifier>,
    ) -> Self {
        self.add_overridden_property_parents(prop, parents);
        self
    }

    /// Clears the overridden property IDs map.
    #[inline]
    pub fn unset_overridden_property_ids(&mut self) {
        self.overridden_property_ids.clear();
    }

    /// Returns a new instance with an empty overridden property IDs map.
    #[inline]
    pub fn without_overridden_property_ids(mut self) -> Self {
        self.unset_overridden_property_ids();
        self
    }

    /// Sets the list of properties initialized with a default value, replacing existing ones.
    #[inline]
    pub fn set_initialized_properties(&mut self, props: impl IntoIterator<Item = StringIdentifier>) {
        self.initialized_properties = props.into_iter().collect();
    }

    /// Returns a new instance with the initialized properties replaced.
    #[inline]
    pub fn with_initialized_properties(mut self, props: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_initialized_properties(props);
        self
    }

    /// Adds a single initialized property.
    #[inline]
    pub fn add_initialized_property(&mut self, prop: StringIdentifier) {
        self.initialized_properties.push(prop);
    }

    /// Returns a new instance with the initialized property added.
    #[inline]
    pub fn with_added_initialized_property(mut self, prop: StringIdentifier) -> Self {
        self.add_initialized_property(prop);
        self
    }

    /// Adds multiple initialized properties.
    #[inline]
    pub fn add_initialized_properties(&mut self, props: impl IntoIterator<Item = StringIdentifier>) {
        self.initialized_properties.extend(props);
    }

    /// Returns a new instance with the initialized properties added.
    #[inline]
    pub fn with_added_initialized_properties(mut self, props: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_initialized_properties(props);
        self
    }

    /// Clears all initialized properties.
    #[inline]
    pub fn unset_initialized_properties(&mut self) {
        self.initialized_properties.clear();
    }

    /// Returns a new instance with no initialized properties.
    #[inline]
    pub fn without_initialized_properties(mut self) -> Self {
        self.unset_initialized_properties();
        self
    }

    /// Sets the class constants, replacing existing ones.
    #[inline]
    pub fn set_constants(&mut self, constants: IndexMap<StringIdentifier, ClassLikeConstantMetadata, RandomState>) {
        self.constants = constants;
    }

    /// Returns a new instance with the class constants replaced.
    #[inline]
    pub fn with_constants(
        mut self,
        constants: IndexMap<StringIdentifier, ClassLikeConstantMetadata, RandomState>,
    ) -> Self {
        self.set_constants(constants);
        self
    }

    /// Adds or updates a single class constant. Returns the previous metadata if the constant existed.
    #[inline]
    pub fn add_constant(
        &mut self,
        name: StringIdentifier,
        constant: ClassLikeConstantMetadata,
    ) -> Option<ClassLikeConstantMetadata> {
        self.constants.insert(name, constant)
    }

    /// Returns a new instance with the class constant added or updated.
    #[inline]
    pub fn with_added_constant(mut self, name: StringIdentifier, constant: ClassLikeConstantMetadata) -> Self {
        self.add_constant(name, constant);
        self
    }

    /// Clears all class constants.
    #[inline]
    pub fn unset_constants(&mut self) {
        self.constants.clear();
    }

    /// Returns a new instance with no class constants.
    #[inline]
    pub fn without_constants(mut self) -> Self {
        self.unset_constants();
        self
    }

    /// Sets the enum cases, replacing existing ones.
    #[inline]
    pub fn set_enum_cases(&mut self, cases: IndexMap<StringIdentifier, EnumCaseMetadata, RandomState>) {
        self.enum_cases = cases;
    }

    /// Returns a new instance with the enum cases replaced.
    #[inline]
    pub fn with_enum_cases(mut self, cases: IndexMap<StringIdentifier, EnumCaseMetadata, RandomState>) -> Self {
        self.set_enum_cases(cases);
        self
    }

    /// Adds or updates a single enum case. Returns the previous metadata if the case existed.
    #[inline]
    pub fn add_enum_case(&mut self, name: StringIdentifier, case_meta: EnumCaseMetadata) -> Option<EnumCaseMetadata> {
        self.enum_cases.insert(name, case_meta)
    }

    /// Returns a new instance with the enum case added or updated.
    #[inline]
    pub fn with_added_enum_case(mut self, name: StringIdentifier, case_meta: EnumCaseMetadata) -> Self {
        self.add_enum_case(name, case_meta);
        self
    }

    /// Clears all enum cases.
    #[inline]
    pub fn unset_enum_cases(&mut self) {
        self.enum_cases.clear();
    }

    /// Returns a new instance with no enum cases.
    #[inline]
    pub fn without_enum_cases(mut self) -> Self {
        self.unset_enum_cases();
        self
    }

    /// Sets the list of invalid dependencies, replacing existing ones.
    #[inline]
    pub fn set_invalid_dependencies(&mut self, deps: impl IntoIterator<Item = StringIdentifier>) {
        self.invalid_dependencies = deps.into_iter().collect();
    }

    /// Returns a new instance with the invalid dependencies replaced.
    #[inline]
    pub fn with_invalid_dependencies(mut self, deps: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.set_invalid_dependencies(deps);
        self
    }

    /// Adds a single invalid dependency.
    #[inline]
    pub fn add_invalid_dependency(&mut self, dep: StringIdentifier) {
        self.invalid_dependencies.push(dep);
    }

    /// Returns a new instance with the invalid dependency added.
    #[inline]
    pub fn with_added_invalid_dependency(mut self, dep: StringIdentifier) -> Self {
        self.add_invalid_dependency(dep);
        self
    }

    /// Adds multiple invalid dependencies.
    #[inline]
    pub fn add_invalid_dependencies(&mut self, deps: impl IntoIterator<Item = StringIdentifier>) {
        self.invalid_dependencies.extend(deps);
    }

    /// Returns a new instance with the invalid dependencies added.
    #[inline]
    pub fn with_added_invalid_dependencies(mut self, deps: impl IntoIterator<Item = StringIdentifier>) -> Self {
        self.add_invalid_dependencies(deps);
        self
    }

    /// Clears all invalid dependencies.
    #[inline]
    pub fn unset_invalid_dependencies(&mut self) {
        self.invalid_dependencies.clear();
    }

    /// Returns a new instance with no invalid dependencies.
    #[inline]
    pub fn without_invalid_dependencies(mut self) -> Self {
        self.unset_invalid_dependencies();
        self
    }

    /// Sets the attributes, replacing existing ones.
    #[inline]
    pub fn set_attributes(&mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) {
        self.attributes = attributes.into_iter().collect();
    }

    /// Returns a new instance with the attributes replaced.
    #[inline]
    pub fn with_attributes(mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) -> Self {
        self.set_attributes(attributes);
        self
    }

    /// Adds a single attribute.
    #[inline]
    pub fn add_attribute(&mut self, attribute: AttributeMetadata) {
        self.attributes.push(attribute);
    }

    /// Returns a new instance with the attribute added.
    #[inline]
    pub fn with_added_attribute(mut self, attribute: AttributeMetadata) -> Self {
        self.add_attribute(attribute);
        self
    }

    /// Adds multiple attributes.
    #[inline]
    pub fn add_attributes(&mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) {
        self.attributes.extend(attributes);
    }

    /// Returns a new instance with the attributes added.
    #[inline]
    pub fn with_added_attributes(mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) -> Self {
        self.add_attributes(attributes);
        self
    }

    /// Clears all attributes.
    #[inline]
    pub fn unset_attributes(&mut self) {
        self.attributes.clear();
    }

    /// Returns a new instance with no attributes.
    #[inline]
    pub fn without_attributes(mut self) -> Self {
        self.unset_attributes();
        self
    }

    /// Sets the backing type for an enum.
    #[inline]
    pub fn set_enum_type(&mut self, enum_type: Option<TAtomic>) {
        self.enum_type = enum_type;
    }

    /// Returns a new instance with the enum backing type set.
    #[inline]
    pub fn with_enum_type(mut self, enum_type: Option<TAtomic>) -> Self {
        self.set_enum_type(enum_type);
        self
    }

    /// Sets the enum backing type to `None`.
    #[inline]
    pub fn unset_enum_type(&mut self) {
        self.enum_type = None;
    }

    /// Returns a new instance with the enum backing type set to `None`.
    #[inline]
    pub fn without_enum_type(mut self) -> Self {
        self.unset_enum_type();
        self
    }

    /// Sets the `@sealed-methods` status.
    #[inline]
    pub fn set_has_sealed_methods(&mut self, sealed: Option<bool>) {
        self.has_sealed_methods = sealed;
    }

    /// Returns a new instance with the `@sealed-methods` status set.
    #[inline]
    pub fn with_has_sealed_methods(mut self, sealed: Option<bool>) -> Self {
        self.set_has_sealed_methods(sealed);
        self
    }

    /// Sets the `@sealed-methods` status to `None`.
    #[inline]
    pub fn unset_has_sealed_methods(&mut self) {
        self.has_sealed_methods = None;
    }

    /// Returns a new instance with the `@sealed-methods` status set to `None`.
    #[inline]
    pub fn without_has_sealed_methods(mut self) -> Self {
        self.unset_has_sealed_methods();
        self
    }

    /// Sets the `@sealed-properties` status.
    #[inline]
    pub fn set_has_sealed_properties(&mut self, sealed: Option<bool>) {
        self.has_sealed_properties = sealed;
    }

    /// Returns a new instance with the `@sealed-properties` status set.
    #[inline]
    pub fn with_has_sealed_properties(mut self, sealed: Option<bool>) -> Self {
        self.set_has_sealed_properties(sealed);
        self
    }

    /// Sets the `@sealed-properties` status to `None`.
    #[inline]
    pub fn unset_has_sealed_properties(&mut self) {
        self.has_sealed_properties = None;
    }

    /// Returns a new instance with the `@sealed-properties` status set to `None`.
    #[inline]
    pub fn without_has_sealed_properties(mut self) -> Self {
        self.unset_has_sealed_properties();
        self
    }

    /// Sets the issues, replacing existing ones.
    #[inline]
    pub fn set_issues(&mut self, issues: impl IntoIterator<Item = Issue>) {
        self.issues = issues.into_iter().collect();
    }

    /// Returns a new instance with the issues replaced.
    #[inline]
    pub fn with_issues(mut self, issues: impl IntoIterator<Item = Issue>) -> Self {
        self.set_issues(issues);
        self
    }

    /// Adds a single issue.
    #[inline]
    pub fn add_issue(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    /// Returns a new instance with the issue added.
    #[inline]
    pub fn with_added_issue(mut self, issue: Issue) -> Self {
        self.add_issue(issue);
        self
    }

    /// Adds multiple issues.
    #[inline]
    pub fn add_issues(&mut self, issues: impl IntoIterator<Item = Issue>) {
        self.issues.extend(issues);
    }

    /// Returns a new instance with the issues added.
    #[inline]
    pub fn with_added_issues(mut self, issues: impl IntoIterator<Item = Issue>) -> Self {
        self.add_issues(issues);
        self
    }

    /// Clears all typing issues.
    #[inline]
    pub fn unset_issues(&mut self) {
        self.issues.clear();
    }

    /// Returns a new instance with no typing issues.
    #[inline]
    pub fn without_issues(mut self) -> Self {
        self.unset_issues();
        self
    }

    /// Get the attribute flags for this class, if it is an attribute.
    pub fn get_attribute_flags(&self) -> Option<AttributeFlags> {
        self.attribute_flags.as_ref().copied()
    }

    /// Set the attribute flags for this class, if it is an attribute.
    pub fn set_attribute_flags(&mut self, flags: Option<AttributeFlags>) {
        self.attribute_flags = flags;
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
