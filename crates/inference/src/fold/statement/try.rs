use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Try;
use mago_hir::ir::statement::TryCatchClause;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_try(
        &mut self,
        span: Span,
        try_statement: &'source Try<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let entry_reachable = self.reachable;
        let entry = self.environment.clone();

        self.reachable = entry_reachable;
        let body = self.infer_statement(try_statement.statement)?;
        let mut protected_exit = body.meta.exit;
        let mut fallthrough = matches!(body.meta.exit, ControlFlow::Fallthrough).then(|| self.environment.clone());

        let mut catch_clauses = Vec::new_in(self.arena);
        for clause in try_statement.catch_clauses {
            self.environment.clone_from(&entry);
            self.reachable = entry_reachable;
            if let Some(variable) = clause.variable {
                self.environment.set(Var::new(self.arena.alloc_slice_copy(variable.name)), TYPE_MIXED);
            }

            let catch_body = self.infer_statement(clause.statement)?;
            protected_exit = combine_exits(protected_exit, catch_body.meta.exit);
            if matches!(catch_body.meta.exit, ControlFlow::Fallthrough) {
                fallthrough = Environment::merge_options(fallthrough, Some(self.environment.clone()), &mut self.ty);
            }

            catch_clauses.push(TryCatchClause {
                span: clause.span,
                r#type: copy_ref_into(clause.r#type, self.arena),
                variable: clause.variable.map(|variable| variable.copy_into(self.arena)),
                statement: self.arena.alloc(catch_body),
            });
        }

        let (finally_clause, exit) = match try_statement.finally_clause {
            Some(finally) => {
                self.environment = fallthrough.unwrap_or_else(|| entry.clone());
                self.reachable = entry_reachable;

                let finally_body = self.infer_statement(finally)?;
                let exit = if matches!(finally_body.meta.exit, ControlFlow::Fallthrough) {
                    protected_exit
                } else {
                    finally_body.meta.exit
                };

                (Some(&*self.arena.alloc(finally_body)), exit)
            }
            None => {
                self.environment = fallthrough.unwrap_or(entry);

                (None, protected_exit)
            }
        };

        let node = Try {
            span: try_statement.span,
            statement: self.arena.alloc(body),
            catch_clauses: catch_clauses.leak(),
            finally_clause,
        };

        Ok(Statement {
            meta: Flow { reachable: entry_reachable, exit },
            span,
            kind: StatementKind::Try(self.arena.alloc(node)),
        })
    }
}

/// Combines two protected-region exits: the region falls through if either path
/// can, otherwise it leaves the way both agree on, or diverges when they differ.
fn combine_exits(left: ControlFlow, right: ControlFlow) -> ControlFlow {
    if matches!(left, ControlFlow::Fallthrough) || matches!(right, ControlFlow::Fallthrough) {
        ControlFlow::Fallthrough
    } else if left == right {
        left
    } else {
        ControlFlow::Diverge
    }
}
