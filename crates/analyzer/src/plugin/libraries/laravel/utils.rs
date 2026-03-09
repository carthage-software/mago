//! Shared utilities for Laravel support.
//!
//! Pure helper functions and constants derived from phpantom_lsp's
//! `src/virtual_members/laravel.rs`.  These encode Laravel framework
//! knowledge at the string level and have no dependency on Mago's type
//! system or class representation.

// ────────────────────────────────────────────────────────────────────────────────
// String conversion helpers
// ────────────────────────────────────────────────────────────────────────────────

/// Converts a `camelCase` or `PascalCase` string to `snake_case`.
///
/// acronym boundaries correctly.
///
/// # Examples
/// ```text
/// "fooBar"     → "foo_bar"
/// "FooBar"     → "foo_bar"
/// "fooBarBaz"  → "foo_bar_baz"
/// "HTMLParser" → "html_parser"   (acronym boundary)
/// "URLName"    → "url_name"      (acronym boundary)
/// "item2Name"  → "item2_name"    (digit → uppercase)
/// ```
pub fn camel_to_snake(input: &str) -> String {
    let mut result = String::with_capacity(input.len() + 4);
    let chars: Vec<char> = input.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                // Insert underscore when: lowercase/digit → uppercase,
                // or uppercase → uppercase followed by lowercase (acronym boundary).
                if prev.is_lowercase()
                    || prev.is_ascii_digit()
                    || (prev.is_uppercase() && chars.get(i + 1).is_some_and(|next| next.is_lowercase()))
                {
                    result.push('_');
                }
            }
            for lc in ch.to_lowercase() {
                result.push(lc);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Converts a `snake_case` string to `camelCase`.
///
/// # Examples
/// ```text
/// "foo_bar"     → "fooBar"
/// "foo_bar_baz" → "fooBarBaz"
/// "full_name"   → "fullName"
/// ```
pub fn snake_to_camel(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut capitalize_next = false;
    for ch in input.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// Converts a `snake_case` string to `PascalCase`.
///
/// # Examples
/// ```text
/// "foo_bar"     → "FooBar"
/// "full_name"   → "FullName"
/// ```
pub fn snake_to_pascal(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut capitalize_next = true;
    for ch in input.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

// ────────────────────────────────────────────────────────────────────────────────
// FQN helpers
// ────────────────────────────────────────────────────────────────────────────────

/// Extracts the short (unqualified) name from a fully-qualified class name.
///
/// # Examples
/// ```text
/// "\\App\\Models\\User"                    → "User"
/// "Illuminate\\Database\\Eloquent\\Model"  → "Model"
/// "User"                                   → "User"
/// ```
pub fn extract_short_name(fqn: &str) -> &str {
    fqn.rsplit('\\').next().unwrap_or(fqn)
}

/// Strips a leading backslash from a FQN if present.
///
/// # Examples
/// ```text
/// "\\App\\Models\\User" → "App\\Models\\User"
/// "App\\Models\\User"   → "App\\Models\\User"
/// ```
pub fn strip_leading_backslash(fqn: &str) -> &str {
    fqn.strip_prefix('\\').unwrap_or(fqn)
}

// ────────────────────────────────────────────────────────────────────────────────
// Laravel FQN constants
// ────────────────────────────────────────────────────────────────────────────────

/// The fully-qualified name of the Eloquent base model.
pub const ELOQUENT_MODEL: &str = "Illuminate\\Database\\Eloquent\\Model";

/// The fully-qualified name of the Eloquent Builder class.
pub const ELOQUENT_BUILDER: &str = "Illuminate\\Database\\Eloquent\\Builder";

/// The fully-qualified name of the Eloquent Collection class.
pub const ELOQUENT_COLLECTION: &str = "Illuminate\\Database\\Eloquent\\Collection";

/// The fully-qualified name of the Eloquent Factory base class.
pub const ELOQUENT_FACTORY: &str = "Illuminate\\Database\\Eloquent\\Factories\\Factory";

/// The `Illuminate\Database\Eloquent\Casts\Attribute` class (modern accessors).
pub const ATTRIBUTE_CLASS: &str = "Illuminate\\Database\\Eloquent\\Casts\\Attribute";

/// The short name of the `CastsAttributes` interface.
pub const CASTS_ATTRIBUTES_SHORT: &str = "CastsAttributes";

/// The fully-qualified name of the `CastsAttributes` interface.
pub const CASTS_ATTRIBUTES: &str = "Illuminate\\Contracts\\Database\\Eloquent\\CastsAttributes";

/// The fully-qualified name of the `Castable` contract.
pub const CASTABLE: &str = "Illuminate\\Contracts\\Database\\Eloquent\\Castable";

/// The default return type for scope methods that don't declare a return
/// type or return `void`.
pub const DEFAULT_SCOPE_RETURN_TYPE: &str = "\\Illuminate\\Database\\Eloquent\\Builder<static>";

/// The fully-qualified name of the `#[Scope]` attribute (Laravel 11+).
pub const SCOPE_ATTRIBUTE: &str = "Illuminate\\Database\\Eloquent\\Attributes\\Scope";

/// The fully-qualified name of the `#[CollectedBy]` attribute.
pub const COLLECTED_BY_ATTRIBUTE: &str = "Illuminate\\Database\\Eloquent\\Attributes\\CollectedBy";

/// The fully-qualified name of the `SoftDeletes` trait.
pub const SOFT_DELETES_TRAIT: &str = "Illuminate\\Database\\Eloquent\\SoftDeletes";

// ────────────────────────────────────────────────────────────────────────────────
// Model / Builder / Factory initialized property lists
// ────────────────────────────────────────────────────────────────────────────────

/// Well-known Eloquent Model properties that are initialized at runtime
/// (either with defaults in the base class or hydrated from the database).
pub const MODEL_INITIALIZED_PROPERTIES: &[&str] = &[
    "$connection",
    "$table",
    "$primaryKey",
    "$keyType",
    "$incrementing",
    "$with",
    "$withCount",
    "$preventsLazyLoading",
    "$perPage",
    "$exists",
    "$wasRecentlyCreated",
    "$escapeWhenCastingToString",
    "$attributes",
    "$original",
    "$changes",
    "$casts",
    "$classCastCache",
    "$attributeCastCache",
    "$dateFormat",
    "$appends",
    "$dispatchesEvents",
    "$observables",
    "$relations",
    "$touches",
    "$timestamps",
    "$usesUniqueIds",
    "$hidden",
    "$visible",
    "$fillable",
    "$guarded",
];

/// Well-known Eloquent Builder properties that are initialized at runtime.
pub const BUILDER_INITIALIZED_PROPERTIES: &[&str] =
    &["$model", "$query", "$eagerLoad", "$localMacros", "$scopes", "$removedScopes", "$passthru"];

/// Well-known Factory properties that are initialized at runtime.
pub const FACTORY_INITIALIZED_PROPERTIES: &[&str] =
    &["$model", "$count", "$states", "$has", "$for", "$afterMaking", "$afterCreating", "$connection", "$faker"];

// ────────────────────────────────────────────────────────────────────────────────
// Hierarchy helpers (lowercased parent checking)
// ────────────────────────────────────────────────────────────────────────────────

/// Lowercased FQN of the Eloquent Model base class (for `all_parent_classes` lookups).
const ELOQUENT_MODEL_LOWER: &str = "illuminate\\database\\eloquent\\model";

/// Lowercased FQN of the Eloquent Builder class.
const ELOQUENT_BUILDER_LOWER: &str = "illuminate\\database\\eloquent\\builder";

/// Lowercased FQN of the Eloquent Factory class.
const ELOQUENT_FACTORY_LOWER: &str = "illuminate\\database\\eloquent\\factories\\factory";

/// Checks if the given `all_parent_classes` set contains the Eloquent Model.
///
/// `all_parent_classes` in Mago stores lowercased FQNs.
pub fn is_eloquent_model_parent(all_parent_classes: &impl Contains) -> bool {
    all_parent_classes.contains_str(ELOQUENT_MODEL_LOWER)
}

/// Checks if the given `all_parent_classes` set contains the Eloquent Builder.
pub fn is_eloquent_builder_parent(all_parent_classes: &impl Contains) -> bool {
    all_parent_classes.contains_str(ELOQUENT_BUILDER_LOWER)
}

/// Checks if the given `all_parent_classes` set contains the Eloquent Factory.
pub fn is_eloquent_factory_parent(all_parent_classes: &impl Contains) -> bool {
    all_parent_classes.contains_str(ELOQUENT_FACTORY_LOWER)
}

/// Abstraction for checking string membership in a set.
///
/// Implemented for `AtomSet` so that the hierarchy helpers work with
/// `ClassLikeMetadata.all_parent_classes`.
pub trait Contains {
    fn contains_str(&self, s: &str) -> bool;
}

impl Contains for mago_atom::AtomSet {
    fn contains_str(&self, s: &str) -> bool {
        self.contains(&mago_atom::atom(s))
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Relationship classification
// ────────────────────────────────────────────────────────────────────────────────

/// Classification of an Eloquent relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipKind {
    /// Returns a single related model (nullable).
    /// e.g. `HasOne`, `BelongsTo`, `MorphOne`, `HasOneThrough`
    Singular,
    /// Returns a collection of related models.
    /// e.g. `HasMany`, `BelongsToMany`, `MorphMany`, `MorphToMany`, `HasManyThrough`
    Collection,
    /// Polymorphic "to" — returns `Model` (unknown concrete type at analysis time).
    MorphTo,
}

/// Short names of singular relationship classes.
pub const SINGULAR_RELATIONSHIPS: &[&str] = &["HasOne", "MorphOne", "BelongsTo", "HasOneThrough"];

/// Short names of collection relationship classes.
pub const COLLECTION_RELATIONSHIPS: &[&str] =
    &["HasMany", "MorphMany", "BelongsToMany", "HasManyThrough", "MorphToMany"];

/// Maps Eloquent relationship builder method names to their corresponding
/// relationship class short names.  Used to synthesize a return type from
/// the method body when no `@return` annotation is present.
pub const RELATIONSHIP_METHOD_MAP: &[(&str, &str)] = &[
    ("hasOne", "HasOne"),
    ("hasMany", "HasMany"),
    ("belongsTo", "BelongsTo"),
    ("belongsToMany", "BelongsToMany"),
    ("morphOne", "MorphOne"),
    ("morphMany", "MorphMany"),
    ("morphTo", "MorphTo"),
    ("morphToMany", "MorphToMany"),
    ("hasManyThrough", "HasManyThrough"),
    ("hasOneThrough", "HasOneThrough"),
];

/// Classifies a relationship class by its short name.
///
/// Returns `None` for unrecognized class names.
///
/// # Examples
/// ```text
/// "HasOne"        → Some(Singular)
/// "HasMany"       → Some(Collection)
/// "MorphTo"       → Some(MorphTo)
/// "BelongsToMany" → Some(Collection)
/// ```
pub fn classify_relationship(short_name: &str) -> Option<RelationshipKind> {
    if short_name == "MorphTo" {
        return Some(RelationshipKind::MorphTo);
    }
    if SINGULAR_RELATIONSHIPS.contains(&short_name) {
        return Some(RelationshipKind::Singular);
    }
    if COLLECTION_RELATIONSHIPS.contains(&short_name) {
        return Some(RelationshipKind::Collection);
    }
    None
}

/// Classifies a relationship by its fully-qualified class name.
///
/// Extracts the short name from the FQN and delegates to [`classify_relationship`].
///
/// # Examples
/// ```text
/// "Illuminate\\Database\\Eloquent\\Relations\\HasMany" → Some(Collection)
/// "\\Illuminate\\Database\\Eloquent\\Relations\\HasOne" → Some(Singular)
/// ```
pub fn classify_relationship_fqn(fqn: &str) -> Option<RelationshipKind> {
    let short = extract_short_name(strip_leading_backslash(fqn));
    classify_relationship(short)
}

/// All known relationship class short names.
pub const RELATIONSHIP_CLASSES: &[&str] = &[
    "HasOne",
    "HasMany",
    "BelongsTo",
    "BelongsToMany",
    "MorphOne",
    "MorphMany",
    "MorphTo",
    "MorphToMany",
    "HasManyThrough",
    "HasOneThrough",
];

/// Checks if a short class name is a known Eloquent relationship class.
pub fn is_relationship_class(short_name: &str) -> bool {
    RELATIONSHIP_CLASSES.contains(&short_name)
}

// ────────────────────────────────────────────────────────────────────────────────
// Accessor helpers
// ────────────────────────────────────────────────────────────────────────────────

/// Checks if a method name is a legacy accessor getter (`getXxxAttribute`).
///
/// # Examples
/// ```text
/// "getFullNameAttribute" → true
/// "getNameAttribute"     → true
/// "getAttribute"         → false  (no property name in the middle)
/// "setNameAttribute"     → false  (setter, not getter)
/// "getattributeX"        → false  (lowercase after "get")
/// ```
pub fn is_legacy_getter_accessor(method_name: &str) -> bool {
    if !method_name.starts_with("get") || !method_name.ends_with("Attribute") {
        return false;
    }
    // Must have at least one character between "get" and "Attribute".
    // "getAttribute" itself (len 12) is a real Eloquent method, not an accessor.
    let middle = &method_name[3..method_name.len() - 9]; // strip "get" (3) and "Attribute" (9)
    if middle.is_empty() {
        return false;
    }
    // The first character of the middle portion must be uppercase.
    middle.starts_with(|c: char| c.is_uppercase())
}

/// Extracts the snake_case property name from a legacy getter accessor.
///
/// # Examples
/// ```text
/// "getFullNameAttribute" → Some("full_name")
/// "getNameAttribute"     → Some("name")
/// "getAttribute"         → None
/// ```
pub fn legacy_getter_property_name(method_name: &str) -> Option<String> {
    if !is_legacy_getter_accessor(method_name) {
        return None;
    }

    let inner = &method_name[3..method_name.len() - 9];
    if inner.is_empty() {
        return None;
    }

    Some(camel_to_snake(inner))
}

/// Generates the legacy accessor method name for a snake_case property name.
///
/// # Examples
/// ```text
/// "full_name" → "getFullNameAttribute"
/// "name"      → "getNameAttribute"
/// ```
pub fn legacy_accessor_method_name(property_name: &str) -> String {
    let pascal = snake_to_pascal(property_name);
    format!("get{}Attribute", pascal)
}

/// Given a snake_case property name, produces all accessor method candidates
/// that could define this property (legacy + modern).
///
/// # Examples
/// ```text
/// "full_name" → ["getFullNameAttribute", "fullName"]
/// ```
pub fn accessor_method_candidates(property_name: &str) -> Vec<String> {
    vec![legacy_accessor_method_name(property_name), snake_to_camel(property_name)]
}

/// Extracts the snake_case property name from a modern accessor method name
/// (camelCase method → snake_case property).
///
/// The method should return `Attribute` and not be a legacy accessor.
///
/// # Examples
/// ```text
/// "fullName" → "full_name"
/// ```
pub fn modern_accessor_property_name(method_name: &str) -> String {
    camel_to_snake(method_name)
}

// ────────────────────────────────────────────────────────────────────────────────
// Scope helpers
// ────────────────────────────────────────────────────────────────────────────────

/// Checks if a method name is a convention-based scope (e.g. `scopeActive`).
///
/// Must be longer than just "scope" — `scope()` alone is not a scope definition.
pub fn is_convention_scope_method(method_name: &str) -> bool {
    method_name.len() > 5
        && method_name.starts_with("scope")
        && method_name[5..].starts_with(|c: char| c.is_uppercase())
}

/// Extracts the callable scope name from a `scopeXxx` method.
///
/// # Examples
/// ```text
/// "scopeActive"   → Some("active")
/// "scopeOfType"   → Some("ofType")
/// "scope"         → None
/// "notAScope"     → None
/// ```
pub fn scope_name_from_method(method_name: &str) -> Option<String> {
    if !is_convention_scope_method(method_name) {
        return None;
    }

    let after_prefix = &method_name[5..];
    // lcfirst: lowercase the first character
    let mut chars = after_prefix.chars();
    match chars.next() {
        Some(first) => {
            let mut name = first.to_lowercase().to_string();
            name.push_str(chars.as_str());
            Some(name)
        }
        None => None,
    }
}

/// Given a scope name (e.g. `active`), produces the convention method name.
///
/// # Examples
/// ```text
/// "active" → "scopeActive"
/// "ofType" → "scopeOfType"
/// ```
pub fn scope_method_name(scope_name: &str) -> String {
    let mut chars = scope_name.chars();
    match chars.next() {
        Some(first) => {
            let upper_first: String = first.to_uppercase().collect();
            format!("scope{}{}", upper_first, chars.as_str())
        }
        None => "scope".to_string(),
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Factory <-> Model naming conventions
// ────────────────────────────────────────────────────────────────────────────────

/// Derives the model FQN from a factory FQN by naming convention.
///
/// # Examples
/// ```text
/// "Database\\Factories\\UserFactory"        → Some("App\\Models\\User")
/// "Database\\Factories\\Admin\\UserFactory"  → Some("App\\Models\\Admin\\User")
/// "\\Database\\Factories\\UserFactory"       → Some("App\\Models\\User")
/// "App\\Models\\User"                        → None  (not a factory)
/// ```
pub fn factory_to_model_fqn(factory_fqn: &str) -> Option<String> {
    let clean = strip_leading_backslash(factory_fqn);

    // The short name must end with `Factory`.
    let short = extract_short_name(clean);
    let model_short = short.strip_suffix("Factory")?;
    if model_short.is_empty() {
        return None;
    }

    // Extract the namespace (everything before the last `\`).
    let ns = clean.rsplit_once('\\').map(|(ns, _)| ns).unwrap_or("");

    // Extract the sub-namespace after `Database\Factories\`.
    let sub_ns = if let Some(after) = ns.strip_prefix("Database\\Factories\\") {
        Some(after)
    } else if ns == "Database\\Factories" {
        None
    } else {
        // Not in the standard factory namespace.
        None
    };

    match sub_ns {
        Some(sub) => Some(format!("App\\Models\\{}\\{}", sub, model_short)),
        None => Some(format!("App\\Models\\{}", model_short)),
    }
}

/// Derives the factory FQN from a model FQN by naming convention.
///
/// # Examples
/// ```text
/// "App\\Models\\User"        → "Database\\Factories\\UserFactory"
/// "App\\Models\\Admin\\User" → "Database\\Factories\\Admin\\UserFactory"
/// "User"                     → "Database\\Factories\\UserFactory"
/// ```
pub fn model_to_factory_fqn(model_fqn: &str) -> String {
    let clean = strip_leading_backslash(model_fqn);

    // Split into namespace + short name.
    let (ns, short) = match clean.rsplit_once('\\') {
        Some((ns, short)) => (ns, short),
        None => return format!("Database\\Factories\\{}Factory", clean),
    };

    // Check for `X\Models\Sub` pattern → `Database\Factories\Sub\ShortFactory`
    if let Some((_prefix, suffix)) = ns.split_once("\\Models\\") {
        return format!("Database\\Factories\\{}\\{}Factory", suffix, short);
    }

    // Check for `X\Models` pattern (model directly in Models namespace)
    if ns.ends_with("\\Models") || ns == "Models" {
        return format!("Database\\Factories\\{}Factory", short);
    }

    // No `Models` segment — put factory in `Database\Factories`
    format!("Database\\Factories\\{}Factory", short)
}

// ────────────────────────────────────────────────────────────────────────────────
// Cast type mapping
// ────────────────────────────────────────────────────────────────────────────────

/// Maps a Laravel cast string to a PHP type string.
///
/// **Note:** phpantom maps `datetime`/`date` to `\Carbon\Carbon`.  Here
/// we use `\Illuminate\Support\Carbon` instead since that is the type
/// Laravel actually exposes.
///
/// # Examples
/// ```text
/// "int"            → Some("int")
/// "integer"        → Some("int")
/// "float"          → Some("float")
/// "string"         → Some("string")
/// "bool"           → Some("bool")
/// "boolean"        → Some("bool")
/// "array"          → Some("array")
/// "object"         → Some("object")
/// "collection"     → Some("\\Illuminate\\Support\\Collection")
/// "date"           → Some("\\Illuminate\\Support\\Carbon")
/// "datetime"       → Some("\\Illuminate\\Support\\Carbon")
/// "immutable_date" → Some("\\Carbon\\CarbonImmutable")
/// "timestamp"      → Some("int")
/// "encrypted"      → Some("string")
/// "decimal:2"      → Some("float")
/// "datetime:Y-m-d" → Some("\\Illuminate\\Support\\Carbon")
/// "Boolean"        → Some("bool")   (case-insensitive)
/// "custom_class"   → None
/// ```
pub fn cast_type_to_php_type(cast_str: &str) -> Option<&'static str> {
    let lower = cast_str.to_lowercase();
    let lower = lower.as_str();

    match lower {
        "int" | "integer" => return Some("int"),
        "real" | "float" | "double" => return Some("float"),
        "string" => return Some("string"),
        "bool" | "boolean" => return Some("bool"),
        "array" | "json" => return Some("array"),
        "object" => return Some("object"),
        "collection" => return Some("\\Illuminate\\Support\\Collection"),
        "datetime" => return Some("\\Illuminate\\Support\\Carbon"),
        "date" => return Some("\\Illuminate\\Support\\Carbon"),
        "timestamp" => return Some("int"),
        "immutable_datetime" => return Some("\\Carbon\\CarbonImmutable"),
        "immutable_date" => return Some("\\Carbon\\CarbonImmutable"),
        "encrypted" => return Some("string"),
        "encrypted:array" => return Some("array"),
        "encrypted:collection" => return Some("\\Illuminate\\Support\\Collection"),
        "encrypted:object" => return Some("object"),
        "hashed" => return Some("string"),
        _ => {}
    }

    // 2. `decimal:N` variants (e.g. `decimal:2`, `decimal:8`) and bare `decimal`.
    if lower.starts_with("decimal:") || lower == "decimal" {
        return Some("float");
    }

    // 3. `datetime:format` variants (e.g. `datetime:Y-m-d`).
    if lower.starts_with("datetime:") {
        return Some("\\Illuminate\\Support\\Carbon");
    }

    // 4. `date:format` variants.
    if lower.starts_with("date:") {
        return Some("\\Illuminate\\Support\\Carbon");
    }

    // 5. `immutable_datetime:format` variants.
    if lower.starts_with("immutable_datetime:") {
        return Some("\\Carbon\\CarbonImmutable");
    }

    // 6. `immutable_date:format` variants.
    if lower.starts_with("immutable_date:") {
        return Some("\\Carbon\\CarbonImmutable");
    }

    // 7. Not a built-in cast — could be a class-based cast.
    //    Return None so the caller can perform deeper inspection
    //    (enum detection, CastsAttributes interface, Castable, etc.).
    None
}

// ────────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use mago_atom::AtomSet;
    use mago_atom::atom;

    // ── String conversions ───────────────────────────────────────────

    #[test]
    fn test_camel_to_snake() {
        assert_eq!(camel_to_snake("FullName"), "full_name");
        assert_eq!(camel_to_snake("Name"), "name");
        assert_eq!(camel_to_snake("name"), "name");
        assert_eq!(camel_to_snake("firstName"), "first_name");
        assert_eq!(camel_to_snake("isAdminUser"), "is_admin_user");
        // Acronym handling
        assert_eq!(camel_to_snake("HTMLParser"), "html_parser");
        assert_eq!(camel_to_snake("URLName"), "url_name");
        // Digit boundary
        assert_eq!(camel_to_snake("item2Name"), "item2_name");
    }

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("foo_bar"), "fooBar");
        assert_eq!(snake_to_camel("full_name"), "fullName");
        assert_eq!(snake_to_camel("name"), "name");
    }

    #[test]
    fn test_snake_to_pascal() {
        assert_eq!(snake_to_pascal("foo_bar"), "FooBar");
        assert_eq!(snake_to_pascal("full_name"), "FullName");
    }

    // ── FQN helpers ──────────────────────────────────────────────────

    #[test]
    fn test_extract_short_name() {
        assert_eq!(extract_short_name("Illuminate\\Database\\Eloquent\\Model"), "Model");
        assert_eq!(extract_short_name("\\App\\Models\\User"), "User");
        assert_eq!(extract_short_name("User"), "User");
    }

    #[test]
    fn test_strip_leading_backslash() {
        assert_eq!(strip_leading_backslash("\\App\\Models\\User"), "App\\Models\\User");
        assert_eq!(strip_leading_backslash("App\\Models\\User"), "App\\Models\\User");
    }

    // ── Hierarchy helpers ────────────────────────────────────────────

    #[test]
    fn test_is_eloquent_model_parent() {
        let mut set = AtomSet::default();
        set.insert(atom("illuminate\\database\\eloquent\\model"));
        assert!(is_eloquent_model_parent(&set));

        let empty = AtomSet::default();
        assert!(!is_eloquent_model_parent(&empty));
    }

    #[test]
    fn test_is_eloquent_builder_parent() {
        let mut set = AtomSet::default();
        set.insert(atom("illuminate\\database\\eloquent\\builder"));
        assert!(is_eloquent_builder_parent(&set));

        let empty = AtomSet::default();
        assert!(!is_eloquent_builder_parent(&empty));
    }

    #[test]
    fn test_is_eloquent_factory_parent() {
        let mut set = AtomSet::default();
        set.insert(atom("illuminate\\database\\eloquent\\factories\\factory"));
        assert!(is_eloquent_factory_parent(&set));

        let empty = AtomSet::default();
        assert!(!is_eloquent_factory_parent(&empty));
    }

    // ── Initialized property lists ───────────────────────────────────

    #[test]
    fn test_model_initialized_properties() {
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$fillable"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$guarded"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$casts"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$hidden"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$table"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$connection"));
        assert!(MODEL_INITIALIZED_PROPERTIES.contains(&"$primaryKey"));
        assert!(!MODEL_INITIALIZED_PROPERTIES.contains(&"$nonExistent"));
    }

    #[test]
    fn test_builder_initialized_properties() {
        assert!(BUILDER_INITIALIZED_PROPERTIES.contains(&"$model"));
        assert!(BUILDER_INITIALIZED_PROPERTIES.contains(&"$query"));
        assert!(BUILDER_INITIALIZED_PROPERTIES.contains(&"$eagerLoad"));
        assert!(!BUILDER_INITIALIZED_PROPERTIES.contains(&"$nonExistent"));
    }

    #[test]
    fn test_factory_initialized_properties() {
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$model"));
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$count"));
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$states"));
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$afterMaking"));
        assert!(FACTORY_INITIALIZED_PROPERTIES.contains(&"$afterCreating"));
        assert!(!FACTORY_INITIALIZED_PROPERTIES.contains(&"$nonExistent"));
    }

    // ── Relationship classification ──────────────────────────────────

    #[test]
    fn test_classify_relationship_fqn() {
        assert_eq!(
            classify_relationship_fqn("Illuminate\\Database\\Eloquent\\Relations\\HasMany"),
            Some(RelationshipKind::Collection),
        );
        assert_eq!(
            classify_relationship_fqn("\\Illuminate\\Database\\Eloquent\\Relations\\HasOne"),
            Some(RelationshipKind::Singular),
        );
        assert_eq!(
            classify_relationship_fqn("\\Illuminate\\Database\\Eloquent\\Relations\\MorphTo"),
            Some(RelationshipKind::MorphTo),
        );
        assert_eq!(classify_relationship_fqn("App\\Models\\User"), None);
    }

    #[test]
    fn test_is_relationship_class() {
        assert!(is_relationship_class("HasOne"));
        assert!(is_relationship_class("HasMany"));
        assert!(is_relationship_class("BelongsTo"));
        assert!(is_relationship_class("MorphTo"));
        assert!(!is_relationship_class("User"));
        assert!(!is_relationship_class("Model"));
    }

    #[test]
    fn test_classify_relationship() {
        assert_eq!(classify_relationship("HasOne"), Some(RelationshipKind::Singular));
        assert_eq!(classify_relationship("HasMany"), Some(RelationshipKind::Collection));
        assert_eq!(classify_relationship("BelongsTo"), Some(RelationshipKind::Singular));
        assert_eq!(classify_relationship("BelongsToMany"), Some(RelationshipKind::Collection));
        assert_eq!(classify_relationship("MorphOne"), Some(RelationshipKind::Singular));
        assert_eq!(classify_relationship("MorphMany"), Some(RelationshipKind::Collection));
        assert_eq!(classify_relationship("MorphTo"), Some(RelationshipKind::MorphTo));
        assert_eq!(classify_relationship("MorphToMany"), Some(RelationshipKind::Collection));
        assert_eq!(classify_relationship("HasManyThrough"), Some(RelationshipKind::Collection));
        assert_eq!(classify_relationship("HasOneThrough"), Some(RelationshipKind::Singular));
        assert_eq!(classify_relationship("NotARelationship"), None);
    }

    // ── Accessor helpers ─────────────────────────────────────────────

    #[test]
    fn test_is_legacy_getter_accessor() {
        assert!(is_legacy_getter_accessor("getFullNameAttribute"));
        assert!(is_legacy_getter_accessor("getNameAttribute"));
        assert!(!is_legacy_getter_accessor("getAttribute")); // real method, not accessor
        assert!(!is_legacy_getter_accessor("setNameAttribute")); // setter
        assert!(!is_legacy_getter_accessor("notAnAccessor"));
    }

    #[test]
    fn test_legacy_getter_property_name() {
        assert_eq!(legacy_getter_property_name("getFullNameAttribute"), Some("full_name".to_string()));
        assert_eq!(legacy_getter_property_name("getNameAttribute"), Some("name".to_string()));
        assert_eq!(legacy_getter_property_name("getAttribute"), None);
    }

    #[test]
    fn test_legacy_accessor_method_name() {
        assert_eq!(legacy_accessor_method_name("full_name"), "getFullNameAttribute");
        assert_eq!(legacy_accessor_method_name("name"), "getNameAttribute");
    }

    #[test]
    fn test_accessor_method_candidates() {
        let candidates = accessor_method_candidates("full_name");
        assert_eq!(candidates, vec!["getFullNameAttribute", "fullName"]);
    }

    #[test]
    fn test_modern_accessor_property_name() {
        assert_eq!(modern_accessor_property_name("fullName"), "full_name");
    }

    // ── Scope helpers ────────────────────────────────────────────────

    #[test]
    fn test_is_convention_scope_method() {
        assert!(is_convention_scope_method("scopeActive"));
        assert!(is_convention_scope_method("scopeOfType"));
        assert!(!is_convention_scope_method("scope")); // too short
        assert!(!is_convention_scope_method("notAScope"));
    }

    #[test]
    fn test_scope_name_from_method() {
        assert_eq!(scope_name_from_method("scopeActive"), Some("active".to_string()));
        assert_eq!(scope_name_from_method("scopeOfType"), Some("ofType".to_string()));
        assert_eq!(scope_name_from_method("scope"), None);
    }

    #[test]
    fn test_scope_method_name() {
        assert_eq!(scope_method_name("active"), "scopeActive");
        assert_eq!(scope_method_name("ofType"), "scopeOfType");
    }

    // ── Factory <-> Model naming──────────────────────────────────────

    #[test]
    fn test_factory_to_model_fqn() {
        assert_eq!(factory_to_model_fqn("Database\\Factories\\UserFactory"), Some("App\\Models\\User".to_string()),);
        assert_eq!(
            factory_to_model_fqn("Database\\Factories\\Admin\\SuperUserFactory"),
            Some("App\\Models\\Admin\\SuperUser".to_string()),
        );
        assert_eq!(factory_to_model_fqn("\\Database\\Factories\\UserFactory"), Some("App\\Models\\User".to_string()),);
        // Not a factory name
        assert_eq!(factory_to_model_fqn("App\\Models\\User"), None);
        // Bare "Factory"
        assert_eq!(factory_to_model_fqn("Database\\Factories\\Factory"), None);
    }

    #[test]
    fn test_model_to_factory_fqn() {
        assert_eq!(model_to_factory_fqn("App\\Models\\User"), "Database\\Factories\\UserFactory",);
        assert_eq!(model_to_factory_fqn("App\\Models\\Admin\\User"), "Database\\Factories\\Admin\\UserFactory",);
        assert_eq!(model_to_factory_fqn("\\App\\Models\\User"), "Database\\Factories\\UserFactory",);
        // Bare name
        assert_eq!(model_to_factory_fqn("User"), "Database\\Factories\\UserFactory",);
        // Non-App namespace with Models segment
        assert_eq!(model_to_factory_fqn("MyApp\\Models\\Sub\\User"), "Database\\Factories\\Sub\\UserFactory",);
        // Models-only namespace
        assert_eq!(model_to_factory_fqn("Models\\User"), "Database\\Factories\\UserFactory",);
    }

    // ── Cast type mapping ────────────────────────────────────────────

    #[test]
    fn test_cast_type_to_php_type() {
        // Scalar types
        assert_eq!(cast_type_to_php_type("int"), Some("int"));
        assert_eq!(cast_type_to_php_type("integer"), Some("int"));
        assert_eq!(cast_type_to_php_type("float"), Some("float"));
        assert_eq!(cast_type_to_php_type("double"), Some("float"));
        assert_eq!(cast_type_to_php_type("real"), Some("float"));
        assert_eq!(cast_type_to_php_type("string"), Some("string"));
        assert_eq!(cast_type_to_php_type("bool"), Some("bool"));
        assert_eq!(cast_type_to_php_type("boolean"), Some("bool"));
        assert_eq!(cast_type_to_php_type("array"), Some("array"));
        assert_eq!(cast_type_to_php_type("json"), Some("array"));
        assert_eq!(cast_type_to_php_type("object"), Some("object"));
        assert_eq!(cast_type_to_php_type("collection"), Some("\\Illuminate\\Support\\Collection"));

        // Date/time types (using Illuminate\Support\Carbon instead of Carbon\Carbon)
        assert_eq!(cast_type_to_php_type("date"), Some("\\Illuminate\\Support\\Carbon"));
        assert_eq!(cast_type_to_php_type("datetime"), Some("\\Illuminate\\Support\\Carbon"));
        assert_eq!(cast_type_to_php_type("immutable_date"), Some("\\Carbon\\CarbonImmutable"));
        assert_eq!(cast_type_to_php_type("immutable_datetime"), Some("\\Carbon\\CarbonImmutable"));
        assert_eq!(cast_type_to_php_type("timestamp"), Some("int"));

        // Date with format suffixes
        assert_eq!(cast_type_to_php_type("datetime:Y-m-d"), Some("\\Illuminate\\Support\\Carbon"));
        assert_eq!(cast_type_to_php_type("date:Y-m-d"), Some("\\Illuminate\\Support\\Carbon"));
        assert_eq!(cast_type_to_php_type("immutable_datetime:Y-m-d"), Some("\\Carbon\\CarbonImmutable"));
        assert_eq!(cast_type_to_php_type("immutable_date:Y-m-d"), Some("\\Carbon\\CarbonImmutable"));

        // Decimal → float
        assert_eq!(cast_type_to_php_type("decimal:2"), Some("float"));
        assert_eq!(cast_type_to_php_type("decimal"), Some("float"));

        // Encrypted types
        assert_eq!(cast_type_to_php_type("encrypted"), Some("string"));
        assert_eq!(cast_type_to_php_type("encrypted:array"), Some("array"));
        assert_eq!(cast_type_to_php_type("encrypted:collection"), Some("\\Illuminate\\Support\\Collection"));
        assert_eq!(cast_type_to_php_type("encrypted:object"), Some("object"));

        assert_eq!(cast_type_to_php_type("hashed"), Some("string"));

        // Case-insensitive matching
        assert_eq!(cast_type_to_php_type("Boolean"), Some("bool"));
        assert_eq!(cast_type_to_php_type("DATETIME"), Some("\\Illuminate\\Support\\Carbon"));
        assert_eq!(cast_type_to_php_type("Integer"), Some("int"));

        // Unknown type → None (caller handles class-based casts)
        assert_eq!(cast_type_to_php_type("SomeCustomClass"), None);
    }
}
