use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst;

use crate::ir::expression::Expression;
use crate::ir::statement::Declare;
use crate::ir::statement::DeclareItem;
use crate::ir::statement::DoWhile;
use crate::ir::statement::For;
use crate::ir::statement::Foreach;
use crate::ir::statement::If;
use crate::ir::statement::Namespace;
use crate::ir::statement::Statement;
use crate::ir::statement::StatementKind;
use crate::ir::statement::StaticItem;
use crate::ir::statement::Switch;
use crate::ir::statement::SwitchCase;
use crate::ir::statement::Try;
use crate::ir::statement::TryCatchClause;
use crate::ir::statement::While;
use crate::ir::statement::definition::DefinitionStatement;
use crate::ir::statement::definition::DefinitionStatementKind;
use crate::lower::Lowering;

pub mod definition;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_statement(
        &mut self,
        statement: &'arena cst::Statement<'arena>,
    ) -> Statement<'arena, (), (), ()> {
        Statement {
            meta: (),
            span: statement.span(),
            kind: match statement {
                cst::Statement::Expression(expression) => {
                    StatementKind::Expression(self.lower_expression_statement(expression))
                }
                cst::Statement::Block(block) => StatementKind::Sequence(self.lower_block(block)),
                cst::Statement::Noop(_) | cst::Statement::ClosingTag(_) | cst::Statement::OpeningTag(_) => {
                    StatementKind::Noop
                }
                cst::Statement::HaltCompiler(_) => StatementKind::HaltCompiler,
                cst::Statement::Unset(unset) => StatementKind::Unset(self.lower_unset(unset)),
                cst::Statement::Echo(echo) => StatementKind::Echo(self.lower_echo(echo)),
                cst::Statement::EchoTag(echo_tag) => StatementKind::Echo(self.lower_echo_tag(echo_tag)),
                cst::Statement::Return(r#return) => StatementKind::Return(self.lower_return(r#return)),
                cst::Statement::Goto(goto) => StatementKind::Goto(self.lower_name(&goto.label)),
                cst::Statement::Label(label) => StatementKind::Label(self.lower_name(&label.name)),
                cst::Statement::Continue(r#continue) => {
                    StatementKind::Continue(self.lower_optional_expression(r#continue.level))
                }
                cst::Statement::Break(r#break) => StatementKind::Break(self.lower_optional_expression(r#break.level)),
                cst::Statement::If(r#if) => StatementKind::If(self.lower_if(r#if)),
                cst::Statement::Switch(switch) => StatementKind::Switch(self.lower_switch(switch)),
                cst::Statement::While(r#while) => StatementKind::While(self.lower_while(r#while)),
                cst::Statement::DoWhile(do_while) => StatementKind::DoWhile(self.lower_do_while(do_while)),
                cst::Statement::For(r#for) => StatementKind::For(self.lower_for(r#for)),
                cst::Statement::Foreach(foreach) => StatementKind::Foreach(self.lower_foreach(foreach)),
                cst::Statement::Namespace(namespace) => StatementKind::Namespace(self.lower_namespace(namespace)),
                cst::Statement::Use(r#use) => {
                    self.resolution.populate_from_use(r#use);

                    StatementKind::Noop
                }
                cst::Statement::Inline(inline) => StatementKind::Inline(inline.value),
                cst::Statement::Class(class) => StatementKind::Definition(self.arena.alloc(DefinitionStatement {
                    meta: (),
                    kind: DefinitionStatementKind::Class(self.lower_class(class)),
                })),
                cst::Statement::Interface(interface) => {
                    StatementKind::Definition(self.arena.alloc(DefinitionStatement {
                        meta: (),
                        kind: DefinitionStatementKind::Interface(self.lower_interface(interface)),
                    }))
                }
                cst::Statement::Trait(r#trait) => StatementKind::Definition(self.arena.alloc(DefinitionStatement {
                    meta: (),
                    kind: DefinitionStatementKind::Trait(self.lower_trait(r#trait)),
                })),
                cst::Statement::Enum(r#enum) => StatementKind::Definition(self.arena.alloc(DefinitionStatement {
                    meta: (),
                    kind: DefinitionStatementKind::Enum(self.lower_enum(r#enum)),
                })),
                cst::Statement::Constant(constant) => {
                    StatementKind::Definition(self.arena.alloc(DefinitionStatement {
                        meta: (),
                        kind: DefinitionStatementKind::Constant(self.lower_constant(constant)),
                    }))
                }
                cst::Statement::Function(function) => {
                    StatementKind::Definition(self.arena.alloc(DefinitionStatement {
                        meta: (),
                        kind: DefinitionStatementKind::Function(self.lower_function(function)),
                    }))
                }
                cst::Statement::Declare(declare) => StatementKind::Declare(self.lower_declare(declare)),
                cst::Statement::Try(r#try) => StatementKind::Try(self.lower_try(r#try)),
                cst::Statement::Global(global) => StatementKind::Global(
                    self.arena
                        .alloc_slice_fill_iter(global.variables.iter().map(|variable| self.lower_variable(variable))),
                ),
                cst::Statement::Static(r#static) => StatementKind::Static(
                    self.arena.alloc_slice_fill_iter(r#static.items.iter().map(|item| self.lower_static_item(item))),
                ),
                _ => {
                    debug_assert!(false, "unhandled statement kind: {:?}", statement);

                    // SAFETY: This code is unreachable because all possible statement kinds have been handled in the match arms above.
                    // The debug assertion ensures that if an unhandled statement kind is encountered during development, it will be caught and fixed.
                    unsafe { std::hint::unreachable_unchecked() }
                }
            },
        }
    }

    pub(crate) fn lower_block(&mut self, block: &'arena cst::Block<'arena>) -> &'arena [Statement<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(block.statements.iter().map(|statement| self.lower_statement(statement)))
    }

    pub(crate) fn colon_delimited_statements_to_statement(
        &mut self,
        colon: Span,
        statements: &'arena [cst::Statement<'arena>],
        end: cst::Keyword<'arena>,
    ) -> &'arena Statement<'arena, (), (), ()> {
        self.arena.alloc(Statement {
            meta: (),
            span: colon.join(end.span),
            kind: StatementKind::Sequence(
                self.arena.alloc_slice_fill_iter(statements.iter().map(|statement| self.lower_statement(statement))),
            ),
        })
    }

    pub(crate) fn statements_to_statement(
        &mut self,
        statements: &'arena [cst::Statement<'arena>],
        fallback_span: Span,
    ) -> &'arena Statement<'arena, (), (), ()> {
        let span = match (statements.first(), statements.last()) {
            (Some(first), Some(last)) => first.span().join(last.span()),
            (Some(statement), None) => statement.span(),
            (None, _) => fallback_span,
        };

        self.arena.alloc(Statement {
            meta: (),
            span,
            kind: StatementKind::Sequence(
                self.arena.alloc_slice_fill_iter(statements.iter().map(|statement| self.lower_statement(statement))),
            ),
        })
    }

    pub(crate) fn lower_unset(
        &mut self,
        unset: &'arena cst::Unset<'arena>,
    ) -> &'arena [Expression<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(unset.values.iter().map(|value| self.lower_expression(value)))
    }

    pub(crate) fn lower_echo(&mut self, echo: &'arena cst::Echo<'arena>) -> &'arena [Expression<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(echo.values.iter().map(|value| self.lower_expression(value)))
    }

    pub(crate) fn lower_echo_tag(
        &mut self,
        echo_tag: &'arena cst::EchoTag<'arena>,
    ) -> &'arena [Expression<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(echo_tag.values.iter().map(|value| self.lower_expression(value)))
    }

    pub(crate) fn lower_expression_statement(
        &mut self,
        expression_statement: &'arena cst::ExpressionStatement<'arena>,
    ) -> &'arena Expression<'arena, (), (), ()> {
        self.arena.alloc(self.lower_expression(expression_statement.expression))
    }

    pub(crate) fn lower_return(
        &mut self,
        r#return: &'arena cst::Return<'arena>,
    ) -> Option<&'arena Expression<'arena, (), (), ()>> {
        self.lower_optional_expression(r#return.value)
    }

    pub(crate) fn lower_optional_expression(
        &mut self,
        expression: Option<&'arena cst::Expression<'arena>>,
    ) -> Option<&'arena Expression<'arena, (), (), ()>> {
        expression.map(|expression| &*self.arena.alloc(self.lower_expression(expression)))
    }

    pub(crate) fn lower_expression_list(
        &mut self,
        expressions: &'arena cst::TokenSeparatedSequence<'arena, &'arena cst::Expression<'arena>>,
    ) -> &'arena [Expression<'arena, (), (), ()>] {
        self.arena
            .alloc_slice_fill_iter(expressions.iter().copied().map(|expression| self.lower_expression(expression)))
    }

    pub(crate) fn lower_if(&mut self, r#if: &'arena cst::If<'arena>) -> &'arena If<'arena, (), (), ()> {
        let condition: &Expression<'arena, (), (), ()> = self.arena.alloc(self.lower_expression(r#if.condition));

        match &r#if.body {
            cst::IfBody::Statement(body) => {
                let then = self.arena.alloc(self.lower_statement(body.statement));
                let r#else =
                    self.lower_statement_else_chain(body.else_if_clauses.as_slice(), body.else_clause.as_ref());

                self.arena.alloc(If { condition, then, r#else })
            }
            cst::IfBody::ColonDelimited(body) => {
                let then =
                    self.colon_delimited_statements_to_statement(body.colon, body.statements.as_slice(), body.endif);
                let r#else = self.lower_colon_else_chain(body.else_if_clauses.as_slice(), body.else_clause.as_ref());

                self.arena.alloc(If { condition, then, r#else })
            }
        }
    }

    fn lower_statement_else_chain(
        &mut self,
        clauses: &'arena [cst::IfStatementBodyElseIfClause<'arena>],
        else_clause: Option<&'arena cst::IfStatementBodyElseClause<'arena>>,
    ) -> Option<&'arena Statement<'arena, (), (), ()>> {
        match clauses.split_first() {
            None => match else_clause {
                Some(clause) => Some(self.arena.alloc(self.lower_statement(clause.statement))),
                None => None,
            },
            Some((clause, rest)) => {
                let condition: &Expression<'arena, (), (), ()> =
                    self.arena.alloc(self.lower_expression(clause.condition));
                let then = self.arena.alloc(self.lower_statement(clause.statement));
                let r#else = self.lower_statement_else_chain(rest, else_clause);
                let nested = self.arena.alloc(If { condition, then, r#else });

                Some(self.arena.alloc(Statement { meta: (), span: clause.span(), kind: StatementKind::If(nested) }))
            }
        }
    }

    fn lower_colon_else_chain(
        &mut self,
        clauses: &'arena [cst::IfColonDelimitedBodyElseIfClause<'arena>],
        else_clause: Option<&'arena cst::IfColonDelimitedBodyElseClause<'arena>>,
    ) -> Option<&'arena Statement<'arena, (), (), ()>> {
        match clauses.split_first() {
            None => match else_clause {
                Some(clause) => Some(self.statements_to_statement(clause.statements.as_slice(), clause.colon)),
                None => None,
            },
            Some((clause, rest)) => {
                let condition = self.arena.alloc(self.lower_expression(clause.condition));
                let then = self.statements_to_statement(clause.statements.as_slice(), clause.colon);
                let r#else = self.lower_colon_else_chain(rest, else_clause);
                let nested = self.arena.alloc(If { condition, then, r#else });

                Some(self.arena.alloc(Statement { meta: (), span: clause.span(), kind: StatementKind::If(nested) }))
            }
        }
    }

    pub(crate) fn lower_switch(&mut self, switch: &'arena cst::Switch<'arena>) -> &'arena Switch<'arena, (), (), ()> {
        let subject = self.arena.alloc(self.lower_expression(switch.expression));
        let cases =
            self.arena.alloc_slice_fill_iter(switch.body.cases().iter().map(|case| self.lower_switch_case(case)));

        self.arena.alloc(Switch { subject, cases })
    }

    fn lower_switch_case(&mut self, case: &'arena cst::SwitchCase<'arena>) -> SwitchCase<'arena, (), (), ()> {
        match case {
            cst::SwitchCase::Expression(case) => {
                let expression = self.arena.alloc(self.lower_expression(case.expression));
                let statement = self.statements_to_statement(case.statements.as_slice(), case.separator.span());

                SwitchCase::Expression(expression, statement)
            }
            cst::SwitchCase::Default(case) => {
                SwitchCase::Default(self.statements_to_statement(case.statements.as_slice(), case.separator.span()))
            }
        }
    }

    pub(crate) fn lower_while(&mut self, r#while: &'arena cst::While<'arena>) -> &'arena While<'arena, (), (), ()> {
        let condition = self.arena.alloc(self.lower_expression(r#while.condition));
        let statement: &Statement<'arena, (), (), ()> = match &r#while.body {
            cst::WhileBody::Statement(statement) => self.arena.alloc(self.lower_statement(statement)),
            cst::WhileBody::ColonDelimited(body) => {
                self.colon_delimited_statements_to_statement(body.colon, body.statements.as_slice(), body.end_while)
            }
        };

        self.arena.alloc(While { condition, statement })
    }

    pub(crate) fn lower_do_while(
        &mut self,
        do_while: &'arena cst::DoWhile<'arena>,
    ) -> &'arena DoWhile<'arena, (), (), ()> {
        let statement = self.arena.alloc(self.lower_statement(do_while.statement));
        let condition = self.arena.alloc(self.lower_expression(do_while.condition));

        self.arena.alloc(DoWhile { statement, condition })
    }

    pub(crate) fn lower_for(&mut self, r#for: &'arena cst::For<'arena>) -> &'arena For<'arena, (), (), ()> {
        let initializations = self.lower_expression_list(&r#for.initializations);
        let conditions = self.lower_expression_list(&r#for.conditions);
        let increments = self.lower_expression_list(&r#for.increments);
        let statement: &Statement<'arena, (), (), ()> = match &r#for.body {
            cst::ForBody::Statement(statement) => self.arena.alloc(self.lower_statement(statement)),
            cst::ForBody::ColonDelimited(body) => {
                self.colon_delimited_statements_to_statement(body.colon, body.statements.as_slice(), body.end_for)
            }
        };

        self.arena.alloc(For { initializations, conditions, increments, statement })
    }

    pub(crate) fn lower_foreach(
        &mut self,
        foreach: &'arena cst::Foreach<'arena>,
    ) -> &'arena Foreach<'arena, (), (), ()> {
        let expression = self.arena.alloc(self.lower_expression(foreach.expression));
        let key: Option<&Expression<'arena, (), (), ()>> = match &foreach.target {
            cst::ForeachTarget::Value(_) => None,
            cst::ForeachTarget::KeyValue(target) => Some(self.arena.alloc(self.lower_expression(target.key))),
        };
        let value = self.arena.alloc(self.lower_expression(foreach.target.value()));
        let statement: &Statement<'arena, (), (), ()> = match &foreach.body {
            cst::ForeachBody::Statement(statement) => self.arena.alloc(self.lower_statement(statement)),
            cst::ForeachBody::ColonDelimited(body) => {
                self.colon_delimited_statements_to_statement(body.colon, body.statements.as_slice(), body.end_foreach)
            }
        };

        self.arena.alloc(Foreach { expression, key, value, statement })
    }

    pub(crate) fn lower_namespace(
        &mut self,
        namespace: &'arena cst::Namespace<'arena>,
    ) -> &'arena Namespace<'arena, (), (), ()> {
        let name = match &namespace.name {
            Some(identifier) => Some(&*self.arena.alloc(self.lower_identifier(identifier, None))),
            None => None,
        };

        self.resolution.enter_namespace(namespace.name.as_ref().map(|identifier| identifier.value()));
        let statement = self.statements_to_statement(namespace.statements().as_slice(), namespace.namespace.span());
        self.resolution.leave_namespace();

        self.arena.alloc(Namespace { name, statement })
    }

    pub(crate) fn lower_declare(
        &mut self,
        declare: &'arena cst::Declare<'arena>,
    ) -> &'arena Declare<'arena, (), (), ()> {
        self.arena.alloc(Declare {
            items: self.arena.alloc_slice_fill_iter(declare.items.iter().map(|item| DeclareItem {
                name: self.lower_name(&item.name),
                value: Some(self.arena.alloc(self.lower_expression(item.value))),
            })),
            statement: match &declare.body {
                cst::DeclareBody::Statement(statement) => self.arena.alloc(self.lower_statement(statement)),
                cst::DeclareBody::ColonDelimited(body) => self.colon_delimited_statements_to_statement(
                    body.colon,
                    body.statements.as_slice(),
                    body.end_declare,
                ),
            },
        })
    }

    fn lower_static_item(&mut self, item: &'arena cst::StaticItem<'arena>) -> StaticItem<'arena, (), (), ()> {
        match item {
            cst::StaticItem::Abstract(item) => {
                StaticItem { variable: self.lower_direct_variable(&item.variable), value: None }
            }
            cst::StaticItem::Concrete(item) => StaticItem {
                variable: self.lower_direct_variable(&item.variable),
                value: Some(self.arena.alloc(self.lower_expression(item.value))),
            },
        }
    }

    pub(crate) fn lower_try(&mut self, r#try: &'arena cst::Try<'arena>) -> &'arena Try<'arena, (), (), ()> {
        let statement = self.statements_to_statement(r#try.block.statements.as_slice(), r#try.block.span());
        let catch_clauses = self
            .arena
            .alloc_slice_fill_iter(r#try.catch_clauses.iter().map(|clause| self.lower_try_catch_clause(clause)));
        let finally_clause = r#try
            .finally_clause
            .as_ref()
            .map(|finally| self.statements_to_statement(finally.block.statements.as_slice(), finally.block.span()));

        self.arena.alloc(Try { statement, catch_clauses, finally_clause })
    }

    fn lower_try_catch_clause(
        &mut self,
        clause: &'arena cst::TryCatchClause<'arena>,
    ) -> TryCatchClause<'arena, (), (), ()> {
        TryCatchClause {
            r#type: self.lower_type(&clause.hint),
            variable: clause.variable.as_ref().map(|variable| self.lower_direct_variable(variable)),
            statement: self.statements_to_statement(clause.block.statements.as_slice(), clause.block.span()),
        }
    }
}
