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
    context: &mut Context<'_, '_, '_>,
) {
    if extension_limit && extends.types.len() > 1 {
        context.report(
            Issue::error(format!(
                "{} `{}` can only extend one other type, found {}.",
                class_like_kind,
                class_like_name,
                extends.types.len()
            ))
            .with_annotation(Annotation::primary(extends.span()).with_message("Multiple extensions found here."))
            .with_annotation(
                Annotation::secondary(class_like_span)
                    .with_message(format!("{class_like_kind} `{class_like_fqcn}` declared here.")),
            )
            .with_help("Remove the extra extensions to ensure only one type is extended."),
        );
    }

    for extended_type in extends.types.iter() {
        let extended_fqcn = context.get_name(&extended_type.span().start);

        if extended_fqcn.eq_ignore_ascii_case(class_like_fqcn) {
            context.report(
                Issue::error(format!("{class_like_kind} `{class_like_name}` cannot extend itself."))
                    .with_annotation(
                        Annotation::primary(extended_type.span())
                            .with_message(format!("{class_like_kind} `{class_like_name}` extends itself here.")),
                    )
                    .with_annotation(
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{class_like_kind} `{class_like_fqcn}` declared here.")),
                    )
                    .with_help("Remove the self-referencing extension."),
            );
        }
    }

    for extended_type in extends.types.iter() {
        let extended_name = extended_type.value();

        if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(extended_name))
            || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(extended_name))
        {
            context.report(
                Issue::error(format!(
                    "{class_like_kind} `{class_like_name}` cannot extend reserved keyword `{extended_name}`."
                ))
                .with_annotation(
                    Annotation::primary(extended_type.span()).with_message("Extension uses a reserved keyword."),
                )
                .with_annotation(
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{class_like_kind} `{class_like_name}` declared here.")),
                )
                .with_help(format!(
                    "Change the extended type to a valid identifier. `{extended_name}` is a reserved keyword."
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
    context: &mut Context<'_, '_, '_>,
) {
    if check_for_self_implement {
        for implemented_type in implements.types.iter() {
            let implemented_fqcn = context.get_name(&implemented_type.span().start);

            if implemented_fqcn.eq_ignore_ascii_case(class_like_fqcn) {
                context.report(
                    Issue::error(format!("{class_like_kind} `{class_like_name}` cannot implement itself."))
                        .with_annotation(
                            Annotation::primary(implemented_type.span())
                                .with_message(format!("{class_like_kind} `{class_like_name}` implements itself here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` declared here.")),
                        )
                        .with_help("Remove the self-referencing implementation."),
                );
            }
        }
    }

    for implemented_type in implements.types.iter() {
        let implemented_name = implemented_type.value();

        if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(implemented_name))
            || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(implemented_name))
        {
            context.report(
                Issue::error(format!(
                    "{class_like_kind} `{class_like_name}` cannot implement reserved keyword `{implemented_name}`."
                ))
                .with_annotation(
                    Annotation::primary(implemented_type.span()).with_message("This is a reserved keyword."),
                )
                .with_annotation(
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{class_like_kind} `{class_like_name}` declared here.")),
                )
                .with_help(format!(
                    "Replace `{implemented_name}` with a valid identifier. Reserved keywords cannot be used as implemented types."
                )),
            );
        }
    }
}
