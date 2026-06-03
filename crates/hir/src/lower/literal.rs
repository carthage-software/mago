use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::literal::Literal;
use crate::ir::literal::LiteralFloat;
use crate::ir::literal::LiteralInteger;
use crate::ir::literal::LiteralKind;
use crate::ir::literal::LiteralString;
use crate::ir::literal::LiteralStringKind;
use crate::lower::Lowering;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_literal(&self, literal: &'arena cst::Literal<'arena>) -> &'arena Literal<'arena> {
        self.arena.alloc(Literal {
            span: literal.span(),
            kind: match literal {
                cst::Literal::Integer(integer) => {
                    LiteralKind::Integer(LiteralInteger { raw: integer.raw, value: integer.value })
                }
                cst::Literal::String(string) => LiteralKind::String(LiteralString {
                    kind: match string.kind {
                        cst::LiteralStringKind::SingleQuoted => LiteralStringKind::SingleQuoted,
                        cst::LiteralStringKind::DoubleQuoted => LiteralStringKind::DoubleQuoted,
                    },
                    raw: string.raw,
                    value: string.value,
                }),
                cst::Literal::Float(float) => LiteralKind::Float(LiteralFloat { raw: float.raw, value: float.value }),
                cst::Literal::Null(_) => LiteralKind::Null,
                cst::Literal::False(_) => LiteralKind::False,
                cst::Literal::True(_) => LiteralKind::True,
            },
        })
    }
}
