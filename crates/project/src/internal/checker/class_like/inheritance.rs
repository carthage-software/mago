use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::consts::RESERVED_KEYWORDS;
use crate::internal::consts::SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED;
use crate::internal::context::Context;

#[inline]
pub fn check_extends(
    extends: &Extends,
    class_like_span: Span,
    class_like_kind: &str,
    class_like_name: &str,
    class_like_fqcn: &str,
    extension_limit: bool,
    context: &mut Context<'_>,
) {
    if extension_limit && extends.types.len() > 1 {
        context.issues.push(
            Issue::error(format!(
                "{} `{}` can only extend one other type, found {}.",
                class_like_kind,
                class_like_name,
                extends.types.len()
            ))
            .with_annotation(Annotation::primary(extends.span()).with_message("Multiple extensions found here."))
            .with_annotation(
                Annotation::secondary(class_like_span)
                    .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_fqcn)),
            )
            .with_help("Remove the extra extensions to ensure only one type is extended."),
        );
    }

    for extended_type in extends.types.iter() {
        let extended_fqcn = context.get_name(&extended_type.span().start);

        if extended_fqcn.eq_ignore_ascii_case(class_like_fqcn) {
            context.issues.push(
                Issue::error(format!("{} `{}` cannot extend itself.", class_like_kind, class_like_name))
                    .with_annotation(
                        Annotation::primary(extended_type.span())
                            .with_message(format!("{} `{}` extends itself here.", class_like_kind, class_like_name)),
                    )
                    .with_annotation(
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_fqcn)),
                    )
                    .with_help("Remove the self-referencing extension."),
            );
        }
    }

    for extended_type in extends.types.iter() {
        let extended_name = context.interner.lookup(extended_type.value());

        if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(extended_name))
            || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(extended_name))
        {
            context.issues.push(
                Issue::error(format!(
                    "{} `{}` cannot extend reserved keyword `{}`.",
                    class_like_kind, class_like_name, extended_name
                ))
                .with_annotation(
                    Annotation::primary(extended_type.span()).with_message("Extension uses a reserved keyword."),
                )
                .with_annotation(
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_name)),
                )
                .with_help(format!(
                    "Change the extended type to a valid identifier. `{}` is a reserved keyword.",
                    extended_name
                )),
            );
        }
    }
}

#[inline]
pub fn check_implements(
    implements: &Implements,
    class_like_span: Span,
    class_like_kind: &str,
    class_like_name: &str,
    class_like_fqcn: &str,
    check_for_self_implement: bool,
    context: &mut Context<'_>,
) {
    if check_for_self_implement {
        for implemented_type in implements.types.iter() {
            let implemented_fqcn = context.get_name(&implemented_type.span().start);

            if implemented_fqcn.eq_ignore_ascii_case(class_like_fqcn) {
                context.issues.push(
                    Issue::error(format!("{} `{}` cannot implement itself.", class_like_kind, class_like_name))
                        .with_annotation(
                            Annotation::primary(implemented_type.span()).with_message(format!(
                                "{} `{}` implements itself here.",
                                class_like_kind, class_like_name
                            )),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_fqcn)),
                        )
                        .with_help("Remove the self-referencing implementation."),
                );
            }
        }
    }

    for implemented_type in implements.types.iter() {
        let implemented_name = context.interner.lookup(implemented_type.value());

        if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(implemented_name))
            || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(implemented_name))
        {
            context.issues.push(
                Issue::error(format!(
                    "{} `{}` cannot implement reserved keyword `{}`.",
                    class_like_kind, class_like_name, implemented_name
                ))
                .with_annotation(
                    Annotation::primary(implemented_type.span()).with_message("This is a reserved keyword."),
                )
                .with_annotation(
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_name)),
                )
                .with_help(format!(
                    "Replace `{}` with a valid identifier. Reserved keywords cannot be used as implemented types.",
                    implemented_name
                )),
            );
        }
    }
}
