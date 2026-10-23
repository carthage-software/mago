//! Decode callable arrays and quoted method names without retaining CST nodes.

use mago_span::HasSpan;
use mago_syntax::cst::*;

use super::range;
use crate::Range;

#[derive(Debug, Clone, Copy)]
pub enum CallableReceiver {
    Literal(Range),
    ClassName(Range),
    Expression(Range),
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub struct CallableReference {
    /// The quoted method name, or the whole `Class::method` string.
    pub method: Range,
    pub receiver: Option<CallableReceiver>,
    pub owner: Option<u32>,
}

#[derive(Default)]
pub(super) struct Callables {
    references: Vec<CallableReference>,
    class_literals: Vec<Range>,
}

impl Callables {
    pub fn finish(mut self) -> (Box<[CallableReference]>, Box<[Range]>) {
        self.class_literals.sort_unstable_by_key(|range| (range.start, range.end));
        self.class_literals.dedup();
        (self.references.into_boxed_slice(), self.class_literals.into_boxed_slice())
    }

    pub fn literal(&mut self, node: &LiteralString<'_>, owner: Option<u32>) {
        if node.value.is_some_and(|value| value.windows(2).any(|part| part == b"::") && callable_name(value)) {
            self.references.push(CallableReference { method: range(node.span), receiver: None, owner });
        }
    }

    pub fn array<'arena>(
        &mut self,
        elements: &TokenSeparatedSequence<'arena, ArrayElement<'arena>>,
        owner: Option<u32>,
    ) {
        let mut values = [None, None];
        let mut next = 0;
        let mut unknown = false;
        let mut methods = Vec::new();
        let mut receivers = Vec::new();
        for entry in elements.iter() {
            let Some((key, value)) = callable_element(entry, next) else {
                if !unknown {
                    methods.extend(values[1].and_then(callable_method));
                    receivers.extend(values[0].and_then(callable_literal));
                }
                unknown = true;
                match entry {
                    ArrayElement::KeyValue(entry) => {
                        methods.extend(callable_method(entry.value));
                        receivers.extend(callable_literal(entry.value));
                    }
                    ArrayElement::Variadic(entry) => {
                        spread_callable_literals(entry.value, &mut methods, &mut receivers)
                    }
                    _ => {}
                }
                continue;
            };
            if key > 1 {
                return;
            }
            if unknown && matches!(entry, ArrayElement::Value(_)) {
                methods.extend(callable_method(value));
                receivers.extend(callable_literal(value));
                continue;
            }
            if key == 0 {
                receivers.clear();
                if unknown {
                    receivers.extend(callable_literal(value));
                }
            } else {
                methods.clear();
                if unknown {
                    methods.extend(callable_method(value));
                }
            }
            values[key as usize] = Some(value);
            next = next.max(key + 1);
        }
        if unknown {
            self.class_literals.extend(receivers);
            self.references.extend(methods.into_iter().map(|method| CallableReference {
                method,
                receiver: Some(CallableReceiver::Unknown),
                owner,
            }));
            return;
        }
        let Some(receiver) = values[0] else { return };
        let Some(method) = values[1] else { return };
        self.class_literals.extend(callable_literal(receiver));
        let Expression::Literal(Literal::String(method)) = method else { return };
        if !method.value.is_some_and(callable_name) {
            return;
        }
        let receiver = match receiver {
            Expression::Literal(Literal::String(literal)) => CallableReceiver::Literal(range(literal.span)),
            Expression::Access(Access::ClassConstant(access)) if matches!(&access.constant, ClassLikeConstantSelector::Identifier(name) if name.value.eq_ignore_ascii_case(b"class")) => {
                CallableReceiver::ClassName(range(access.class.span()))
            }
            _ => CallableReceiver::Expression(range(receiver.span())),
        };
        self.references.push(CallableReference { method: range(method.span), receiver: Some(receiver), owner });
    }
}

fn callable_element<'arena>(entry: &ArrayElement<'arena>, next: u64) -> Option<(u64, &'arena Expression<'arena>)> {
    match entry {
        ArrayElement::Value(entry) => Some((next, entry.value.unparenthesized())),
        ArrayElement::KeyValue(entry) => {
            let key = match entry.key.unparenthesized() {
                Expression::Literal(Literal::Integer(key)) => {
                    key.value.filter(|value| i64::try_from(*value).is_ok())?
                }
                Expression::Literal(Literal::Float(key)) => {
                    let value = key.value.0;
                    if !value.is_finite() || value < i64::MIN as f64 || value >= i64::MAX as f64 {
                        return None;
                    }
                    u64::try_from(value.trunc() as i64).ok()?
                }
                Expression::Literal(Literal::String(key)) => match key.value? {
                    b"0" => 0,
                    b"1" => 1,
                    _ => 2,
                },
                Expression::Literal(Literal::False(_)) => 0,
                Expression::Literal(Literal::True(_)) => 1,
                Expression::Literal(Literal::Null(_)) => 2,
                _ => return None,
            };
            Some((key, entry.value.unparenthesized()))
        }
        _ => None,
    }
}

fn callable_method(expression: &Expression<'_>) -> Option<Range> {
    let Expression::Literal(Literal::String(method)) = expression.unparenthesized() else { return None };
    method.value.is_some_and(callable_name).then(|| range(method.span))
}

fn callable_literal(expression: &Expression<'_>) -> Option<Range> {
    let Expression::Literal(Literal::String(literal)) = expression.unparenthesized() else { return None };
    Some(range(literal.span))
}

fn spread_callable_literals(expression: &Expression<'_>, methods: &mut Vec<Range>, receivers: &mut Vec<Range>) {
    let elements = match expression.unparenthesized() {
        Expression::Array(array) => &array.elements,
        Expression::LegacyArray(array) => &array.elements,
        _ => return,
    };
    for entry in elements.iter() {
        let value = match entry {
            ArrayElement::Value(entry) => entry.value,
            ArrayElement::KeyValue(entry) => entry.value,
            ArrayElement::Variadic(entry) => {
                spread_callable_literals(entry.value, methods, receivers);
                continue;
            }
            ArrayElement::Missing(_) => continue,
        };
        methods.extend(callable_method(value));
        receivers.extend(callable_literal(value));
    }
}

fn callable_name(value: &[u8]) -> bool {
    let name = value.rsplit(|byte| *byte == b':').next().unwrap_or(value);
    name.first().is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_' || *byte >= 0x80)
        && name.iter().all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte >= 0x80)
}
