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
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct UseWpFunctionsRule {
    meta: &'static RuleMeta,
    cfg: UseWpFunctionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct UseWpFunctionsConfig {
    pub level: Level,
}

impl Default for UseWpFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for UseWpFunctionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for UseWpFunctionsRule {
    type Config = UseWpFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Use WordPress API Functions",
            code: "use-wp-functions",
            description: indoc! {"
                This rule encourages using WordPress's wrapper functions instead of native PHP functions for
                common tasks like HTTP requests, filesystem operations, and data handling. The WordPress APIs
                provide a consistent, secure, and reliable abstraction that works across different hosting
                environments.
            "},
            good_example: indoc! {r#"
                <?php

                // For remote requests:
                $response = wp_remote_get('https://example.com/api/data');

                // For filesystem operations:
                global $wp_filesystem;
                require_once ABSPATH . 'wp-admin/includes/file.php';
                WP_Filesystem();
                $wp_filesystem->put_contents( '/path/to/my-file.txt', 'data' );
            "#},
            bad_example: indoc! {r#"
                <?php

                // For remote requests:
                $ch = curl_init();
                curl_setopt($ch, CURLOPT_URL, 'https://example.com/api/data');
                // ...

                // For filesystem operations:
                file_put_contents('/path/to/my-file.txt', 'data');
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::WordPress),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        // Define function mappings with their WordPress alternatives
        const FUNCTION_MAPPINGS: &[(&[&str], &str, &str)] = &[
            // HTTP functions
            (
                &["curl_init", "curl_exec", "curl_setopt", "curl_close", "curl_getinfo"],
                "wp_remote_get() or wp_remote_post()",
                "Use WordPress HTTP API functions for better compatibility and security",
            ),
            // Filesystem functions
            (
                &[
                    "file_put_contents",
                    "fopen",
                    "fwrite",
                    "fread",
                    "fclose",
                    "unlink",
                    "rmdir",
                    "mkdir",
                    "copy",
                    "rename",
                    "chmod",
                ],
                "WP_Filesystem API",
                "Use WP_Filesystem API for file operations",
            ),
            // JSON functions
            (&["json_encode"], "wp_json_encode()", "Use wp_json_encode() for better security and compatibility"),
            (
                &["json_decode"],
                "wp_json_decode() (if available) or validate the result",
                "Consider using WordPress alternatives or validate JSON decode results",
            ),
            // Mail functions
            (&["mail"], "wp_mail()", "Use wp_mail() for better compatibility and filtering support"),
            // Cache functions
            (
                &["apc_fetch", "apc_store", "memcache_get", "memcache_set"],
                "wp_cache_get() and wp_cache_set()",
                "Use WordPress object cache functions",
            ),
        ];

        // Check if this function call matches any of our target functions
        for (function_names, wp_function, _message) in FUNCTION_MAPPINGS {
            if let Some(matched_function) = function_call_matches_any(ctx, function_call, function_names) {
                // Special case for file_get_contents - only flag if used for HTTP
                if matched_function == "file_get_contents" && !is_http_url_in_args(function_call) {
                    continue;
                }

                let issue =
                    Issue::new(self.cfg.level(), format!("Use WordPress function instead of `{}`", matched_function))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(function_call.span())
                                .with_message(format!("Replace with `{}`", wp_function)),
                        )
                        .with_note("Native PHP functions lack WordPress compatibility features")
                        .with_help(format!("Use `{}`", wp_function));

                ctx.collector.report(issue);
                return; // Only report one issue per function call
            }
        }
    }
}

/// Check if function call arguments contain HTTP URL
fn is_http_url_in_args(function_call: &FunctionCall) -> bool {
    let argument_list = &function_call.argument_list;

    for argument in argument_list.arguments.iter() {
        if let Expression::Literal(Literal::String(string_literal)) = &argument.value()
            && let Some(value) = extract_string_value(string_literal)
            && (value.starts_with("http://") || value.starts_with("https://"))
        {
            return true;
        }
    }

    false
}

/// Extract string value from a literal string
fn extract_string_value<'a>(string_literal: &'a LiteralString<'a>) -> Option<&'a str> {
    string_literal.value
}
