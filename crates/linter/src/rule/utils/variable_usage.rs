//! Shared variable-usage walker for the `no-redundant-variable`, `no-dead-store`,
//! `no-unused-static`, `no-unused-global`, and `no-unused-closure-capture` rules.

use foldhash::HashMap;
use foldhash::HashSet;

use mago_span::Span;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

pub trait Recorder<'arena> {
    fn declare_external(&mut self, name: &'arena str);
    fn record_write(&mut self, name: &'arena str, span: Span);
    fn record_read(&mut self, name: &'arena str);
    fn record_read_then_write(&mut self, name: &'arena str, span: Span);
    fn record_call_argument(&mut self, name: &'arena str, span: Span);
    fn record_unset(&mut self, name: &'arena str);
    fn enter_arm(&mut self);
    fn exit_arm(&mut self);
    fn bail(&mut self);
    fn is_bailed(&self) -> bool;
    fn is_seen(&self, name: &str) -> bool;
    fn tracked_names(&self) -> Vec<&'arena str>;

    fn enter_rescan(&mut self) {}
    fn exit_rescan(&mut self) {}

    fn rescans_loops(&self) -> bool {
        false
    }
}

#[derive(Default)]
pub struct RedundantVarInfo {
    pub do_not_flag: bool,
    pub pending_write: Option<Span>,
}

#[derive(Default)]
pub struct RedundantRecorder<'arena> {
    pub info: HashMap<&'arena str, RedundantVarInfo>,
    pub bailed: bool,
    rescan_depth: u32,
}

impl<'arena> RedundantRecorder<'arena> {
    fn entry(&mut self, name: &'arena str) -> &mut RedundantVarInfo {
        self.info.entry(name).or_default()
    }

    fn in_rescan(&self) -> bool {
        self.rescan_depth > 0
    }
}

impl<'arena> Recorder<'arena> for RedundantRecorder<'arena> {
    fn declare_external(&mut self, name: &'arena str) {
        self.entry(name).do_not_flag = true;
    }

    fn record_write(&mut self, name: &'arena str, span: Span) {
        if self.in_rescan() {
            return;
        }

        self.entry(name).pending_write = Some(span);
    }

    fn record_read(&mut self, name: &'arena str) {
        self.entry(name).pending_write = None;
    }

    fn record_read_then_write(&mut self, name: &'arena str, span: Span) {
        if self.in_rescan() {
            self.entry(name).pending_write = None;
            return;
        }

        self.entry(name).pending_write = Some(span);
    }

    fn record_call_argument(&mut self, name: &'arena str, span: Span) {
        if self.is_seen(name) {
            self.record_read(name);
        } else {
            self.record_write(name, span);
        }
    }

    fn record_unset(&mut self, name: &'arena str) {
        self.record_read(name);
    }

    fn enter_arm(&mut self) {}

    fn exit_arm(&mut self) {}

    fn bail(&mut self) {
        self.bailed = true;
    }

    fn is_bailed(&self) -> bool {
        self.bailed
    }

    fn is_seen(&self, name: &str) -> bool {
        self.info.contains_key(name)
    }

    fn tracked_names(&self) -> Vec<&'arena str> {
        self.info.keys().copied().collect()
    }

    fn enter_rescan(&mut self) {
        self.rescan_depth += 1;
    }

    fn exit_rescan(&mut self) {
        self.rescan_depth = self.rescan_depth.saturating_sub(1);
    }

    fn rescans_loops(&self) -> bool {
        true
    }
}

#[derive(Default)]
pub struct DeadStoreVarInfo {
    pub do_not_flag: bool,
    pub pending_write: Option<(Span, u32)>,
    pub dead_stores: Vec<Span>,
}

#[derive(Default)]
pub struct DeadStoreRecorder<'arena> {
    pub info: HashMap<&'arena str, DeadStoreVarInfo>,
    pub bailed: bool,
    arm_counter: u32,
    arm_stack: Vec<u32>,
}

impl<'arena> DeadStoreRecorder<'arena> {
    fn entry(&mut self, name: &'arena str) -> &mut DeadStoreVarInfo {
        self.info.entry(name).or_default()
    }

    fn current_arm(&self) -> u32 {
        self.arm_stack.last().copied().unwrap_or(0)
    }
}

impl<'arena> Recorder<'arena> for DeadStoreRecorder<'arena> {
    fn declare_external(&mut self, name: &'arena str) {
        self.entry(name).do_not_flag = true;
    }

    fn record_write(&mut self, name: &'arena str, span: Span) {
        let arm = self.current_arm();
        let info = self.entry(name);
        let prev = info.pending_write.replace((span, arm));
        if let Some((prev_span, prev_arm)) = prev
            && prev_arm == arm
        {
            info.dead_stores.push(prev_span);
        }
    }

    fn record_read(&mut self, name: &'arena str) {
        self.entry(name).pending_write = None;
    }

    fn record_read_then_write(&mut self, name: &'arena str, span: Span) {
        let arm = self.current_arm();
        self.entry(name).pending_write = Some((span, arm));
    }

    fn record_call_argument(&mut self, name: &'arena str, span: Span) {
        if self.is_seen(name) {
            self.record_read(name);
        } else {
            self.record_write(name, span);
        }
    }

    fn record_unset(&mut self, name: &'arena str) {
        self.record_read(name);
    }

    fn enter_arm(&mut self) {
        self.arm_counter += 1;
        self.arm_stack.push(self.arm_counter);
    }

    fn exit_arm(&mut self) {
        self.arm_stack.pop();
    }

    fn bail(&mut self) {
        self.bailed = true;
    }

    fn is_bailed(&self) -> bool {
        self.bailed
    }

    fn is_seen(&self, name: &str) -> bool {
        self.info.contains_key(name)
    }

    fn tracked_names(&self) -> Vec<&'arena str> {
        self.info.keys().copied().collect()
    }
}

/// Collects every direct variable name referenced (read, written, unset, passed
/// as a call argument, captured by a nested closure, etc.) within a body.
///
/// Unlike [`RedundantRecorder`] / [`DeadStoreRecorder`], this collector ignores
/// `declare_external`. That makes `static $x;`, `global $x;`, parameters, and
/// closure `use ($x)` captures invisible from the collector's point of view —
/// only *other* references to those names show up in [`UsageCollector::referenced`].
///
/// The collector also bails (sets `bailed = true`) when the body contains
/// variable variables (`$$x`, `${expr}`) or calls `extract()`, since those
/// introduce names the linter cannot resolve.
///
/// Populate `interest` before walking with the names the caller cares about;
/// `compact('foo')` matches against this set so any match resolves to a read
/// without the collector having to fabricate `&'arena` strings on the fly.
#[derive(Default)]
pub struct UsageCollector<'arena> {
    pub interest: HashSet<&'arena str>,
    pub referenced: HashSet<&'arena str>,
    pub bailed: bool,
}

impl<'arena> Recorder<'arena> for UsageCollector<'arena> {
    fn declare_external(&mut self, _name: &'arena str) {}

    fn record_write(&mut self, name: &'arena str, _span: Span) {
        self.referenced.insert(name);
    }

    fn record_read(&mut self, name: &'arena str) {
        self.referenced.insert(name);
    }

    fn record_read_then_write(&mut self, name: &'arena str, _span: Span) {
        self.referenced.insert(name);
    }

    fn record_call_argument(&mut self, name: &'arena str, _span: Span) {
        self.referenced.insert(name);
    }

    fn record_unset(&mut self, name: &'arena str) {
        self.referenced.insert(name);
    }

    fn enter_arm(&mut self) {}

    fn exit_arm(&mut self) {}

    fn bail(&mut self) {
        self.bailed = true;
    }

    fn is_bailed(&self) -> bool {
        self.bailed
    }

    fn is_seen(&self, name: &str) -> bool {
        self.interest.contains(name) || self.referenced.contains(name)
    }

    fn tracked_names(&self) -> Vec<&'arena str> {
        self.interest.iter().copied().collect()
    }
}

pub fn analyze<'arena, R: Recorder<'arena> + Default + Sync + Send>(
    parameter_list: &FunctionLikeParameterList<'arena>,
    body: &Block<'arena>,
    use_clause: Option<&ClosureUseClause<'arena>>,
) -> R {
    let mut walker: UsageWalker<'arena, R> =
        UsageWalker { rec: R::default(), ctx: ExprCtx::Read, excluded: Vec::new() };

    for param in parameter_list.parameters.iter() {
        walker.rec.declare_external(param.variable.name);
    }
    if let Some(use_clause) = use_clause {
        for cap in use_clause.variables.iter() {
            walker.rec.declare_external(cap.variable.name);
        }
    }

    walker.walk_block(body, &mut ());
    walker.rec
}

/// Walks `body` with a [`UsageCollector`] pre-loaded with `interest` names so
/// that `compact('name')` resolves to a read of `$name` for any name in the set.
///
/// Returns the collector after the walk; callers should check `bailed` first
/// before consulting `referenced`.
pub fn collect_used_names<'arena>(body: &Block<'arena>, interest: HashSet<&'arena str>) -> UsageCollector<'arena> {
    let collector = UsageCollector { interest, referenced: HashSet::default(), bailed: false };
    let mut walker: UsageWalker<'arena, UsageCollector<'arena>> =
        UsageWalker { rec: collector, ctx: ExprCtx::Read, excluded: Vec::new() };
    walker.walk_block(body, &mut ());
    walker.rec
}

pub fn function_like_parts<'ast, 'arena>(
    node: Node<'ast, 'arena>,
) -> Option<(&'ast FunctionLikeParameterList<'arena>, &'ast Block<'arena>, Option<&'ast ClosureUseClause<'arena>>)> {
    match node {
        Node::Function(f) => Some((&f.parameter_list, &f.body, None)),
        Node::Method(m) => match &m.body {
            MethodBody::Concrete(b) => Some((&m.parameter_list, b, None)),
            _ => None,
        },
        Node::Closure(c) => Some((&c.parameter_list, &c.body, c.use_clause.as_ref())),
        _ => None,
    }
}

pub fn is_silenced_name(name: &str) -> bool {
    let bare = name.strip_prefix('$').unwrap_or(name);
    bare.starts_with('_')
}

#[derive(Clone, Copy, PartialEq)]
enum ExprCtx {
    Read,
    Write,
}

struct UsageWalker<'arena, R: Recorder<'arena>> {
    rec: R,
    ctx: ExprCtx,
    excluded: Vec<&'arena str>,
}

impl<'arena, R: Recorder<'arena>> UsageWalker<'arena, R> {
    fn with_ctx<F: FnOnce(&mut Self)>(&mut self, ctx: ExprCtx, f: F) {
        let saved = self.ctx;
        self.ctx = ctx;
        f(self);
        self.ctx = saved;
    }
}

impl<'ast, 'arena, R: Recorder<'arena> + Sync + Send> MutWalker<'ast, 'arena, ()> for UsageWalker<'arena, R> {
    fn walk_in_direct_variable(&mut self, d: &'ast DirectVariable<'arena>, _: &mut ()) {
        if self.excluded.contains(&d.name) {
            return;
        }

        match self.ctx {
            ExprCtx::Read => self.rec.record_read(d.name),
            ExprCtx::Write => self.rec.record_write(d.name, d.span),
        }
    }

    fn walk_in_indirect_variable(&mut self, _: &'ast IndirectVariable<'arena>, _: &mut ()) {
        self.rec.bail();
    }

    fn walk_in_nested_variable(&mut self, _: &'ast NestedVariable<'arena>, _: &mut ()) {
        self.rec.bail();
    }

    fn walk_assignment(&mut self, a: &'ast Assignment<'arena>, ctx: &mut ()) {
        if self.rec.is_bailed() {
            return;
        }

        self.with_ctx(ExprCtx::Read, |w| w.walk_expression(a.rhs, ctx));

        let is_simple = a.operator.is_assign();
        match a.lhs {
            Expression::Variable(Variable::Direct(d)) if !self.excluded.contains(&d.name) => {
                if is_simple {
                    self.rec.record_write(d.name, d.span);
                } else {
                    self.rec.record_read_then_write(d.name, d.span);
                }
            }
            Expression::Variable(Variable::Indirect(_) | Variable::Nested(_)) => {
                self.rec.bail();
            }
            Expression::List(_) | Expression::Array(_) | Expression::LegacyArray(_) => {
                self.with_ctx(ExprCtx::Write, |w| w.walk_expression(a.lhs, ctx));
            }
            _ => self.with_ctx(ExprCtx::Read, |w| w.walk_expression(a.lhs, ctx)),
        }
    }

    fn walk_unary_prefix(&mut self, u: &'ast UnaryPrefix<'arena>, ctx: &mut ()) {
        use UnaryPrefixOperator::*;
        if self.rec.is_bailed() {
            return;
        }

        let is_inc_dec = matches!(u.operator, PreIncrement(_) | PreDecrement(_));
        match (is_inc_dec, u.operand) {
            (true, Expression::Variable(Variable::Direct(d))) if !self.excluded.contains(&d.name) => {
                self.rec.record_read_then_write(d.name, d.span);
            }
            (true, Expression::Variable(Variable::Indirect(_) | Variable::Nested(_))) => {
                self.rec.bail();
            }
            _ => self.with_ctx(ExprCtx::Read, |w| w.walk_expression(u.operand, ctx)),
        }
    }

    fn walk_unary_postfix(&mut self, u: &'ast UnaryPostfix<'arena>, ctx: &mut ()) {
        use UnaryPostfixOperator::*;
        if self.rec.is_bailed() {
            return;
        }

        let is_inc_dec = matches!(u.operator, PostIncrement(_) | PostDecrement(_));
        match (is_inc_dec, u.operand) {
            (true, Expression::Variable(Variable::Direct(d))) if !self.excluded.contains(&d.name) => {
                self.rec.record_read_then_write(d.name, d.span);
            }
            (true, Expression::Variable(Variable::Indirect(_) | Variable::Nested(_))) => {
                self.rec.bail();
            }
            _ => self.with_ctx(ExprCtx::Read, |w| w.walk_expression(u.operand, ctx)),
        }
    }

    fn walk_foreach(&mut self, fe: &'ast Foreach<'arena>, ctx: &mut ()) {
        if self.rec.is_bailed() {
            return;
        }

        self.with_ctx(ExprCtx::Read, |w| w.walk_expression(fe.expression, ctx));
        match &fe.target {
            ForeachTarget::Value(v) => self.with_ctx(ExprCtx::Write, |w| w.walk_expression(v.value, ctx)),
            ForeachTarget::KeyValue(kv) => self.with_ctx(ExprCtx::Write, |w| {
                w.walk_expression(kv.key, ctx);
                w.walk_expression(kv.value, ctx);
            }),
        }

        self.rec.enter_arm();
        let mut walk_body = |w: &mut Self| match &fe.body {
            ForeachBody::Statement(s) => w.walk_statement(s, ctx),
            ForeachBody::ColonDelimited(b) => {
                for s in b.statements.iter() {
                    w.walk_statement(s, ctx);
                }
            }
        };

        walk_body(self);

        if self.rec.rescans_loops() {
            self.rec.enter_rescan();
            walk_body(self);
            self.rec.exit_rescan();
        }

        self.rec.exit_arm();
    }

    fn walk_if(&mut self, if_stmt: &'ast If<'arena>, ctx: &mut ()) {
        if self.rec.is_bailed() {
            return;
        }

        self.walk_expression(if_stmt.condition, ctx);
        match &if_stmt.body {
            IfBody::Statement(body) => {
                self.rec.enter_arm();
                self.walk_statement(body.statement, ctx);
                self.rec.exit_arm();
                for elseif in body.else_if_clauses.iter() {
                    self.walk_expression(elseif.condition, ctx);
                    self.rec.enter_arm();
                    self.walk_statement(elseif.statement, ctx);
                    self.rec.exit_arm();
                }

                if let Some(else_clause) = &body.else_clause {
                    self.rec.enter_arm();
                    self.walk_statement(else_clause.statement, ctx);
                    self.rec.exit_arm();
                }
            }
            IfBody::ColonDelimited(body) => {
                self.rec.enter_arm();
                for s in body.statements.iter() {
                    self.walk_statement(s, ctx);
                }

                self.rec.exit_arm();
                for elseif in body.else_if_clauses.iter() {
                    self.walk_expression(elseif.condition, ctx);
                    self.rec.enter_arm();
                    for s in elseif.statements.iter() {
                        self.walk_statement(s, ctx);
                    }
                    self.rec.exit_arm();
                }

                if let Some(else_clause) = &body.else_clause {
                    self.rec.enter_arm();
                    for s in else_clause.statements.iter() {
                        self.walk_statement(s, ctx);
                    }
                    self.rec.exit_arm();
                }
            }
        }
    }

    fn walk_while(&mut self, w: &'ast While<'arena>, ctx: &mut ()) {
        self.walk_expression(w.condition, ctx);
        self.rec.enter_arm();
        let mut walk_body = |w_: &mut Self| match &w.body {
            WhileBody::Statement(s) => w_.walk_statement(s, ctx),
            WhileBody::ColonDelimited(b) => {
                for s in b.statements.iter() {
                    w_.walk_statement(s, ctx);
                }
            }
        };

        walk_body(self);

        if self.rec.rescans_loops() {
            self.rec.enter_rescan();
            walk_body(self);
            self.rec.exit_rescan();
        }

        self.rec.exit_arm();
    }

    fn walk_do_while(&mut self, d: &'ast DoWhile<'arena>, ctx: &mut ()) {
        self.rec.enter_arm();
        self.walk_statement(d.statement, ctx);

        if self.rec.rescans_loops() {
            self.rec.enter_rescan();
            self.walk_statement(d.statement, ctx);
            self.rec.exit_rescan();
        }

        self.rec.exit_arm();
        self.walk_expression(d.condition, ctx);
    }

    fn walk_for(&mut self, f: &'ast For<'arena>, ctx: &mut ()) {
        for e in f.initializations.iter() {
            self.walk_expression(e, ctx);
        }

        for e in f.conditions.iter() {
            self.walk_expression(e, ctx);
        }

        self.rec.enter_arm();
        let mut walk_increments_and_body = |w: &mut Self| {
            for e in f.increments.iter() {
                w.walk_expression(e, ctx);
            }

            match &f.body {
                ForBody::Statement(s) => w.walk_statement(s, ctx),
                ForBody::ColonDelimited(b) => {
                    for s in b.statements.iter() {
                        w.walk_statement(s, ctx);
                    }
                }
            }
        };

        walk_increments_and_body(self);

        if self.rec.rescans_loops() {
            self.rec.enter_rescan();
            walk_increments_and_body(self);
            self.rec.exit_rescan();
        }

        self.rec.exit_arm();
    }

    fn walk_try(&mut self, tr: &'ast Try<'arena>, ctx: &mut ()) {
        self.rec.enter_arm();
        self.walk_block(&tr.block, ctx);
        self.rec.exit_arm();
        for catch in tr.catch_clauses.iter() {
            self.rec.enter_arm();
            if let Some(var) = &catch.variable {
                self.rec.record_write(var.name, var.span);
            }

            self.walk_block(&catch.block, ctx);
            self.rec.exit_arm();
        }

        if let Some(finally) = &tr.finally_clause {
            self.rec.enter_arm();
            self.walk_block(&finally.block, ctx);
            self.rec.exit_arm();
        }
    }

    fn walk_switch(&mut self, sw: &'ast Switch<'arena>, ctx: &mut ()) {
        self.walk_expression(sw.expression, ctx);
        for case in sw.body.cases() {
            self.rec.enter_arm();
            if let Some(cond) = case.expression() {
                self.walk_expression(cond, ctx);
            }

            for s in case.statements() {
                self.walk_statement(s, ctx);
            }

            self.rec.exit_arm();
        }
    }

    fn walk_match(&mut self, m: &'ast Match<'arena>, ctx: &mut ()) {
        self.walk_expression(m.expression, ctx);
        for arm in m.arms.iter() {
            self.rec.enter_arm();
            match arm {
                MatchArm::Expression(ea) => {
                    for cond in ea.conditions.iter() {
                        self.walk_expression(cond, ctx);
                    }
                    self.walk_expression(ea.expression, ctx);
                }
                MatchArm::Default(da) => {
                    self.walk_expression(da.expression, ctx);
                }
            }

            self.rec.exit_arm();
        }
    }

    fn walk_unset(&mut self, u: &'ast Unset<'arena>, ctx: &mut ()) {
        for v in u.values.iter() {
            if let Expression::Variable(Variable::Direct(d)) = v {
                self.rec.record_unset(d.name);
            } else {
                self.walk_expression(v, ctx);
            }
        }
    }

    fn walk_global(&mut self, g: &'ast Global<'arena>, _: &mut ()) {
        for v in g.variables.iter() {
            if let Variable::Direct(d) = v {
                self.rec.declare_external(d.name);
            } else {
                self.rec.bail();
                return;
            }
        }
    }

    fn walk_static(&mut self, s: &'ast Static<'arena>, ctx: &mut ()) {
        for item in s.items.iter() {
            self.rec.declare_external(item.variable().name);
            if let StaticItem::Concrete(c) = item {
                self.walk_expression(c.value, ctx);
            }
        }
    }

    fn walk_argument_list(&mut self, args: &'ast ArgumentList<'arena>, ctx: &mut ()) {
        for arg in args.arguments.iter() {
            let value = match arg {
                Argument::Positional(p) => p.value,
                Argument::Named(n) => n.value,
            };

            if let Expression::Variable(Variable::Direct(d)) = value
                && !self.excluded.contains(&d.name)
            {
                self.rec.record_call_argument(d.name, d.span);
                continue;
            }

            self.walk_expression(value, ctx);
        }
    }

    fn walk_function_call(&mut self, fc: &'ast FunctionCall<'arena>, ctx: &mut ()) {
        if let Some(name) = simple_function_name(fc.function) {
            if name.eq_ignore_ascii_case("extract") {
                self.rec.bail();
                return;
            }

            if name.eq_ignore_ascii_case("compact") {
                for arg in fc.argument_list.arguments.iter() {
                    let value = match arg {
                        Argument::Positional(p) => p.value,
                        Argument::Named(n) => n.value,
                    };

                    if let Expression::Literal(Literal::String(lit)) = value {
                        let raw = lit.raw;
                        if raw.len() < 2 {
                            continue;
                        }

                        let inner = &raw[1..raw.len() - 1];
                        if inner.is_empty() {
                            continue;
                        }

                        for key in self.rec.tracked_names() {
                            if key.strip_prefix('$').map(|s| s == inner).unwrap_or(false) {
                                self.rec.record_read(key);
                            }
                        }
                    }
                }

                return;
            }
        }

        self.walk_expression(fc.function, ctx);
        self.walk_argument_list(&fc.argument_list, ctx);
    }

    fn walk_closure(&mut self, c: &'ast Closure<'arena>, _: &mut ()) {
        if let Some(use_clause) = &c.use_clause {
            for cap in use_clause.variables.iter() {
                let span = cap.variable.span;
                self.rec.record_read(cap.variable.name);
                if cap.ampersand.is_some() {
                    self.rec.record_write(cap.variable.name, span);
                }
            }
        }
    }

    fn walk_arrow_function(&mut self, a: &'ast ArrowFunction<'arena>, ctx: &mut ()) {
        let added = a.parameter_list.parameters.iter().map(|p| p.variable.name).collect::<Vec<_>>();
        let saved_len = self.excluded.len();
        self.excluded.extend(added);
        self.walk_expression(a.expression, ctx);
        self.excluded.truncate(saved_len);
    }

    fn walk_function(&mut self, _: &'ast Function<'arena>, _: &mut ()) {}
    fn walk_method(&mut self, _: &'ast Method<'arena>, _: &mut ()) {}
    fn walk_class(&mut self, _: &'ast Class<'arena>, _: &mut ()) {}
    fn walk_interface(&mut self, _: &'ast Interface<'arena>, _: &mut ()) {}
    fn walk_trait(&mut self, _: &'ast Trait<'arena>, _: &mut ()) {}
    fn walk_enum(&mut self, _: &'ast Enum<'arena>, _: &mut ()) {}
    fn walk_anonymous_class(&mut self, _: &'ast AnonymousClass<'arena>, _: &mut ()) {}
}

fn simple_function_name<'arena>(expr: &Expression<'arena>) -> Option<&'arena str> {
    if let Expression::Identifier(Identifier::Local(local)) = expr { Some(local.value) } else { None }
}
