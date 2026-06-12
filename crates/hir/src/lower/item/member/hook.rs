use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::delimited::Delimited;
use crate::ir::item::member::hook::Hook;
use crate::ir::item::member::hook::HookBody;
use crate::ir::item::member::hook::HookBodyKind;
use crate::ir::item::member::hook::HookFlag;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_property_hooks(
        &mut self,
        hook_list: &'scratch cst::PropertyHookList<'scratch>,
    ) -> Delimited<'arena, Hook<'arena, (), (), ()>> {
        Delimited {
            span: hook_list.left_brace.join(hook_list.right_brace),
            items: self.arena.alloc_slice_fill_iter(hook_list.hooks.iter().map(|hook| self.lower_property_hook(hook))),
        }
    }

    fn lower_property_hook(&mut self, hook: &'scratch cst::PropertyHook<'scratch>) -> Hook<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&hook.attribute_lists);
        let version_constraint = self.lower_version_constraint(&hook.attribute_lists);
        let modifiers = self.lower_modifiers(&hook.modifiers);
        let name = self.lower_name(&hook.name);
        let parameters = hook.parameter_list.as_ref().map(|parameter_list| self.lower_parameter_list(parameter_list));

        let document = self.phpdoc_resolution.get(hook.span());
        let annotation = self.lower_item_annotation(document.as_ref(), None);

        let mut flags = U8Flags::new();
        if hook.ampersand.is_some() {
            flags.set(HookFlag::ReturnsByReference);
        }

        Hook {
            span: hook.span(),
            annotation,
            attributes,
            version_constraint,
            flags,
            modifiers,
            name,
            parameters,
            body: self.lower_property_hook_body(&hook.body),
        }
    }

    fn lower_property_hook_body(
        &mut self,
        body: &'scratch cst::PropertyHookBody<'scratch>,
    ) -> Option<HookBody<'arena, (), (), ()>> {
        match body {
            cst::PropertyHookBody::Abstract(_) => None,
            cst::PropertyHookBody::Concrete(concrete) => match concrete {
                cst::PropertyHookConcreteBody::Block(block) => {
                    Some(HookBody { span: block.span(), kind: HookBodyKind::Statements(self.lower_block(block)) })
                }
                cst::PropertyHookConcreteBody::Expression(expression) => Some(HookBody {
                    span: expression.span(),
                    kind: HookBodyKind::Expression(self.arena.alloc(self.lower_expression(expression.expression))),
                }),
            },
        }
    }
}
