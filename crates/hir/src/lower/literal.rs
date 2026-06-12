use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::literal::Literal;
use crate::ir::literal::LiteralFloat;
use crate::ir::literal::LiteralInteger;
use crate::ir::literal::LiteralKind;
use crate::ir::literal::LiteralString;
use crate::ir::literal::LiteralStringKind;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_literal(&mut self, literal: &'scratch cst::Literal<'scratch>) -> &'arena Literal<'arena> {
        self.arena.alloc(Literal {
            span: literal.span(),
            kind: match literal {
                cst::Literal::Integer(integer) => LiteralKind::Integer(LiteralInteger {
                    span: integer.span(),
                    raw: self.interner.intern(integer.raw),
                    value: integer.value,
                }),
                cst::Literal::String(string) => LiteralKind::String(LiteralString {
                    span: string.span(),
                    kind: match string.kind {
                        cst::LiteralStringKind::SingleQuoted => LiteralStringKind::SingleQuoted,
                        cst::LiteralStringKind::DoubleQuoted => LiteralStringKind::DoubleQuoted,
                    },
                    raw: self.interner.intern(string.raw),
                    value: string.value.map(|value| self.interner.intern(value)),
                }),
                cst::Literal::Float(float) => LiteralKind::Float(LiteralFloat {
                    span: float.span(),
                    raw: self.interner.intern(float.raw),
                    value: float.value,
                }),
                cst::Literal::Null(_) => LiteralKind::Null,
                cst::Literal::False(_) => LiteralKind::False,
                cst::Literal::True(_) => LiteralKind::True,
            },
        })
    }
}
