use mago_allocator::Arena;
use mago_hir::ir::statement::If;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;

/// The outcome of inferring one branch of an `if`: its typed statement, the
/// environment it leaves if it falls through (`None` if it diverges or is
/// unreachable), and the control-flow exit of the path (`None` if unreachable).
struct Branch<'source, 'arena, A: Arena> {
    statement: Statement<'arena, SymbolId, Flow, Type<'arena>>,
    fallthrough: Option<Environment<'source, 'arena, A>>,
    exit: Option<ControlFlow>,
}

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_if(
        &mut self,
        span: Span,
        conditional: &'source If<'source, SymbolId, S, E>,
    ) -> Statement<'arena, SymbolId, Flow, Type<'arena>> {
        let entry_reachable = self.reachable;
        let (condition, when_true, when_false) = self.analyze_condition(conditional.condition);
        let entry = self.environment.clone();

        self.reachable = entry_reachable;
        let then = self.infer_branch(&entry, when_true, conditional.then);

        self.reachable = entry_reachable;
        let (otherwise, else_fallthrough, else_exit) = match conditional.r#else {
            Some(statement) => {
                let branch = self.infer_branch(&entry, when_false, statement);

                (Some(&*self.arena.alloc(branch.statement)), branch.fallthrough, branch.exit)
            }
            None => (None, when_false.clone(), when_false.map(|_| ControlFlow::Fallthrough)),
        };

        let exit = merge_exits(then.exit, else_exit);
        if let Some(environment) = self.merge_condition_environments(then.fallthrough, else_fallthrough) {
            self.environment = environment;
        }

        let conditional = If {
            span: conditional.span,
            condition: self.arena.alloc(condition),
            then: self.arena.alloc(then.statement),
            r#else: otherwise,
        };

        Statement { meta: Flow { reachable: true, exit }, span, kind: StatementKind::If(self.arena.alloc(conditional)) }
    }

    fn infer_branch(
        &mut self,
        entry: &Environment<'source, 'arena, A>,
        environment: Option<Environment<'source, 'arena, A>>,
        statement: &'source Statement<'source, SymbolId, S, E>,
    ) -> Branch<'source, 'arena, A> {
        match environment {
            Some(environment) => {
                self.environment = environment;
                let statement = self.infer_statement(statement);
                let fallthrough = match statement.meta.exit {
                    ControlFlow::Fallthrough => Some(self.environment.clone()),
                    _ => None,
                };

                Branch { exit: Some(statement.meta.exit), fallthrough, statement }
            }
            None => {
                self.environment.clone_from(entry);
                self.reachable = false;
                let statement = self.infer_statement(statement);

                Branch { statement, fallthrough: None, exit: None }
            }
        }
    }
}

/// Combines the exits of an `if`'s reachable paths: it falls through if any path
/// does, otherwise leaves the way both agree on (or [`ControlFlow::Diverge`] if
/// they leave differently). An unreachable path (`None`) does not constrain it.
fn merge_exits(then: Option<ControlFlow>, otherwise: Option<ControlFlow>) -> ControlFlow {
    match (then, otherwise) {
        (Some(then), Some(otherwise)) => {
            if then == ControlFlow::Fallthrough || otherwise == ControlFlow::Fallthrough {
                ControlFlow::Fallthrough
            } else if then == otherwise {
                then
            } else {
                ControlFlow::Diverge
            }
        }
        (Some(exit), None) | (None, Some(exit)) => exit,
        (None, None) => ControlFlow::Diverge,
    }
}
