//! Preset configurations for the `init` command.
//!
//! This module provides predefined formatter configurations that can be used to quickly
//! set up `mago.toml` with common coding standards. Each preset is a complete
//! `[formatter]` TOML section.

use strum_macros::EnumString;

/// Enum representing the available formatter presets.
#[derive(Debug, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum FormatterPreset {
    Laravel,
    Psr12,
    Default,
}

/// Returns the TOML configuration string for the given formatter preset.
///
/// # Arguments
///
/// * `preset` - The `FormatterPreset` to get the configuration for.
///
/// # Returns
///
/// A static string slice containing the TOML configuration for the formatter.
pub fn get_preset_config(preset: FormatterPreset) -> &'static str {
    match preset {
        FormatterPreset::Laravel => LARAVEL_PRESET,
        FormatterPreset::Psr12 => PSR12_PRESET,
        FormatterPreset::Default => DEFAULT_PRESET,
    }
}

/// The default formatter preset, based on PER-CS.
const DEFAULT_PRESET: &str = r#"[formatter]
print-width = 120
tab-width = 4
use-tabs = false
single-quote = true
trailing-comma = true
control-brace-style = "same-line"
empty-line-before-return = true
empty-line-after-opening-brace = false
"#;

/// The PSR-12 formatter preset.
const PSR12_PRESET: &str = r#"[formatter]
print-width = 120
tab-width = 4
use-tabs = false
end-of-line = "lf"
single-quote = true
trailing-comma = false
remove-trailing-close-tag = true
control-brace-style = "same_line"
closure-brace-style = "same_line"
function-brace-style = "next_line"
method-brace-style = "next_line"
classlike-brace-style = "next_line"
inline-empty-control-braces = false
inline-empty-closure-braces = true
inline-empty-function-braces = false
inline-empty-method-braces = false
inline-empty-constructor-braces = false
inline-empty-classlike-braces = false
inline-empty-anonymous-class-braces = true
method-chain-breaking-style = "next_line"
first-method-chain-on-new-line = true
preserve-breaking-member-access-chain = false
preserve-breaking-argument-list = false
preserve-breaking-array-like = true
preserve-breaking-parameter-list = true
preserve-breaking-attribute-list = false
preserve-breaking-conditional-expression = false
break-promoted-properties-list = true
line-before-binary-operator = true
always-break-named-arguments-list = false
always-break-attribute-named-argument-lists = false
array-table-style-alignment = true
align-assignment-like = false
sort-uses = false
sort-class-methods = false
separate-use-types = true
expand-use-groups = false
null-type-hint = "question"
parentheses-around-new-in-member-access = false
parentheses-in-new-expression = true
parentheses-in-exit-and-die = true
parentheses-in-attribute = false
space-before-arrow-function-parameter-list-parenthesis = false
space-before-closure-parameter-list-parenthesis = true
space-before-hook-parameter-list-parenthesis = false
space-before-closure-use-clause-parenthesis = true
space-after-cast-unary-prefix-operators = true
space-after-reference-unary-prefix-operator = false
space-after-error-control-unary-prefix-operator = false
space-after-logical-not-unary-prefix-operator = false
space-after-bitwise-not-unary-prefix-operator = false
space-after-increment-unary-prefix-operator = false
space-after-decrement-unary-prefix-operator = false
space-after-additive-unary-prefix-operator = false
space-around-concatenation-binary-operator = true
space-around-assignment-in-declare = true
space-within-grouping-parenthesis = true
empty-line-after-control-structure = false
empty-line-after-opening-tag = true
empty-line-after-declare = true
empty-line-after-namespace = true
empty-line-after-use = true
empty-line-after-symbols = true
empty-line-between-same-symbols = true
empty-line-after-class-like-constant = false
empty-line-after-enum-case = false
empty-line-after-trait-use = false
empty-line-after-property = false
empty-line-after-method = true
empty-line-before-return = false
empty-line-before-dangling-comments = true
separate-class-like-members = true
"#;

/// The Laravel formatter preset.
const LARAVEL_PRESET: &str = r#"[formatter]
print-width = 120
tab-width = 4
use-tabs = false
end-of-line = "lf"
single-quote = true
trailing-comma = true
remove-trailing-close-tag = true
control-brace-style = "same_line"
closure-brace-style = "same_line"
function-brace-style = "next_line"
method-brace-style = "next_line"
classlike-brace-style = "next_line"
inline-empty-control-braces = false
inline-empty-closure-braces = false
inline-empty-function-braces = false
inline-empty-method-braces = false
inline-empty-constructor-braces = true
inline-empty-classlike-braces = true
inline-empty-anonymous-class-braces = false
method-chain-breaking-style = "next_line"
first-method-chain-on-new-line = true
preserve-breaking-member-access-chain = false
preserve-breaking-argument-list = false
preserve-breaking-array-like = true
preserve-breaking-parameter-list = false
preserve-breaking-attribute-list = false
preserve-breaking-conditional-expression = false
break-promoted-properties-list = true
line-before-binary-operator = true
always-break-named-arguments-list = false
always-break-attribute-named-argument-lists = false
array-table-style-alignment = true
align-assignment-like = false
sort-uses = true
sort-class-methods = false
separate-use-types = true
expand-use-groups = true
null-type-hint = "question"
parentheses-around-new-in-member-access = false
parentheses-in-new-expression = false
parentheses-in-exit-and-die = true
parentheses-in-attribute = false
space-before-arrow-function-parameter-list-parenthesis = false
space-before-closure-parameter-list-parenthesis = true
space-before-hook-parameter-list-parenthesis = false
space-before-closure-use-clause-parenthesis = true
space-after-cast-unary-prefix-operators = true
space-after-reference-unary-prefix-operator = false
space-after-error-control-unary-prefix-operator = false
space-after-logical-not-unary-prefix-operator = false
space-after-bitwise-not-unary-prefix-operator = false
space-after-increment-unary-prefix-operator = false
space-after-decrement-unary-prefix-operator = false
space-after-additive-unary-prefix-operator = false
space-around-concatenation-binary-operator = false
space-around-assignment-in-declare = true
space-within-grouping-parenthesis = true
empty-line-after-control-structure = false
empty-line-after-opening-tag = true
empty-line-after-declare = true
empty-line-after-namespace = true
empty-line-after-use = true
empty-line-after-symbols = true
empty-line-between-same-symbols = true
empty-line-after-class-like-constant = false
empty-line-after-enum-case = false
empty-line-after-trait-use = false
empty-line-after-property = false
empty-line-after-method = true
empty-line-before-return = true
empty-line-before-dangling-comments = true
separate-class-like-members = true
"#;
