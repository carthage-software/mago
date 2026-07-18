//! Per-call context for [`expand`](super::expand_with): contextual
//! class names (`self` / `static` / `parent`) and per-stage feature
//! toggles that control which atom-resolution rules fire.

use crate::path::Path;

/// Caller-controlled options for [`expand_with`](super::expand_with).
///
/// Each toggleable expansion stage has its own field; the
/// analyzer turns stages on or off depending on what the calling site
/// needs. Defaults: class constants, global constants, and aliases
/// evaluate; conditionals, default-fill, constraint substitution, and
/// final-function collapse stay off.
///
/// # Field summary
///
/// - **`self_class` / `static_class` / `parent_class`** - class names
///   PHP's `self`, `static`, `$this`, `parent` resolve to in the current
///   scope. `None` keeps the keyword atom intact.
/// - **`evaluate_class_constants`** - gate `Foo::CONST` resolution to the
///   constant's declared type.
/// - **`evaluate_global_constants`** - gate `\GLOBAL_NAME` resolution.
/// - **`evaluate_aliases`** - gate `@type Foo = ...` alias body resolution.
/// - **`evaluate_conditional`** - when true, decide `T is U ? A : B` via
///   the lattice.
/// - **`fill_template_defaults`** - when true, generic class references
///   missing type arguments have their slots filled with each parameter's
///   declared upper bound (or `mixed`).
/// - **`substitute_template_constraints`** - when true, free
///   `GenericParameter T` atoms are replaced by `T`'s constraint.
///   Must stay off in any context comparing two template parameters
///   for identity.
/// - **`function_is_final`** - when true, the `static` modality on
///   named-object atoms is dropped even without a `static_class`
///   binding (the function is `final`, so `static` cannot widen).
#[derive(Debug, Clone, Copy)]
pub struct ExpansionContext<'arena> {
    pub self_class: Option<Path<'arena>>,
    pub static_class: Option<Path<'arena>>,
    pub parent_class: Option<Path<'arena>>,
    pub evaluate_class_constants: bool,
    pub evaluate_global_constants: bool,
    pub evaluate_aliases: bool,
    pub evaluate_conditional: bool,
    pub fill_template_defaults: bool,
    pub substitute_template_constraints: bool,
    pub function_is_final: bool,
}

impl Default for ExpansionContext<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            self_class: None,
            static_class: None,
            parent_class: None,
            evaluate_class_constants: true,
            evaluate_global_constants: true,
            evaluate_aliases: true,
            evaluate_conditional: false,
            fill_template_defaults: false,
            substitute_template_constraints: false,
            function_is_final: false,
        }
    }
}

impl<'arena> ExpansionContext<'arena> {
    #[must_use]
    #[inline]
    pub const fn with_self_class(mut self, class: Path<'arena>) -> Self {
        self.self_class = Some(class);
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_static_class(mut self, class: Path<'arena>) -> Self {
        self.static_class = Some(class);
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_parent_class(mut self, class: Path<'arena>) -> Self {
        self.parent_class = Some(class);
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_evaluate_class_constants(mut self, on: bool) -> Self {
        self.evaluate_class_constants = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_evaluate_global_constants(mut self, on: bool) -> Self {
        self.evaluate_global_constants = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_evaluate_aliases(mut self, on: bool) -> Self {
        self.evaluate_aliases = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_evaluate_conditional(mut self, on: bool) -> Self {
        self.evaluate_conditional = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_fill_template_defaults(mut self, on: bool) -> Self {
        self.fill_template_defaults = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_substitute_template_constraints(mut self, on: bool) -> Self {
        self.substitute_template_constraints = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_function_is_final(mut self, on: bool) -> Self {
        self.function_is_final = on;
        self
    }
}
