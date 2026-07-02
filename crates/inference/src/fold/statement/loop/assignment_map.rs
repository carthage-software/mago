use std::collections::BTreeMap;
use std::collections::BTreeSet;

use mago_hir::ir::argument::Argument;
use mago_hir::ir::expression::Access;
use mago_hir::ir::expression::AccessKind;
use mago_hir::ir::expression::ArrayElement;
use mago_hir::ir::expression::ArrayElementKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::operator::UnaryPrefixOperator;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::SwitchCase;
use mago_hir::ir::variable::Variable;

/// The assignment dependency graph of a loop: each assigned root place mapped to
/// the roots its value depends on. `$a = $b` records an edge `$a -> $b`; `$a++`
/// or a call argument records the self-edge `$a -> $a`.
#[derive(Default)]
struct Graph<'ir> {
    edges: BTreeMap<&'ir [u8], BTreeSet<&'ir [u8]>>,
}

impl<'ir> Graph<'ir> {
    fn add_edge(&mut self, root: &'ir [u8], dependency: &'ir [u8]) {
        self.edges.entry(root).or_default().insert(dependency);
    }

    /// The first assigned root in sorted order — the chain whose depth bounds the
    /// number of re-folds, matching the analyzer's `first_variable_id`.
    fn first_root(&self) -> Option<&'ir [u8]> {
        self.edges.keys().next().copied()
    }

    /// Removes a root and returns the roots it depends on, so a chain is walked
    /// once and cycles cannot loop forever.
    fn take_dependencies(&mut self, root: &'ir [u8]) -> Option<BTreeSet<&'ir [u8]>> {
        self.edges.remove(root)
    }

    fn has_root(&self, root: &'ir [u8]) -> bool {
        self.edges.contains_key(root)
    }
}

/// The number of times a loop body must be re-folded for types to propagate
/// through its assignment chains, the way the analyzer bounds its fixed point.
///
/// A dependency graph maps each assigned root place to the roots its value
/// depends on (`$a = $b` gives `$a -> {$b}`); the longest chain from the first
/// assigned root, clamped to `limit`, is the bound. A loop with no assignments
/// has depth `0` and needs only a single fold.
pub(crate) fn assignment_depth<I, S, E>(
    conditions: &[&Expression<'_, I, S, E>],
    increments: &[&Expression<'_, I, S, E>],
    body: &Statement<'_, I, S, E>,
    limit: usize,
) -> usize {
    let mut graph = Graph::default();

    for condition in conditions {
        collect_expression(condition, &mut graph);
    }
    collect_statement(body, &mut graph);
    for increment in increments {
        collect_expression(increment, &mut graph);
    }

    match graph.first_root() {
        Some(first) => chain_depth(first, &mut graph, limit),
        None => 0,
    }
}

/// The depth of the dependency chain rooted at `place`, clamped to `maximum`.
/// Short-circuits once `maximum` is reached so deep graphs cost `O(maximum)`.
fn chain_depth<'ir>(place: &'ir [u8], graph: &mut Graph<'ir>, maximum: usize) -> usize {
    if maximum == 0 {
        return 0;
    }

    let Some(dependencies) = graph.take_dependencies(place) else {
        return 0;
    };

    let mut deepest = 0;
    for dependency in dependencies {
        let mut depth = 1;
        if depth < maximum && graph.has_root(dependency) {
            depth += chain_depth(dependency, graph, maximum - 1);
        }

        if depth > deepest {
            deepest = depth;
            if deepest >= maximum {
                return maximum;
            }
        }
    }

    deepest
}

fn record<'ir, I, S, E>(target: &Expression<'ir, I, S, E>, source: &'ir [u8], graph: &mut Graph<'ir>) {
    if let Some(root) = root_variable(target) {
        graph.add_edge(root, source);
    }
}

fn collect_statement<'ir, I, S, E>(statement: &Statement<'ir, I, S, E>, map: &mut Graph<'ir>) {
    match &statement.kind {
        StatementKind::Expression(expression) => collect_expression(expression, map),
        StatementKind::Sequence(statements) => {
            for statement in *statements {
                collect_statement(statement, map);
            }
        }
        StatementKind::If(conditional) => {
            collect_expression(conditional.condition, map);
            collect_statement(conditional.then, map);
            if let Some(otherwise) = conditional.r#else {
                collect_statement(otherwise, map);
            }
        }
        StatementKind::Switch(switch) => {
            collect_expression(switch.subject, map);
            for case in switch.cases.items {
                match case {
                    SwitchCase::Expression(value, body) => {
                        collect_expression(value, map);
                        collect_statement(body, map);
                    }
                    SwitchCase::Default(body) => collect_statement(body, map),
                }
            }
        }
        StatementKind::While(while_loop) => {
            collect_expression(while_loop.condition, map);
            collect_statement(while_loop.statement, map);
        }
        StatementKind::DoWhile(do_while) => {
            collect_statement(do_while.statement, map);
            collect_expression(do_while.condition, map);
        }
        StatementKind::For(for_loop) => {
            for expression in for_loop.initializations.iter().chain(for_loop.conditions).chain(for_loop.increments) {
                collect_expression(expression, map);
            }
            collect_statement(for_loop.statement, map);
        }
        StatementKind::Foreach(foreach) => {
            collect_expression(foreach.expression, map);
            collect_statement(foreach.statement, map);
        }
        StatementKind::Try(try_statement) => {
            collect_statement(try_statement.statement, map);
            for catch in try_statement.catch_clauses {
                collect_statement(catch.statement, map);
            }
            if let Some(finally) = try_statement.finally_clause {
                collect_statement(finally, map);
            }
        }
        StatementKind::Return(Some(value)) => collect_expression(value, map),
        StatementKind::Echo(expressions) => {
            for expression in *expressions {
                collect_expression(expression, map);
            }
        }
        StatementKind::Unset(operands) => {
            for operand in operands.items {
                record(operand, b"unset", map);
            }
        }
        _ => {}
    }
}

fn collect_expression<'ir, I, S, E>(expression: &Expression<'ir, I, S, E>, map: &mut Graph<'ir>) {
    match &expression.kind {
        ExpressionKind::Parenthesized(inner) => collect_expression(inner, map),
        ExpressionKind::Assignment(assignment) => {
            let source = root_variable(assignment.right).unwrap_or(b"isset");
            match array_like_targets(assignment.left) {
                Some(targets) => {
                    for target in targets {
                        record(target, source, map);
                    }
                }
                None => record(assignment.left, source, map),
            }

            collect_expression(assignment.left, map);
            collect_expression(assignment.right, map);
        }
        ExpressionKind::UnaryPostfix(postfix) => record(postfix.operand, b"isset", map),
        ExpressionKind::UnaryPrefix(prefix) => {
            if matches!(prefix.operator, UnaryPrefixOperator::PreIncrement | UnaryPrefixOperator::PreDecrement) {
                record(prefix.operand, b"isset", map);
            } else {
                collect_expression(prefix.operand, map);
            }
        }
        ExpressionKind::Binary(binary) => {
            collect_expression(binary.left, map);
            collect_expression(binary.right, map);
        }
        ExpressionKind::Conditional(conditional) => {
            collect_expression(conditional.condition, map);
            if let Some(then) = conditional.then {
                collect_expression(then, map);
            }
            collect_expression(conditional.r#else, map);
        }
        ExpressionKind::Array(elements) | ExpressionKind::List(elements) => {
            for element in elements.items {
                if let Some(value) = element_value(element) {
                    collect_expression(value, map);
                }
            }
        }
        ExpressionKind::ArrayAppend(base) => collect_expression(base, map),
        ExpressionKind::Access(access) => collect_access(access, map),
        ExpressionKind::Call(call) => {
            for argument in call.arguments.items {
                record(argument_value(argument), b"isset", map);
            }
        }
        ExpressionKind::Match(match_expression) => collect_expression(match_expression.subject, map),
        _ => {}
    }
}

fn collect_access<'ir, I, S, E>(access: &Access<'ir, I, S, E>, map: &mut Graph<'ir>) {
    match &access.kind {
        AccessKind::Array(base, index) => {
            collect_expression(base, map);
            collect_expression(index, map);
        }
        AccessKind::Property(base, _)
        | AccessKind::NullsafeProperty(base, _)
        | AccessKind::StaticProperty(base, _)
        | AccessKind::ClassConstant(base, _) => collect_expression(base, map),
    }
}

/// The root variable a writable place is rooted in: `$a`, `$a[$k]`, `$a->b`,
/// `$a[]`, `&$a` all root in `$a`. `None` for anything not anchored in a variable.
fn root_variable<'ir, I, S, E>(expression: &Expression<'ir, I, S, E>) -> Option<&'ir [u8]> {
    match &expression.kind {
        ExpressionKind::Parenthesized(inner) => root_variable(inner),
        ExpressionKind::Variable(Variable::Direct(direct)) => Some(direct.name),
        ExpressionKind::ArrayAppend(base) => root_variable(base),
        ExpressionKind::Access(access) => match &access.kind {
            AccessKind::Array(base, _)
            | AccessKind::Property(base, _)
            | AccessKind::NullsafeProperty(base, _)
            | AccessKind::StaticProperty(base, _)
            | AccessKind::ClassConstant(base, _) => root_variable(base),
        },
        ExpressionKind::UnaryPrefix(prefix) if matches!(prefix.operator, UnaryPrefixOperator::Reference) => {
            root_variable(prefix.operand)
        }
        _ => None,
    }
}

fn element_value<'ast, 'ir, I, S, E>(
    element: &'ast ArrayElement<'ir, I, S, E>,
) -> Option<&'ast Expression<'ir, I, S, E>> {
    match &element.kind {
        ArrayElementKind::Value(value) | ArrayElementKind::Variadic(value) => Some(value),
        ArrayElementKind::KeyValue(_, value) => Some(value),
        ArrayElementKind::Missing => None,
    }
}

fn argument_value<'ast, 'ir, I, S, E>(argument: &'ast Argument<'ir, I, S, E>) -> &'ast Expression<'ir, I, S, E> {
    match argument {
        Argument::Value(value) | Argument::Variadic(value) | Argument::Named(_, value) => value,
    }
}

fn array_like_targets<'ast, 'ir, I, S, E>(
    expression: &'ast Expression<'ir, I, S, E>,
) -> Option<impl Iterator<Item = &'ast Expression<'ir, I, S, E>>> {
    let (ExpressionKind::Array(elements) | ExpressionKind::List(elements)) = &expression.kind else {
        return None;
    };

    Some(elements.items.iter().filter_map(element_value))
}
