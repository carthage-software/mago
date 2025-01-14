use indoc::indoc;

use mago_ast::ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

const ALIAS_TO_FUNCTION: [(&str, &str); 68] = [
    // @internal aliases
    ("diskfreespace", "disk_free_space"),
    ("dns_check_record", "checkdnsrr"),
    ("dns_get_mx", "getmxrr"),
    ("session_commit", "session_write_close"),
    ("stream_register_wrapper", "stream_wrapper_register"),
    ("set_file_buffer", "stream_set_write_buffer"),
    ("socket_set_blocking", "stream_set_blocking"),
    ("socket_get_status", "stream_get_meta_data"),
    ("socket_set_timeout", "stream_set_timeout"),
    ("socket_getopt", "socket_get_option"),
    ("socket_setopt", "socket_set_option"),
    ("chop", "rtrim"),
    ("close", "closedir"),
    ("doubleval", "floatval"),
    ("fputs", "fwrite"),
    ("get_required_files", "get_included_files"),
    ("ini_alter", "ini_set"),
    ("is_double", "is_float"),
    ("is_integer", "is_int"),
    ("is_long", "is_int"),
    ("is_real", "is_float"),
    ("is_writeable", "is_writable"),
    ("join", "implode"),
    ("key_exists", "array_key_exists"),
    ("magic_quotes_runtime", "set_magic_quotes_runtime"),
    ("pos", "current"),
    ("show_source", "highlight_file"),
    ("sizeof", "count"),
    ("strchr", "strstr"),
    ("user_error", "trigger_error"),
    // @IMAP aliases
    ("imap_create", "imap_createmailbox"),
    ("imap_fetchtext", "imap_body"),
    ("imap_header", "imap_headerinfo"),
    ("imap_listmailbox", "imap_list"),
    ("imap_listsubscribed", "imap_lsub"),
    ("imap_rename", "imap_renamemailbox"),
    ("imap_scan", "imap_listscan"),
    ("imap_scanmailbox", "imap_listscan"),
    // @ldap aliases
    ("ldap_close", "ldap_unbind"),
    ("ldap_modify", "ldap_mod_replace"),
    // @mysqli aliases
    ("mysqli_execute", "mysqli_stmt_execute"),
    ("mysqli_set_opt", "mysqli_options"),
    ("mysqli_escape_string", "mysqli_real_escape_string"),
    // @pg aliases
    ("pg_exec", "pg_query"),
    // @oci aliases
    ("oci_free_cursor", "oci_free_statement"),
    // @odbc aliases
    ("odbc_do", "odbc_exec"),
    ("odbc_field_precision", "odbc_field_len"),
    // @mbreg aliases
    ("mbereg", "mb_ereg"),
    ("mbereg_match", "mb_ereg_match"),
    ("mbereg_replace", "mb_ereg_replace"),
    ("mbereg_search", "mb_ereg_search"),
    ("mbereg_search_getpos", "mb_ereg_search_getpos"),
    ("mbereg_search_getregs", "mb_ereg_search_getregs"),
    ("mbereg_search_init", "mb_ereg_search_init"),
    ("mbereg_search_pos", "mb_ereg_search_pos"),
    ("mbereg_search_regs", "mb_ereg_search_regs"),
    ("mbereg_search_setpos", "mb_ereg_search_setpos"),
    ("mberegi", "mb_eregi"),
    ("mberegi_replace", "mb_eregi_replace"),
    ("mbregex_encoding", "mb_regex_encoding"),
    ("mbsplit", "mb_split"),
    // @openssl aliases
    ("openssl_get_publickey", "openssl_pkey_get_public"),
    ("openssl_get_privatekey", "openssl_pkey_get_private"),
    // @sodium aliases
    ("sodium_crypto_scalarmult_base", "sodium_crypto_box_publickey_from_secretkey"),
    // @exif aliases
    ("read_exif_data", "exif_read_data"),
    // @ftp aliases
    ("ftp_quit", "ftp_close"),
    // @posix aliases
    ("posix_errno", "posix_get_last_error"),
    // @pcntl aliases
    ("pcntl_errno", "pcntl_get_last_error"),
];

#[derive(Clone, Copy, Debug)]
pub struct NoFunctionAliasesRule;

impl Rule for NoFunctionAliasesRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Function Aliases", Level::Note)
            .with_description(indoc! {"
                Detects usage of function aliases (e.g., `diskfreespace` instead of `disk_free_space`)
                and suggests calling the canonical (original) function name instead.
                This is primarily for consistency and clarity.
            "})
            .with_example(RuleUsageExample::valid(
                "Using canonical function names",
                indoc! {r#"
                    <?php

                    // 'disk_free_space' is the proper name instead of 'diskfreespace'
                    $freeSpace = disk_free_space("/");
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using an aliased function",
                indoc! {r#"
                    <?php

                    // 'diskfreespace' is an alias for 'disk_free_space'
                    $freeSpace = diskfreespace("/");
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for NoFunctionAliasesRule {
    fn walk_in_function_call<'ast>(&self, function_call: &'ast FunctionCall, context: &mut LintContext<'a>) {
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return;
        };

        let function_name = context.resolve_function_name(identifier);

        let original_name = ALIAS_TO_FUNCTION.iter().find_map(|&(alias, original)| {
            if alias.eq_ignore_ascii_case(function_name) {
                Some(original)
            } else {
                None
            }
        });

        if let Some(original_name) = original_name {
            // Build the diagnostic message
            let issue = Issue::new(context.level(), format!("Function alias `{}` should not be used.", function_name))
                .with_annotation(
                    Annotation::primary(identifier.span())
                        .with_message(format!("This function is an alias of `{}`.", original_name)),
                )
                .with_note(format!("The function `{}` is an alias of `{}`.", function_name, original_name))
                .with_help(format!("Consider using the function `{}` instead.", original_name));

            context.report_with_fix(issue, |p| {
                p.replace(identifier.span().into(), format!("\\{}", original_name), SafetyClassification::Safe)
            });
        }
    }
}
