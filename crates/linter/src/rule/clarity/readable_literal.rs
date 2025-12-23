use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const DEFAULT_MIN_DIGITS: usize = 5;

#[derive(Debug, Clone)]
pub struct ReadableLiteralRule {
    meta: &'static RuleMeta,
    cfg: ReadableLiteralConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReadableLiteralConfig {
    pub level: Level,
    /// Minimum number of digits before suggesting separators.
    #[serde(alias = "min-digits")]
    pub min_digits: usize,
}

impl Default for ReadableLiteralConfig {
    fn default() -> Self {
        Self { level: Level::Warning, min_digits: DEFAULT_MIN_DIGITS }
    }
}

impl Config for ReadableLiteralConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        false
    }
}

impl LintRule for ReadableLiteralRule {
    type Config = ReadableLiteralConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Readable Literal",
            code: "readable-literal",
            description: indoc! {"
                Enforces using underscore separators in numeric literals for improved readability.
            "},
            good_example: indoc! {r"
                <?php

                $a = 1_000_000;
                $b = 0xCAFE_F00D;
                $c = 0b0101_1111;
            "},
            bad_example: indoc! {r"
                <?php

                $a = 1000000;
                $b = 0xCAFEF00D;
                $c = 0b01011111;
            "},
            category: Category::Clarity,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP74)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::LiteralInteger, NodeKind::LiteralFloat];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let (raw, span) = match node {
            Node::LiteralInteger(integer) => (integer.raw, integer.span()),
            Node::LiteralFloat(float) => (float.raw, float.span()),
            _ => return,
        };

        if raw.contains('_') {
            return;
        }

        let digit_count = count_significant_digits(raw);
        if digit_count < self.cfg.min_digits {
            return;
        }

        let suggested = suggest_separated_literal(raw);

        let issue = Issue::new(self.cfg.level(), "Numeric literal could use underscore separators for readability.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(span).with_message("Consider adding underscore separators to this literal"),
            )
            .with_note("Underscore separators improve readability of large numeric literals.")
            .with_help(format!("For example, `{raw}` could be written as `{suggested}`."));

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(span, suggested));
        });
    }
}

/// Count the number of significant digits in a numeric literal.
/// This excludes prefixes (0x, 0b, 0o), signs, decimal points, and exponents.
fn count_significant_digits(raw: &str) -> usize {
    let raw_bytes = raw.as_bytes();

    if raw_bytes.len() >= 2 && raw_bytes[0] == b'0' {
        match raw_bytes[1] {
            b'x' | b'X' => {
                return raw[2..].chars().filter(|c| c.is_ascii_hexdigit()).count();
            }
            b'b' | b'B' => {
                return raw[2..].chars().filter(|c| *c == '0' || *c == '1').count();
            }
            b'o' | b'O' => {
                return raw[2..].chars().filter(|c| ('0'..='7').contains(c)).count();
            }
            b'0'..=b'7' if !raw.contains('.') => {
                return raw[1..].chars().filter(|c| ('0'..='7').contains(c)).count();
            }
            _ => {}
        }
    }

    raw.chars().filter(|c| c.is_ascii_digit()).count()
}

/// Suggest a version of the literal with underscore separators.
/// Preserves the original case of prefixes.
fn suggest_separated_literal(raw: &str) -> String {
    let raw_bytes = raw.as_bytes();

    if raw_bytes.len() >= 2 && raw_bytes[0] == b'0' {
        match raw_bytes[1] {
            b'x' | b'X' => {
                // Hex: group by 4 digits, preserve prefix case
                let prefix = &raw[..2];
                let digits: String = raw[2..].chars().filter(|c| c.is_ascii_hexdigit()).collect();
                return format!("{}{}", prefix, group_digits_from_right(&digits, 4));
            }
            b'b' | b'B' => {
                // Binary: group by 4 digits, preserve prefix case
                let prefix = &raw[..2];
                let digits: String = raw[2..].chars().filter(|c| *c == '0' || *c == '1').collect();
                return format!("{}{}", prefix, group_digits_from_right(&digits, 4));
            }
            b'o' | b'O' => {
                // Explicit octal: group by 3 digits, preserve prefix case
                let prefix = &raw[..2];
                let digits: String = raw[2..].chars().filter(|c| ('0'..='7').contains(c)).collect();
                return format!("{}{}", prefix, group_digits_from_right(&digits, 3));
            }
            b'0'..=b'7' if !raw.contains('.') => {
                // Legacy octal: group by 3 digits after the leading 0
                let digits: String = raw[1..].chars().filter(|c| ('0'..='7').contains(c)).collect();
                return format!("0{}", group_digits_from_right(&digits, 3));
            }
            _ => {}
        }
    }

    let raw_lower = raw.to_lowercase();
    if raw.contains('.') || raw_lower.contains('e') {
        return suggest_separated_float(raw);
    }

    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    group_digits_from_right(&digits, 3)
}

/// Group digits from right to left with underscore separators.
fn group_digits_from_right(digits: &str, group_size: usize) -> String {
    let chars: Vec<char> = digits.chars().collect();
    let mut result = String::new();

    for (i, ch) in chars.iter().rev().enumerate() {
        if i > 0 && i % group_size == 0 {
            result.push('_');
        }

        result.push(*ch);
    }

    result.chars().rev().collect()
}

/// Suggest a separated version of a float literal.
/// Preserves the original case of exponent marker (e/E).
fn suggest_separated_float(raw: &str) -> String {
    let exp_pos = raw.find(['e', 'E']);
    let (mantissa, exponent) = if let Some(pos) = exp_pos { (&raw[..pos], Some(&raw[pos..])) } else { (raw, None) };

    let (integer_part, fractional_part) = if let Some(dot_pos) = mantissa.find('.') {
        (&mantissa[..dot_pos], Some(&mantissa[dot_pos..]))
    } else {
        (mantissa, None)
    };

    let integer_digits: String = integer_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let grouped_integer =
        if integer_digits.len() >= 4 { group_digits_from_right(&integer_digits, 3) } else { integer_digits };

    let grouped_fractional = fractional_part.map(|frac| {
        let digits: String = frac[1..].chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 { format!(".{}", group_digits_from_left(&digits, 3)) } else { frac.to_string() }
    });

    let mut result = grouped_integer;
    if let Some(frac) = grouped_fractional {
        result.push_str(&frac);
    }

    if let Some(exp) = exponent {
        result.push_str(exp);
    }

    result
}

fn group_digits_from_left(digits: &str, group_size: usize) -> String {
    let chars: Vec<char> = digits.chars().collect();
    let mut result = String::new();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && i % group_size == 0 {
            result.push('_');
        }
        result.push(*ch);
    }

    result
}
