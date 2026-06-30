use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::ArrayElement;
use mago_hir::ir::expression::ArrayElementKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::well_known::TYPE_ARRAY_KEY;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_array(
        &mut self,
        span: Span,
        elements: &Delimited<'source, ArrayElement<'source, SymbolId, S, E>>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut items = Vec::new_in(self.arena);
        let mut entries = Vec::new_in(self.source);
        let mut value_atoms = Vec::new_in(self.source);
        let mut shapeable = true;
        let mut has_never = false;
        let mut next_int_key: i64 = 0;

        for element in elements.items {
            let kind = match element.kind {
                ArrayElementKind::Value(value) => {
                    let value = self.infer_expression(value)?;
                    has_never |= value.meta.is_never();
                    value_atoms.extend_from_slice(value.meta.atoms);
                    if shapeable {
                        push_entry(&mut entries, ArrayKey::Int(next_int_key), value.meta);
                        next_int_key += 1;
                    }

                    ArrayElementKind::Value(self.arena.alloc(value))
                }
                ArrayElementKind::KeyValue(key, value) => {
                    let key = self.infer_expression(key)?;
                    let value = self.infer_expression(value)?;
                    has_never |= key.meta.is_never() || value.meta.is_never();
                    value_atoms.extend_from_slice(value.meta.atoms);
                    if shapeable {
                        match self.array_key_of(key.meta) {
                            Some(array_key) => {
                                if let ArrayKey::Int(index) = array_key
                                    && index >= next_int_key
                                {
                                    next_int_key = index + 1;
                                }
                                push_entry(&mut entries, array_key, value.meta);
                            }
                            None => shapeable = false,
                        }
                    }

                    ArrayElementKind::KeyValue(self.arena.alloc(key), self.arena.alloc(value))
                }
                ArrayElementKind::Variadic(value) => {
                    shapeable = false;
                    let value = self.infer_expression(value)?;
                    has_never |= value.meta.is_never();

                    ArrayElementKind::Variadic(self.arena.alloc(value))
                }
                ArrayElementKind::Missing => {
                    shapeable = false;

                    ArrayElementKind::Missing
                }
            };

            items.push(ArrayElement { span: element.span, kind });
        }

        let meta = if has_never {
            TYPE_NEVER
        } else if shapeable {
            self.closed_array(&entries)
        } else {
            let value_param = if value_atoms.is_empty() { TYPE_MIXED } else { self.ty.union_of(&value_atoms) };
            let atom = self.ty.unsealed_keyed_array_atom(TYPE_ARRAY_KEY, value_param, true);

            self.ty.union_of(&[atom])
        };

        Ok(Expression {
            meta,
            span,
            kind: ExpressionKind::Array(Delimited { span: elements.span, items: items.leak() }),
        })
    }

    /// Resolves a literal array key the way PHP coerces it: `true`/`false`/floats
    /// become integers, `null` becomes `""`, and integer-like strings (canonical
    /// decimal, no leading zeros) become integers. Returns `None` for any key
    /// that is not statically known.
    pub(crate) fn array_key_of(&mut self, key: Type<'arena>) -> Option<ArrayKey<'arena>> {
        let [atom] = key.atoms else {
            return None;
        };

        Some(match atom {
            Atom::Int(IntAtom::Literal(value)) => ArrayKey::Int(*value),
            Atom::True => ArrayKey::Int(1),
            Atom::False => ArrayKey::Int(0),
            Atom::Null => ArrayKey::String(self.ty.intern(b"")),
            Atom::Float(FloatAtom::Literal(value)) => ArrayKey::Int(value.0.into_inner() as i64),
            Atom::String(string) => match string.literal {
                StringLiteral::Value(value) => match canonical_int_key(value) {
                    Some(index) => ArrayKey::Int(index),
                    None => ArrayKey::String(self.ty.intern(value)),
                },
                _ => return None,
            },
            _ => return None,
        })
    }
}

/// Appends an entry, applying PHP's last-write-wins on duplicate keys while
/// keeping the key's first position.
pub(crate) fn push_entry<'arena, A>(
    entries: &mut Vec<'_, KnownItem<'arena>, A>,
    key: ArrayKey<'arena>,
    value: Type<'arena>,
) where
    A: Arena,
{
    if let Some(existing) = entries.iter_mut().find(|entry| entry.key == key) {
        existing.value = value;
    } else {
        entries.push(KnownItem { key, value, optional: false });
    }
}

/// A string is used as an integer array key only when it is a canonical decimal
/// integer: optional `-`, no leading zeros, no `-0`, and within `i64`.
fn canonical_int_key(bytes: &[u8]) -> Option<i64> {
    let digits = match bytes.first() {
        Some(b'-') => &bytes[1..],
        _ => bytes,
    };

    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    if digits.len() > 1 && digits[0] == b'0' {
        return None;
    }
    if bytes.first() == Some(&b'-') && digits == b"0" {
        return None;
    }

    std::str::from_utf8(bytes).ok()?.parse::<i64>().ok()
}
