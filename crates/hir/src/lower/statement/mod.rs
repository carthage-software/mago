use mago_allocator::Arena;
use mago_allocator::copy::copy_slice_into;
use mago_allocator::vec::Vec;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst;

use crate::ir::delimited::Delimited;
use crate::ir::expression::Expression;
use crate::ir::identifier::Identifier;
use crate::ir::identifier::IdentifierKind;
use crate::ir::item::statement::ItemStatement;
use crate::ir::item::statement::ItemStatementKind;
use crate::ir::item::statement::constant::Constant;
use crate::ir::statement::Declare;
use crate::ir::statement::DeclareItem;
use crate::ir::statement::DoWhile;
use crate::ir::statement::For;
use crate::ir::statement::Foreach;
use crate::ir::statement::GlobalItem;
use crate::ir::statement::If;
use crate::ir::statement::Namespace;
use crate::ir::statement::Statement;
use crate::ir::statement::StatementKind;
use crate::ir::statement::StaticItem;
use crate::ir::statement::Switch;
use crate::ir::statement::SwitchCase;
use crate::ir::statement::Try;
use crate::ir::statement::TryCatchClause;
use crate::ir::statement::UseItem;
use crate::ir::statement::UseItemKind;
use crate::ir::statement::While;
use crate::ir::statement::annotation::VariableBindingAnnotation;
use crate::ir::variable::Variable;
use crate::lower::Lowering;
use crate::lower::settings::DefineConstantLowering;

pub mod annotation;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_statement(
        &mut self,
        statement: &'scratch cst::Statement<'scratch>,
    ) -> Statement<'arena, (), (), ()> {
        let document = self.phpdoc_resolution.get(statement.span());
        let span = statement.span();

        if let Some(lowered) = self.lower_define_call(statement, document.as_ref(), span) {
            return lowered;
        }

        let mut bindings = self.collect_var_bindings(document.as_ref());

        let kind = if bindings.is_empty() {
            self.lower_statement_kind(statement)
        } else {
            match statement {
                cst::Statement::Expression(expression) => {
                    let lowered = self.lower_expression_statement(expression);

                    StatementKind::Expression(self.fold_assignment_statement(lowered, &mut bindings))
                }
                cst::Statement::Return(r#return) => {
                    let lowered = self.lower_return(r#return);

                    StatementKind::Return(self.fold_returned_expression(lowered, &mut bindings))
                }
                cst::Statement::Foreach(foreach) => {
                    let mut folded = *self.lower_foreach(foreach);
                    folded.value = self.fold_assignment_target(folded.value, &mut bindings);
                    folded.key = folded.key.map(|key| self.fold_assignment_target(key, &mut bindings));

                    StatementKind::Foreach(self.arena.alloc(folded))
                }
                cst::Statement::Static(r#static) => {
                    StatementKind::Static(self.arena.alloc_slice_fill_iter(r#static.items.iter().map(|item| {
                        let mut item = self.lower_static_item(item);
                        item.type_annotation = bindings.take_named(item.variable.name);
                        item
                    })))
                }
                cst::Statement::Global(global) => {
                    StatementKind::Global(self.arena.alloc_slice_fill_iter(global.variables.iter().map(|variable| {
                        let mut item = self.lower_global_item(variable);
                        if let Variable::Direct(direct) = item.variable {
                            item.type_annotation = bindings.take_named(direct.name);
                        }

                        item
                    })))
                }
                _ => self.lower_statement_kind(statement),
            }
        };

        if bindings.named.is_empty() {
            return Statement { meta: (), span, kind };
        }

        let mut statements = Vec::new_in(self.arena);
        for (variable, type_annotation, _, span) in bindings.named {
            statements.push(Statement {
                meta: (),
                span,
                kind: StatementKind::VariableBindingAnnotation(self.arena.alloc(VariableBindingAnnotation {
                    span,
                    variable,
                    type_annotation,
                })),
            });
        }

        statements.push(Statement { meta: (), span, kind });

        Statement { meta: (), span, kind: StatementKind::Sequence(statements.leak()) }
    }

    fn lower_define_call(
        &mut self,
        statement: &'scratch cst::Statement<'scratch>,
        document: Option<&mago_phpdoc_syntax::cst::Document<'scratch>>,
        span: Span,
    ) -> Option<Statement<'arena, (), (), ()>> {
        if self.settings.define_constant_lowering == DefineConstantLowering::Disabled {
            return None;
        }

        let cst::Statement::Expression(expression_statement) = statement else {
            return None;
        };

        let cst::Expression::Call(cst::Call::Function(call)) = expression_statement.expression else {
            return None;
        };

        let cst::Expression::Identifier(identifier) = call.function else {
            return None;
        };

        if identifier.value() != b"define" {
            return None;
        }

        let [name_argument, value_argument] = call.argument_list.arguments.as_slice() else {
            return None;
        };

        let cst::Expression::Literal(cst::Literal::String(name_string)) = name_argument.value() else {
            return None;
        };

        let name_value = name_string.value?;
        let annotation = self.lower_item_annotation(document, None);
        let value = self.arena.alloc(self.lower_expression(value_argument.value()));
        let constant = self.arena.alloc(Constant {
            span,
            attributes: &[],
            version_constraint: &[],
            annotation,
            name: Identifier {
                span: name_string.span(),
                value: self.interner.intern(name_value),
                kind: IdentifierKind::Local,
            },
            value,
            flattened: false,
        });

        let definition = StatementKind::Item(self.arena.alloc(ItemStatement {
            meta: (),
            span,
            kind: ItemStatementKind::Constant(constant),
        }));

        let constant_statement = Statement { meta: (), span, kind: definition };

        match self.settings.define_constant_lowering {
            DefineConstantLowering::Statement => Some(constant_statement),
            DefineConstantLowering::StatementAndCall => {
                let call_statement = Statement { meta: (), span, kind: self.lower_statement_kind(statement) };
                Some(Statement {
                    meta: (),
                    span,
                    kind: StatementKind::Sequence(self.arena.alloc_slice_copy(&[constant_statement, call_statement])),
                })
            }
            DefineConstantLowering::Disabled => None,
        }
    }

    fn lower_statement_kind(
        &mut self,
        statement: &'scratch cst::Statement<'scratch>,
    ) -> StatementKind<'arena, (), (), ()> {
        match statement {
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
                let items = self.lower_use(r#use);
                for item in items.iter() {
                    self.namespace_resolution.add_import(item.kind, item.item.value, item.alias);
                }

                StatementKind::Use(copy_slice_into(items, self.arena))
            }
            cst::Statement::Inline(inline) => StatementKind::Inline(self.interner.intern(inline.value)),
            cst::Statement::Class(class) => StatementKind::Item(self.arena.alloc(ItemStatement {
                meta: (),
                span: class.span(),
                kind: ItemStatementKind::Class(self.lower_class(class)),
            })),
            cst::Statement::Interface(interface) => StatementKind::Item(self.arena.alloc(ItemStatement {
                meta: (),
                span: interface.span(),
                kind: ItemStatementKind::Interface(self.lower_interface(interface)),
            })),
            cst::Statement::Trait(r#trait) => StatementKind::Item(self.arena.alloc(ItemStatement {
                meta: (),
                span: r#trait.span(),
                kind: ItemStatementKind::Trait(self.lower_trait(r#trait)),
            })),
            cst::Statement::Enum(r#enum) => StatementKind::Item(self.arena.alloc(ItemStatement {
                meta: (),
                span: r#enum.span(),
                kind: ItemStatementKind::Enum(self.lower_enum(r#enum)),
            })),
            cst::Statement::Constant(constant) => {
                // A `const A = 1, B = 2;` declaration lowers to one item-statement per
                // declared constant. A single declarator stays a plain item; multiple
                // declarators become a sequence, preserving source order.
                let arena = self.arena;
                let item_statements: std::vec::Vec<&'arena ItemStatement<'arena, (), (), ()>> = self
                    .lower_constant(constant)
                    .into_iter()
                    .map(|constant| {
                        let span = constant.span;
                        &*arena.alloc(ItemStatement {
                            meta: (),
                            span,
                            kind: ItemStatementKind::Constant(arena.alloc(constant)),
                        })
                    })
                    .collect();

                if let [single] = item_statements.as_slice() {
                    StatementKind::Item(single)
                } else {
                    StatementKind::Sequence(arena.alloc_slice_fill_iter(
                        item_statements.iter().map(|&item| Statement {
                            meta: (),
                            span: item.span,
                            kind: StatementKind::Item(item),
                        }),
                    ))
                }
            }
            cst::Statement::Function(function) => StatementKind::Item(self.arena.alloc(ItemStatement {
                meta: (),
                span: function.span(),
                kind: ItemStatementKind::Function(self.lower_function(function)),
            })),
            cst::Statement::Declare(declare) => StatementKind::Declare(self.lower_declare(declare)),
            cst::Statement::Try(r#try) => StatementKind::Try(self.lower_try(r#try)),
            cst::Statement::Global(global) => StatementKind::Global(
                self.arena
                    .alloc_slice_fill_iter(global.variables.iter().map(|variable| self.lower_global_item(variable))),
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
        }
    }

    pub(crate) fn lower_block(
        &mut self,
        block: &'scratch cst::Block<'scratch>,
    ) -> &'arena [Statement<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(block.statements.iter().map(|statement| self.lower_statement(statement)))
    }

    pub(crate) fn colon_delimited_statements_to_statement(
        &mut self,
        colon: Span,
        statements: &'scratch [cst::Statement<'scratch>],
        end: cst::Keyword<'scratch>,
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
        statements: &'scratch [cst::Statement<'scratch>],
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
        unset: &'scratch cst::Unset<'scratch>,
    ) -> Delimited<'arena, Expression<'arena, (), (), ()>> {
        Delimited {
            span: unset.left_parenthesis.join(unset.right_parenthesis),
            items: self.arena.alloc_slice_fill_iter(unset.values.iter().map(|value| self.lower_expression(value))),
        }
    }

    pub(crate) fn lower_echo(
        &mut self,
        echo: &'scratch cst::Echo<'scratch>,
    ) -> &'arena [Expression<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(echo.values.iter().map(|value| self.lower_expression(value)))
    }

    pub(crate) fn lower_echo_tag(
        &mut self,
        echo_tag: &'scratch cst::EchoTag<'scratch>,
    ) -> &'arena [Expression<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(echo_tag.values.iter().map(|value| self.lower_expression(value)))
    }

    pub(crate) fn lower_expression_statement(
        &mut self,
        expression_statement: &'scratch cst::ExpressionStatement<'scratch>,
    ) -> &'arena Expression<'arena, (), (), ()> {
        self.arena.alloc(self.lower_expression(expression_statement.expression))
    }

    pub(crate) fn lower_return(
        &mut self,
        r#return: &'scratch cst::Return<'scratch>,
    ) -> Option<&'arena Expression<'arena, (), (), ()>> {
        self.lower_optional_expression(r#return.value)
    }

    pub(crate) fn lower_optional_expression(
        &mut self,
        expression: Option<&'scratch cst::Expression<'scratch>>,
    ) -> Option<&'arena Expression<'arena, (), (), ()>> {
        expression.map(|expression| &*self.arena.alloc(self.lower_expression(expression)))
    }

    pub(crate) fn lower_expression_list(
        &mut self,
        expressions: &'scratch cst::TokenSeparatedSequence<'scratch, &'scratch cst::Expression<'scratch>>,
    ) -> &'arena [Expression<'arena, (), (), ()>] {
        self.arena
            .alloc_slice_fill_iter(expressions.iter().copied().map(|expression| self.lower_expression(expression)))
    }

    pub(crate) fn lower_if(&mut self, r#if: &'scratch cst::If<'scratch>) -> &'arena If<'arena, (), (), ()> {
        let condition: &Expression<'arena, (), (), ()> = self.arena.alloc(self.lower_expression(r#if.condition));

        match &r#if.body {
            cst::IfBody::Statement(body) => {
                let then = self.arena.alloc(self.lower_statement(body.statement));
                let r#else =
                    self.lower_statement_else_chain(body.else_if_clauses.as_slice(), body.else_clause.as_ref());

                self.arena.alloc(If { span: r#if.span(), condition, then, r#else })
            }
            cst::IfBody::ColonDelimited(body) => {
                let then =
                    self.colon_delimited_statements_to_statement(body.colon, body.statements.as_slice(), body.endif);
                let r#else = self.lower_colon_else_chain(body.else_if_clauses.as_slice(), body.else_clause.as_ref());

                self.arena.alloc(If { span: r#if.span(), condition, then, r#else })
            }
        }
    }

    fn lower_statement_else_chain(
        &mut self,
        clauses: &'scratch [cst::IfStatementBodyElseIfClause<'scratch>],
        else_clause: Option<&'scratch cst::IfStatementBodyElseClause<'scratch>>,
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
                let nested = self.arena.alloc(If { span: clause.span(), condition, then, r#else });

                Some(self.arena.alloc(Statement { meta: (), span: clause.span(), kind: StatementKind::If(nested) }))
            }
        }
    }

    fn lower_colon_else_chain(
        &mut self,
        clauses: &'scratch [cst::IfColonDelimitedBodyElseIfClause<'scratch>],
        else_clause: Option<&'scratch cst::IfColonDelimitedBodyElseClause<'scratch>>,
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
                let nested = self.arena.alloc(If { span: clause.span(), condition, then, r#else });

                Some(self.arena.alloc(Statement { meta: (), span: clause.span(), kind: StatementKind::If(nested) }))
            }
        }
    }

    pub(crate) fn lower_switch(
        &mut self,
        switch: &'scratch cst::Switch<'scratch>,
    ) -> &'arena Switch<'arena, (), (), ()> {
        let subject = self.arena.alloc(self.lower_expression(switch.expression));
        let cases = Delimited {
            span: switch.body.span(),
            items: self
                .arena
                .alloc_slice_fill_iter(switch.body.cases().iter().map(|case| self.lower_switch_case(case))),
        };

        self.arena.alloc(Switch { span: switch.span(), subject, cases })
    }

    fn lower_switch_case(&mut self, case: &'scratch cst::SwitchCase<'scratch>) -> SwitchCase<'arena, (), (), ()> {
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

    pub(crate) fn lower_while(&mut self, r#while: &'scratch cst::While<'scratch>) -> &'arena While<'arena, (), (), ()> {
        let condition = self.arena.alloc(self.lower_expression(r#while.condition));
        let statement: &Statement<'arena, (), (), ()> = match &r#while.body {
            cst::WhileBody::Statement(statement) => self.arena.alloc(self.lower_statement(statement)),
            cst::WhileBody::ColonDelimited(body) => {
                self.colon_delimited_statements_to_statement(body.colon, body.statements.as_slice(), body.end_while)
            }
        };

        self.arena.alloc(While { span: r#while.span(), condition, statement })
    }

    pub(crate) fn lower_do_while(
        &mut self,
        do_while: &'scratch cst::DoWhile<'scratch>,
    ) -> &'arena DoWhile<'arena, (), (), ()> {
        let statement = self.arena.alloc(self.lower_statement(do_while.statement));
        let condition = self.arena.alloc(self.lower_expression(do_while.condition));

        self.arena.alloc(DoWhile { span: do_while.span(), statement, condition })
    }

    pub(crate) fn lower_for(&mut self, r#for: &'scratch cst::For<'scratch>) -> &'arena For<'arena, (), (), ()> {
        let initializations = self.lower_expression_list(&r#for.initializations);
        let conditions = self.lower_expression_list(&r#for.conditions);
        let increments = self.lower_expression_list(&r#for.increments);
        let statement: &Statement<'arena, (), (), ()> = match &r#for.body {
            cst::ForBody::Statement(statement) => self.arena.alloc(self.lower_statement(statement)),
            cst::ForBody::ColonDelimited(body) => {
                self.colon_delimited_statements_to_statement(body.colon, body.statements.as_slice(), body.end_for)
            }
        };

        self.arena.alloc(For { span: r#for.span(), initializations, conditions, increments, statement })
    }

    pub(crate) fn lower_foreach(
        &mut self,
        foreach: &'scratch cst::Foreach<'scratch>,
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

        self.arena.alloc(Foreach { span: foreach.span(), expression, key, value, statement })
    }

    pub(crate) fn lower_namespace(
        &mut self,
        namespace: &'scratch cst::Namespace<'scratch>,
    ) -> &'arena Namespace<'arena, (), (), ()> {
        let name = match &namespace.name {
            Some(identifier) => Some(&*self.arena.alloc(self.lower_identifier(identifier, None))),
            None => None,
        };

        self.namespace_resolution.enter_namespace(namespace.name.as_ref().map(|identifier| identifier.value()));
        let statement = self.statements_to_statement(namespace.statements().as_slice(), namespace.namespace.span());
        self.namespace_resolution.leave_namespace();

        self.arena.alloc(Namespace { span: namespace.span(), name, statement })
    }

    pub(crate) fn lower_use(&self, r#use: &'scratch cst::Use<'scratch>) -> &'scratch [UseItem<'scratch>] {
        let mut items = Vec::new_in(self.scratch);

        match &r#use.items {
            cst::UseItems::Sequence(sequence) => {
                for item in sequence.items.iter() {
                    items.push(self.lower_use_item(item, UseItemKind::Default, None));
                }
            }
            cst::UseItems::TypedSequence(sequence) => {
                let kind = lower_use_type(&sequence.r#type);
                for item in sequence.items.iter() {
                    items.push(self.lower_use_item(item, kind, None));
                }
            }
            cst::UseItems::TypedList(list) => {
                let kind = lower_use_type(&list.r#type);
                for item in list.items.iter() {
                    items.push(self.lower_use_item(item, kind, Some(&list.namespace)));
                }
            }
            cst::UseItems::MixedList(list) => {
                for maybe in list.items.iter() {
                    let kind = maybe.r#type.as_ref().map(lower_use_type).unwrap_or(UseItemKind::Default);
                    items.push(self.lower_use_item(&maybe.item, kind, Some(&list.namespace)));
                }
            }
        }

        items.leak()
    }

    fn lower_use_item(
        &self,
        item: &'scratch cst::UseItem<'scratch>,
        kind: UseItemKind,
        prefix: Option<&'scratch cst::Identifier<'scratch>>,
    ) -> UseItem<'scratch> {
        let (mut value, identifier_kind) = match prefix {
            Some(prefix) => {
                let mut joined = Vec::new_in(self.scratch);
                joined.extend_from_slice(prefix.value());
                joined.push(b'\\');
                joined.extend_from_slice(item.name.value());

                let identifier_kind = if prefix.is_fully_qualified() {
                    IdentifierKind::FullyQualified
                } else {
                    IdentifierKind::Qualified
                };

                (&*joined.leak(), identifier_kind)
            }
            None => (item.name.value(), use_identifier_kind(&item.name)),
        };

        if let [b'\\', rest @ ..] = value {
            value = rest;
        }

        let alias = item.alias.as_ref().map(|alias| alias.identifier.value);

        UseItem {
            span: item.span(),
            kind,
            item: Identifier { span: item.name.span(), value, kind: identifier_kind },
            alias,
        }
    }

    pub(crate) fn lower_declare(
        &mut self,
        declare: &'scratch cst::Declare<'scratch>,
    ) -> &'arena Declare<'arena, (), (), ()> {
        self.arena.alloc(Declare {
            span: declare.span(),
            items: Delimited {
                span: declare.left_parenthesis.join(declare.right_parenthesis),
                items: self.arena.alloc_slice_fill_iter(declare.items.iter().map(|item| DeclareItem {
                    span: item.span(),
                    name: self.lower_name(&item.name),
                    value: Some(self.arena.alloc(self.lower_expression(item.value))),
                })),
            },
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

    fn lower_static_item(&mut self, item: &'scratch cst::StaticItem<'scratch>) -> StaticItem<'arena, (), (), ()> {
        match item {
            cst::StaticItem::Abstract(item) => StaticItem {
                span: item.span(),
                variable: self.lower_direct_variable(&item.variable),
                type_annotation: None,
                value: None,
            },
            cst::StaticItem::Concrete(item) => StaticItem {
                span: item.span(),
                variable: self.lower_direct_variable(&item.variable),
                type_annotation: None,
                value: Some(self.arena.alloc(self.lower_expression(item.value))),
            },
        }
    }

    fn lower_global_item(&mut self, variable: &'scratch cst::Variable<'scratch>) -> GlobalItem<'arena, (), (), ()> {
        let span = variable.span();
        let lowered = self.lower_variable(variable);
        if let Variable::Direct(direct) = lowered {
            self.body_effects.accessed_globals.push(direct);
        }

        GlobalItem { span, variable: lowered, type_annotation: None }
    }

    pub(crate) fn lower_try(&mut self, r#try: &'scratch cst::Try<'scratch>) -> &'arena Try<'arena, (), (), ()> {
        let statement = self.statements_to_statement(r#try.block.statements.as_slice(), r#try.block.span());
        let catch_clauses = self
            .arena
            .alloc_slice_fill_iter(r#try.catch_clauses.iter().map(|clause| self.lower_try_catch_clause(clause)));
        let finally_clause = r#try
            .finally_clause
            .as_ref()
            .map(|finally| self.statements_to_statement(finally.block.statements.as_slice(), finally.block.span()));

        self.arena.alloc(Try { span: r#try.span(), statement, catch_clauses, finally_clause })
    }

    fn lower_try_catch_clause(
        &mut self,
        clause: &'scratch cst::TryCatchClause<'scratch>,
    ) -> TryCatchClause<'arena, (), (), ()> {
        TryCatchClause {
            span: clause.span(),
            r#type: self.lower_type(&clause.hint),
            variable: clause.variable.as_ref().map(|variable| self.lower_direct_variable(variable)),
            statement: self.statements_to_statement(clause.block.statements.as_slice(), clause.block.span()),
        }
    }
}

fn lower_use_type(r#type: &cst::UseType<'_>) -> UseItemKind {
    match r#type {
        cst::UseType::Function(_) => UseItemKind::Function,
        cst::UseType::Const(_) => UseItemKind::Const,
    }
}

fn use_identifier_kind(identifier: &cst::Identifier<'_>) -> IdentifierKind {
    match identifier {
        cst::Identifier::Local(_) => IdentifierKind::Local,
        cst::Identifier::Qualified(_) => IdentifierKind::Qualified,
        cst::Identifier::FullyQualified(_) => IdentifierKind::FullyQualified,
    }
}
