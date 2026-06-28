use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_flags::U8Flags;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::literal::Literal;
use mago_hir::ir::literal::LiteralKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::float::LiteralFloat;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::atom::payload::scalar::string::StringRefinementFlag;
use mago_oracle::ty::well_known::LITERAL_INT;
use mago_oracle::ty::well_known::LITERAL_STRING;
use mago_oracle::ty::well_known::TYPE_FALSE;
use mago_oracle::ty::well_known::TYPE_NULL;
use mago_oracle::ty::well_known::TYPE_TRUE;
use mago_span::Span;
use ordered_float::OrderedFloat;

use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_literal(
        &mut self,
        span: Span,
        literal: &'source Literal<'source>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let meta = match literal.kind {
            LiteralKind::String(literal_string) => match literal_string.value {
                Some(value) => {
                    let literal = StringLiteral::Value(self.ty.intern(value));

                    let mut flags = U8Flags::empty();
                    if !value.is_empty() {
                        flags = flags.with(StringRefinementFlag::NonEmpty);
                        if value != b"0" {
                            flags = flags.with(StringRefinementFlag::Truthy);
                        }
                    }

                    let atom = self.ty.string(StringAtom { literal, casing: StringCasing::Unspecified, flags });
                    self.ty.union_of(&[atom])
                }
                None => self.ty.union_of(&[LITERAL_STRING]),
            },
            LiteralKind::Integer(literal_integer) => self.ty.union_of(&[match literal_integer.value {
                Some(v) => {
                    if v > i64::MAX as u64 {
                        Atom::Float(FloatAtom::Literal(LiteralFloat(OrderedFloat(v as f64))))
                    } else {
                        Atom::Int(IntAtom::Literal(v as i64))
                    }
                }
                None => LITERAL_INT,
            }]),
            LiteralKind::Float(literal_float) => {
                self.ty.union_of(&[Atom::Float(FloatAtom::Literal(LiteralFloat(literal_float.value)))])
            }
            LiteralKind::True => TYPE_TRUE,
            LiteralKind::False => TYPE_FALSE,
            LiteralKind::Null => TYPE_NULL,
        };

        Expression { meta, span, kind: ExpressionKind::Literal(self.arena.alloc(literal.copy_into(self.arena))) }
    }
}
