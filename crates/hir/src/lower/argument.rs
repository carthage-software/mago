use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::ir::delimited::Delimited;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_argument_list(
        &mut self,
        argument_list: &'scratch cst::ArgumentList<'scratch>,
    ) -> Delimited<'arena, Argument<'arena, (), (), ()>> {
        Delimited {
            span: argument_list.left_parenthesis.join(argument_list.right_parenthesis),
            items: self
                .arena
                .alloc_slice_fill_iter(argument_list.arguments.iter().map(|argument| self.lower_argument(argument))),
        }
    }

    pub(crate) fn lower_argument(
        &mut self,
        argument: &'scratch cst::Argument<'scratch>,
    ) -> Argument<'arena, (), (), ()> {
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
        argument_list: &'scratch cst::PartialArgumentList<'scratch>,
    ) -> Delimited<'arena, PartialArgument<'arena, (), (), ()>> {
        Delimited {
            span: argument_list.left_parenthesis.join(argument_list.right_parenthesis),
            items: self.arena.alloc_slice_fill_iter(
                argument_list.arguments.iter().map(|argument| self.lower_partial_argument(argument)),
            ),
        }
    }

    fn lower_partial_argument(
        &mut self,
        argument: &'scratch cst::PartialArgument<'scratch>,
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
            cst::PartialArgument::Placeholder(placeholder) => PartialArgument::Placeholder(placeholder.span()),
            cst::PartialArgument::VariadicPlaceholder(placeholder) => {
                PartialArgument::VariadicPlaceholder(placeholder.span())
            }
        }
    }
}
