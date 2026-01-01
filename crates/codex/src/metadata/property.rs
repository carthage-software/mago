use serde::Deserialize;
use serde::Serialize;

use mago_atom::AtomMap;
use mago_span::Span;

use crate::metadata::flags::MetadataFlags;
use crate::metadata::property_hook::PropertyHookMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;
use crate::visibility::Visibility;

/// Contains metadata associated with a declared class property in PHP.
///
/// This includes information about its name, location, visibility (potentially asymmetric),
/// type hints, default values, and various modifiers (`static`, `readonly`, `abstract`, etc.).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PropertyMetadata {
    /// The identifier (name) of the property, including the leading '$'.
    pub name: VariableIdentifier,

    /// The specific source code location (span) of the property's name identifier itself.
    /// `None` if the location is unknown or not relevant (e.g., for synthetic properties).
    pub name_span: Option<Span>,

    /// The source code location (span) covering the entire property declaration statement.
    /// `None` if the location is unknown or not relevant.
    pub span: Option<Span>,

    /// The visibility level required for reading the property's value.
    ///
    /// In PHP, this corresponds to the primary visibility keyword specified
    /// (e.g., the `public` in `public private(set) string $prop;`).
    ///
    /// If no asymmetric visibility is specified (e.g., `public string $prop`),
    /// this level applies to both reading and writing. Defaults to `Public`.
    pub read_visibility: Visibility,

    /// The visibility level required for writing/modifying the property's value.
    ///
    /// In PHP, this can differ from `read_visibility` using asymmetric visibility syntax
    /// like `private(set)` (e.g., `public private(set) string $prop;`).
    ///
    /// If asymmetric visibility is not used, this implicitly matches `read_visibility`.
    /// Defaults to `Public`.
    pub write_visibility: Visibility,

    /// The explicit type declaration (type hint) associated with the property, if any.
    ///
    /// e.g., for `public string $name;`, this would contain the metadata for `string`.
    pub type_declaration_metadata: Option<TypeMetadata>,

    /// The type metadata for the property's type, if any.
    ///
    /// This is either the same as `type_declaration_metadata` or the type provided
    /// in a docblock comment (e.g., `@var string`).
    pub type_metadata: Option<TypeMetadata>,

    /// The type inferred from the property's default value, if it has one.
    ///
    /// e.g., for `public $count = 0;`, this would contain the metadata for `int(0)`.
    /// This can be used to compare against `type_signature` for consistency checks.
    pub default_type_metadata: Option<TypeMetadata>,

    /// Flags indicating various properties of the property.
    pub flags: MetadataFlags,

    /// Metadata for property hooks (get/set).
    ///
    /// Key is the hook name atom ("get" or "set").
    /// Only present for PHP 8.4+ hooked properties.
    pub hooks: AtomMap<PropertyHookMetadata>,
}

impl PropertyMetadata {
    /// Creates new `PropertyMetadata` with basic defaults (public, non-static, non-readonly, etc.).
    /// Name is mandatory. Spans, types, and flags can be set using modifier methods.
    #[inline]
    #[must_use]
    pub fn new(name: VariableIdentifier, flags: MetadataFlags) -> Self {
        Self {
            name,
            name_span: None,
            span: None,
            read_visibility: Visibility::Public,
            write_visibility: Visibility::Public,
            type_declaration_metadata: None,
            type_metadata: None,
            default_type_metadata: None,
            flags,
            hooks: AtomMap::default(),
        }
    }

    #[inline]
    pub fn set_default_type_metadata(&mut self, default_type_metadata: Option<TypeMetadata>) {
        self.default_type_metadata = default_type_metadata;
    }

    #[inline]
    pub fn set_type_declaration_metadata(&mut self, type_declaration_metadata: Option<TypeMetadata>) {
        if self.type_metadata.is_none() {
            self.type_metadata.clone_from(&type_declaration_metadata);
        }

        self.type_declaration_metadata = type_declaration_metadata;
    }

    #[inline]
    pub fn set_type_metadata(&mut self, type_metadata: Option<TypeMetadata>) {
        self.type_metadata = type_metadata;
    }

    /// Returns a reference to the property's name identifier.
    #[inline]
    #[must_use]
    pub fn get_name(&self) -> &VariableIdentifier {
        &self.name
    }

    /// Checks if the property is effectively final (private read or write access).
    #[inline]
    #[must_use]
    pub fn is_final(&self) -> bool {
        self.read_visibility.is_private() || self.write_visibility.is_private()
    }

    /// Sets the span for the property name identifier.
    #[inline]
    pub fn set_name_span(&mut self, name_span: Option<Span>) {
        self.name_span = name_span;
    }

    /// Sets the overall span for the property declaration.
    #[inline]
    pub fn set_span(&mut self, span: Option<Span>) {
        self.span = span;
    }

    /// Sets both read and write visibility levels. Updates `is_asymmetric`. Ensures virtual properties remain symmetric.
    #[inline]
    pub fn set_visibility(&mut self, read: Visibility, write: Visibility) {
        self.read_visibility = read;
        self.write_visibility = write;
        self.update_asymmetric();
    }

    /// Sets whether the property uses property hooks. Updates `is_asymmetric`.
    #[inline]
    pub fn set_is_virtual(&mut self, is_virtual: bool) {
        self.flags.set(MetadataFlags::VIRTUAL_PROPERTY, is_virtual);

        self.update_asymmetric();
    }

    /// Also ensures virtual properties are not asymmetric.
    #[inline]
    fn update_asymmetric(&mut self) {
        if self.flags.is_virtual_property() {
            if self.read_visibility != self.write_visibility {
                // If virtual and somehow asymmetric, force symmetry (prefer read)
                self.write_visibility = self.read_visibility;
            }

            self.flags &= !MetadataFlags::ASYMMETRIC_PROPERTY;
        } else if self.read_visibility == self.write_visibility {
            // If both visibilities are the same, ensure no asymmetric flag is set
            self.flags &= !MetadataFlags::ASYMMETRIC_PROPERTY;
        } else {
            // Otherwise, set the asymmetric flag
            self.flags |= MetadataFlags::ASYMMETRIC_PROPERTY;
        }
    }
}
