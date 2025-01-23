use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

const FFI_CLASSES: [&str; 3] = ["FFI", "FFI\\Cdata", "FFI\\Ctype"];

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
}

impl NoFFIRule {
    fn report(&self, identifier: &Identifier, node: Option<&impl HasSpan>, context: &mut LintContext<'_>) {
        let class_name = context.lookup_name(identifier);

        if FFI_CLASSES.iter().any(|ffi| ffi.eq_ignore_ascii_case(class_name)) {
            let mut issue = Issue::new(
               context.level(),
               format!("Potentionally unsafe use of FFI class `{}`.", class_name),
            )
            .with_annotation(Annotation::primary(identifier.span()))
            .with_note("FFI (Foreign Function Interface) allows interaction with code written in other languages such as C, C++, and Rust.")
            .with_note("This can introduce potential security risks and stability issues if not handled carefully.")
            .with_note("Make sure you understand the implications and potential vulnerabilities before using FFI in production.")
            .with_note("If you are confident in your use of FFI and understand the risks, you can disable this rule in your Mago configuration.")
            .with_help("If possible, consider using alternative solutions within PHP to avoid relying on FFI.");

            if let Some(node) = node {
                issue = issue.with_annotation(Annotation::secondary(node.span()));
            }

            context.report(issue);
        }
    }
}

impl<'a> Walker<LintContext<'a>> for NoFFIRule {
    fn walk_in_static_method_call<'ast>(
        &self,
        static_method_call: &'ast StaticMethodCall,
        context: &mut LintContext<'a>,
    ) {
        let Expression::Identifier(class_identifier) = static_method_call.class.as_ref() else {
            return;
        };

        self.report(class_identifier, Some(static_method_call), context);
    }

    fn walk_in_class_constant_access<'ast>(
        &self,
        class_constant_access: &'ast ClassConstantAccess,
        context: &mut LintContext<'a>,
    ) {
        let Expression::Identifier(class_identifier) = class_constant_access.class.as_ref() else {
            return;
        };

        self.report(class_identifier, Some(class_constant_access), context);
    }

    fn walk_in_instantiation<'ast>(&self, instantiation: &'ast Instantiation, context: &mut LintContext<'a>) {
        let Expression::Identifier(class_identifier) = instantiation.class.as_ref() else {
            return;
        };

        self.report(class_identifier, Some(instantiation), context);
    }

    fn walk_in_hint<'ast>(&self, hint: &'ast Hint, context: &mut LintContext<'a>) {
        let Hint::Identifier(identifier) = &hint else {
            return;
        };

        self.report(identifier, Option::<&Hint>::None, context);
    }
}
