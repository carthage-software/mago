use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_syntax::cst;

use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

#[derive(Clone, Copy)]
enum AvailabilityClaim {
    Since,
    Until,
}

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_version_constraint(
        &self,
        attribute_lists: &'scratch cst::Sequence<'scratch, cst::AttributeList<'scratch>>,
    ) -> &'arena [PHPVersionRange] {
        if !self.settings.lower_availability_attributes || attribute_lists.is_empty() {
            return &[];
        }

        let mut ranges = Vec::new_in(self.arena);
        for attribute_list in attribute_lists.iter() {
            for attribute in attribute_list.attributes.iter() {
                let resolved =
                    self.namespace_resolution.resolve_name(NameResolutionKind::Default, attribute.name.value());
                let Some(claim) = recognize_availability_claim(resolved) else {
                    continue;
                };

                let Some(argument_list) = attribute.argument_list.as_ref() else {
                    continue;
                };

                let Some(version) = first_version_argument(argument_list) else {
                    continue;
                };

                match claim {
                    AvailabilityClaim::Since => push_since(&mut ranges, version),
                    AvailabilityClaim::Until => push_until(&mut ranges, version),
                }
            }
        }

        ranges.leak()
    }
}

fn recognize_availability_claim(resolved_name: &[u8]) -> Option<AvailabilityClaim> {
    let mut bytes = resolved_name;
    while let Some(rest) = bytes.strip_prefix(b"\\") {
        bytes = rest;
    }

    if bytes.len() < 5 || !bytes[..5].eq_ignore_ascii_case(b"Mago\\") {
        return None;
    }

    let suffix = &bytes[5..];
    if suffix.eq_ignore_ascii_case(b"AvailableSince") {
        Some(AvailabilityClaim::Since)
    } else if suffix.eq_ignore_ascii_case(b"AvailableUntil") {
        Some(AvailabilityClaim::Until)
    } else {
        None
    }
}

fn first_version_argument<'scratch>(argument_list: &'scratch cst::ArgumentList<'scratch>) -> Option<PHPVersion> {
    let argument = argument_list.arguments.iter().next()?;

    literal_u32(argument.value()).map(decode_decimal_version_id)
}

fn literal_u32(expression: &cst::Expression<'_>) -> Option<u32> {
    match expression.unparenthesized() {
        cst::Expression::Literal(cst::Literal::Integer(cst::LiteralInteger { value: Some(value), .. })) => {
            (*value).try_into().ok()
        }
        cst::Expression::UnaryPrefix(cst::UnaryPrefix { operator: cst::UnaryPrefixOperator::Plus(_), operand }) => {
            literal_u32(operand)
        }
        _ => None,
    }
}

fn decode_decimal_version_id(decimal: u32) -> PHPVersion {
    let major = decimal / 10_000;
    let minor = (decimal / 100) % 100;
    let patch = decimal % 100;

    PHPVersion::new(major, minor, patch)
}

fn push_since<A>(ranges: &mut Vec<'_, PHPVersionRange, A>, version: PHPVersion)
where
    A: Arena,
{
    ranges.push(PHPVersionRange::from(version));
}

fn push_until<A>(ranges: &mut Vec<'_, PHPVersionRange, A>, version: PHPVersion)
where
    A: Arena,
{
    if let Some(last) = ranges.last_mut()
        && last.max.is_none()
    {
        last.max = Some(version);
        return;
    }

    ranges.push(PHPVersionRange::until(version));
}
