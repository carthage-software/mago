/// Configuration for the lowering pass that controls how PHP static-analysis scoping rules are
/// modeled in the produced IR.
///
/// Every option defaults to `true`, which reproduces the behavior of the established CST-based
/// scanner/analyzer. Setting an option to `false` selects the stricter, lexically-scoped
/// interpretation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LowerSettings {
    /// When a class imports a type alias (`@import-type X from Y`), treat that alias as if it were
    /// declared on the importing class too, so it can be re-imported from it (`@import-type X from
    /// <importer>`). This matches how Psalm/PHPStan model imported aliases.
    pub re_export_type_aliases: bool,

    /// Make every `@type`/`@import-type` alias visible program-wide rather than only within the
    /// class that declares it. A bare reference to an alias declared on another class then resolves
    /// to that alias without an explicit import.
    pub program_wide_type_aliases: bool,

    /// Keep a class's `@template` parameters in scope inside its `static` methods (and the closures
    /// nested within them). Lexically, a static method has no access to the instance-level type
    /// parameters, but the established analyzer keeps them visible.
    pub inherit_static_templates: bool,

    /// Interpret the `#[Deprecated]` attribute as a deprecation marker, setting the deprecated flag
    /// on the annotated symbol without the consumer having to inspect attributes.
    pub lower_deprecation_attributes: bool,

    /// Interpret the `Mago\Available*` / `Mago\Optional*` / `Mago\Required*` attributes into a
    /// structured version constraint on the annotated symbol.
    pub lower_availability_attributes: bool,

    /// How a `define('NAME', value)` call with a literal name is represented in the IR.
    pub define_constant_lowering: DefineConstantLowering,

    /// Infer `@assert-if-true`/`@assert-if-false` annotations for a function-like that has no explicit
    /// assertions and whose body is a single boolean `return` (or arrow expression) over a parameter,
    /// e.g. `return $x !== null;` or `return $x instanceof Foo;`. Matches the established scanner.
    pub infer_assertions: bool,
}

/// Controls whether (and how) a `define('NAME', value)` call is normalized into a `const`
/// definition in the IR.
///
/// Only the literal-name form is recognized; dynamic `define()` calls are always left as calls.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DefineConstantLowering {
    /// Replace the `define()` call with a `const NAME = value;` definition.
    Statement,
    /// Emit a `const NAME = value;` definition and also keep the original `define()` call.
    StatementAndCall,
    /// Leave the `define()` call untouched.
    Disabled,
}

impl Default for LowerSettings {
    fn default() -> Self {
        Self {
            re_export_type_aliases: true,
            program_wide_type_aliases: true,
            inherit_static_templates: true,
            lower_deprecation_attributes: true,
            lower_availability_attributes: true,
            define_constant_lowering: DefineConstantLowering::Statement,
            infer_assertions: true,
        }
    }
}
