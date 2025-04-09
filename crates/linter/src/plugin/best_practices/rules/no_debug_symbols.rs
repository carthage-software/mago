use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoDebugSymbolsRule;

impl Rule for NoDebugSymbolsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Debug Symbols", Level::Note).with_description(indoc! {"
            Flags calls to debug functions like `var_dump`, `print_r`, `debug_backtrace`, etc.
            in production code. Debug functions are useful for debugging, but they can expose
            sensitive information or degrade performance in production environments.
        "})
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(function_call) = node else { return LintDirective::default() };

        let Expression::Identifier(function_identifier) = function_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(function_identifier);

        if DEBUG_FUNCTIONS.contains(&function_name) {
            let issue = Issue::new(context.level(), format!("Usage of debug function `{}` detected.", function_name))
                .with_annotation(
                    Annotation::primary(function_call.span())
                        .with_message(format!("Function `{}` is called here.", function_name)),
                )
                .with_note("Avoid using debug functions like `var_dump`, `print_r`, etc. in production code.")
                .with_help("Remove the debug function call.");

            context.report(issue);
        }

        LintDirective::default()
    }
}

const DEBUG_FUNCTIONS: [&str; 50] = [
    // PHP built-in debug functions
    "var_dump",
    "var_export",
    "print_r",
    "debug_zval_dump",
    "debug_print_backtrace",
    "phpinfo",
    // Symfony var-dumper functions
    "dump",
    // Symfony / Laravel dump+die function
    "dd",
    // Xdebug functions
    "xdebug_debug_zval",
    "xdebug_break",
    "xdebug_call_class",
    "xdebug_call_file",
    "xdebug_call_function",
    "xdebug_call_line",
    "xdebug_code_coverage_started",
    "xdebug_connect_to_client",
    "xdebug_debug_zval",
    "xdebug_debug_zval_stdout",
    "xdebug_dump_superglobals",
    "xdebug_get_code_coverage",
    "xdebug_get_collected_errors",
    "xdebug_get_function_count",
    "xdebug_get_function_stack",
    "xdebug_get_gc_run_count",
    "xdebug_get_gc_total_collected_roots",
    "xdebug_get_gcstats_filename",
    "xdebug_get_headers",
    "xdebug_get_monitored_functions",
    "xdebug_get_profiler_filename",
    "xdebug_get_stack_depth",
    "xdebug_get_tracefile_name",
    "xdebug_info",
    "xdebug_is_debugger_active",
    "xdebug_memory_usage",
    "xdebug_notify",
    "xdebug_peak_memory_usage",
    "xdebug_print_function_stack",
    "xdebug_set_filter",
    "xdebug_start_code_coverage",
    "xdebug_start_error_collection",
    "xdebug_start_function_monitor",
    "xdebug_start_gcstats",
    "xdebug_start_trace",
    "xdebug_stop_code_coverage",
    "xdebug_stop_error_collection",
    "xdebug_stop_function_monitor",
    "xdebug_stop_gcstats",
    "xdebug_stop_trace",
    "xdebug_time_index",
    "xdebug_var_dump",
];
