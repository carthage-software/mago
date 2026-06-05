use mago_syntax::cst;

use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::lower::Lowering;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_argument_list(
        &mut self,
        argument_list: &'arena cst::ArgumentList<'arena>,
    ) -> &'arena [Argument<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(argument_list.arguments.iter().map(|argument| self.lower_argument(argument)))
    }

    pub(crate) fn lower_argument(&mut self, argument: &'arena cst::Argument<'arena>) -> Argument<'arena, (), (), ()> {
        match argument {
            cst::Argument::Positional(positional) => {
                let value = self.arena.alloc(self.lower_expression(positional.value));

                if positional.ellipsis.is_some() { Argument::Variadic(value) } else { Argument::Value(value) }
            }
            cst::Argument::Named(named) => {
                let name = self.lower_name(&named.name);

                Argument::Named(name, self.arena.alloc(self.lower_expression(named.value)))
            }
        }
    }

    pub(crate) fn lower_partial_argument_list(
        &mut self,
        argument_list: &'arena cst::PartialArgumentList<'arena>,
    ) -> &'arena [PartialArgument<'arena, (), (), ()>] {
        self.arena
            .alloc_slice_fill_iter(argument_list.arguments.iter().map(|argument| self.lower_partial_argument(argument)))
    }

    fn lower_partial_argument(
        &mut self,
        argument: &'arena cst::PartialArgument<'arena>,
    ) -> PartialArgument<'arena, (), (), ()> {
        match argument {
            cst::PartialArgument::Positional(positional) => {
                let value = self.arena.alloc(self.lower_expression(positional.value));

                if positional.ellipsis.is_some() {
                    PartialArgument::Variadic(value)
                } else {
                    PartialArgument::Value(value)
                }
            }
            cst::PartialArgument::Named(named) => {
                let name = self.lower_name(&named.name);

                PartialArgument::Named(name, self.arena.alloc(self.lower_expression(named.value)))
            }
            cst::PartialArgument::NamedPlaceholder(named) => {
                PartialArgument::NamedPlaceholder(self.lower_name(&named.name))
            }
            cst::PartialArgument::Placeholder(_) => PartialArgument::Placeholder,
            cst::PartialArgument::VariadicPlaceholder(_) => PartialArgument::VariadicPlaceholder,
        }
    }
}
