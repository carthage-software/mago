use mago_span::HasSpan;
use mago_span::Span;
use mago_word::Word;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;
use crate::ttype::atomic::TAtomic;

/// Describes when a callable passed to a parameter is invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallableInvocationKind {
    /// The callable is invoked before the function returns.
    Immediately,
    /// The callable may be stored or invoked after the function returns.
    Later,
}

/// A statically known throw from a catch body, with nested catches that may handle it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallableExceptionRethrow {
    /// The exception type thrown by the catch body.
    pub exception_type: Word,
    /// Exception types from nested catch clauses surrounding this throw.
    #[cfg_attr(feature = "serde", serde(default))]
    pub caught_types: Vec<Word>,
    /// Whether the thrown type came from a caught exception variable.
    #[cfg_attr(feature = "serde", serde(default))]
    pub caught_variable: bool,
}

/// One catch arm surrounding an immediately invoked callable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallableExceptionCatchArm {
    /// Exception types matched by this catch arm.
    pub exception_types: Vec<Word>,
    /// Whether the original callback exception may be rethrown through an unknown expression.
    pub rethrows: bool,
    /// Statically known exception types thrown by this catch arm.
    #[cfg_attr(feature = "serde", serde(default))]
    pub rethrows_types: Vec<Word>,
    /// Types of caught exceptions rethrown by this catch arm.
    #[cfg_attr(feature = "serde", serde(default))]
    pub rethrows_caught_types: Vec<Word>,
    /// Statically known throws that may be caught by nested catch clauses.
    #[cfg_attr(feature = "serde", serde(default))]
    pub rethrows_with_catches: Vec<CallableExceptionRethrow>,
}

/// The ordered catch arms of one try statement surrounding a callback invocation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallableExceptionCatchGroup {
    /// Catch arms in source order.
    pub arms: Vec<CallableExceptionCatchArm>,
    /// Whether a `finally` block replaces exceptions from the associated try/catch paths.
    #[cfg_attr(feature = "serde", serde(default))]
    pub overrides_pending_exceptions: bool,
}

/// Nested try/catch handling surrounding one invocation of an immediately invoked callable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallableExceptionCatchSummary {
    /// Catch groups ordered from the innermost try statement to the outermost.
    pub catch_groups: Vec<CallableExceptionCatchGroup>,
}

/// Contains metadata associated with a single parameter within a function, method, or closure signature.
///
/// This captures details like the parameter's name, type hint, attributes, default value,
/// pass-by-reference status, variadic nature, and other PHP features like property promotion.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct FunctionLikeParameterMetadata {
    /// Attributes attached to the parameter declaration.
    pub attributes: Vec<AttributeMetadata>,

    /// The identifier (name) of the parameter, including the leading '$'.
    pub name: VariableIdentifier,

    /// The native type declaration from the function signature.
    ///
    /// This is the type hint specified in the code (e.g., `string $name`), not from docblocks.
    /// Can be `None` if no type hint is specified in the signature.
    pub type_declaration_metadata: Option<TypeMetadata>,

    /// The explicit type declaration (type hint) or docblock type (`@param`).
    ///
    /// If there's a docblock `@param` annotation, this will contain that type (with `from_docblock=true`).
    /// Otherwise, this will be the same as `type_declaration_metadata`.
    /// Can be `None` if no type is specified.
    pub type_metadata: Option<TypeMetadata>,

    /// The type specified by a `@param-out` docblock tag.
    ///
    /// This indicates the expected type of a pass-by-reference parameter *after* the function executes.
    pub out_type: Option<TypeMetadata>,

    /// The type specified by a `@param-closure-this` docblock tag.
    ///
    /// When this parameter receives a closure, that closure is invoked with `$this` bound to an
    /// instance of this type. A closure literal passed here is analyzed with `$this` set accordingly.
    pub closure_this_type: Option<TypeMetadata>,

    /// Whether a callable passed to this parameter is invoked immediately or later.
    #[cfg_attr(feature = "serde", serde(default))]
    pub callable_invocation_kind: Option<CallableInvocationKind>,

    /// Catch and conservative escape summaries for each statically recognized callback invocation in the function body.
    #[cfg_attr(feature = "serde", serde(default))]
    pub callable_exception_catches: Option<Vec<CallableExceptionCatchSummary>>,

    /// Namespaced function names whose absence is required for the corresponding catch summary to apply.
    /// Each index refers to an entry in `callable_exception_catches`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub callable_exception_catch_global_fallbacks: Vec<(usize, Word)>,

    /// The inferred type of the parameter's default value, if `has_default` is true and the
    /// type could be determined.
    ///
    /// `None` if there is no default or the default value's type couldn't be inferred.
    pub default_type: Option<TypeMetadata>,

    /// The source code location (span) covering the entire parameter declaration.
    pub span: Span,

    /// The specific source code location (span) of the parameter's name identifier.
    pub name_span: Span,

    /// Flags indicating various properties of the parameter.
    pub flags: MetadataFlags,
}

/// Contains metadata associated with a single parameter within a function, method, or closure signature.
///
/// This captures details like the parameter's name, type hint, attributes, default value,
/// pass-by-reference status, variadic nature, and other PHP features like property promotion.
impl FunctionLikeParameterMetadata {
    /// Creates new `FunctionLikeParameterMetadata` for a basic parameter.
    /// Initializes most flags to false and optional fields to None.
    ///
    /// # Arguments
    ///
    /// * `name`: The identifier (name) of the parameter (e.g., `$userId`).
    /// * `span`: The source code location covering the entire parameter declaration.
    /// * `name_span`: The source code location of the parameter's name identifier (`$userId`).
    #[must_use]
    pub fn new(name: VariableIdentifier, span: Span, name_span: Span, flags: MetadataFlags) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            flags,
            span,
            name_span,
            type_declaration_metadata: None,
            type_metadata: None,
            out_type: None,
            closure_this_type: None,
            callable_invocation_kind: None,
            callable_exception_catches: None,
            callable_exception_catch_global_fallbacks: Vec::new(),
            default_type: None,
        }
    }

    /// Returns a reference to the parameter's name identifier (e.g., `$userId`).
    #[inline]
    #[must_use]
    pub fn get_name(&self) -> &VariableIdentifier {
        &self.name
    }

    /// Returns the span covering the entire parameter declaration.
    #[inline]
    #[must_use]
    pub fn get_span(&self) -> Span {
        self.span
    }

    /// Returns the span covering the parameter's name identifier.
    #[inline]
    #[must_use]
    pub fn get_name_span(&self) -> Span {
        self.name_span
    }

    /// Returns a reference to the parameter's type metadata (effective type with docblock).
    #[inline]
    #[must_use]
    pub fn get_type_metadata(&self) -> Option<&TypeMetadata> {
        self.type_metadata.as_ref()
    }

    /// Returns a reference to the parameter's native type declaration metadata.
    #[inline]
    #[must_use]
    pub fn get_type_declaration_metadata(&self) -> Option<&TypeMetadata> {
        self.type_declaration_metadata.as_ref()
    }

    /// Returns a reference to the inferred type of the default value, if known.
    #[inline]
    #[must_use]
    pub fn get_default_type(&self) -> Option<&TypeMetadata> {
        self.default_type.as_ref()
    }

    /// Sets the attributes, replacing any existing ones.
    pub fn set_attributes(&mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) {
        self.attributes = attributes.into_iter().collect();
    }

    /// Returns a new instance with the attributes replaced.
    #[must_use]
    pub fn with_attributes(mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) -> Self {
        self.set_attributes(attributes);
        self
    }

    /// Sets the parameter's type metadata (effective type with docblock).
    #[inline]
    pub fn set_type_metadata(&mut self, type_metadata: Option<TypeMetadata>) {
        self.type_metadata = type_metadata;
    }

    /// Sets the parameter's native type declaration metadata.
    ///
    /// If `type_metadata` is not set, it will be initialized with the same value.
    #[inline]
    pub fn set_type_declaration_metadata(&mut self, type_declaration: Option<TypeMetadata>) {
        if self.type_metadata.is_none() {
            self.type_metadata.clone_from(&type_declaration);
        }

        self.type_declaration_metadata = type_declaration;
    }

    /// Adds `null` to a docblock-only type when the parameter defaults to `null`.
    ///
    /// `@param string $x` with `$x = null` accepts `null`, the same way PHP makes a native
    /// `string $x = null` nullable. This does not depend on the PHP version, because an
    /// untyped parameter can default to `null` on every version.
    ///
    /// A parameter with a native type is left alone. The scanner makes that native type
    /// nullable where PHP does, and a docblock type keeps the `null` when merged with it.
    pub fn add_null_from_null_default(&mut self) {
        if self.type_declaration_metadata.is_some()
            || !self.default_type.as_ref().is_some_and(|default_type| default_type.type_union.is_null())
        {
            return;
        }

        let Some(type_metadata) = self.type_metadata.as_mut() else {
            return;
        };

        if type_metadata.type_union.accepts_null() || type_metadata.type_union.types.iter().any(TAtomic::is_conditional)
        {
            return;
        }

        type_metadata.type_union = type_metadata.type_union.clone().as_nullable();
    }
}

impl HasSpan for FunctionLikeParameterMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
