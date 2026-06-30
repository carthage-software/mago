use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::argument::PartialArgument;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_partial_argument(
        &mut self,
        argument: &PartialArgument<'source, SymbolId, S, E>,
    ) -> InferenceResult<PartialArgument<'arena, SymbolId, Flow, Type<'arena>>> {
        let argument = match argument {
            PartialArgument::Value(expression) => {
                let expression = self.infer_expression(expression)?;

                PartialArgument::Value(self.arena.alloc(expression))
            }
            PartialArgument::Variadic(expression) => {
                let expression = self.infer_expression(expression)?;

                PartialArgument::Variadic(self.arena.alloc(expression))
            }
            PartialArgument::Named(name, expression) => {
                let expression = self.infer_expression(expression)?;

                PartialArgument::Named(name.copy_into(self.arena), self.arena.alloc(expression))
            }
            PartialArgument::Placeholder(span) => PartialArgument::Placeholder(*span),
            PartialArgument::NamedPlaceholder(name) => PartialArgument::NamedPlaceholder(name.copy_into(self.arena)),
            PartialArgument::VariadicPlaceholder(span) => PartialArgument::VariadicPlaceholder(*span),
        };

        Ok(argument)
    }
}
