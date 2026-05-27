use foldhash::HashMap;
use foldhash::fast::RandomState;
use indexmap::IndexMap;
use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;

use crate::flags::attribute::AttributeFlags;
use crate::identifier::method::MethodIdentifier;
use crate::issue::ScanningIssueKind;
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

    /// Applies a patch to this class, refining member definitions while preserving everything else.
    ///
    /// Type annotations on existing members (methods via `CodebaseMetadata::extend()`,
    /// properties, constants, `@template` declarations, pseudo-methods, and type aliases)
    /// can be refined by a patch. Structural information (hierarchy, override chains,
    /// initialization state, enum cases) is never altered.
    ///
    /// Patches are not required to re-declare hierarchy. If a patch does declare a parent
    /// class, implemented interfaces, `@require-extends`, or `@require-implements`, those
    /// declarations must match the original exactly — a mismatch is a hard error indicating
    /// the patch targets a different class. `use` trait declarations are never valid in a
    /// patch and are always reported as an error.
    ///
    /// Diagnostics about the patch itself (mismatches, attempts to introduce new members)
    /// are returned to the caller rather than pushed onto `self.issues`. The patched class
    /// is vendor/built-in metadata; its `issues` list is reserved for diagnostics about
    /// that definition. Patch diagnostics belong to the patch and the caller routes them to
    /// the codebase-level patch-diagnostics collection.
    #[must_use = "patch diagnostics must be routed to CodebaseMetadata::patch_diagnostics"]
    pub fn apply_patch(&mut self, patch: ClassLikeMetadata, inherited_methods: &WordSet) -> Vec<Issue> {
        // Patches redeclare the same class by name, so their member identifiers
        // already carry the correct class name and merge cleanly.
        debug_assert_eq!(self.name, patch.name, "patch class name must match the patched class");

        let mut diagnostics: Vec<Issue> = Vec::new();

        if self.kind != patch.kind {
            diagnostics.push(
                Issue::error(format!(
                    "Patch declares `{}` as a {} but the original symbol is a {}; patch members are ignored.",
                    patch.original_name,
                    patch.kind.as_str(),
                    self.kind.as_str(),
                ))
                .with_code(ScanningIssueKind::PatchKindMismatch)
                .with_annotation(Annotation::primary(patch.span)),
            );
            return diagnostics;
        }

        // Hierarchy declarations must match the original exactly if declared; a mismatch
        // means the patch is describing a different class.
        let hierarchy_mismatch = (patch.direct_parent_class.is_some()
            && patch.direct_parent_class != self.direct_parent_class)
            || (!patch.direct_parent_interfaces.is_empty()
                && patch.direct_parent_interfaces != self.direct_parent_interfaces)
            || (!patch.require_extends.is_empty() && patch.require_extends != self.require_extends)
            || (!patch.require_implements.is_empty() && patch.require_implements != self.require_implements);

        if hierarchy_mismatch {
            diagnostics.push(
                Issue::error(format!(
                    "Patch for `{}` declares hierarchy that does not match the original; patch members are ignored.",
                    patch.original_name,
                ))
                .with_code(ScanningIssueKind::PatchHierarchyMismatch)
                .with_annotation(Annotation::primary(patch.span)),
            );
            return diagnostics;
        }

        // `readonly class` is a structural modifier — a patch cannot add or remove it.
        if patch.flags.contains(MetadataFlags::READONLY) != self.flags.contains(MetadataFlags::READONLY) {
            diagnostics.push(
                Issue::warning(format!(
                    "Patch declares `{}` as a {} class but the original is {}; readonly modifier is ignored.",
                    patch.original_name,
                    if patch.flags.contains(MetadataFlags::READONLY) { "readonly" } else { "non-readonly" },
                    if self.flags.contains(MetadataFlags::READONLY) { "readonly" } else { "non-readonly" },
                ))
                .with_code(ScanningIssueKind::PatchKindMismatch)
                .with_annotation(Annotation::primary(patch.span)),
            );
        }

        if !patch.used_traits.is_empty() {
            diagnostics.push(
                Issue::warning(format!(
                    "Patch for `{}` declares `use` traits; patches refine member type information only and trait usage declarations are ignored.",
                    patch.original_name,
                ))
                .with_code(ScanningIssueKind::PatchDeclaresTrait)
                .with_annotation(Annotation::primary(patch.span)),
            );
        }

        // Template types
        //
        // Patches can add @template declarations to un-annotated vendor code or refine existing
        // constraints. Existing templates are overridden by name; new names are appended.
        // template_variance is keyed by position in the template_types IndexMap and must be
        // rebuilt after the merge. template_readonly is name-keyed so it can be extended
        // directly without rebuilding.
        if !patch.template_types.is_empty() {
            // Collect name → variance for the current (original) state.
            let mut name_to_variance: HashMap<Word, Variance> = self
                .template_types
                .keys()
                .enumerate()
                .map(|(i, name)| (*name, self.template_variance.get(i).copied().unwrap_or(Variance::Invariant)))
                .collect();

            // Patch overrides existing entries and contributes new ones.
            name_to_variance.extend(
                patch
                    .template_types
                    .keys()
                    .enumerate()
                    .map(|(i, name)| (*name, patch.template_variance.get(i).copied().unwrap_or(Variance::Invariant))),
            );

            // Extend the IndexMap: existing names get updated definitions, new ones are appended.
            self.template_types.extend(patch.template_types);

            // Rebuild position-indexed variance vec to match the merged IndexMap order.
            // template_readonly is name-keyed so it doesn't need rebuilding — just extend it.
            self.template_variance = self
                .template_types
                .keys()
                .map(|name| name_to_variance.get(name).copied().unwrap_or(Variance::Invariant))
                .collect();
            self.template_readonly.extend(patch.template_readonly);
        }

        // Methods
        //
        // Real-method type info is patched in place by `FunctionLikeMetadata::apply_patch`,
        // called from `CodebaseMetadata::extend` when a patch function-like meets an Occupied
        // slot. Pseudo-methods (@method annotations) are purely class-level and always update
        // structural maps. For real methods: if already declared in the original, the type info
        // flows through function_likes and no structural change is needed here. If not declared
        // in the original (e.g. the method is inherited), the patch is introducing an override
        // for this class — add it to the structural maps so the populator sees it as a declared
        // override. If the method exists nowhere in the inheritance chain, it is a new method
        // — warn and ignore.
        for method_name in &patch.methods {
            if patch.pseudo_methods.contains(method_name) || patch.static_pseudo_methods.contains(method_name) {
                continue;
            }
            if inherited_methods.contains(method_name) {
                if self.methods.insert(*method_name) {
                    if let Some(id) = patch.inheritable_method_ids.get(method_name) {
                        self.inheritable_method_ids.insert(*method_name, *id);
                    }
                }
            } else if !self.methods.contains(method_name) {
                diagnostics.push(
                    Issue::error(format!(
                        "Patch for `{}` declares method `{}` which does not exist in the original \
                         or any of its ancestors; patches cannot introduce new methods.",
                        patch.original_name, method_name,
                    ))
                    .with_code(ScanningIssueKind::PatchIntroducesNewMethod)
                    .with_annotation(Annotation::primary(patch.span)),
                );
            }
        }

        for name in patch.pseudo_methods.iter().chain(patch.static_pseudo_methods.iter()) {
            if let Some(id) = patch.declaring_method_ids.get(name) {
                self.declaring_method_ids.insert(*name, *id);
            }
            if let Some(id) = patch.appearing_method_ids.get(name) {
                self.appearing_method_ids.insert(*name, *id);
            }
            if let Some(id) = patch.inheritable_method_ids.get(name) {
                self.inheritable_method_ids.insert(*name, *id);
            }
        }

        self.pseudo_methods.extend(patch.pseudo_methods);
        self.static_pseudo_methods.extend(patch.static_pseudo_methods);

        // Properties
        //
        // Patches can refine type annotations on existing properties. New properties are
        // rejected because they would assert a property exists at runtime when it does not.
        // Exception: magic properties (@property/@property-read/@property-write) are pure
        // type annotations for __get/__set magic and carry no runtime existence claim.
        // Initialization state and override chains are structural and must not be touched.
        for (name, prop_metadata) in patch.properties {
            if let Some(slot) = self.properties.get_mut(&name) {
                // Patches can only refine type annotations. Structural attributes
                // (visibility, modifiers, hooks) must match the original exactly;
                // if they differ the patch is wrong and we should say so rather than
                // silently discarding the mismatch.
                let visibility_mismatch = prop_metadata.read_visibility != slot.read_visibility
                    || prop_metadata.write_visibility != slot.write_visibility;
                // READONLY, STATIC, ABSTRACT: any mismatch is structural.
                // FINAL: only an error when removed (vendor has it, patch doesn't);
                //        adding final via a patch is allowed.
                let structural_flag_mismatch =
                    [MetadataFlags::READONLY, MetadataFlags::STATIC, MetadataFlags::ABSTRACT]
                        .iter()
                        .any(|&f| prop_metadata.flags.contains(f) != slot.flags.contains(f))
                        || (slot.flags.contains(MetadataFlags::FINAL)
                            && !prop_metadata.flags.contains(MetadataFlags::FINAL));
                let has_hooks = !prop_metadata.hooks.is_empty();

                if visibility_mismatch || structural_flag_mismatch || has_hooks {
                    diagnostics.push(
                        Issue::error(format!(
                            "Patch for `{}::{}` declares structural attributes (visibility, modifiers, \
                             or hooks) that differ from the original; only type annotations are applied.",
                            patch.original_name, name,
                        ))
                        .with_code(ScanningIssueKind::PatchPropertyStructuralMismatch)
                        .with_annotation(Annotation::primary(prop_metadata.span.unwrap_or(patch.span))),
                    );
                }

                slot.type_declaration_metadata = prop_metadata.type_declaration_metadata;
                slot.type_metadata = prop_metadata.type_metadata;
            } else if prop_metadata.flags.is_magic_property() {
                self.add_property(name, prop_metadata);
            } else {
                diagnostics.push(
                    Issue::error(format!(
                        "Patch declares property `{}::{}` which does not exist in the original; \
                         patches cannot introduce new properties.",
                        patch.original_name, name,
                    ))
                    .with_code(ScanningIssueKind::PatchIntroducesNewProperty)
                    .with_annotation(Annotation::primary(patch.span)),
                );
            }
        }

        // Constants
        //
        // Same rule as properties: only existing constants can have their type annotations
        // refined; new constants are rejected.
        for (name, const_metadata) in patch.constants {
            if let Some(slot) = self.constants.get_mut(&name) {
                let visibility_mismatch = const_metadata.visibility != slot.visibility;
                // ABSTRACT: any mismatch is structural.
                // FINAL: only an error when removed (vendor has it, patch doesn't);
                //        adding final via a patch is allowed.
                let structural_flag_mismatch = const_metadata.flags.contains(MetadataFlags::ABSTRACT)
                    != slot.flags.contains(MetadataFlags::ABSTRACT)
                    || (slot.flags.contains(MetadataFlags::FINAL)
                        && !const_metadata.flags.contains(MetadataFlags::FINAL));

                if visibility_mismatch || structural_flag_mismatch {
                    diagnostics.push(
                        Issue::error(format!(
                            "Patch for `{}::{}` declares structural attributes (visibility or modifiers) \
                             that differ from the original; only type annotations are applied.",
                            patch.original_name, name,
                        ))
                        .with_code(ScanningIssueKind::PatchConstantStructuralMismatch)
                        .with_annotation(Annotation::primary(const_metadata.span)),
                    );
                }

                slot.type_declaration = const_metadata.type_declaration;
                slot.type_metadata = const_metadata.type_metadata;
            } else {
                diagnostics.push(
                    Issue::error(format!(
                        "Patch declares constant `{}::{}` which does not exist in the original; \
                         patches cannot introduce new constants.",
                        patch.original_name, name,
                    ))
                    .with_code(ScanningIssueKind::PatchIntroducesNewConstant)
                    .with_annotation(Annotation::primary(patch.span)),
                );
            }
        }

        // Enum cases are structural (they define the valid runtime values of an enum)
        // and cannot be modified by a patch.
        if !patch.enum_cases.is_empty() {
            diagnostics.push(
                Issue::error(format!(
                    "Patch for `{}` declares enum case(s); enum cases are structural and cannot be \
                     refined — patch enum cases are ignored.",
                    patch.original_name,
                ))
                .with_code(ScanningIssueKind::PatchEnumCasesIgnored)
                .with_annotation(Annotation::primary(patch.span)),
            );
        }

        // Type aliases
        self.type_aliases.extend(patch.type_aliases);

        // Scan-time issues on the patch itself (malformed docblocks, bad type
        // annotations, etc.) are not validation errors about the application — they
        // are diagnostics about the patch source and belong in the same bucket.
        diagnostics.extend(patch.issues);

        diagnostics
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

/// Collects all method names reachable through the ancestors of `class_meta`.
///
/// Does not include methods defined directly on `class_meta` itself.
#[must_use]
pub fn collect_ancestor_methods(class_meta: &ClassLikeMetadata, class_likes: &WordMap<ClassLikeMetadata>) -> WordSet {
    let mut visited = WordSet::default();
    let mut methods = WordSet::default();
    collect_ancestor_methods_inner(class_meta, class_likes, &mut visited, &mut methods);
    methods
}

fn collect_ancestor_methods_inner(
    class_meta: &ClassLikeMetadata,
    class_likes: &WordMap<ClassLikeMetadata>,
    visited: &mut WordSet,
    methods: &mut WordSet,
) {
    if !visited.insert(class_meta.name) {
        return;
    }
    if let Some(parent_name) = class_meta.direct_parent_class
        && let Some(parent_meta) = class_likes.get(&parent_name)
    {
        methods.extend(parent_meta.methods.iter().copied());
        collect_ancestor_methods_inner(parent_meta, class_likes, visited, methods);
    }
    for interface_name in &class_meta.direct_parent_interfaces {
        if let Some(interface_meta) = class_likes.get(interface_name) {
            methods.extend(interface_meta.methods.iter().copied());
            collect_ancestor_methods_inner(interface_meta, class_likes, visited, methods);
        }
    }
    for trait_name in &class_meta.used_traits {
        if let Some(trait_meta) = class_likes.get(trait_name) {
            methods.extend(trait_meta.methods.iter().copied());
            collect_ancestor_methods_inner(trait_meta, class_likes, visited, methods);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::once;

    use mago_span::Span;
    use mago_word::WordSet;
    use mago_word::word;

    use crate::identifier::method::MethodIdentifier;
    use crate::issue::ScanningIssueKind;
    use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
    use crate::metadata::enum_case::EnumCaseMetadata;
    use crate::metadata::flags::MetadataFlags;
    use crate::metadata::property::PropertyMetadata;
    use crate::misc::GenericParent;
    use crate::misc::VariableIdentifier;
    use crate::symbol::SymbolKind;
    use crate::ttype;
    use crate::ttype::template::GenericTemplate;
    use crate::ttype::template::variance::Variance;
    use crate::visibility::Visibility;

    use super::ClassLikeMetadata;

    fn has_code(issues: &[mago_reporting::Issue], kind: ScanningIssueKind) -> bool {
        let code = kind.to_string();
        issues.iter().any(|i| i.code.as_deref() == Some(code.as_str()))
    }

    fn make(name: &str) -> ClassLikeMetadata {
        let a = word(name);
        ClassLikeMetadata::new(a, a, Span::dummy(0, 10), None, MetadataFlags::empty())
    }

    #[test]
    fn apply_patch_adds_override_for_inherited_real_method() {
        let class_name = word("VendorClass");
        let mut vendored = make("VendorClass");
        let method_existing = word("existing");
        vendored.methods.insert(method_existing);
        vendored.declaring_method_ids.insert(method_existing, MethodIdentifier::new(class_name, method_existing));

        let mut patch = make("VendorClass");
        let method_override = word("inherited_method");
        patch.methods.insert(method_override);
        patch.declaring_method_ids.insert(method_override, MethodIdentifier::new(class_name, method_override));
        patch.inheritable_method_ids.insert(method_override, MethodIdentifier::new(class_name, method_override));

        let inherited: WordSet = once(method_override).collect();
        let issues = vendored.apply_patch(patch, &inherited);

        // The override is added to the real method set and to inheritable_method_ids.
        // declaring_method_ids and appearing_method_ids are filled in later by the populator
        // from self.methods, so apply_patch does not seed them here.
        assert!(vendored.methods.contains(&method_override));
        assert!(vendored.inheritable_method_ids.contains_key(&method_override));

        // No warning: patch overrides of inherited methods are expected and intentional.
        assert!(issues.is_empty());
    }

    #[test]
    fn apply_patch_adds_pseudo_methods() {
        let class_name = word("VendorClass");
        let mut vendored = make("VendorClass");

        let mut patch = make("VendorClass");
        let pseudo = word("magicMethod");
        patch.pseudo_methods.insert(pseudo);
        patch.declaring_method_ids.insert(pseudo, MethodIdentifier::new(class_name, pseudo));
        patch.appearing_method_ids.insert(pseudo, MethodIdentifier::new(class_name, pseudo));
        patch.inheritable_method_ids.insert(pseudo, MethodIdentifier::new(class_name, pseudo));

        let issues = vendored.apply_patch(patch, &WordSet::default());

        // Pseudo-method added to the right sets and ID maps.
        assert!(vendored.pseudo_methods.contains(&pseudo));
        assert!(vendored.declaring_method_ids.contains_key(&pseudo));
        assert!(vendored.appearing_method_ids.contains_key(&pseudo));
        assert!(vendored.inheritable_method_ids.contains_key(&pseudo));

        // Must not appear as a real method.
        assert!(!vendored.methods.contains(&pseudo));

        // No issues.
        assert!(issues.is_empty());
    }

    #[test]
    fn apply_patch_accepts_new_magic_property() {
        let mut vendored = make("VendorClass");

        let mut patch = make("VendorClass");
        let prop_magic = word("$magic");
        patch.properties.insert(
            prop_magic,
            PropertyMetadata::new(VariableIdentifier(prop_magic), MetadataFlags::PATCH | MetadataFlags::MAGIC_PROPERTY),
        );

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(vendored.properties.contains_key(&prop_magic));
        assert!(issues.is_empty());
    }

    #[test]
    fn apply_patch_does_not_touch_initialized_or_override_maps() {
        let mut vendored = make("VendorClass");
        let prop = word("$x");
        vendored.properties.insert(prop, PropertyMetadata::new(VariableIdentifier(prop), MetadataFlags::empty()));
        vendored.initialized_properties.insert(prop);
        vendored.overridden_property_ids.insert(prop, once(word("ParentClass")).collect());

        let mut patch = make("VendorClass");
        patch.properties.insert(prop, PropertyMetadata::new(VariableIdentifier(prop), MetadataFlags::PATCH));
        // patch has no initialized_properties entry and no overridden_property_ids

        let issues = vendored.apply_patch(patch, &WordSet::default());

        // initialized_properties must not be cleared by the patch
        assert!(vendored.initialized_properties.contains(&prop));
        // overridden_property_ids must not be cleared by the patch
        assert!(vendored.overridden_property_ids.contains_key(&prop));
        assert!(issues.is_empty());
    }

    #[test]
    fn apply_patch_adds_template_types() {
        let class_name = word("VendorClass");
        let mut vendored = make("VendorClass");

        let mut patch = make("VendorClass");
        let t = word("T");
        patch.template_types.insert(t, GenericTemplate::new(GenericParent::ClassLike(class_name), ttype::get_mixed()));
        patch.template_variance.push(Variance::Covariant);
        patch.template_readonly.insert(t);

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(vendored.template_types.contains_key(&t));
        assert_eq!(vendored.template_variance.first().copied(), Some(Variance::Covariant));
        assert!(vendored.template_readonly.contains(&t));
        assert!(issues.is_empty());
    }

    #[test]
    fn apply_patch_refines_existing_template_and_appends_new() {
        let class_name = word("VendorClass");
        let mut vendored = make("VendorClass");
        let t = word("T");
        let u = word("U");

        // Original has T (invariant, constraint = mixed) and U (covariant).
        vendored
            .template_types
            .insert(t, GenericTemplate::new(GenericParent::ClassLike(class_name), ttype::get_mixed()));
        vendored
            .template_types
            .insert(u, GenericTemplate::new(GenericParent::ClassLike(class_name), ttype::get_mixed()));
        vendored.template_variance = vec![Variance::Invariant, Variance::Covariant];

        // Patch refines T (now contravariant) and adds V (invariant).
        let mut patch = make("VendorClass");
        let v = word("V");
        patch.template_types.insert(t, GenericTemplate::new(GenericParent::ClassLike(class_name), ttype::get_int()));
        patch.template_types.insert(v, GenericTemplate::new(GenericParent::ClassLike(class_name), ttype::get_string()));
        patch.template_variance = vec![Variance::Contravariant, Variance::Invariant];

        let issues = vendored.apply_patch(patch, &WordSet::default());

        // T refined, U preserved, V appended → order T=0, U=1, V=2
        assert_eq!(vendored.template_types.keys().copied().collect::<Vec<_>>(), [t, u, v]);
        assert_eq!(vendored.template_variance, [Variance::Contravariant, Variance::Covariant, Variance::Invariant]);

        assert!(issues.is_empty());
    }

    #[test]
    fn apply_patch_preserves_original_only_readonly_template() {
        // Original: T is readonly. Patch adds U but does not re-declare T.
        // T must remain readonly after the patch — the patch can add readonly
        // entries but must not strip existing ones by omitting them.
        let class_name = word("VendorClass");
        let mut vendored = make("VendorClass");
        let t = word("T");
        vendored
            .template_types
            .insert(t, GenericTemplate::new(GenericParent::ClassLike(class_name), ttype::get_mixed()));
        vendored.template_variance.push(Variance::Invariant);
        vendored.template_readonly.insert(t);

        let mut patch = make("VendorClass");
        let u = word("U");
        patch.template_types.insert(u, GenericTemplate::new(GenericParent::ClassLike(class_name), ttype::get_mixed()));
        patch.template_variance.push(Variance::Invariant);

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(vendored.template_readonly.contains(&t), "T should remain readonly after patch");
        assert!(!vendored.template_readonly.contains(&u), "U was not declared readonly by the patch");
        assert!(issues.is_empty());
    }

    #[test]
    fn apply_patch_rejects_kind_mismatch() {
        let mut vendored = make("VendorClass");
        let mut patch = make("VendorClass");
        patch.kind = SymbolKind::Interface;

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(has_code(&issues, ScanningIssueKind::PatchKindMismatch));
    }

    #[test]
    fn apply_patch_rejects_trait_use() {
        let mut vendored = make("VendorClass");
        let mut patch = make("VendorClass");
        patch.used_traits.insert(word("SomeTrait"));

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(has_code(&issues, ScanningIssueKind::PatchDeclaresTrait));
    }

    #[test]
    fn apply_patch_rejects_hierarchy_mismatch() {
        let mut vendored = make("VendorClass");
        vendored.direct_parent_class = Some(word("ActualParent"));

        let mut patch = make("VendorClass");
        patch.direct_parent_class = Some(word("WrongParent"));

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(has_code(&issues, ScanningIssueKind::PatchHierarchyMismatch));
    }

    #[test]
    fn apply_patch_rejects_new_method_not_in_ancestors() {
        let mut vendored = make("VendorClass");
        let mut patch = make("VendorClass");
        patch.methods.insert(word("newMethod"));

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(has_code(&issues, ScanningIssueKind::PatchIntroducesNewMethod));
    }

    #[test]
    fn apply_patch_rejects_new_property() {
        let mut vendored = make("VendorClass");
        let mut patch = make("VendorClass");
        let prop = word("$newProp");
        patch.properties.insert(prop, PropertyMetadata::new(VariableIdentifier(prop), MetadataFlags::PATCH));

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(has_code(&issues, ScanningIssueKind::PatchIntroducesNewProperty));
    }

    #[test]
    fn apply_patch_rejects_new_constant() {
        let mut vendored = make("VendorClass");
        let mut patch = make("VendorClass");
        let c = word("NEW_CONST");
        patch
            .constants
            .insert(c, ClassLikeConstantMetadata::new(c, Span::dummy(0, 5), Visibility::Public, MetadataFlags::PATCH));

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(has_code(&issues, ScanningIssueKind::PatchIntroducesNewConstant));
    }

    #[test]
    fn apply_patch_rejects_enum_cases() {
        let mut vendored = make("VendorClass");
        let mut patch = make("VendorClass");
        let case = word("CaseA");
        patch
            .enum_cases
            .insert(case, EnumCaseMetadata::new(case, Span::dummy(0, 3), Span::dummy(0, 5), MetadataFlags::PATCH));

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(has_code(&issues, ScanningIssueKind::PatchEnumCasesIgnored));
    }

    #[test]
    fn apply_patch_rejects_property_structural_mismatch() {
        let mut vendored = make("VendorClass");
        let prop = word("$x");
        vendored.properties.insert(prop, PropertyMetadata::new(VariableIdentifier(prop), MetadataFlags::empty()));

        let mut patch = make("VendorClass");
        patch.properties.insert(
            prop,
            PropertyMetadata::new(VariableIdentifier(prop), MetadataFlags::PATCH | MetadataFlags::STATIC),
        );

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(has_code(&issues, ScanningIssueKind::PatchPropertyStructuralMismatch));
    }

    #[test]
    fn apply_patch_rejects_constant_structural_mismatch() {
        let mut vendored = make("VendorClass");
        let c = word("MY_CONST");
        vendored.constants.insert(
            c,
            ClassLikeConstantMetadata::new(c, Span::dummy(0, 5), Visibility::Private, MetadataFlags::empty()),
        );

        let mut patch = make("VendorClass");
        patch
            .constants
            .insert(c, ClassLikeConstantMetadata::new(c, Span::dummy(0, 5), Visibility::Public, MetadataFlags::PATCH));

        let issues = vendored.apply_patch(patch, &WordSet::default());

        assert!(has_code(&issues, ScanningIssueKind::PatchConstantStructuralMismatch));
    }
}
