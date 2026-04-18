//! `Pattern::AstNode` and `Pattern::AstLeafNode` adapters for Mago.
//!
//! A `MagoNodePattern` mirrors the shape of a single AST node: a [`NodeKind`] + ordered
//! child patterns. The engine's matcher, when given a `MagoNodePattern`, checks that the
//! target node has the same kind and that each positional child matches recursively.
//!
//! `MagoLeafNodePattern` is for leaf tokens identified purely by text + kind (e.g. a
//! specific identifier name).

use grit_pattern_matcher::binding::Binding;
use grit_pattern_matcher::context::StaticDefinitions;
use grit_pattern_matcher::pattern::AstLeafNodePattern;
use grit_pattern_matcher::pattern::AstNodePattern;
use grit_pattern_matcher::pattern::Matcher;
use grit_pattern_matcher::pattern::Pattern;
use grit_pattern_matcher::pattern::PatternName;
use grit_pattern_matcher::pattern::PatternOrPredicate;
use grit_pattern_matcher::pattern::ResolvedPattern;
use grit_pattern_matcher::pattern::State;
use grit_util::AnalysisLogs;
use grit_util::error::GritResult;
use mago_syntax::ast::NodeKind;

use crate::binding::MagoBinding;
use crate::context::MagoExecContext;
use crate::query_context::MagoQueryContext;
use crate::resolved_pattern::MagoResolvedPattern;

/// Matches an interior AST node by kind + positional children.
///
/// The `children` vector holds one pattern per expected child in the same order Mago's
/// `visit_children` produces them. A `None` slot in the vector means "any child at this
/// position" (equivalent to `Pattern::Top`).
#[derive(Debug, Clone)]
pub struct MagoNodePattern {
    pub kind: NodeKind,
    pub children: Vec<Pattern<MagoQueryContext>>,
}

impl MagoNodePattern {
    pub fn new(kind: NodeKind, children: Vec<Pattern<MagoQueryContext>>) -> Self {
        Self { kind, children }
    }
}

impl AstNodePattern<MagoQueryContext> for MagoNodePattern {
    const INCLUDES_TRIVIA: bool = false;

    fn children<'a>(
        &'a self,
        _definitions: &'a StaticDefinitions<MagoQueryContext>,
    ) -> Vec<PatternOrPredicate<'a, MagoQueryContext>> {
        self.children.iter().map(PatternOrPredicate::Pattern).collect()
    }

    fn matches_kind_of(
        &self,
        node: &<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Node<'_>,
    ) -> bool {
        node.kind() == self.kind
    }
}

impl PatternName for MagoNodePattern {
    fn name(&self) -> &'static str {
        "MAGO_AST_NODE"
    }
}

impl Matcher<MagoQueryContext> for MagoNodePattern {
    fn execute<'a>(
        &'a self,
        binding: &MagoResolvedPattern<'a>,
        init_state: &mut State<'a, MagoQueryContext>,
        context: &'a MagoExecContext<'a>,
        logs: &mut AnalysisLogs,
    ) -> GritResult<bool> {
        let Some(b) = binding.get_last_binding() else {
            return Ok(false);
        };
        let Some(node) = b.singleton() else {
            return Ok(false);
        };

        if node.kind() != self.kind {
            return Ok(false);
        }

        let target_children: Vec<_> = grit_util::AstNode::children(&node).collect();

        let mut running_state = init_state.clone();
        let mut p_idx = 0usize;
        let mut t_idx = 0usize;

        while p_idx < self.children.len() || t_idx < target_children.len() {
            if let Some(dots_info) = match_dots_shape(self.children.get(p_idx)) {
                let is_last = p_idx + 1 == self.children.len();
                let absorb_end = if is_last {
                    target_children.len()
                } else {
                    // Greedy absorption: reserve exactly as many target children as the
                    // remaining non-dots pattern positions need, and let the Dots take
                    // the rest. Then walk from the rightmost viable boundary inward and
                    // pick the first position where the next pattern matches. This makes
                    // `f(^head, ^...middle, ^tail)` bind `middle` to everything between
                    // the first and last argument, instead of collapsing it to empty.
                    let required_after =
                        self.children[(p_idx + 1)..].iter().filter(|p| match_dots_shape(Some(*p)).is_none()).count();
                    let max_absorb_end = target_children.len().saturating_sub(required_after);
                    if max_absorb_end < t_idx {
                        return Ok(false);
                    }

                    let next_pattern = &self.children[p_idx + 1];
                    let mut consumed_to: Option<usize> = None;
                    for scan in (t_idx..=max_absorb_end).rev() {
                        if scan >= target_children.len() {
                            continue;
                        }
                        let mut trial = running_state.clone();
                        let b = MagoResolvedPattern::from_node_binding(target_children[scan]);
                        if next_pattern.execute(&b, &mut trial, context, logs)? {
                            consumed_to = Some(scan);
                            break;
                        }
                    }
                    match consumed_to {
                        Some(n) => n,
                        None => return Ok(false),
                    }
                };

                // If the dots were named (`^...name`), bind the absorbed slice to the
                // variable as a `MagoBinding::NodeList` so templates can splice it.
                if let DotsShape::Named(var) = dots_info {
                    let absorbed: Vec<crate::node::MagoNode<'a>> = target_children[t_idx..absorb_end].to_vec();
                    let resolved = MagoResolvedPattern::Binding(vec![crate::binding::MagoBinding::NodeList(absorbed)]);
                    if !var.execute(&resolved, &mut running_state, context, logs)? {
                        return Ok(false);
                    }
                }

                t_idx = absorb_end;
                p_idx += 1;
                continue;
            }

            let p_on_mod =
                p_idx < self.children.len() && pattern_kind(&self.children[p_idx]) == Some(NodeKind::Modifier);
            let t_on_mod = t_idx < target_children.len() && target_children[t_idx].kind() == NodeKind::Modifier;
            if p_on_mod || t_on_mod {
                let p_end = scan_pattern_modifier_run(&self.children, p_idx);
                let t_end = scan_target_modifier_run(&target_children, t_idx);
                if !subset_match_modifiers(
                    &self.children[p_idx..p_end],
                    &target_children[t_idx..t_end],
                    &mut running_state,
                    context,
                    logs,
                )? {
                    return Ok(false);
                }
                p_idx = p_end;
                t_idx = t_end;
                continue;
            }

            if p_idx >= self.children.len() || t_idx >= target_children.len() {
                return Ok(false);
            }
            let target = target_children[t_idx];
            let mut cur_state = running_state.clone();
            let b = MagoResolvedPattern::from_node_binding(target);
            if self.children[p_idx].execute(&b, &mut cur_state, context, logs)? {
                running_state = cur_state;
                p_idx += 1;
                t_idx += 1;
            } else {
                return Ok(false);
            }
        }

        *init_state = running_state;
        Ok(true)
    }
}

/// Shape of a dots-like child pattern in [`MagoNodePattern::children`].
enum DotsShape<'a> {
    /// Bare `Pattern::Dots` with no associated variable.
    Unnamed,
    /// `Pattern::And([Dots, Variable])` produced by `^...name`; carries the variable.
    Named(&'a grit_pattern_matcher::pattern::Variable),
}

/// Returns `Some` if the pattern at this child position is a dots-shape that should drive
/// the sequence-absorption branch of [`MagoNodePattern::execute`].
fn match_dots_shape<'a>(p: Option<&'a Pattern<MagoQueryContext>>) -> Option<DotsShape<'a>> {
    match p? {
        Pattern::Dots => Some(DotsShape::Unnamed),
        Pattern::And(and) if crate::compiler::is_named_dots_and(and) => {
            // `is_named_dots_and` guarantees the layout `[Dots, Variable(_)]`.
            match &and.patterns[1] {
                Pattern::Variable(var) => Some(DotsShape::Named(var)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Returns the [`NodeKind`] a given child pattern matches against, or `None` if the
/// pattern isn't kind-anchored (metavariables, `Pattern::Dots`, etc.).
fn pattern_kind(p: &Pattern<MagoQueryContext>) -> Option<NodeKind> {
    match p {
        Pattern::AstNode(n) => Some(n.kind),
        Pattern::AstLeafNode(n) => Some(n.kind),
        _ => None,
    }
}

/// Returns the end index of the contiguous modifier run in `patterns` starting at `start`.
fn scan_pattern_modifier_run(patterns: &[Pattern<MagoQueryContext>], start: usize) -> usize {
    let mut end = start;
    while end < patterns.len() && pattern_kind(&patterns[end]) == Some(NodeKind::Modifier) {
        end += 1;
    }
    end
}

/// Returns the end index of the contiguous modifier run in `targets` starting at `start`.
fn scan_target_modifier_run<'a>(targets: &[crate::node::MagoNode<'a>], start: usize) -> usize {
    let mut end = start;
    while end < targets.len() && targets[end].kind() == NodeKind::Modifier {
        end += 1;
    }
    end
}

/// Subset matcher: every pattern modifier must find a distinct target modifier that
/// matches it. Order does not matter. Returns `true` if the subset relationship holds.
fn subset_match_modifiers<'a>(
    patterns: &'a [Pattern<MagoQueryContext>],
    targets: &[crate::node::MagoNode<'a>],
    state: &mut State<'a, MagoQueryContext>,
    context: &'a MagoExecContext<'a>,
    logs: &mut AnalysisLogs,
) -> GritResult<bool> {
    if patterns.len() > targets.len() {
        return Ok(false);
    }
    let mut used = vec![false; targets.len()];
    for p in patterns {
        let mut matched = false;
        for (i, t) in targets.iter().enumerate() {
            if used[i] {
                continue;
            }
            let mut trial = state.clone();
            let b = MagoResolvedPattern::from_node_binding(*t);
            if p.execute(&b, &mut trial, context, logs)? {
                used[i] = true;
                *state = trial;
                matched = true;
                break;
            }
        }
        if !matched {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Matches a leaf AST node with a specific text payload.
///
/// Used for identifier names, variable names, literal values, or anywhere a single
/// token's spelling matters.
#[derive(Debug, Clone)]
pub struct MagoLeafNodePattern {
    pub kind: NodeKind,
    pub text: String,
}

impl MagoLeafNodePattern {
    pub fn new(kind: NodeKind, text: impl Into<String>) -> Self {
        Self { kind, text: text.into() }
    }
}

impl AstLeafNodePattern<MagoQueryContext> for MagoLeafNodePattern {
    fn text(&self) -> Option<&str> {
        Some(&self.text)
    }
}

impl PatternName for MagoLeafNodePattern {
    fn name(&self) -> &'static str {
        "MAGO_AST_LEAF"
    }
}

impl Matcher<MagoQueryContext> for MagoLeafNodePattern {
    fn execute<'a>(
        &'a self,
        binding: &MagoResolvedPattern<'a>,
        _state: &mut State<'a, MagoQueryContext>,
        _context: &'a MagoExecContext<'a>,
        _logs: &mut AnalysisLogs,
    ) -> GritResult<bool> {
        let Some(bind) = binding.get_last_binding() else {
            return Ok(false);
        };
        let Some(node) = bind.singleton() else {
            return Ok(false);
        };
        if node.kind() != self.kind {
            return Ok(false);
        }
        match grit_util::AstNode::text(&node) {
            Ok(text) => Ok(text.trim() == self.text),
            Err(_) => Ok(false),
        }
    }
}

impl<'a> MagoBinding<'a> {
    /// Equivalent to [`grit_pattern_matcher::binding::Binding::singleton`].
    pub fn node(&self) -> Option<<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Node<'a>> {
        use grit_pattern_matcher::binding::Binding;
        self.singleton()
    }
}
