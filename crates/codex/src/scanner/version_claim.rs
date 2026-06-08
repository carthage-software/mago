use mago_allocator::Arena;
use mago_php_version::PHPVersion;
use mago_syntax::ast::ArgumentList;
use mago_syntax::ast::AttributeList;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::LiteralInteger;
use mago_syntax::ast::LiteralString;
use mago_syntax::ast::Sequence;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_word::Word;
use mago_word::word;

use crate::metadata::version_constraint::VersionConstraint;
use crate::scanner::Context;

/// Outcome of evaluating the `Mago\*` version-gating attributes on a symbol
/// against the configured PHP version.
#[derive(Debug, Clone, Default)]
pub struct VersionVerdict {
    pub constraint: VersionConstraint,
    pub optional: Option<bool>,
    pub type_override: Option<TypeOverride>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeOverride {
    Typed(Word),
    Untyped,
}

impl VersionVerdict {
    /// Convenience verdict used when no claims are present (the common case).
    #[inline]
    #[must_use]
    pub const fn always() -> Self {
        Self { constraint: VersionConstraint::unconstrained(), optional: None, type_override: None }
    }

    /// Returns `true` when `version` falls within the version constraint
    /// resolved on this verdict.
    #[inline]
    #[must_use]
    pub fn is_available(&self, version: PHPVersion) -> bool {
        self.constraint.allows_version(version)
    }
}

/// Walks the attribute list a single time and resolves all `Mago\*` claims
/// against `version`. Returns [`VersionVerdict::always`] (the cheap default)
/// when no `Mago\*` attributes are present, so callers pay nothing on the
/// overwhelmingly common no-attribute path.
pub fn evaluate_version_attributes<'arena, A>(
    attribute_lists: &'arena Sequence<'arena, AttributeList<'arena>>,
    context: &Context<'_, 'arena, A>,
    version: PHPVersion,
) -> VersionVerdict
where
    A: Arena,
{
    if attribute_lists.is_empty() {
        return VersionVerdict::always();
    }

    let mut verdict = VersionVerdict::always();

    let mut optional_latest = PHPVersion::from_version_id(0);
    let mut type_latest = PHPVersion::from_version_id(0);
    let mut have_optional = false;
    let mut have_type = false;

    for attribute_list in attribute_lists {
        for attribute in &attribute_list.attributes {
            let resolved = context.resolved_names.get(&attribute.name);
            let Some(kind) = recognize(resolved) else {
                continue;
            };

            let Some(argument_list) = attribute.argument_list.as_ref() else {
                continue;
            };

            apply_claim(
                kind,
                argument_list,
                version,
                &mut verdict,
                &mut optional_latest,
                &mut have_optional,
                &mut type_latest,
                &mut have_type,
            );
        }
    }

    verdict
}

/// Decodes a `PHP_VERSION_ID`-style decimal integer (e.g. `80400`) into a
/// [`PHPVersion`]. This is the integer convention used by the
/// `Mago\Available*` / `Mago\Optional*` / `Mago\Required*` attribute
/// arguments — distinct from [`PHPVersion::to_version_id`], which packs
/// the major/minor/patch components into separate bytes of a `u32`.
#[inline]
#[must_use]
fn decode_decimal_version_id(decimal: u32) -> PHPVersion {
    let major = decimal / 10_000;
    let minor = (decimal / 100) % 100;
    let patch = decimal % 100;
    PHPVersion::new(major, minor, patch)
}

#[derive(Debug, Clone, Copy)]
enum ClaimKind {
    AvailableSince,
    AvailableUntil,
    OptionalSince,
    OptionalUntil,
    RequiredSince,
    RequiredUntil,
    TypedWithSince,
    TypedWithUntil,
    UntypedSince,
    UntypedUntil,
}

fn recognize(resolved_name: &[u8]) -> Option<ClaimKind> {
    let bytes = mago_bytes::trim_start_byte(resolved_name, b'\\');
    if bytes.len() < 5 || !bytes[..5].eq_ignore_ascii_case(b"Mago\\") {
        return None;
    }

    let suffix = &bytes[5..];
    if suffix.eq_ignore_ascii_case(b"AvailableSince") {
        Some(ClaimKind::AvailableSince)
    } else if suffix.eq_ignore_ascii_case(b"AvailableUntil") {
        Some(ClaimKind::AvailableUntil)
    } else if suffix.eq_ignore_ascii_case(b"OptionalSince") {
        Some(ClaimKind::OptionalSince)
    } else if suffix.eq_ignore_ascii_case(b"OptionalUntil") {
        Some(ClaimKind::OptionalUntil)
    } else if suffix.eq_ignore_ascii_case(b"RequiredSince") {
        Some(ClaimKind::RequiredSince)
    } else if suffix.eq_ignore_ascii_case(b"RequiredUntil") {
        Some(ClaimKind::RequiredUntil)
    } else if suffix.eq_ignore_ascii_case(b"TypedWithSince") {
        Some(ClaimKind::TypedWithSince)
    } else if suffix.eq_ignore_ascii_case(b"TypedWithUntil") {
        Some(ClaimKind::TypedWithUntil)
    } else if suffix.eq_ignore_ascii_case(b"UntypedSince") {
        Some(ClaimKind::UntypedSince)
    } else if suffix.eq_ignore_ascii_case(b"UntypedUntil") {
        Some(ClaimKind::UntypedUntil)
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_claim<'arena>(
    kind: ClaimKind,
    argument_list: &'arena ArgumentList<'arena>,
    version: PHPVersion,
    verdict: &mut VersionVerdict,
    optional_latest: &mut PHPVersion,
    have_optional: &mut bool,
    type_latest: &mut PHPVersion,
    have_type: &mut bool,
) {
    let positional: Vec<&Expression<'_>> =
        argument_list.arguments.iter().map(mago_syntax::ast::Argument::value).collect();

    let parse_version = |expr: &Expression<'_>| literal_u32(expr).map(decode_decimal_version_id);

    match kind {
        ClaimKind::AvailableSince => {
            if let Some(v) = positional.first().and_then(|e| parse_version(e)) {
                verdict.constraint.push_since(v);
            }
        }
        ClaimKind::AvailableUntil => {
            if let Some(v) = positional.first().and_then(|e| parse_version(e)) {
                verdict.constraint.push_until(v);
            }
        }
        ClaimKind::OptionalSince => {
            if let Some(v) = positional.first().and_then(|e| parse_version(e))
                && version >= v
                && (!*have_optional || v >= *optional_latest)
            {
                *optional_latest = v;
                *have_optional = true;
                verdict.optional = Some(true);
            }
        }
        ClaimKind::RequiredSince => {
            if let Some(v) = positional.first().and_then(|e| parse_version(e))
                && version >= v
                && (!*have_optional || v >= *optional_latest)
            {
                *optional_latest = v;
                *have_optional = true;
                verdict.optional = Some(false);
            }
        }
        ClaimKind::OptionalUntil => {
            if let Some(v) = positional.first().and_then(|e| parse_version(e))
                && version <= v
                && (!*have_optional || v >= *optional_latest)
            {
                *optional_latest = v;
                *have_optional = true;
                verdict.optional = Some(true);
            }
        }
        ClaimKind::RequiredUntil => {
            if let Some(v) = positional.first().and_then(|e| parse_version(e))
                && version <= v
                && (!*have_optional || v >= *optional_latest)
            {
                *optional_latest = v;
                *have_optional = true;
                verdict.optional = Some(false);
            }
        }
        ClaimKind::TypedWithSince => {
            if let (Some(ty), Some(v)) =
                (positional.first().and_then(|e| literal_string(e)), positional.get(1).and_then(|e| parse_version(e)))
                && version >= v
                && (!*have_type || v >= *type_latest)
            {
                *type_latest = v;
                *have_type = true;
                verdict.type_override = Some(TypeOverride::Typed(ty));
            }
        }
        ClaimKind::TypedWithUntil => {
            if let (Some(ty), Some(v)) =
                (positional.first().and_then(|e| literal_string(e)), positional.get(1).and_then(|e| parse_version(e)))
                && version <= v
                && (!*have_type || v >= *type_latest)
            {
                *type_latest = v;
                *have_type = true;
                verdict.type_override = Some(TypeOverride::Typed(ty));
            }
        }
        ClaimKind::UntypedSince => {
            if let Some(v) = positional.first().and_then(|e| parse_version(e))
                && version >= v
                && (!*have_type || v >= *type_latest)
            {
                *type_latest = v;
                *have_type = true;
                verdict.type_override = Some(TypeOverride::Untyped);
            }
        }
        ClaimKind::UntypedUntil => {
            if let Some(v) = positional.first().and_then(|e| parse_version(e))
                && version <= v
                && (!*have_type || v >= *type_latest)
            {
                *type_latest = v;
                *have_type = true;
                verdict.type_override = Some(TypeOverride::Untyped);
            }
        }
    }
}

fn literal_u32(expression: &Expression<'_>) -> Option<u32> {
    match expression.unparenthesized() {
        Expression::Literal(Literal::Integer(LiteralInteger { value: Some(value), .. })) => (*value).try_into().ok(),
        Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Plus(_), operand }) => {
            literal_u32(operand)
        }
        _ => None,
    }
}

fn literal_string(expression: &Expression<'_>) -> Option<Word> {
    match expression.unparenthesized() {
        Expression::Literal(Literal::String(LiteralString { value: Some(value), .. })) => Some(word(value)),
        _ => None,
    }
}
