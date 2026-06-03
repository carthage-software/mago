use std::collections::BTreeMap;

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

use crate::assertion::Assertion;
use crate::issue::ScanningIssueKind;
use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::class_like::TemplateTypes;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::metadata::version_constraint::VersionConstraint;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::template::GenericTemplate;
use crate::visibility::Visibility;

/// Contains metadata specific to methods defined within classes, interfaces, enums, or traits.
///
/// This complements the more general `FunctionLikeMetadata`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MethodMetadata {
    /// Marks whether this method is declared as `final`, preventing further overriding.
    pub is_final: bool,

    /// Marks whether this method is declared as `abstract`, requiring implementation in subclasses.
    pub is_abstract: bool,

    /// Marks whether this method is declared as `static`, allowing it to be called without an instance.
    pub is_static: bool,

    /// Marks whether this method is a constructor (`__construct`).
    pub is_constructor: bool,

    /// Marks whether this method is declared as `public`, `protected`, or `private`.
    pub visibility: Visibility,

    /// A map of constraints defined by `@where` docblock tags.
    ///
    /// The key is the name of a class-level template parameter (e.g., `T`), and the value
    /// is the `TUnion` type constraint that `T` must satisfy for this specific method
    /// to be considered callable.
    pub where_constraints: WordMap<TypeMetadata>,
}

/// Distinguishes between different kinds of callable constructs in PHP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FunctionLikeKind {
    /// Represents a standard function declared in the global scope or a namespace (`function foo() {}`).
    Function,
    /// Represents a method defined within a class, trait, enum, or interface (`class C { function bar() {} }`).
    Method,
    /// Represents an anonymous function created using `function() {}`.
    Closure,
    /// Represents an arrow function (short closure syntax) introduced in PHP 7.4 (`fn() => ...`).
    ArrowFunction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionLikeMetadata {
    /// The kind of function-like structure this metadata represents.
    pub kind: FunctionLikeKind,

    /// The source code location (span) covering the entire function/method/closure definition.
    /// For closures/arrow functions, this covers the `function(...) { ... }` or `fn(...) => ...` part.
    pub span: Span,

    /// The lookup name of the function or method. For named functions and
    /// methods this is the lowercased identifier (PHP-style case-insensitive
    /// lookup); for closures and arrow functions it is the synthetic
    /// `{closure:path:line:col}` word produced by
    /// [`crate::build_synthetic_name`]. Always set.
    /// Example: `processrequest`, `__construct`, `my_global_func`,
    /// `{closure:src/foo.php:12:5}`.
    pub name: Word,

    /// The original-case name. Matches the source identifier for named
    /// functions/methods, and matches [`Self::name`] verbatim for closures.
    pub original_name: Word,

    /// The specific source code location (span) of the function or method name identifier.
    /// `None` if the function/method has no name (closures/arrow functions).
    pub name_span: Option<Span>,

    /// Ordered list of metadata for each parameter defined in the signature.
    pub parameters: Vec<FunctionLikeParameterMetadata>,

    /// The explicit return type declaration (type hint).
    ///
    /// Example: For `function getName(): string`, this holds metadata for `string`.
    /// `None` if no return type is specified.
    pub return_type_declaration_metadata: Option<TypeMetadata>,

    /// The explicit return type declaration (type hint) or docblock type (`@return`).
    ///
    /// Example: For `function getName(): string`, this holds metadata for `string`,
    /// or for ` /** @return string */ function getName() { .. }`, this holds metadata for `string`.
    /// `None` if neither is specified.
    pub return_type_metadata: Option<TypeMetadata>,

    /// Generic type parameters (templates) defined for the function/method (e.g., `@template T`).
    /// Stores the template name and its constraint (defining entity and bound type).
    /// Example: `{ "T" => (GenericParent::FunctionLike(("funcName", "")), TUnion::object()) }`
    pub template_types: TemplateTypes,

    /// Attributes attached to the function/method/closure declaration (`#[Attribute] function foo() {}`).
    pub attributes: Vec<AttributeMetadata>,

    /// Specific metadata relevant only to methods (visibility, final, static, etc.).
    /// This is `Some` if `kind` is `FunctionLikeKind::Method`, `None` otherwise.
    pub method_metadata: Option<MethodMetadata>,

    /// Contains context information needed for resolving types within this function's scope
    /// (e.g., `use` statements, current namespace, class context). Often populated during analysis.
    pub type_resolution_context: Option<TypeResolutionContext>,

    /// A list of types that this function/method might throw, derived from `@throws` docblock tags
    /// or inferred from `throw` statements within the body.
    pub thrown_types: Vec<TypeMetadata>,

    /// List of issues specifically related to parsing or interpreting this function's docblock.
    pub issues: Vec<Issue>,

    /// Assertions about parameter types or variable types that are guaranteed to be true
    /// *after* this function/method returns normally. From `@psalm-assert`, `@phpstan-assert`, etc.
    /// Maps variable/parameter name to a list of type assertions.
    pub assertions: BTreeMap<Word, Vec<Assertion>>,

    /// Assertions about parameter/variable types that are guaranteed to be true if this
    /// function/method returns `true`. From `@psalm-assert-if-true`, etc.
    pub if_true_assertions: BTreeMap<Word, Vec<Assertion>>,

    /// Assertions about parameter/variable types that are guaranteed to be true if this
    /// function/method returns `false`. From `@psalm-assert-if-false`, etc.
    pub if_false_assertions: BTreeMap<Word, Vec<Assertion>>,

    /// Set when the assertions in `if_true_assertions` / `if_false_assertions` were
    /// auto-inferred from the body rather than declared explicitly via docblock. The
    /// populator uses this to know it can safely override them with assertions
    /// inherited from a parent method, so explicit contracts on a parent always win.
    pub assertions_inferred: bool,

    /// Names of variables this function/method imports from the global scope via a
    /// `global $x;` statement anywhere in its body. Used by the invocation post-processor
    /// to invalidate caller-side narrowings of those globals on every call, since the
    /// callee can reassign them behind the caller's back.
    pub globals_accessed: WordSet,

    /// Tracks whether this function/method has a docblock comment.
    /// Used to determine if docblock inheritance should occur implicitly.
    pub has_docblock: bool,

    pub flags: MetadataFlags,

    /// PHP version range in which this function-like is available, derived
    /// from `Mago\AvailableSince` / `Mago\AvailableUntil` attributes during
    /// scanning.
    pub version_constraint: VersionConstraint,
}

impl FunctionLikeKind {
    /// Checks if this kind represents a class/trait/enum/interface method.
    #[inline]
    #[must_use]
    pub const fn is_method(&self) -> bool {
        matches!(self, Self::Method)
    }

    /// Checks if this kind represents a globally/namespace-scoped function.
    #[inline]
    #[must_use]
    pub const fn is_function(&self) -> bool {
        matches!(self, Self::Function)
    }

    /// Checks if this kind represents an anonymous function (`function() {}`).
    #[inline]
    #[must_use]
    pub const fn is_closure(&self) -> bool {
        matches!(self, Self::Closure)
    }

    /// Checks if this kind represents an arrow function (`fn() => ...`).
    #[inline]
    #[must_use]
    pub const fn is_arrow_function(&self) -> bool {
        matches!(self, Self::ArrowFunction)
    }
}

/// Contains comprehensive metadata for any function-like structure in PHP.
impl FunctionLikeMetadata {
    /// Creates new `FunctionLikeMetadata` with basic information and default flags.
    ///
    /// Pass the lookup `name` (lowercased identifier for named functions/methods,
    /// synthetic `{closure:...}` word for closures) and `original_name` (source
    /// casing for named items, identical to `name` for closures).
    #[must_use]
    pub fn new(kind: FunctionLikeKind, name: Word, original_name: Word, span: Span, flags: MetadataFlags) -> Self {
        let method_metadata = if kind.is_method() { Some(MethodMetadata::default()) } else { None };

        Self {
            kind,
            span,
            flags,
            name,
            original_name,
            name_span: None,
            parameters: vec![],
            return_type_declaration_metadata: None,
            return_type_metadata: None,
            template_types: TemplateTypes::default(),
            attributes: vec![],
            method_metadata,
            type_resolution_context: None,
            thrown_types: vec![],
            assertions: BTreeMap::new(),
            if_true_assertions: BTreeMap::new(),
            if_false_assertions: BTreeMap::new(),
            assertions_inferred: false,
            globals_accessed: WordSet::default(),
            has_docblock: false,
            issues: vec![],
            version_constraint: VersionConstraint::unconstrained(),
        }
    }

    /// Returns `true` when this function-like is available in the given PHP
    /// version.
    #[inline]
    #[must_use]
    pub fn is_available_in_version(&self, version: PHPVersion) -> bool {
        self.version_constraint.allows_version(version)
    }

    /// Returns `true` when this function-like is available across the entire
    /// supplied [`PHPVersionRange`].
    #[inline]
    #[must_use]
    pub fn is_available_in_version_range(&self, range: PHPVersionRange) -> bool {
        self.version_constraint.allows_version_range(range)
    }

    /// Returns the kind of function-like (Function, Method, Closure, `ArrowFunction`).
    #[inline]
    #[must_use]
    pub fn get_kind(&self) -> FunctionLikeKind {
        self.kind
    }

    /// Returns a mutable slice of the parameter metadata.
    #[inline]
    pub fn get_parameters_mut(&mut self) -> &mut [FunctionLikeParameterMetadata] {
        &mut self.parameters
    }

    /// Returns a reference to specific parameter metadata by name, if it exists.
    #[inline]
    #[must_use]
    pub fn get_parameter(&self, name: Word) -> Option<&FunctionLikeParameterMetadata> {
        self.parameters.iter().find(|parameter| parameter.get_name().0 == name)
    }

    /// Returns a mutable reference to specific parameter metadata by name, if it exists.
    #[inline]
    pub fn get_parameter_mut(&mut self, name: Word) -> Option<&mut FunctionLikeParameterMetadata> {
        self.parameters.iter_mut().find(|parameter| parameter.get_name().0 == name)
    }

    /// Returns a mutable reference to the template type parameters.
    #[inline]
    pub fn get_template_types_mut(&mut self) -> &mut TemplateTypes {
        &mut self.template_types
    }

    /// Returns a slice of the attributes.
    #[inline]
    #[must_use]
    pub fn get_attributes(&self) -> &[AttributeMetadata] {
        &self.attributes
    }

    /// Returns a mutable reference to the method-specific info, if this is a method.
    #[inline]
    pub fn get_method_metadata_mut(&mut self) -> Option<&mut MethodMetadata> {
        self.method_metadata.as_mut()
    }

    /// Returns a mutable slice of docblock issues.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }

    /// Sets the parameters, replacing existing ones.
    #[inline]
    pub fn set_parameters(&mut self, parameters: impl IntoIterator<Item = FunctionLikeParameterMetadata>) {
        self.parameters = parameters.into_iter().collect();
    }

    /// Returns a new instance with the parameters replaced.
    #[inline]
    #[must_use]
    pub fn with_parameters(mut self, parameters: impl IntoIterator<Item = FunctionLikeParameterMetadata>) -> Self {
        self.set_parameters(parameters);
        self
    }

    #[inline]
    pub fn set_return_type_metadata(&mut self, return_type: Option<TypeMetadata>) {
        self.return_type_metadata = return_type;
    }

    #[inline]
    pub fn set_return_type_declaration_metadata(&mut self, return_type: Option<TypeMetadata>) {
        if self.return_type_metadata.is_none() {
            self.return_type_metadata.clone_from(&return_type);
        }

        self.return_type_declaration_metadata = return_type;
    }

    /// Adds a single template type definition.
    #[inline]
    pub fn add_template_type(&mut self, name: Word, constraint: GenericTemplate) {
        self.template_types.insert(name, constraint);
    }

    /// Applies a patch to this entry in place, refining type information only.
    ///
    /// Refined fields — return type, per-parameter types, `@param-out`, default-value types,
    /// `@throws`, `@template`, and assertions — are each copied only when the patch specifies
    /// them, so a sparsely-typed patch never erases richer existing information. Structural
    /// identity (span, file, kind, parameter count, visibility) is left to whichever non-patch
    /// source declared the symbol. Diagnostics are appended to `patch.issues`; the full set of
    /// patching rules is documented in the `[source]` patching guide.
    pub fn apply_patch(&mut self, patch: &mut FunctionLikeMetadata) {
        // A parameter count or name mismatch means types cannot be mapped positionally;
        // reject the patch wholesale rather than risk a silent misapply.
        if self.report_parameter_count_mismatch(patch) || self.report_parameter_name_mismatch(patch) {
            return;
        }

        self.report_method_structural_mismatch(patch);

        self.patch_return_type(patch);
        self.patch_parameters(patch);
        self.patch_templates(patch);
        self.patch_throws_and_assertions(patch);
    }

    /// Reports a parameter count mismatch between the patch and the original.
    ///
    /// Returns `true` when the counts differ, in which case the patch must be rejected
    /// wholesale — there is no sensible positional mapping.
    fn report_parameter_count_mismatch(&self, patch: &mut FunctionLikeMetadata) -> bool {
        if patch.parameters.len() == self.parameters.len() {
            return false;
        }

        patch.issues.push(
            Issue::error(format!(
                "Patch for `{}` declares {} parameter(s) but the original has {}; \
                 patches cannot change the number of parameters.",
                patch.original_name,
                patch.parameters.len(),
                self.parameters.len(),
            ))
            .with_code(ScanningIssueKind::PatchFunctionParameterMismatch)
            .with_annotation(Annotation::primary(patch.span))
            .with_help(format!(
                "Declare exactly {} parameter(s) in the patch to match the original signature. \
                 Patches refine parameter types only and cannot add or remove parameters.",
                self.parameters.len(),
            )),
        );

        true
    }

    /// Reports a parameter name mismatch at any position.
    ///
    /// Types are refined by position, so a name mismatch (a wrong order, or a patch drifted
    /// out of sync with the vendor code) would apply them to the wrong parameter. Returns
    /// `true` on the first mismatch so the patch is rejected wholesale.
    fn report_parameter_name_mismatch(&self, patch: &mut FunctionLikeMetadata) -> bool {
        for (index, (base_param, patch_param)) in self.parameters.iter().zip(patch.parameters.iter()).enumerate() {
            if base_param.name != patch_param.name {
                patch.issues.push(
                    Issue::error(format!(
                        "Patch for `{}` names parameter #{} `{}` but the original declares `{}` there; \
                         patches refine parameter types by position and the names must match.",
                        patch.original_name,
                        index + 1,
                        patch_param.name.0,
                        base_param.name.0,
                    ))
                    .with_code(ScanningIssueKind::PatchFunctionParameterNameMismatch)
                    .with_annotation(Annotation::primary(patch_param.name_span))
                    .with_help(
                        "Declare the patch's parameters with the same names in the same order as the \
                         original signature so their types are applied to the intended parameters.",
                    ),
                );
                return true;
            }
        }

        false
    }

    /// Reports structural method-attribute mismatches.
    ///
    /// For methods, visibility, static, and removing final are all structural changes a patch
    /// may not make. Adding final is allowed. This is reported as an error but does not abort
    /// the patch — type annotations are still applied.
    ///
    /// `abstract` is deliberately excluded: it is implied by writing the method with a trailing
    /// `;` instead of a `{}` body — the idiomatic form for a signature-only type patch — rather
    /// than being an explicit modifier the author chose. The patch never changes the original's
    /// abstractness either way, so a difference is harmless and would only produce a spurious
    /// error for the natural patch syntax.
    fn report_method_structural_mismatch(&self, patch: &mut FunctionLikeMetadata) {
        let (Some(patch_m), Some(base_m)) = (&patch.method_metadata, &self.method_metadata) else {
            return;
        };

        let visibility_mismatch = patch_m.visibility != base_m.visibility;
        let static_mismatch = patch_m.is_static != base_m.is_static;
        let final_removed = base_m.is_final && !patch_m.is_final;

        if visibility_mismatch || static_mismatch || final_removed {
            patch.issues.push(
                Issue::error(format!(
                    "Patch for `{}` declares structural attributes (visibility, static, or \
                     removing final) that differ from the original; only type annotations are applied.",
                    patch.original_name,
                ))
                .with_code(ScanningIssueKind::PatchMethodStructuralMismatch)
                .with_annotation(Annotation::primary(patch.span))
                .with_help(
                    "Declare the method with the same visibility and the same `static` and \
                     `final` modifiers as the original (adding `final` is allowed); \
                     a patch may only refine the method's types.",
                ),
            );
        }
    }

    /// Refines the return type (declaration and docblock) when the patch specifies it.
    fn patch_return_type(&mut self, patch: &FunctionLikeMetadata) {
        if let Some(decl) = &patch.return_type_declaration_metadata {
            self.return_type_declaration_metadata = Some(decl.clone());
        }
        if let Some(ty) = &patch.return_type_metadata {
            self.return_type_metadata = Some(ty.clone());
        }
    }

    /// Refines per-parameter types by position; each field is copied only when the patch
    /// specifies it, so a sparsely-typed patch does not erase richer existing information.
    fn patch_parameters(&mut self, patch: &FunctionLikeMetadata) {
        for (slot, replacement) in self.parameters.iter_mut().zip(patch.parameters.iter()) {
            if let Some(decl) = &replacement.type_declaration_metadata {
                slot.type_declaration_metadata = Some(decl.clone());
            }
            if let Some(ty) = &replacement.type_metadata {
                slot.type_metadata = Some(ty.clone());
            }
            if let Some(out) = &replacement.out_type {
                slot.out_type = Some(out.clone());
            }
            if let Some(default) = &replacement.default_type {
                slot.default_type = Some(default.clone());
            }
        }
    }

    /// Merges `@template` declarations from the patch.
    fn patch_templates(&mut self, patch: &FunctionLikeMetadata) {
        if !patch.template_types.is_empty() {
            self.template_types.extend(patch.template_types.iter().map(|(k, v)| (*k, v.clone())));
        }
    }

    /// Replaces `@throws` and `@psalm-assert`-style annotations when the patch declares any.
    fn patch_throws_and_assertions(&mut self, patch: &FunctionLikeMetadata) {
        if !patch.thrown_types.is_empty() {
            self.thrown_types.clone_from(&patch.thrown_types);
        }
        if !patch.assertions.is_empty() {
            self.assertions = patch.assertions.clone();
        }
        if !patch.if_true_assertions.is_empty() {
            self.if_true_assertions = patch.if_true_assertions.clone();
        }
        if !patch.if_false_assertions.is_empty() {
            self.if_false_assertions = patch.if_false_assertions.clone();
        }
    }
}
