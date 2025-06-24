use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;

/// Describes the origin and nature of a class name resolution.
///
/// This enum provides a clear, type-safe alternative to using multiple boolean flags,
/// capturing all possible ways a class name can be identified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResolutionOrigin {
    /// The resolution is definitively invalid (e.g., from a numeric literal).
    Invalid,
    /// Resolved from a direct identifier (`Foo`), or the `self` or `parent` keywords.
    Named { is_parent: bool, is_self: bool },
    /// Resolved from the `static` keyword. `can_extend` is true if the class is not final.
    Static { can_extend: bool },
    /// Resolved from an object instance (e.g., `$obj` in `$obj::foo()`). `is_this` is true for `$this`.
    Object { is_this: bool },
    /// Resolved from a literal string that is known to be a class name (e.g., `MyClass::class`).
    LiteralClassString,
    /// Resolved from a generic `class-string` type where the concrete class is unknown.
    GenericClassString,
    /// Resolved from a generic `string` type, which may or may not be a valid class name at runtime.
    GenericString,
    /// Resolved from an `object` type where the specific class is not known.
    GenericObject,
    /// Resolved from a `mixed` type, which could potentially be a class name.
    Mixed,
    /// Resolved from an `any` type.
    Any,
}

/// Represents the result of resolving an expression that is expected to be a class name.
///
/// This struct is used to analyze expressions in contexts like `new <expr>`, `<expr>::method()`,
/// `<expr>::$property`, or `<expr>::CONSTANT`, where `<expr>` must resolve to a valid class identifier.
/// It carries the resolved fully-qualified class name (if known) and metadata about how the
/// resolution was performed via `ResolutionOrigin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct ResolvedClassname {
    /// The fully qualified class name (`StringIdentifier`) if a specific class could be identified.
    /// This is `None` for ambiguous or generic types like `object`, `class-string`, or `mixed`.
    pub fq_class_id: Option<StringIdentifier>,
    /// Describes how the class name was resolved.
    pub origin: ResolutionOrigin,
}

impl ResolvedClassname {
    /// Creates a new `ResolvedClassname`.
    #[inline]
    const fn new(fq_class_id: Option<StringIdentifier>, origin: ResolutionOrigin) -> Self {
        Self { fq_class_id, origin }
    }

    /// Creates a `ResolvedClassname` that is definitively invalid.
    #[inline]
    const fn invalid() -> Self {
        Self { fq_class_id: None, origin: ResolutionOrigin::Invalid }
    }

    /// Creates a `ResolvedClassname` that is definitively invalid.
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Invalid)
    }

    /// Checks if the resolution might result in an invalid class name at runtime.
    ///
    /// This is true for vague types like a generic `string`, `mixed`, or `any`, where the
    /// actual value is not guaranteed to be a valid class name. It is also true if the
    /// resolution is known to be `Invalid`.
    pub fn is_possibly_invalid(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Mixed | ResolutionOrigin::Any | ResolutionOrigin::Invalid)
    }

    /// Checks if the class name is resolved from the `static` keyword.
    pub fn is_static(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Static { .. })
    }

    /// Checks if the class name is resolved from the `static` keyword and the class is not final,
    pub fn can_extend_static(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Static { can_extend: true })
    }

    /// Checks if the resolution is from a generic `class-string` type,
    /// which means it could be any class name that is a valid `class-string`.
    pub fn is_from_class_string(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::GenericClassString | ResolutionOrigin::LiteralClassString)
    }

    /// Checks if the resolution is from a generic `object` type,
    /// which means it could be any object type.
    pub fn is_from_generic_object(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::GenericObject)
    }

    /// Checks if the resolution is a class instance (e.g., from an object or `static`).
    pub fn is_object_instance(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Object { .. } | ResolutionOrigin::Static { .. })
    }

    /// Checks if the resolution is from a `parent` keyword.
    pub fn is_from_parent(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Named { is_parent: true, .. })
    }
}

/// Resolves an AST `Expression` to one or more `ResolvedClassname` instances.
///
/// This function analyzes various forms of expressions that can represent a class name
/// in PHP. For expressions that can resolve to a union of types (e.g., a variable
/// with type `class-string<A>|class-string<B>`), this function will return a vector
/// containing a `ResolvedClassname` for each possible resolution.
///
/// It reports errors for syntactically invalid uses (e.g., `self` outside a class)
/// or when an expression's type is fundamentally incompatible with being a class name.
pub fn resolve_classnames_from_expression<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    class_expression: &Expression,
    class_is_analyzed: bool,
) -> Result<Vec<ResolvedClassname>, AnalysisError> {
    let mut possible_types = vec![];
    match class_expression.unparenthesized() {
        Expression::Identifier(name_node) => {
            let fq_class_id = *context.resolved_names.get(name_node);

            possible_types.push(ResolvedClassname::new(
                Some(fq_class_id),
                ResolutionOrigin::Named { is_parent: false, is_self: false },
            ));
        }
        Expression::Self_(self_keyword) => {
            if let Some(self_class) = block_context.scope.get_class_like() {
                possible_types.push(ResolvedClassname::new(
                    Some(self_class.name),
                    ResolutionOrigin::Named { is_parent: false, is_self: true },
                ));
            } else {
                possible_types.push(ResolvedClassname::invalid());
                context.buffer.report(
                    TypingIssueKind::SelfOutsideClassScope,
                    Issue::error("Cannot use `self` keyword outside of a class context.")
                        .with_annotation(Annotation::primary(self_keyword.span()).with_message("`self` used here"))
                        .with_note("The `self` keyword refers to the current class and can only be used within a class method.")
                );
            };
        }
        Expression::Static(static_keyword) => {
            if let Some(self_class) = block_context.scope.get_class_like() {
                let origin = ResolutionOrigin::Static { can_extend: !self_class.is_final() };
                possible_types.push(ResolvedClassname::new(Some(self_class.name), origin));
            } else {
                possible_types.push(ResolvedClassname::invalid());
                context.buffer.report(
                    TypingIssueKind::StaticOutsideClassScope,
                    Issue::error("Cannot use `static` keyword outside of a class scope.")
                        .with_annotation(Annotation::primary(static_keyword.span()).with_message("`static` used here"))
                        .with_note(
                            "The `static` keyword refers to the called class at runtime and requires a class scope.",
                        ),
                );
            }
        }
        Expression::Parent(parent_keyword) => {
            if let Some(self_meta) = block_context.scope.get_class_like() {
                if let Some(parent_id_ref) = self_meta.get_direct_parent_class_ref() {
                    possible_types.push(ResolvedClassname::new(
                        Some(*parent_id_ref),
                        ResolutionOrigin::Named { is_parent: true, is_self: false },
                    ));
                } else {
                    context.buffer.report(
                        TypingIssueKind::InvalidParentType,
                        Issue::error(format!(
                            "Cannot use `parent` as the current type (`{}`) does not have a parent class.",
                            context.interner.lookup(&self_meta.original_name)
                        ))
                        .with_annotation(Annotation::primary(parent_keyword.span()).with_message("`parent` used here"))
                        .with_annotation(
                            Annotation::secondary(self_meta.get_name_span().unwrap_or_else(|| self_meta.get_span()))
                                .with_message(format!(
                                    "Class `{}` has no parent",
                                    context.interner.lookup(&self_meta.original_name)
                                )),
                        ),
                    );

                    possible_types.push(ResolvedClassname::invalid());
                }
            } else {
                context.buffer.report(
                    TypingIssueKind::ParentOutsideClassScope,
                    Issue::error("Cannot use `parent` keyword outside of a class context.")
                        .with_annotation(Annotation::primary(parent_keyword.span()).with_message("`parent` used here"))
                        .with_note("The `parent` keyword refers to the parent class and must be used inside a class."),
                );

                possible_types.push(ResolvedClassname::invalid());
            }
        }
        expression => {
            // If the expression is not already analyzed, we analyze it now.
            if !class_is_analyzed {
                let was_inside_call = block_context.inside_call;
                block_context.inside_call = true;
                expression.analyze(context, block_context, artifacts)?;
                block_context.inside_call = was_inside_call;
            }

            let expression_type = artifacts.get_expression_type(expression);

            for atomic in expression_type.map(|u| u.types.iter()).unwrap_or_default() {
                if let Some(resolved_classname) = get_class_name_from_atomic(context.interner, atomic) {
                    possible_types.push(resolved_classname);
                } else {
                    possible_types.push(ResolvedClassname::invalid());
                    context.buffer.report(
                        TypingIssueKind::InvalidClassStringExpression,
                        Issue::error(format!(
                            "Expression of type `{}` cannot be used as a class name.",
                            atomic.get_id(Some(context.interner))
                        ))
                        .with_annotation(Annotation::primary(expression.span()).with_message("This expression is used as a class name"))
                        .with_note("To use an expression as a class name, it must evaluate to a string that is a valid class name (e.g., a `class-string` type).")
                    );
                }
            }
        }
    };

    Ok(possible_types)
}

/// Resolves a `TAtomic` type to a `ResolvedClassname` if it can represent a class identifier.
///
/// This function handles various atomic types:
/// - `class-string` types: Extracts the specific class or marks as generic.
/// - Object types: Uses the object's own class name.
/// - String types: Marks as a generic string, as the value is unknown.
/// - `mixed`, `any`, `object`: Resolved with a corresponding generic origin.
///
/// Returns `None` for atomic types that can never be a class name (e.g., int, bool, array).
pub fn get_class_name_from_atomic(interner: &ThreadedInterner, atomic: &TAtomic) -> Option<ResolvedClassname> {
    #[inline]
    fn get_class_name_from_atomic_impl(
        interner: &ThreadedInterner,
        atomic: &TAtomic,
        from_generic_class_string: bool,
    ) -> Option<ResolvedClassname> {
        if let Some(literal_string) = atomic.get_literal_string_value() {
            // A literal string value is treated as a generic string because, while its value
            // is known, it's not guaranteed to be a class name without further checks.
            // It's different from `MyClass::class` which is guaranteed.
            return Some(ResolvedClassname::new(
                Some(interner.intern(literal_string)),
                ResolutionOrigin::GenericString,
            ));
        }

        if let TAtomic::Scalar(TScalar::ClassLikeString(class_string)) = atomic {
            return Some(match class_string {
                TClassLikeString::Any { .. } => ResolvedClassname::new(None, ResolutionOrigin::GenericClassString),
                TClassLikeString::OfType { constraint, .. } | TClassLikeString::Generic { constraint, .. } => {
                    // This is a `class-string<T>`. We resolve `T` to get the class name.
                    get_class_name_from_atomic_impl(interner, constraint.as_ref(), true)?
                }
                TClassLikeString::Literal { value } => {
                    ResolvedClassname::new(Some(*value), ResolutionOrigin::LiteralClassString)
                }
            });
        }

        if let Some(object_name) = atomic.get_object_or_enum_name() {
            let origin = if from_generic_class_string {
                ResolutionOrigin::GenericClassString
            } else {
                ResolutionOrigin::Object { is_this: atomic.is_this() }
            };

            return Some(ResolvedClassname::new(Some(object_name), origin));
        }

        if let TAtomic::Object(TObject::Any) = atomic {
            return Some(ResolvedClassname::new(None, ResolutionOrigin::GenericObject));
        }

        if atomic.is_any_string() {
            return Some(ResolvedClassname::new(None, ResolutionOrigin::GenericString));
        }

        if atomic.is_any() {
            return Some(ResolvedClassname::new(None, ResolutionOrigin::Any));
        }

        if atomic.is_mixed() {
            return Some(ResolvedClassname::new(None, ResolutionOrigin::Mixed));
        }

        // This type cannot be interpreted as a class name.
        None
    }

    get_class_name_from_atomic_impl(interner, atomic, false)
}
