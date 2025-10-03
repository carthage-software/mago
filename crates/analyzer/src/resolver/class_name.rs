use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

/// Describes the origin and nature of a class name resolution.
///
/// This enum provides a clear, type-safe alternative to using multiple boolean flags,
/// capturing all possible ways a class name can be identified.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    AnyClassString,
    /// Resolved from a generic `string` type, which may or may not be a valid class name at runtime.
    AnyString,
    /// Resolved from `class-string<T>`.
    SpecificClassLikeString(TClassLikeString),
    /// Resolved from an `object` type where the specific class is not known.
    AnyObject,
    /// Resolved from a `mixed` type, which could potentially be a class name.
    Mixed,
}

/// Represents the result of resolving an expression that is expected to be a class name.
///
/// This struct is used to analyze expressions in contexts like `new <expr>`, `<expr>::method()`,
/// `<expr>::$property`, or `<expr>::CONSTANT`, where `<expr>` must resolve to a valid class identifier.
/// It carries the resolved fully-qualified class name (if known) and metadata about how the
/// resolution was performed via `ResolutionOrigin`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResolvedClassname {
    /// The fully qualified class name (`Atom`) if a specific class could be identified.
    /// This is `None` for ambiguous or generic types like `object`, `class-string`, or `mixed`.
    pub fqcn: Option<Atom>,
    /// Describes how the class name was resolved.
    pub origin: ResolutionOrigin,
    /// A list of other `ResolvedClassname` instances that this class name intersects with.
    pub intersections: Vec<ResolvedClassname>,
    /// Indicates if the class name refers to a final class or an enum (which is implicitly final).
    pub is_final: bool,
}

impl ResolvedClassname {
    /// Creates a new `ResolvedClassname`.
    #[inline]
    const fn new(fq_class_id: Option<Atom>, origin: ResolutionOrigin, is_final: bool) -> Self {
        Self { fqcn: fq_class_id, origin, intersections: Vec::new(), is_final }
    }

    /// Creates a `ResolvedClassname` that is definitively invalid.
    #[inline]
    const fn invalid() -> Self {
        Self { fqcn: None, origin: ResolutionOrigin::Invalid, intersections: Vec::new(), is_final: false }
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
    pub const fn is_possibly_invalid(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Mixed | ResolutionOrigin::Invalid)
    }

    /// Checks if the class name is resolved from `mixed` or `any` type,
    /// which means it could potentially be any class name.
    #[inline]
    pub const fn is_from_mixed(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Mixed)
    }

    /// Checks if the class name is resolved from the `static` keyword.
    pub const fn is_static(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Static { .. })
    }

    /// Checks if the class name is resolved from the `self` keyword.
    pub const fn is_self(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Named { is_self: true, .. })
    }

    /// Checks if the class name is resolved from the `static` keyword and the class is not final,
    pub const fn can_extend_static(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Static { can_extend: true })
    }

    /// Checks if the resolution is from a generic `class-string` type,
    /// which means it could be any class name that is a valid `class-string`.
    pub const fn is_from_class_string(&self) -> bool {
        matches!(
            self.origin,
            ResolutionOrigin::AnyClassString
                | ResolutionOrigin::LiteralClassString
                | ResolutionOrigin::SpecificClassLikeString(_)
        )
    }

    /// Checks if the resolution is from a literal `class-string` (e.g., `MyClass::class`).
    pub const fn is_from_literal_class_string(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::LiteralClassString)
    }

    /// Checks if the resolution is from a generic `object` type,
    /// which means it could be any object type.
    pub const fn is_from_any_object(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::AnyObject)
    }

    /// Checks if the resolution is a class instance (e.g., from an object).
    pub const fn is_object_instance(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Object { .. })
    }

    /// Checks if the resolution is from an identifier or `self`, or `parent` keyword.
    #[inline]
    pub const fn is_named(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Named { .. })
    }

    /// Checks if the resolution is from a `self`, `static`, or `parent` keyword.
    #[inline]
    pub const fn is_relative(&self) -> bool {
        matches!(
            self.origin,
            ResolutionOrigin::Named { is_self: true, .. }
                | ResolutionOrigin::Named { is_parent: true, .. }
                | ResolutionOrigin::Static { .. }
        )
    }

    /// Checks if the resolution is from the `parent` keyword.
    #[inline]
    pub const fn is_parent(&self) -> bool {
        matches!(self.origin, ResolutionOrigin::Named { is_parent: true, .. })
    }

    #[inline]
    pub fn get_object_type(&self, codebase: &CodebaseMetadata) -> TAtomic {
        let mut object_atomic = if let ResolutionOrigin::SpecificClassLikeString(class_string) = &self.origin {
            class_string.get_object_type(codebase)
        } else {
            TAtomic::Object(match self.fqcn {
                Some(fqcn) => {
                    let lowercase_fqcn = ascii_lowercase_atom(&fqcn);

                    if codebase.symbols.contains_enum(&lowercase_fqcn) {
                        TObject::Enum(TEnum::new(fqcn))
                    } else {
                        TObject::Named(TNamedObject::new(fqcn))
                    }
                }
                None => TObject::Any,
            })
        };

        for intersection_class in &self.intersections {
            object_atomic.add_intersection_type(intersection_class.get_object_type(codebase));
        }

        object_atomic
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
pub fn resolve_classnames_from_expression<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    class_expression: &Expression<'arena>,
    class_is_analyzed: bool,
) -> Result<Vec<ResolvedClassname>, AnalysisError> {
    let mut possible_types = vec![];
    match class_expression.unparenthesized() {
        Expression::Identifier(name_node) => {
            let fqcn = atom(context.resolved_names.get(name_node));

            possible_types.push(ResolvedClassname::new(
                Some(fqcn),
                ResolutionOrigin::Named { is_parent: false, is_self: false },
                context.codebase.is_enum_or_final_class(&fqcn),
            ));
        }
        Expression::Self_(self_keyword) => {
            if let Some(self_class) = block_context.scope.get_class_like() {
                let origin = ResolutionOrigin::Named { is_parent: false, is_self: true };
                let mut class_name = ResolvedClassname::new(
                    Some(self_class.original_name),
                    origin,
                    self_class.kind.is_enum() || self_class.flags.is_final(),
                );

                class_name.intersections = get_intersections_from_metadata(context, self_class);

                possible_types.push(class_name);
            } else {
                possible_types.push(ResolvedClassname::invalid());
                context.collector.report_with_code(
                    IssueCode::SelfOutsideClassScope,
                    Issue::error("Cannot use `self` keyword outside of a class context.")
                        .with_annotation(Annotation::primary(self_keyword.span()).with_message("`self` used here"))
                        .with_note("The `self` keyword refers to the current class and can only be used within a class method.")
                );
            };
        }
        Expression::Static(static_keyword) => {
            if let Some(self_class) = block_context.scope.get_class_like() {
                let origin = ResolutionOrigin::Static { can_extend: !self_class.flags.is_final() };
                let mut classname = ResolvedClassname::new(
                    Some(self_class.original_name),
                    origin,
                    self_class.kind.is_enum() || self_class.flags.is_final(),
                );
                classname.intersections = get_intersections_from_metadata(context, self_class);

                possible_types.push(classname);
            } else {
                possible_types.push(ResolvedClassname::invalid());
                context.collector.report_with_code(
                    IssueCode::StaticOutsideClassScope,
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
                if let Some(parent_metadata) =
                    self_meta.direct_parent_class.as_ref().and_then(|id| context.codebase.get_class_like(id))
                {
                    let origin = ResolutionOrigin::Named { is_parent: true, is_self: false };
                    let mut classname = ResolvedClassname::new(Some(parent_metadata.original_name), origin, false);
                    classname.intersections = get_intersections_from_metadata(context, self_meta);

                    possible_types.push(classname);
                } else {
                    context.collector.report_with_code(
                        IssueCode::InvalidParentType,
                        Issue::error(format!(
                            "Cannot use `parent` as the current type (`{}`) does not have a parent class.",
                            self_meta.original_name
                        ))
                        .with_annotation(Annotation::primary(parent_keyword.span()).with_message("`parent` used here"))
                        .with_annotation(
                            Annotation::secondary(self_meta.name_span.unwrap_or(self_meta.span))
                                .with_message(format!("Class `{}` has no parent", self_meta.original_name)),
                        ),
                    );

                    possible_types.push(ResolvedClassname::invalid());
                }
            } else {
                context.collector.report_with_code(
                    IssueCode::ParentOutsideClassScope,
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
                if let Some(resolved_classname) = get_class_name_from_atomic(context.codebase, atomic) {
                    possible_types.push(resolved_classname);
                } else {
                    possible_types.push(ResolvedClassname::invalid());
                    context.collector.report_with_code(
                        IssueCode::InvalidClassStringExpression,
                        Issue::error(format!(
                            "Expression of type `{}` cannot be used as a class name.",
                            atomic.get_id()
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
pub fn get_class_name_from_atomic(codebase: &CodebaseMetadata, atomic: &TAtomic) -> Option<ResolvedClassname> {
    #[inline]
    fn get_class_name_from_atomic_impl(
        codebase: &CodebaseMetadata,
        atomic: &TAtomic,
        active_class_string: Option<&TClassLikeString>,
    ) -> Option<ResolvedClassname> {
        let mut class_name = match atomic {
            TAtomic::GenericParameter(parameter) => parameter
                .constraint
                .types
                .iter()
                .filter_map(|constraint_atomic| {
                    get_class_name_from_atomic_impl(codebase, constraint_atomic, active_class_string)
                })
                .next()
                .unwrap_or_else(ResolvedClassname::invalid),
            TAtomic::Object(object) => match object {
                TObject::Any => {
                    let origin = if let Some(class_string) = active_class_string {
                        ResolutionOrigin::SpecificClassLikeString(class_string.clone())
                    } else {
                        ResolutionOrigin::AnyObject
                    };

                    ResolvedClassname::new(None, origin, false)
                }
                TObject::Enum(enum_object) => {
                    let origin = if let Some(class_string) = active_class_string {
                        ResolutionOrigin::SpecificClassLikeString(class_string.clone())
                    } else {
                        ResolutionOrigin::Object { is_this: atomic.is_this() }
                    };

                    ResolvedClassname::new(Some(enum_object.name), origin, true)
                }
                TObject::Named(named_object) => {
                    let origin = if let Some(class_string) = active_class_string {
                        ResolutionOrigin::SpecificClassLikeString(class_string.clone())
                    } else {
                        ResolutionOrigin::Object { is_this: atomic.is_this() }
                    };

                    ResolvedClassname::new(
                        Some(named_object.name),
                        origin,
                        codebase.is_enum_or_final_class(&named_object.name),
                    )
                }
                TObject::WithProperties(_) => {
                    let origin = if let Some(class_string) = active_class_string {
                        ResolutionOrigin::SpecificClassLikeString(class_string.clone())
                    } else {
                        ResolutionOrigin::AnyObject
                    };

                    ResolvedClassname::new(None, origin, false)
                }
            },
            TAtomic::Scalar(TScalar::ClassLikeString(class_string)) => {
                match class_string {
                    TClassLikeString::Any { .. } => {
                        ResolvedClassname::new(None, ResolutionOrigin::AnyClassString, false)
                    }
                    TClassLikeString::OfType { constraint, .. } | TClassLikeString::Generic { constraint, .. } => {
                        // This is a `class-string<T>`. We resolve `T` to get the class name.
                        get_class_name_from_atomic_impl(codebase, constraint.as_ref(), Some(class_string))?
                    }
                    TClassLikeString::Literal { value } => ResolvedClassname::new(
                        Some(*value),
                        ResolutionOrigin::LiteralClassString,
                        codebase.is_enum_or_final_class(value),
                    ),
                }
            }
            TAtomic::Scalar(scalar) => {
                if let Some(literal_string) = atomic.get_literal_string_value() {
                    // A literal string value is treated as a generic string because, while its value
                    // is known, it's not guaranteed to be a class name without further checks.
                    // It's different from `MyClass::class` which is guaranteed.
                    let class_id = atom(literal_string);

                    ResolvedClassname::new(Some(class_id), ResolutionOrigin::AnyString, false)
                } else if scalar.is_string() {
                    ResolvedClassname::new(None, ResolutionOrigin::AnyString, false)
                } else {
                    return None; // This type cannot be interpreted as a class name.
                }
            }
            TAtomic::Mixed(_) => ResolvedClassname::new(None, ResolutionOrigin::Mixed, false),
            _ => {
                // This type cannot be interpreted as a class name.
                return None;
            }
        };

        if let Some(intersections) = atomic.get_intersection_types() {
            let intersection_class_names = intersections
                .iter()
                .filter_map(|intersection| get_class_name_from_atomic_impl(codebase, intersection, None))
                .collect::<Vec<_>>();

            class_name.intersections = intersection_class_names;
        }

        Some(class_name)
    }

    get_class_name_from_atomic_impl(codebase, atomic, None)
}

fn get_intersections_from_metadata(context: &Context<'_, '_>, metadata: &ClassLikeMetadata) -> Vec<ResolvedClassname> {
    if metadata.kind.is_enum() {
        return vec![];
    }

    let mut intersections = vec![];
    for required_interface in &metadata.require_implements {
        let Some(interface_metadata) = context.codebase.get_interface(required_interface) else {
            continue;
        };

        intersections.extend(get_intersections_from_metadata(context, interface_metadata));
        intersections.push(ResolvedClassname::new(
            Some(interface_metadata.original_name),
            ResolutionOrigin::Named { is_parent: false, is_self: false },
            false,
        ));
    }

    for required_class in &metadata.require_extends {
        let Some(parent_class_metadata) = context.codebase.get_class_like(required_class) else {
            continue;
        };

        intersections.extend(get_intersections_from_metadata(context, parent_class_metadata));
        intersections.push(ResolvedClassname::new(
            Some(parent_class_metadata.original_name),
            ResolutionOrigin::Named { is_parent: true, is_self: false },
            false,
        ));
    }

    intersections
}

pub fn report_non_existent_class_like(context: &mut Context, span: Span, classname: &Atom) {
    context.collector.report_with_code(
        IssueCode::NonExistentClassLike,
        Issue::error(format!("Class, Interface, or Trait `{classname}` does not exist."))
            .with_annotation(
                Annotation::primary(span).with_message("This expression refers to a non-existent class-like type"),
            )
            .with_help(format!("Ensure the `{classname}` is defined in the codebase.")),
    );
}
