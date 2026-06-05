use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::expression::Expression;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::expression::selector::MemberSelector;
use crate::lower::Lowering;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_member_selector(
        &mut self,
        selector: &'arena cst::ClassLikeMemberSelector<'arena>,
    ) -> MemberSelector<'arena, (), (), ()> {
        match selector {
            cst::ClassLikeMemberSelector::Identifier(identifier) => MemberSelector::Name(self.lower_name(identifier)),
            cst::ClassLikeMemberSelector::Variable(variable) => match variable {
                cst::Variable::Direct(direct) => MemberSelector::Variable(self.lower_direct_variable(direct)),
                _ => MemberSelector::Expression(self.arena.alloc(Expression {
                    meta: (),
                    span: variable.span(),
                    kind: ExpressionKind::Variable(self.lower_variable(variable)),
                })),
            },
            cst::ClassLikeMemberSelector::Expression(selector) => {
                MemberSelector::Expression(self.arena.alloc(self.lower_expression(selector.expression)))
            }
            cst::ClassLikeMemberSelector::Missing(span) => MemberSelector::Expression(self.arena.alloc(Expression {
                meta: (),
                span: *span,
                kind: ExpressionKind::SyntaxError,
            })),
        }
    }

    pub(crate) fn lower_constant_selector(
        &mut self,
        selector: &'arena cst::ClassLikeConstantSelector<'arena>,
    ) -> ConstantSelector<'arena, (), (), ()> {
        match selector {
            cst::ClassLikeConstantSelector::Identifier(identifier) => {
                ConstantSelector::Name(self.lower_name(identifier))
            }
            cst::ClassLikeConstantSelector::Expression(selector) => {
                ConstantSelector::Expression(self.arena.alloc(self.lower_expression(selector.expression)))
            }
            cst::ClassLikeConstantSelector::Missing(span) => {
                ConstantSelector::Expression(self.arena.alloc(Expression {
                    meta: (),
                    span: *span,
                    kind: ExpressionKind::SyntaxError,
                }))
            }
        }
    }
}
