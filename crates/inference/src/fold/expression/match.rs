use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::Match;
use mago_hir::ir::expression::MatchArm;
use mago_hir::ir::expression::MatchArmKind;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;
use crate::reconciler::meet_with;
use crate::reconciler::subtract_with;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    /// Infers a `match`, threading the still-unmatched subject type through the
    /// arms in order: each arm sees the subject narrowed to the values that can
    /// reach it (`meet` with the arm's conditions), and the fall-through to later
    /// arms is the subject with those values removed (`subtract`). An arm whose
    /// narrowed subject is `never` is unreachable and contributes neither a result
    /// type nor a branch environment; the result is the union of the reachable
    /// arms' results, and the environment afterwards is their join.
    pub fn infer_match(
        &mut self,
        span: Span,
        match_expression: &'source Match<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let subject = self.infer_expression(match_expression.subject);
        let subject_variable = direct_variable(&subject);
        let entry = self.environment.clone();

        let mut remaining = subject.meta;
        let mut result_atoms = Vec::new_in(self.source);
        let mut joined: Option<Environment<'source, 'arena, A>> = None;

        let mut arms = Vec::new_in(self.arena);
        for arm in match_expression.arms.items {
            let kind = match arm.kind {
                MatchArmKind::Expression(conditions, result) => {
                    self.environment = entry.clone();
                    let mut typed_conditions = Vec::new_in(self.arena);
                    let mut condition_atoms = Vec::new_in(self.source);
                    for condition in conditions {
                        let typed = self.infer_expression(condition);
                        condition_atoms.extend_from_slice(typed.meta.atoms);
                        typed_conditions.push(typed);
                    }

                    let conditions_type = self.ty.union_of(&condition_atoms);
                    let matched = meet_with(&mut self.ty, self.symbols, remaining, conditions_type);
                    let result = self.infer_arm_result(
                        &entry,
                        subject_variable,
                        matched,
                        result,
                        &mut result_atoms,
                        &mut joined,
                    );

                    remaining = subtract_with(&mut self.ty, self.symbols, remaining, conditions_type);
                    MatchArmKind::Expression(typed_conditions.leak(), result)
                }
                MatchArmKind::Default(result) => {
                    let result = self.infer_arm_result(
                        &entry,
                        subject_variable,
                        remaining,
                        result,
                        &mut result_atoms,
                        &mut joined,
                    );

                    remaining = TYPE_NEVER;
                    MatchArmKind::Default(result)
                }
            };

            arms.push(MatchArm { span: arm.span, kind });
        }

        let arms = arms.leak();
        let meta = if result_atoms.is_empty() { TYPE_NEVER } else { self.ty.union_of(&result_atoms) };
        self.environment = joined.unwrap_or(entry);

        let match_expression = Match {
            span: match_expression.span,
            subject: self.arena.alloc(subject),
            arms: Delimited { span: match_expression.arms.span, items: arms },
        };

        Expression { meta, span, kind: ExpressionKind::Match(self.arena.alloc(match_expression)) }
    }

    /// Infers one arm's result with the subject narrowed to `matched`. When the
    /// arm is reachable (`matched` has inhabitants) its result joins the union
    /// and, unless it diverges, its environment joins `joined`.
    fn infer_arm_result(
        &mut self,
        entry: &Environment<'source, 'arena, A>,
        subject_variable: Option<Var<'arena>>,
        matched: Type<'arena>,
        result: &'source Expression<'source, SymbolId, S, E>,
        result_atoms: &mut Vec<'source, Atom<'arena>, A>,
        joined: &mut Option<Environment<'source, 'arena, A>>,
    ) -> &'arena Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let reachable = !matched.is_never();

        self.environment.clone_from(entry);
        if reachable && let Some(variable) = subject_variable {
            self.environment.insert(variable, matched);
        }

        let typed = self.infer_expression(result);
        if reachable {
            result_atoms.extend_from_slice(typed.meta.atoms);
            if !typed.meta.is_never() {
                let contribution = self.environment.clone();
                *joined = Some(match joined.take() {
                    Some(existing) => self.union_environments(existing, contribution),
                    None => contribution,
                });
            }
        }

        self.arena.alloc(typed)
    }
}

fn direct_variable<'arena>(expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>) -> Option<Var<'arena>> {
    match &expression.kind {
        ExpressionKind::Parenthesized(inner) => direct_variable(inner),
        ExpressionKind::Variable(Variable::Direct(direct)) => Some(Var::new(direct.name)),
        _ => None,
    }
}
