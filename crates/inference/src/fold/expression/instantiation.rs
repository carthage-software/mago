use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::argument::Argument;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::Instantiation;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::Symbol;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_instantiation(
        &mut self,
        span: Span,
        instantiation: &'source Instantiation<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let arguments = match &instantiation.arguments {
            Some(arguments) => {
                let mut items = Vec::new_in(self.arena);
                for argument in arguments.items {
                    let typed = match argument {
                        Argument::Value(value) => Argument::Value(self.arena.alloc(self.infer_expression(value)?)),
                        Argument::Variadic(value) => {
                            Argument::Variadic(self.arena.alloc(self.infer_expression(value)?))
                        }
                        Argument::Named(name, value) => {
                            Argument::Named(name.copy_into(self.arena), self.arena.alloc(self.infer_expression(value)?))
                        }
                    };

                    items.push(typed);
                }

                Some(Delimited { span: arguments.span, items: items.leak() })
            }
            None => None,
        };

        let meta = self.instantiated_type(instantiation.class);
        let class = self.infer_expression(instantiation.class)?;

        let node = Instantiation { span: instantiation.span, class: self.arena.alloc(class), arguments };
        Ok(Expression { meta, span, kind: ExpressionKind::Instantiation(self.arena.alloc(node)) })
    }

    /// The object type a `new ...` produces: an instance of the resolved class
    /// (its canonical name), falling back to the written name for an unknown class.
    /// `mixed` for a dynamic class expression (`new $variable`).
    fn instantiated_type(&mut self, class: &'source Expression<'source, SymbolId, S, E>) -> Type<'arena> {
        let name = if let Some(symbol) = self.resolve_class(class) {
            symbol.path().as_bytes()
        } else if let ExpressionKind::Identifier(identifier) | ExpressionKind::Constant(identifier) = &class.kind {
            self.arena.alloc_slice_copy(identifier.value)
        } else {
            return TYPE_MIXED;
        };

        let atom = self.ty.named_object_atom(name);
        self.ty.union_of(&[atom])
    }
}
