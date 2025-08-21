use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_class_like_constant<'ast, 'arena>(
    class_like_constant: &'ast ClassLikeConstant<'arena>,
    class_like_span: Span,
    class_like_kind: &str,
    class_like_name: &str,
    class_like_fqcn: &str,
    context: &mut Context<'_, 'ast, 'arena>,
) {
    let first_item = class_like_constant.first_item();
    let first_item_name = first_item.name.value;

    let mut last_final: Option<Span> = None;
    let mut last_visibility: Option<Span> = None;
    for modifier in class_like_constant.modifiers.iter() {
        match modifier {
            Modifier::Readonly(k)
            | Modifier::Static(k)
            | Modifier::Abstract(k)
            | Modifier::PrivateSet(k)
            | Modifier::ProtectedSet(k)
            | Modifier::PublicSet(k) => {
                context.report(
                    Issue::error(format!("`{}` modifier is not allowed on constants", k.value))
                        .with_annotation(Annotation::primary(modifier.span()))
                        .with_annotations([
                            Annotation::secondary(first_item.span()).with_message(format!(
                                "{class_like_kind} constant `{class_like_name}::{first_item_name}` is declared here."
                            )),
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` is declared here.")),
                        ]),
                );
            }
            Modifier::Final(_) => {
                if !context.version.is_supported(Feature::FinalConstants) {
                    context.report(
                        Issue::error("Final class constants are only available in PHP 8.1 and above.").with_annotation(
                            Annotation::primary(modifier.span()).with_message("Final modifier used here."),
                        ),
                    );
                }

                if let Some(last_final) = last_final {
                    context.report(
                        Issue::error("duplicate `final` modifier on constant")
                            .with_annotation(Annotation::primary(modifier.span()))
                            .with_annotations([
                                Annotation::secondary(last_final).with_message("previous `final` modifier"),
                                Annotation::secondary(first_item.span()).with_message(format!(
                                    "{class_like_kind} constant `{class_like_name}::{first_item_name}` is declared here."
                                )),
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{class_like_kind} `{class_like_fqcn}` is declared here."
                                )),
                            ]),
                    );
                }

                last_final = Some(modifier.span());
            }
            Modifier::Private(_) | Modifier::Protected(_) | Modifier::Public(_) => {
                if !context.version.is_supported(Feature::ClassLikeConstantVisibilityModifiers) {
                    context.report(
                        Issue::error(
                            "Visibility modifiers for class constants are only available in PHP 7.1 and above.",
                        )
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Visibility modifier used here."),
                        ),
                    );
                }

                if let Some(last_visibility) = last_visibility {
                    context.report(
                        Issue::error("duplicate visibility modifier on constant")
                            .with_annotation(Annotation::primary(modifier.span()))
                            .with_annotations([
                                Annotation::secondary(last_visibility).with_message("previous visibility modifier"),
                                Annotation::secondary(first_item.span()).with_message(format!(
                                    "{class_like_kind} constant `{class_like_name}::{first_item_name}` is declared here."
                                )),
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{class_like_kind} `{class_like_fqcn}` is declared here."
                                )),
                            ]),
                    );
                }

                last_visibility = Some(modifier.span());
            }
        }
    }

    if let Some(type_hint) = &class_like_constant.hint
        && !context.version.is_supported(Feature::TypedClassLikeConstants)
    {
        context.report(
            Issue::error("Typed class constants are only available in PHP 8.3 and above.")
                .with_annotation(Annotation::primary(type_hint.span()).with_message("Type hint used here.")),
        );
    };

    for item in class_like_constant.items.iter() {
        let item_name = item.name.value;

        if !item.value.is_constant(&context.version, false) {
            context.report(
                Issue::error(format!(
                    "Constant `{class_like_name}::{item_name}` value contains a non-constant expression."
                ))
                .with_annotation(Annotation::primary(item.value.span()))
                .with_annotations([
                    Annotation::secondary(item.name.span()).with_message(format!(
                        "{class_like_kind} constant `{class_like_name}::{item_name}` is declared here."
                    )),
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{class_like_kind} `{class_like_fqcn}` is declared here.")),
                ]),
            );
        }
    }
}
