//! Property-related providers for the analyzer plugin system.

use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;

use crate::plugin::provider::Provider;

/// Provider for checking if a property should be considered initialized.
///
/// This allows plugins to mark properties as "initialized" even without explicit
/// initialization in the constructor. Useful for ORMs with auto-generated fields
/// like Doctrine's `#[ORM\GeneratedValue]`, or DI frameworks with `#[Inject]` attributes.
pub trait PropertyInitializationProvider: Provider {
    /// Check if a property should be considered initialized.
    ///
    /// Returns `true` if the property is considered initialized by this provider,
    /// `false` to let other providers or default logic decide.
    fn is_property_initialized(&self, class_metadata: &ClassLikeMetadata, property_metadata: &PropertyMetadata)
    -> bool;
}
