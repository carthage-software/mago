//! Property initialization provider for Laravel Eloquent classes.
//!
//! Marks well-known framework properties as initialized so the analyzer
//! does not report `UninitializedProperty` false positives on Model,
//! Builder, and Factory subclasses.
//!
//! In phpantom these properties are handled through virtual member synthesis.
//! In Mago's architecture, this provider compensates by telling the analyzer
//! these properties are initialized at runtime.
//!
//! Derived from phpantom_lsp's `LaravelModelProvider` and `LaravelFactoryProvider`.

use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;

use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::property::PropertyInitializationProvider;

use super::utils::BUILDER_INITIALIZED_PROPERTIES;
use super::utils::FACTORY_INITIALIZED_PROPERTIES;
use super::utils::MODEL_INITIALIZED_PROPERTIES;
use super::utils::is_eloquent_builder_parent;
use super::utils::is_eloquent_factory_parent;
use super::utils::is_eloquent_model_parent;

static META: ProviderMeta = ProviderMeta::new(
    "laravel-property-init",
    "Laravel Property Initialization",
    "Marks Eloquent Model, Builder, and Factory properties as initialized",
);

/// Property initialization provider for Laravel Eloquent classes.
///
/// Handles three cases:
///
/// 1. **Well-known named properties** — properties like `$fillable`, `$guarded`,
///    `$casts`, `$table`, etc. that are defined in the base Model/Builder/Factory
///    classes with default values or are hydrated at runtime.
///
/// 2. **Typed-but-undefaulted Model properties** — any non-static, non-readonly
///    property with a type declaration but no default value on a Model subclass
///    is treated as initialized (database-backed attribute hydrated at runtime).
///
/// 3. **Builder and Factory properties** — well-known properties on Builder and
///    Factory subclasses that are initialized in the base constructors.
pub struct LaravelPropertyInit;

impl Provider for LaravelPropertyInit {
    fn meta() -> &'static ProviderMeta
    where
        Self: Sized,
    {
        &META
    }
}

impl PropertyInitializationProvider for LaravelPropertyInit {
    fn is_property_initialized(
        &self,
        class_metadata: &ClassLikeMetadata,
        property_metadata: &PropertyMetadata,
    ) -> bool {
        let prop_name = property_metadata.name.0.as_str();

        // Check Model hierarchy
        if is_eloquent_model_parent(&class_metadata.all_parent_classes) {
            // Well-known Model properties
            if MODEL_INITIALIZED_PROPERTIES.contains(&prop_name) {
                return true;
            }

            // Typed-but-undefaulted non-static, non-readonly properties on Models
            // are database-backed columns hydrated at runtime.
            if is_typed_model_column(property_metadata) {
                return true;
            }
        }

        // Check Builder hierarchy
        if is_eloquent_builder_parent(&class_metadata.all_parent_classes) {
            if BUILDER_INITIALIZED_PROPERTIES.contains(&prop_name) {
                return true;
            }
        }

        // Check Factory hierarchy
        if is_eloquent_factory_parent(&class_metadata.all_parent_classes) {
            if FACTORY_INITIALIZED_PROPERTIES.contains(&prop_name) {
                return true;
            }
        }

        false
    }
}

/// Checks if a property looks like a database-backed column on a Model:
/// - Has a type declaration (typed)
/// - Does not have a default value
/// - Is not static
/// - Is not readonly
fn is_typed_model_column(property_metadata: &PropertyMetadata) -> bool {
    let has_type = property_metadata.type_declaration_metadata.is_some();
    let has_default = property_metadata.flags.has_default();
    let is_static = property_metadata.flags.is_static();
    let is_readonly = property_metadata.flags.is_readonly();

    has_type && !has_default && !is_static && !is_readonly
}

#[cfg(test)]
mod tests {
    use mago_atom::atom;
    use mago_codex::metadata::flags::MetadataFlags;
    use mago_codex::metadata::ttype::TypeMetadata;
    use mago_codex::misc::VariableIdentifier;
    use mago_codex::ttype::atomic::TAtomic;
    use mago_codex::ttype::union::TUnion;
    use mago_span::Span;

    use super::*;

    fn make_property(name: &str, flags: MetadataFlags) -> PropertyMetadata {
        PropertyMetadata::new(VariableIdentifier(atom(name)), flags)
    }

    fn dummy_type_metadata() -> TypeMetadata {
        TypeMetadata::new(TUnion::from_atomic(TAtomic::Never), Span::zero())
    }

    #[test]
    fn test_is_known_model_property() {
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$fillable"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$guarded"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$casts"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$hidden"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$table"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$connection"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$primaryKey"));
    }

    #[test]
    fn test_is_known_factory_property() {
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$model"));
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$count"));
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$states"));
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$afterMaking"));
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$afterCreating"));
    }

    #[test]
    fn test_is_known_builder_property() {
        assert!(BUILDER_INITIALIZED_PROPERTIES.contains(&"$model"));
        assert!(BUILDER_INITIALIZED_PROPERTIES.contains(&"$query"));
        assert!(BUILDER_INITIALIZED_PROPERTIES.contains(&"$eagerLoad"));
    }

    #[test]
    fn test_is_typed_model_column_true() {
        // Typed, no default, not static, not readonly
        let mut prop = make_property("$name", MetadataFlags::empty());
        prop.type_declaration_metadata = Some(dummy_type_metadata());
        assert!(is_typed_model_column(&prop));
    }

    #[test]
    fn test_is_typed_model_column_false_has_default() {
        let mut prop = make_property("$name", MetadataFlags::HAS_DEFAULT);
        prop.type_declaration_metadata = Some(dummy_type_metadata());
        assert!(!is_typed_model_column(&prop));
    }

    #[test]
    fn test_is_typed_model_column_false_static() {
        let mut prop = make_property("$name", MetadataFlags::STATIC);
        prop.type_declaration_metadata = Some(dummy_type_metadata());
        assert!(!is_typed_model_column(&prop));
    }

    #[test]
    fn test_is_typed_model_column_false_readonly() {
        let mut prop = make_property("$name", MetadataFlags::READONLY);
        prop.type_declaration_metadata = Some(dummy_type_metadata());
        assert!(!is_typed_model_column(&prop));
    }

    #[test]
    fn test_is_typed_model_column_false_no_type() {
        let prop = make_property("$name", MetadataFlags::empty());
        assert!(!is_typed_model_column(&prop));
    }
}