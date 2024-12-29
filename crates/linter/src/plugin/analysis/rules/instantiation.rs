use mago_ast::*;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct InstantiationRule;

impl Rule for InstantiationRule {
    fn get_name(&self) -> &'static str {
        "instantiation"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Error)
    }
}

impl InstantiationRule {}

impl<'a> Walker<LintContext<'a>> for InstantiationRule {
    fn walk_in_instantiation(&self, instantiation: &Instantiation, context: &mut LintContext<'a>) {
        let Expression::Identifier(class_identifier) = &instantiation.class else {
            return;
        };

        let class_name_identifier = context.semantics.names.get(class_identifier);
        let class_name = context.lookup(&class_identifier.value()).to_string();
        let class_fqcn = context.lookup(class_name_identifier).to_string();

        let Some(reflection) = context.codebase.get_named_class_like(context.interner, class_name_identifier) else {
            let issue = Issue::error(format!("Instantiated class `{}` does not exist.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Class `{}` does not exist.", class_fqcn)),
                )
                .with_help(format!(
                    "Ensure the class `{}` is defined or imported before instantiating it.",
                    class_fqcn
                ));

            context.report(issue);

            return;
        };

        if reflection.is_interface() {
            let issue = Issue::error(format!("Cannot instantiate interface `{}`.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Interface `{}` cannot be instantiated.", class_fqcn)),
                )
                .with_note("Interfaces are abstract types that cannot be instantiated directly.")
                .with_note("Use a class that implements the interface instead.");

            context.report(issue);

            return;
        }

        if reflection.is_trait() {
            let issue = Issue::error(format!("Cannot instantiate trait `{}`.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Trait `{}` cannot be instantiated.", class_fqcn)),
                )
                .with_note("Traits are abstract types that cannot be instantiated directly.")
                .with_note("Use a class that implements the trait instead.");

            context.report(issue);

            return;
        }

        if reflection.is_enum() {
            let issue = Issue::error(format!("Cannot instantiate enum `{}`.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Enum `{}` cannot be instantiated.", class_fqcn)),
                )
                .with_note("Enums are types that represent a fixed set of constants.")
                .with_note("Use one of the enum cases instead.");

            context.report(issue);

            return;
        }

        if reflection.is_abstract {
            let issue = Issue::error(format!("Cannot instantiate abstract class `{}`.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Abstract class `{}` cannot be instantiated.", class_fqcn)),
                )
                .with_note("Abstract classes are incomplete types that cannot be instantiated directly.")
                .with_note("Use a concrete subclass instead.");

            context.report(issue);
        }
    }
}
