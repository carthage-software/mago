use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_hint(hint: &Hint, context: &mut Context<'_, '_, '_>) {
    match hint {
        Hint::Parenthesized(parenthesized_hint) => {
            if !parenthesized_hint.hint.is_parenthesizable() {
                let val = context.get_code_snippet(parenthesized_hint.hint);

                context.report(
                    Issue::error(format!("Type `{val}` cannot be parenthesized."))
                        .with_annotation(
                            Annotation::primary(parenthesized_hint.hint.span())
                                .with_message("Invalid parenthesized type."),
                        )
                        .with_annotation(
                            Annotation::secondary(parenthesized_hint.span())
                                .with_message("Parenthesized type defined here."),
                        )
                        .with_note("Only union or intersection types can be enclosed in parentheses.")
                        .with_help("Remove the parentheses around the type."),
                );
            }
        }
        Hint::Nullable(nullable_hint) => {
            if !context.version.is_supported(Feature::NullableTypeHint) {
                context.report(
                    Issue::error("The `?` nullable type hint is only available in PHP 7.1 and above.")
                        .with_annotation(
                            Annotation::primary(hint.span()).with_message("`?` nullable type hint used here."),
                        )
                        .with_help("Upgrade to PHP 7.1 or above to use the `?` nullable type hint."),
                );
            }

            if nullable_hint.hint.is_standalone() || nullable_hint.hint.is_complex() {
                let val = context.get_code_snippet(nullable_hint.hint);

                context.report(
                    Issue::error(format!("Type `{val}` cannot be nullable."))
                        .with_annotation(
                            Annotation::primary(nullable_hint.hint.span()).with_message("Invalid nullable type."),
                        )
                        .with_annotation(
                            Annotation::secondary(nullable_hint.span()).with_message("Nullable type defined here."),
                        )
                        .with_help("Replace the type or remove the nullable modifier."),
                );
            }
        }
        Hint::Union(union_hint) => {
            if !union_hint.left.is_unionable() {
                let val = context.get_code_snippet(union_hint.left);

                context.report(
                    Issue::error(format!("Type `{val}` cannot be part of a union."))
                        .with_annotation(
                            Annotation::primary(union_hint.left.span()).with_message("Invalid union type."),
                        )
                        .with_annotation(
                            Annotation::secondary(union_hint.pipe).with_message("Union operator `|` used here."),
                        )
                        .with_note("Intersection and standalone types cannot be part of a union.")
                        .with_help("Replace the type or remove it from the union."),
                );
            }

            if !union_hint.right.is_unionable() {
                let val = context.get_code_snippet(union_hint.right);

                context.report(
                    Issue::error(format!("Type `{val}` cannot be part of a union."))
                        .with_annotation(
                            Annotation::primary(union_hint.right.span()).with_message("Invalid union type."),
                        )
                        .with_annotation(
                            Annotation::secondary(union_hint.pipe).with_message("Union operator `|` used here."),
                        )
                        .with_note("Intersection and standalone types cannot be part of a union.")
                        .with_help("Replace the type or remove it from the union."),
                );
            }
        }
        Hint::Intersection(intersection_hint) => {
            if !context.version.is_supported(Feature::PureIntersectionTypes) {
                context.report(
                    Issue::error("Intersection types are only available in PHP 8.1 and above.")
                    .with_annotation(
                        Annotation::primary(intersection_hint.span()).with_message("Intersection type used here."),
                    )
                    .with_note(
                        "Intersection types allow combining multiple types into a single type, but are only available in PHP 8.2 and above.",
                    )
                    .with_help("Upgrade to PHP 8.2 or above to use intersection types."),
                );
            }

            if !intersection_hint.left.is_intersectable() {
                let val = context.get_code_snippet(intersection_hint.left);

                context.report(
                    Issue::error(format!("Type `{val}` cannot be part of an intersection."))
                        .with_annotation(
                            Annotation::primary(intersection_hint.left.span())
                                .with_message("Invalid intersection type."),
                        )
                        .with_annotation(
                            Annotation::secondary(intersection_hint.ampersand)
                                .with_message("Intersection operator `&` used here."),
                        )
                        .with_note("Union and standalone types cannot be part of an intersection.")
                        .with_help("Replace the type or remove it from the intersection."),
                );
            }

            if !intersection_hint.right.is_intersectable() {
                let val = context.get_code_snippet(intersection_hint.right);

                context.report(
                    Issue::error(format!("Type `{val}` cannot be part of an intersection."))
                        .with_annotation(
                            Annotation::primary(intersection_hint.right.span())
                                .with_message("Invalid intersection type."),
                        )
                        .with_annotation(
                            Annotation::secondary(intersection_hint.ampersand)
                                .with_message("Intersection operator `&` used here."),
                        )
                        .with_note("Union and standalone types cannot be part of an intersection.")
                        .with_help("Replace the type or remove it from the intersection."),
                );
            }
        }
        Hint::True(hint) if !context.version.is_supported(Feature::TrueTypeHint) => {
            context.report(
                Issue::error("The `true` type hint is only available in PHP 8.2 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("`true` type hint used here."))
                    .with_help("Upgrade to PHP 8.2 or above to use the `true` type hint."),
            );
        }
        Hint::False(hint) if context.hint_depth == 1 && !context.version.is_supported(Feature::FalseTypeHint) => {
            context.report(
                Issue::error("The `false` type hint is only available in PHP 8.2 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("`false` type hint used here."))
                    .with_help("Upgrade to PHP 8.2 or above to use the `false` type hint."),
            );
        }
        Hint::False(hint)
            if context.hint_depth != 1 && !context.version.is_supported(Feature::FalseCompoundTypeHint) =>
        {
            context.report(
                Issue::error("The compound `false` type hint is only available in PHP 8.0 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("`false` type hint used here."))
                    .with_help("Upgrade to PHP 8.0 or above to use the `false` type hint."),
            );
        }
        Hint::Null(hint) if context.hint_depth == 1 && !context.version.is_supported(Feature::NullTypeHint) => {
            context.report(
                Issue::error("The `null` type hint is only available in PHP 8.2 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("`null` type hint used here."))
                    .with_help("Upgrade to PHP 8.2 or above to use the `null` type hint."),
            );
        }
        Hint::Null(hint) if context.hint_depth != 1 && !context.version.is_supported(Feature::NullCompoundTypeHint) => {
            context.report(
                Issue::error("The compound `null` type hint is only available in PHP 8.0 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("`null` type hint used here."))
                    .with_help("Upgrade to PHP 8.0 or above to use the `null` type hint."),
            );
        }
        Hint::Iterable(hint) if !context.version.is_supported(Feature::IterableTypeHint) => {
            context.report(
                Issue::error("The `iterable` type hint is only available in PHP 7.1 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("`iterable` type hint used here."))
                    .with_help("Upgrade to PHP 7.1 or above to use the `iterable` type hint."),
            );
        }
        Hint::Void(hint) if !context.version.is_supported(Feature::VoidTypeHint) => {
            context.report(
                Issue::error("The `void` type hint is only available in PHP 7.1 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("`void` type hint used here."))
                    .with_help("Upgrade to PHP 7.1 or above to use the `void` type hint."),
            );
        }
        Hint::Mixed(hint) if !context.version.is_supported(Feature::MixedTypeHint) => {
            context.report(
                Issue::error("The `mixed` type hint is only available in PHP 8.0 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("`mixed` type hint used here."))
                    .with_help("Upgrade to PHP 8.0 or above to use the `mixed` type hint."),
            );
        }
        Hint::Never(hint) if !context.version.is_supported(Feature::NeverTypeHint) => {
            context.report(
                Issue::error("The `never` type hint is only available in PHP 8.1 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("`never` type hint used here."))
                    .with_help("Upgrade to PHP 8.1 or above to use the `never` type hint."),
            );
        }
        _ => {}
    }

    if context.version.is_supported(Feature::DisjunctiveNormalForm) {
        return;
    }

    let is_dnf = match hint {
        Hint::Intersection(inter) if inter.left.is_union() || inter.right.is_union() => true,
        Hint::Union(union) if union.left.is_intersection() || union.right.is_intersection() => true,
        _ => false,
    };

    if !is_dnf {
        return;
    }

    context.report(
        Issue::error("Disjunctive Normal Form (DNF) types are only available in PHP 8.2 and above.")
            .with_annotation(Annotation::primary(hint.span()).with_message("DNF type used here.")),
    );
}
