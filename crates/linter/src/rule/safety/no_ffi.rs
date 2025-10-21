use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoFfiRule {
    meta: &'static RuleMeta,
    cfg: NoFfiConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoFfiConfig {
    pub level: Level,
}

impl Default for NoFfiConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoFfiConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoFfiRule {
    type Config = NoFfiConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No FFI",
            code: "no-ffi",
            description: indoc! {"
                Detects unsafe use of the PHP FFI (Foreign Function Interface) extension.

                The FFI extension allows interaction with code written in other languages, such as C, C++, and Rust.
                This can introduce potential security risks and stability issues if not handled carefully.

                If you are confident in your use of FFI and understand the risks, you can disable this rule in your Mago configuration.
            "},
            good_example: indoc! {r#"
                <?php

                // Using a safe alternative to FFI
                $data = 'some data';
                $hash = hash('sha256', $data);
            "#},
            bad_example: indoc! {r#"
                <?php

                use FFI;

                $ffi = FFI::cdef("void* malloc(size_t size);");
                $ffi->malloc(1024); // Allocate memory but never free it
            "#},
            category: Category::Safety,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::StaticMethodCall, NodeKind::ClassConstantAccess, NodeKind::Instantiation, NodeKind::Hint];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let identifier = match node {
            Node::StaticMethodCall(static_method_call) => {
                if let Expression::Identifier(identifier) = static_method_call.class {
                    identifier
                } else {
                    return;
                }
            }
            Node::ClassConstantAccess(class_constant_access) => {
                if let Expression::Identifier(identifier) = class_constant_access.class {
                    identifier
                } else {
                    return;
                }
            }
            Node::Instantiation(instantiation) => {
                if let Expression::Identifier(identifier) = instantiation.class {
                    identifier
                } else {
                    return;
                }
            }
            Node::Hint(Hint::Identifier(identifier)) => identifier,
            _ => return,
        };

        let class_name = ctx.lookup_name(identifier);

        if FFI_CLASSES.iter().any(|ffi| ffi.eq_ignore_ascii_case(class_name)) {
            ctx.collector.report(
                Issue::new(
                    self.cfg.level(),
                    format!("Potentially unsafe use of FFI class `{}`.", class_name),
                )
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(identifier.span())
                    .with_message("This class is part of the FFI extension"))
                .with_note("FFI (Foreign Function Interface) allows interaction with code written in other languages such as C, C++, and Rust.")
                .with_note("This can introduce potential security risks and stability issues if not handled carefully.")
                .with_note("Make sure you understand the implications and potential vulnerabilities before using FFI in production.")
                .with_note("If you are confident in your use of FFI and understand the risks, you can disable this rule in your Mago configuration.")
                .with_help("If possible, consider using alternative solutions within PHP to avoid relying on FFI"),
            );
        }
    }
}

const FFI_CLASSES: [&str; 3] = ["FFI", "FFI\\Cdata", "FFI\\Ctype"];
