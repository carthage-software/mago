use mago_atom::AtomSet;
use mago_php_version::PHPVersion;

/// Configuration settings that control the behavior of the Mago analyzer.
///
/// This struct allows you to enable/disable specific checks, suppress categories of issues,
/// and tune the analyzer's performance and strictness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// The target PHP version for the analysis.
    pub version: PHPVersion,

    /// Find and report expressions whose results are not used (e.g., `$a + $b;`). Defaults to `false`.
    pub find_unused_expressions: bool,

    /// Find and report unused definitions (e.g., private methods that are never called). Defaults to `false`.
    pub find_unused_definitions: bool,

    /// Analyze code that appears to be unreachable. Defaults to `false`.
    pub analyze_dead_code: bool,

    /// Track the literal values of class properties when they are assigned.
    /// This improves type inference but may increase memory usage. Defaults to `true`.
    pub memoize_properties: bool,

    /// Allow accessing array keys that may not be defined without reporting an issue. Defaults to `true`.
    pub allow_possibly_undefined_array_keys: bool,

    /// Enable checking for unhandled thrown exceptions.
    ///
    /// When `true`, the analyzer will report any exception that is thrown but not caught
    /// in a `try-catch` block or documented in a `@throws` tag.
    ///
    /// This check is disabled by default (`false`) as it can be computationally expensive.
    pub check_throws: bool,

    /// Exceptions to ignore including all subclasses (hierarchy-aware).
    ///
    /// When an exception class is in this set, any exception of that class or any of its
    /// subclasses will be ignored during `check_throws` analysis.
    ///
    /// For example, adding `LogicException` will ignore `LogicException`, `InvalidArgumentException`,
    /// `OutOfBoundsException`, and all other subclasses.
    pub unchecked_exceptions: AtomSet,

    /// Exceptions to ignore (exact class match only, not subclasses).
    ///
    /// When an exception class is in this set, only that exact class will be ignored
    /// during `check_throws` analysis. Parent classes and subclasses are not affected.
    pub unchecked_exception_classes: AtomSet,

    /// Check for missing `#[Override]` attributes on overriding methods.
    ///
    /// When enabled, the analyzer reports methods that override a parent method without
    /// the `#[Override]` attribute (PHP 8.3+).
    ///
    /// Defaults to `true`.
    pub check_missing_override: bool,

    /// Find and report unused function/method parameters.
    ///
    /// When enabled, the analyzer reports parameters that are declared but never used
    /// within the function body.
    ///
    /// Defaults to `true`.
    pub find_unused_parameters: bool,

    /// Enforce strict checks when accessing list elements by index.
    ///
    /// When `true`, the analyzer requires that any integer used to access a `list`
    /// element is provably non-negative (e.g., of type `int<0, max>`). This helps
    /// prevent potential runtime errors from using a negative index.
    ///
    /// When `false` (the default), any `int` is permitted as an index, offering
    /// more flexibility at the cost of type safety.
    pub strict_list_index_checks: bool,

    /// Disable comparisons to boolean literals (`true`/`false`).
    ///
    /// When enabled, comparisons to boolean literals will not be reported as issues.
    ///
    /// Defaults to `false`.
    pub no_boolean_literal_comparison: bool,

    /// Check for missing type hints on parameters, properties, and return types.
    ///
    /// When enabled, the analyzer will report warnings for function parameters, class properties,
    /// and function return types that lack explicit type declarations. The analyzer uses its
    /// type system knowledge to avoid false positives - for instance, it won't require a type hint
    /// on a property if adding one would conflict with a parent class or trait that has no type hint.
    ///
    /// Defaults to `false`.
    pub check_missing_type_hints: bool,

    /// Check for missing type hints (both parameters and return types) in closures when `check_missing_type_hints` is enabled.
    ///
    /// When `true`, closures (anonymous functions declared with `function() {}`) will be
    /// checked for missing type hints. When `false`, closures are ignored, which is useful
    /// because closures often rely on type inference.
    ///
    /// Defaults to `false`.
    pub check_closure_missing_type_hints: bool,

    /// Check for missing type hints (both parameters and return types) in arrow functions when `check_missing_type_hints` is enabled.
    ///
    /// When `true`, arrow functions (declared with `fn() => ...`) will be checked for missing
    /// type hints. When `false`, arrow functions are ignored, which is useful because arrow
    /// functions often rely on type inference and are typically short, making types obvious.
    ///
    /// Defaults to `false`.
    pub check_arrow_function_missing_type_hints: bool,

    /// Register superglobals (e.g., `$_GET`, `$_POST`, `$_SERVER`) in the analysis context.
    ///
    /// If disabled, super globals won't be available unless explicitly imported using
    /// the `global` keyword.
    ///
    /// Defaults to `true`.
    pub register_super_globals: bool,

    /// Enable colored output in terminal environments that support it. Defaults to `true`.
    ///
    /// This setting is primarily used for enabling/disabling colored diffs in
    /// issue reports.
    pub use_colors: bool,

    /// **Internal use only.**
    ///
    /// Enables a diffing mode for incremental analysis, used by integrations like LSPs.
    /// This avoids re-analyzing unchanged code in the same session. Defaults to `false`.
    pub diff: bool,

    /// Trust symbol existence checks to narrow types.
    ///
    /// When enabled, conditional checks like `method_exists()`, `property_exists()`,
    /// `function_exists()`, and `defined()` will narrow the type within the conditional block,
    /// suppressing errors for symbols that are verified to exist at runtime.
    ///
    /// When disabled, these checks are ignored and the analyzer requires explicit type hints,
    /// which is stricter but may produce more false positives for dynamic code.
    ///
    /// Defaults to `true`.
    pub trust_existence_checks: bool,

    /// Method names treated as class initializers (like `__construct`).
    ///
    /// Properties initialized in these methods count as "definitely initialized"
    /// just like in the constructor. This is useful for frameworks that use
    /// lifecycle methods like `PHPUnit`'s `setUp()` or framework `boot()` methods.
    ///
    /// Example: `["setUp", "initialize", "boot"]`
    ///
    /// Defaults to empty (no additional initializers).
    pub class_initializers: AtomSet,

    /// Enable property initialization checking (`missing-constructor`, `uninitialized-property`).
    ///
    /// When `false`, disables both `missing-constructor` and `uninitialized-property` issues
    /// entirely. This is useful for projects that prefer to rely on runtime errors for
    /// property initialization.
    ///
    /// Defaults to `false`.
    pub check_property_initialization: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new(PHPVersion::LATEST)
    }
}

impl Settings {
    pub fn new(version: PHPVersion) -> Self {
        Self {
            version,
            find_unused_expressions: false,
            find_unused_definitions: false,
            analyze_dead_code: false,
            memoize_properties: true,
            allow_possibly_undefined_array_keys: true,
            check_throws: false,
            unchecked_exceptions: AtomSet::default(),
            unchecked_exception_classes: AtomSet::default(),
            use_colors: true,
            check_missing_override: false,
            find_unused_parameters: false,
            strict_list_index_checks: false,
            no_boolean_literal_comparison: false,
            check_missing_type_hints: false,
            check_closure_missing_type_hints: false,
            check_arrow_function_missing_type_hints: false,
            register_super_globals: true,
            diff: false,
            trust_existence_checks: true,
            class_initializers: AtomSet::default(),
            check_property_initialization: false,
        }
    }
}
