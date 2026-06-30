use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Switch;
use mago_hir::ir::statement::SwitchCase;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;
use crate::reconciler::subtract_with;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_switch(
        &mut self,
        span: Span,
        switch: &'source Switch<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let reachable = self.reachable;
        let mut entry = self.environment.clone();
        let subject = self.infer_expression(switch.subject)?;
        let subject_place = self.place_id(&subject);
        if let Some(place) = subject_place {
            entry.set(place, subject.meta);
        }

        let mut has_default = false;
        let mut escapes = false;
        let mut last_exit = ControlFlow::Fallthrough;
        let mut after = None;
        let mut excluded = TYPE_NEVER;

        let mut cases = Vec::new_in(self.arena);
        for case in switch.cases.items {
            let reachable_subject = subtract_with(&mut self.ty, self.symbols, subject.meta, excluded);
            self.environment.clone_from(&entry);
            if let Some(place) = subject_place {
                self.environment.set(place, reachable_subject);
            }
            self.reachable = reachable;

            let (typed_case, exit, value) = match case {
                SwitchCase::Expression(value, body) => {
                    let value = self.infer_expression(value)?;
                    let value_meta = value.meta;
                    let body = self.infer_statement(body)?;
                    let exit = body.meta.exit;

                    (SwitchCase::Expression(self.arena.alloc(value), self.arena.alloc(body)), exit, Some(value_meta))
                }
                SwitchCase::Default(body) => {
                    has_default = true;
                    let body = self.infer_statement(body)?;
                    let exit = body.meta.exit;

                    (SwitchCase::Default(self.arena.alloc(body)), exit, None)
                }
            };

            last_exit = exit;
            escapes |= matches!(exit, ControlFlow::Break(_) | ControlFlow::Continue(_));
            if !matches!(exit, ControlFlow::Return | ControlFlow::Diverge) {
                after = Environment::merge_options(after, Some(self.environment.clone()), &mut self.ty);
            }

            // A case that does not fall through consumes the values that matched it
            // (they break/return/diverge out), so a later case can no longer be one
            // of them — but only a literal value can be subtracted soundly.
            if let Some(value) = value
                && !matches!(exit, ControlFlow::Fallthrough)
                && self.array_key_of(value).is_some()
            {
                excluded = self.union(excluded, value);
            }

            cases.push(typed_case);
        }

        if !has_default || matches!(last_exit, ControlFlow::Fallthrough) {
            escapes = true;
        }

        let exit = if escapes { ControlFlow::Fallthrough } else { ControlFlow::Diverge };
        if let Some(environment) = Environment::merge_options(after, Some(entry), &mut self.ty) {
            self.environment = environment;
        }

        let node = Switch {
            span: switch.span,
            subject: self.arena.alloc(subject),
            cases: Delimited { span: switch.cases.span, items: cases.leak() },
        };

        Ok(Statement { meta: Flow { reachable, exit }, span, kind: StatementKind::Switch(self.arena.alloc(node)) })
    }
}
