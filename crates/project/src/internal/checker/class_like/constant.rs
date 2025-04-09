use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_class_like_constant(
    class_like_constant: &ClassLikeConstant,
    class_like_span: Span,
    class_like_kind: &str,
    class_like_name: &str,
    class_like_fqcn: &str,
    context: &mut Context,
) {
    let first_item = class_like_constant.first_item();
    let first_item_name = context.interner.lookup(&first_item.name.value);

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
                context.issues.push(
                    Issue::error(format!(
                        "`{}` modifier is not allowed on constants",
                        context.interner.lookup(&k.value),
                    ))
                    .with_annotation(Annotation::primary(modifier.span()))
                    .with_annotations([
                        Annotation::secondary(first_item.span()).with_message(format!(
                            "{} constant `{}::{}` is declared here.",
                            class_like_kind, class_like_name, first_item_name
                        )),
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{} `{}` is declared here.", class_like_kind, class_like_fqcn)),
                    ]),
                );
            }
            Modifier::Final(_) => {
                if !context.version.is_supported(Feature::FinalConstants) {
                    context.issues.push(
                        Issue::error("Final class constants are only available in PHP 8.1 and above.").with_annotation(
                            Annotation::primary(modifier.span()).with_message("Final modifier used here."),
                        ),
                    );
                }

                if let Some(last_final) = last_final {
                    context.issues.push(
                        Issue::error("duplicate `final` modifier on constant")
                            .with_annotation(Annotation::primary(modifier.span()))
                            .with_annotations([
                                Annotation::secondary(last_final).with_message("previous `final` modifier"),
                                Annotation::secondary(first_item.span()).with_message(format!(
                                    "{} constant `{}::{}` is declared here.",
                                    class_like_kind, class_like_name, first_item_name
                                )),
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is declared here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ]),
                    );
                }

                last_final = Some(modifier.span());
            }
            Modifier::Private(_) | Modifier::Protected(_) | Modifier::Public(_) => {
                if !context.version.is_supported(Feature::ClassLikeConstantVisibilityModifiers) {
                    context.issues.push(
                        Issue::error(
                            "Visibility modifiers for class constants are only available in PHP 7.1 and above.",
                        )
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Visibility modifier used here."),
                        ),
                    );
                }

                if let Some(last_visibility) = last_visibility {
                    context.issues.push(
                        Issue::error("duplicate visibility modifier on constant")
                            .with_annotation(Annotation::primary(modifier.span()))
                            .with_annotations([
                                Annotation::secondary(last_visibility).with_message("previous visibility modifier"),
                                Annotation::secondary(first_item.span()).with_message(format!(
                                    "{} constant `{}::{}` is declared here.",
                                    class_like_kind, class_like_name, first_item_name
                                )),
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is declared here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ]),
                    );
                }

                last_visibility = Some(modifier.span());
            }
        }
    }

    if let Some(type_hint) = &class_like_constant.hint {
        if !context.version.is_supported(Feature::TypedClassLikeConstants) {
            context.issues.push(
                Issue::error("Typed class constants are only available in PHP 8.3 and above.")
                    .with_annotation(Annotation::primary(type_hint.span()).with_message("Type hint used here.")),
            );
        };
    }

    for item in class_like_constant.items.iter() {
        let item_name = context.interner.lookup(&item.name.value);

        if !item.value.is_constant(context.version, false) {
            context.issues.push(
                Issue::error(format!(
                    "Constant `{}::{}` value contains a non-constant expression.",
                    class_like_name, item_name
                ))
                .with_annotation(Annotation::primary(item.value.span()))
                .with_annotations([
                    Annotation::secondary(item.name.span()).with_message(format!(
                        "{} constant `{}::{}` is declared here.",
                        class_like_kind, class_like_name, item_name
                    )),
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{} `{}` is declared here.", class_like_kind, class_like_fqcn)),
                ]),
            );
        }
    }
}
