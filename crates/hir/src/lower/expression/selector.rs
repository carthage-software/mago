use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::expression::Expression;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::expression::selector::ConstantSelectorKind;
use crate::ir::expression::selector::MemberSelector;
use crate::ir::expression::selector::MemberSelectorKind;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_member_selector(
        &mut self,
        selector: &'scratch cst::ClassLikeMemberSelector<'scratch>,
    ) -> MemberSelector<'arena, (), (), ()> {
        let span = selector.span();
        let kind = match selector {
            cst::ClassLikeMemberSelector::Identifier(identifier) => {
                MemberSelectorKind::Name(self.lower_name(identifier))
            }
            cst::ClassLikeMemberSelector::Variable(variable) => match variable {
                cst::Variable::Direct(direct) => MemberSelectorKind::Variable(self.lower_direct_variable(direct)),
                _ => MemberSelectorKind::Expression(self.arena.alloc(Expression {
                    span: variable.span(),
                    meta: (),
                    kind: ExpressionKind::Variable(self.lower_variable(variable)),
                })),
            },
            cst::ClassLikeMemberSelector::Expression(selector) => {
                MemberSelectorKind::Expression(self.arena.alloc(self.lower_expression(selector.expression)))
            }
            cst::ClassLikeMemberSelector::Missing(_) => MemberSelectorKind::Missing,
        };

        MemberSelector { span, kind }
    }

    pub(crate) fn lower_constant_selector(
        &mut self,
        selector: &'scratch cst::ClassLikeConstantSelector<'scratch>,
    ) -> ConstantSelector<'arena, (), (), ()> {
        let span = selector.span();
        let kind = match selector {
            cst::ClassLikeConstantSelector::Identifier(identifier) => {
                ConstantSelectorKind::Name(self.lower_name(identifier))
            }
            cst::ClassLikeConstantSelector::Expression(selector) => {
                ConstantSelectorKind::Expression(self.arena.alloc(self.lower_expression(selector.expression)))
            }
            cst::ClassLikeConstantSelector::Missing(_) => ConstantSelectorKind::Missing,
        };

        ConstantSelector { span, kind }
    }
}
