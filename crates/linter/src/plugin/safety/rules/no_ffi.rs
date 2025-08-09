use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoFFIRule;

impl Rule for NoFFIRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No FFI", Level::Error)
            .with_description(indoc! {"
                Detects unsafe use of the PHP FFI (Foreign Function Interface) extension.

                The FFI extension allows interaction with code written in other languages, such as C, C++, and Rust.
                This can introduce potential security risks and stability issues if not handled carefully.
            "})
            .with_example(RuleUsageExample::invalid(
                "Using the FFI extension",
                indoc! {r#"
                    <?php

                    use FFI;

                    $ffi = FFI::cdef("void* malloc(size_t size);");
                    $ffi->malloc(1024); // Allocate memory but never free it
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let identifier = match node {
            Node::StaticMethodCall(static_method_call) => {
                if let Expression::Identifier(identifier) = static_method_call.class.as_ref() {
                    identifier
                } else {
                    return LintDirective::default();
                }
            }
            Node::ClassConstantAccess(class_constant_access) => {
                if let Expression::Identifier(identifier) = class_constant_access.class.as_ref() {
                    identifier
                } else {
                    return LintDirective::default();
                }
            }
            Node::Instantiation(instantiation) => {
                if let Expression::Identifier(identifier) = instantiation.class.as_ref() {
                    identifier
                } else {
                    return LintDirective::default();
                }
            }

            Node::Hint(Hint::Identifier(identifier)) => identifier,
            _ => return LintDirective::default(),
        };

        let class_name = context.lookup_name(identifier);

        if FFI_CLASSES.iter().any(|ffi| ffi.eq_ignore_ascii_case(class_name)) {
            context.report(
                Issue::new(
                   context.level(),
                   format!("Potentially unsafe use of FFI class `{class_name}`."),
                )
                .with_annotation(Annotation::primary(identifier.span()).with_message("This class is part of the FFI extension."))
                .with_note("FFI (Foreign Function Interface) allows interaction with code written in other languages such as C, C++, and Rust.")
                .with_note("This can introduce potential security risks and stability issues if not handled carefully.")
                .with_note("Make sure you understand the implications and potential vulnerabilities before using FFI in production.")
                .with_note("If you are confident in your use of FFI and understand the risks, you can disable this rule in your Mago configuration.")
                .with_help("If possible, consider using alternative solutions within PHP to avoid relying on FFI.")
            );

            LintDirective::Prune
        } else {
            LintDirective::Continue
        }
    }
}

const FFI_CLASSES: [&str; 3] = ["FFI", "FFI\\Cdata", "FFI\\Ctype"];
