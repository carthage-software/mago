use mago_syntax::cst;

use crate::ir::hook::Hook;
use crate::ir::hook::HookBody;
use crate::lower::Lowering;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_property_hooks(
        &mut self,
        hook_list: &'arena cst::PropertyHookList<'arena>,
    ) -> &'arena [Hook<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(hook_list.hooks.iter().map(|hook| self.lower_property_hook(hook)))
    }

    fn lower_property_hook(&mut self, hook: &'arena cst::PropertyHook<'arena>) -> Hook<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&hook.attribute_lists);
        let modifiers = self.lower_modifiers(&hook.modifiers);
        let name = self.lower_name(&hook.name);
        let parameters = match &hook.parameter_list {
            Some(parameter_list) => self.lower_parameter_list(parameter_list),
            None => &[],
        };

        Hook {
            attributes,
            modifiers,
            return_by_reference: hook.ampersand.is_some(),
            name,
            is_variadic: false,
            parameters,
            body: self.lower_property_hook_body(&hook.body),
        }
    }

    fn lower_property_hook_body(
        &mut self,
        body: &'arena cst::PropertyHookBody<'arena>,
    ) -> Option<HookBody<'arena, (), (), ()>> {
        match body {
            cst::PropertyHookBody::Abstract(_) => None,
            cst::PropertyHookBody::Concrete(concrete) => match concrete {
                cst::PropertyHookConcreteBody::Block(block) => Some(HookBody::Statements(self.lower_block(block))),
                cst::PropertyHookConcreteBody::Expression(expression) => {
                    Some(HookBody::Expression(self.arena.alloc(self.lower_expression(expression.expression))))
                }
            },
        }
    }
}
