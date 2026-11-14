use std::borrow::Cow;

use mago_twig_syntax::cst::Expression;
use mago_twig_syntax::cst::UnaryOperator;

/// A mapping key as PHP stores it: integer-like keys collapse to integers.
#[derive(Hash, PartialEq, Eq)]
pub(super) enum Key<'arena> {
    Integer(i64),
    Bytes(Cow<'arena, [u8]>),
}

/// The key a literal mapping key expression stores to, or `None` when it is not a literal.
pub(super) fn literal<'arena>(mut expression: &Expression<'arena>) -> Option<Key<'arena>> {
    let mut negative = false;
    let mut arithmetic = false;
    loop {
        match expression {
            Expression::Parenthesized(group) => expression = group.inner,
            Expression::Unary(unary) => match unary.operator {
                UnaryOperator::MinusSign(_) => {
                    arithmetic = true;
                    negative = !negative;
                    expression = unary.operand;
                }
                UnaryOperator::PlusSign(_) => {
                    arithmetic = true;
                    expression = unary.operand;
                }
                UnaryOperator::Not(_) => return None,
            },
            _ => break,
        }
    }

    match expression {
        Expression::Bool(value) => {
            Some(Key::Integer(if negative { -i64::from(value.value) } else { i64::from(value.value) }))
        }
        Expression::Null(_) => Some(if arithmetic { Key::Integer(0) } else { Key::Bytes(Cow::Borrowed(b"")) }),
        Expression::Number(value) => {
            let raw = if value.raw.contains(&b'_') {
                Cow::Owned(value.raw.iter().copied().filter(|byte| *byte != b'_').collect::<Vec<_>>())
            } else {
                Cow::Borrowed(value.raw)
            };
            let raw = std::str::from_utf8(&raw).ok()?;
            if let Ok(number) = raw.parse::<i128>() {
                return i64::try_from(if negative { -number } else { number }).ok().map(Key::Integer);
            }

            let number = raw.parse::<f64>().ok()?.copysign(if negative { -1.0 } else { 1.0 });
            if number.is_finite() && number >= i64::MIN as f64 && number < i64::MAX as f64 {
                Some(Key::Integer(number as i64))
            } else {
                None
            }
        }
        Expression::String(value) if !arithmetic => {
            let bytes = decode(value.raw)?;
            if let Ok(text) = std::str::from_utf8(&bytes)
                && let Ok(number) = text.parse::<i64>()
                && number.to_string() == text
            {
                return Some(Key::Integer(number));
            }

            Some(Key::Bytes(bytes))
        }
        _ => None,
    }
}

/// Decodes the escapes of a string literal's body.
fn decode(raw: &[u8]) -> Option<Cow<'_, [u8]>> {
    let quoted = matches!(raw.first(), Some(b'\'' | b'"'));
    let body = if quoted { raw.get(1..raw.len().checked_sub(1)?)? } else { raw };
    if raw.first() == Some(&b'"') && body.windows(2).any(|pair| pair == b"#{") {
        return None;
    }
    if !body.iter().any(|byte| matches!(byte, b'\\' | b'\r')) {
        return Some(Cow::Borrowed(body));
    }

    let mut output = Vec::with_capacity(body.len());
    let mut cursor = 0;
    while cursor < body.len() {
        let byte = body[cursor];
        cursor += 1;
        if byte == b'\r' {
            output.push(b'\n');
            cursor += usize::from(body.get(cursor) == Some(&b'\n'));
        } else if byte != b'\\' || cursor == body.len() {
            output.push(byte);
        } else {
            let escape = body[cursor];
            cursor += 1;
            match escape {
                b'\r' => {
                    output.push(b'\n');
                    cursor += usize::from(body.get(cursor) == Some(&b'\n'));
                }
                b'n' => output.push(b'\n'),
                b'r' => output.push(b'\r'),
                b't' => output.push(b'\t'),
                b'v' => output.push(11),
                b'f' => output.push(12),
                b'0'..=b'7' => {
                    let mut value = escape - b'0';
                    for _ in 0..2 {
                        let Some(next @ b'0'..=b'7') = body.get(cursor).copied() else { break };
                        value = value.wrapping_mul(8).wrapping_add(next - b'0');
                        cursor += 1;
                    }
                    output.push(value);
                }
                b'x' if body.get(cursor).is_some_and(u8::is_ascii_hexdigit) => {
                    let mut value = 0;
                    for _ in 0..2 {
                        let Some(next) = body.get(cursor).copied().filter(u8::is_ascii_hexdigit) else { break };
                        value = value * 16
                            + if next.is_ascii_digit() { next - b'0' } else { next.to_ascii_lowercase() - b'a' + 10 };
                        cursor += 1;
                    }
                    output.push(value);
                }
                other => output.push(other),
            }
        }
    }

    Some(Cow::Owned(output))
}
