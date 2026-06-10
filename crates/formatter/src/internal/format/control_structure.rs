use mago_allocator::Arena;
use mago_allocator::vec_in;
use mago_span::HasSpan;
use mago_syntax::cst::Break;
use mago_syntax::cst::Continue;
use mago_syntax::cst::DoWhile;
use mago_syntax::cst::Expression;
use mago_syntax::cst::For;
use mago_syntax::cst::ForBody;
use mago_syntax::cst::ForColonDelimitedBody;
use mago_syntax::cst::Foreach;
use mago_syntax::cst::ForeachBody;
use mago_syntax::cst::ForeachColonDelimitedBody;
use mago_syntax::cst::ForeachKeyValueTarget;
use mago_syntax::cst::ForeachTarget;
use mago_syntax::cst::ForeachValueTarget;
use mago_syntax::cst::If;
use mago_syntax::cst::IfBody;
use mago_syntax::cst::IfColonDelimitedBody;
use mago_syntax::cst::IfColonDelimitedBodyElseClause;
use mago_syntax::cst::IfColonDelimitedBodyElseIfClause;
use mago_syntax::cst::IfStatementBody;
use mago_syntax::cst::IfStatementBodyElseClause;
use mago_syntax::cst::IfStatementBodyElseIfClause;
use mago_syntax::cst::Statement;
use mago_syntax::cst::Switch;
use mago_syntax::cst::SwitchBody;
use mago_syntax::cst::SwitchBraceDelimitedBody;
use mago_syntax::cst::SwitchCase;
use mago_syntax::cst::SwitchCaseSeparator;
use mago_syntax::cst::SwitchColonDelimitedBody;
use mago_syntax::cst::SwitchDefaultCase;
use mago_syntax::cst::SwitchExpressionCase;
use mago_syntax::cst::While;
use mago_syntax::cst::WhileBody;
use mago_syntax::cst::WhileColonDelimitedBody;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::block::print_block_of_nodes;
use crate::internal::format::format_token;
use crate::internal::format::misc;
use crate::internal::format::misc::print_colon_delimited_body;
use crate::internal::format::statement::print_statement_sequence;
use crate::settings::BraceStyle;
use crate::wrap;

impl<'arena, A> Format<'arena, A> for If<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, If, {
            Document::Group(Group::new(vec_in![f.arena;
                self.r#if.format(f),
                misc::print_condition(
                    f,
                    self.left_parenthesis,
                    self.condition,
                    self.right_parenthesis,
                ),
                self.body.format(f),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for IfBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IfBody, {
            match &self {
                IfBody::Statement(b) => b.format(f),
                IfBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for IfStatementBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IfStatementBody, {
            let mut parts = vec_in![f.arena; misc::print_clause(f, self.statement, false)];

            for else_if_clause in &self.else_if_clauses {
                parts.push(else_if_clause.format(f));
            }

            if let Some(else_clause) = &self.else_clause {
                parts.push(else_clause.format(f));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena, A> Format<'arena, A> for IfStatementBodyElseClause<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IfStatementBodyElseClause, {
            Document::Group(Group::new(
                vec_in![f.arena; self.r#else.format(f), misc::print_clause(f, self.statement, false)],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for IfStatementBodyElseIfClause<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IfStatementBodyElseIfClause, {
            Document::Group(Group::new(vec_in![f.arena;
                self.elseif.format(f),
                misc::print_condition(
                    f,
                    self.left_parenthesis,
                    self.condition,
                    self.right_parenthesis,
                ),
                misc::print_clause(f, self.statement, false),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for IfColonDelimitedBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        {
            let node = mago_syntax::cst::Node::IfColonDelimitedBody(self);
            f.enter_node(node);
            let was_wrapped_in_parens = f.is_wrapped_in_parens;
            let needed_to_wrap_in_parens = f.need_parens(node);
            f.is_wrapped_in_parens |= needed_to_wrap_in_parens;
            let leading = f.print_leading_comments(node.span());
            let doc = {
                let mut parts = vec_in![f.arena; Document::String(b":")];
                let mut statements = print_statement_sequence(f, &self.statements);
                if !statements.is_empty() {
                    if let Some(Statement::ClosingTag(_)) = self.statements.first() {
                        statements.insert(0, Document::String(b" "));
                        parts.push(Document::Array(statements));
                    } else {
                        statements.insert(0, Document::Line(Line::hard()));
                        parts.push(Document::Indent(statements));
                    }
                }

                parts.push(match self.statements.last() {
                    Some(Statement::OpeningTag(_)) => Document::space(),
                    _ => Document::Line(Line::hard()),
                });

                for else_if_clause in &self.else_if_clauses {
                    parts.push(else_if_clause.format(f));

                    parts.push(match else_if_clause.statements.last() {
                        Some(Statement::OpeningTag(_)) => Document::space(),
                        _ => Document::Line(Line::hard()),
                    });
                }
                if let Some(else_clause) = &self.else_clause {
                    parts.push(else_clause.format(f));
                    parts.push(match else_clause.statements.last() {
                        Some(Statement::OpeningTag(_)) => Document::space(),
                        _ => Document::Line(Line::hard()),
                    });
                }
                parts.push(self.endif.format(f));
                parts.push(self.terminator.format(f));
                Document::Group(Group::new(parts))
            };
            let trailing = f.print_trailing_comments_for_node(node);
            let has_leading_comments = leading.is_some();
            let doc = f.print_comments(leading, doc, trailing);
            let doc = if needed_to_wrap_in_parens { f.add_parens(doc, node, has_leading_comments) } else { doc };
            f.leave_node();
            f.is_wrapped_in_parens = was_wrapped_in_parens;
            doc
        }
    }
}

impl<'arena, A> Format<'arena, A> for IfColonDelimitedBodyElseIfClause<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IfColonDelimitedBodyElseIfClause, {
            let mut parts = vec_in![f.arena; self.elseif.format(f)];

            let condition = misc::print_condition(f, self.left_parenthesis, self.condition, self.right_parenthesis);
            let is_first_stmt_closing_tag = matches!(self.statements.first(), Some(Statement::ClosingTag(_)));
            if is_first_stmt_closing_tag {
                parts.push(Document::Indent(vec_in![f.arena; condition, Document::String(b":")]));
            } else {
                parts.push(condition);
                parts.push(Document::String(b":"));
            }

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                if is_first_stmt_closing_tag {
                    statements.insert(0, Document::String(b" "));
                    parts.push(Document::Array(statements));
                } else {
                    statements.insert(0, Document::Line(Line::hard()));
                    parts.push(Document::Indent(statements));
                }
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena, A> Format<'arena, A> for IfColonDelimitedBodyElseClause<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IfColonDelimitedBodyElseClause, {
            let mut parts = vec_in![f.arena; self.r#else.format(f), Document::String(b":")];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                if let Some(Statement::ClosingTag(_)) = self.statements.first() {
                    statements.insert(0, Document::String(b" "));
                    parts.push(Document::Array(statements));
                } else {
                    statements.insert(0, Document::Line(Line::hard()));
                    parts.push(Document::Indent(statements));
                }
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena, A> Format<'arena, A> for DoWhile<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, DoWhile, {
            Document::Group(Group::new(vec_in![f.arena;
                self.r#do.format(f),
                misc::print_clause(f, self.statement, false),
                self.r#while.format(f),
                misc::print_condition(
                    f,
                    self.left_parenthesis,
                    self.condition,
                    self.right_parenthesis,
                ),
                self.terminator.format(f),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for For<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, For, {
            let mut contents = vec_in![f.arena;
                self.r#for.format(f),
                Document::space(),
                format_token(f, self.left_parenthesis, b"("),
            ];

            fn format_expressions<'arena, A>(
                f: &mut FormatterState<'_, 'arena, A>,
                exprs: &'arena [&'arena Expression<'arena>],
            ) -> Document<'arena, A>
            where
                A: Arena,
            {
                let Some(first) = exprs.first() else {
                    return Document::empty();
                };

                let first = first.format(f);
                let rest = exprs[1..].iter().map(|expression| expression.format(f)).collect::<Vec<_>>();

                if rest.is_empty() {
                    first
                } else {
                    let mut contents = vec_in![f.arena; first, Document::String(b",")];
                    for (i, expression) in rest.into_iter().enumerate() {
                        if i != 0 {
                            contents.push(Document::String(b","));
                        }

                        contents.push(Document::Indent(vec_in![f.arena; Document::Line(Line::default()), expression]));
                    }

                    Document::Group(Group::new(contents))
                }
            }

            contents.push(Document::Group(Group::new(vec_in![f.arena;
                Document::Indent(vec_in![f.arena;
                    Document::Line(Line::soft()),
                    format_expressions(f, self.initializations.as_slice()),
                    Document::String(b";"),
                    if self.conditions.is_empty() { Document::empty() } else { Document::Line(Line::default()) },
                    format_expressions(f, self.conditions.as_slice()),
                    Document::String(b";"),
                    if self.increments.is_empty() { Document::empty() } else { Document::Line(Line::default()) },
                    format_expressions(f, self.increments.as_slice()),
                ]),
                Document::Line(Line::soft()),
            ])));

            contents.push(format_token(f, self.right_parenthesis, b")"));
            contents.push(self.body.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena, A> Format<'arena, A> for ForColonDelimitedBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ForColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_for, &self.terminator)
        })
    }
}

impl<'arena, A> Format<'arena, A> for ForBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ForBody, {
            match self {
                ForBody::Statement(s) => {
                    let stmt = s.format(f);

                    misc::adjust_clause(f, s, stmt, false)
                }
                ForBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for Switch<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Switch, {
            Document::Array(vec_in![f.arena;
                self.switch.format(f),
                misc::print_condition(
                    f,
                    self.left_parenthesis,
                    self.expression,
                    self.right_parenthesis,
                ),
                self.body.format(f),
            ])
        })
    }
}

impl<'arena, A> Format<'arena, A> for SwitchBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, SwitchBody, {
            match self {
                SwitchBody::BraceDelimited(b) => Document::Array(vec_in![f.arena;
                    match f.settings.control_brace_style {
                        BraceStyle::SameLine => Document::space(),
                        BraceStyle::NextLine | BraceStyle::AlwaysNextLine => {
                            if b.cases.is_empty() && f.settings.inline_empty_control_braces {
                                Document::space()
                            } else {
                                Document::Line(Line::hard())
                            }
                        }
                    },
                    b.format(f),
                ]),
                SwitchBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for SwitchColonDelimitedBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, SwitchColonDelimitedBody, {
            let mut contents = vec_in![f.arena; Document::String(b":")];
            for case in &self.cases {
                contents.push(Document::Indent(vec_in![f.arena; Document::Line(Line::hard()), case.format(f)]));
            }

            if let Some(comment) = f.print_dangling_comments(self.colon.join(self.end_switch.span), true) {
                contents.push(comment);
            } else {
                contents.push(Document::Line(Line::hard()));
            }

            contents.push(self.end_switch.format(f));
            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena, A> Format<'arena, A> for SwitchBraceDelimitedBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, SwitchBraceDelimitedBody, {
            print_block_of_nodes(
                f,
                &self.left_brace,
                &self.cases,
                &self.right_brace,
                f.settings.inline_empty_control_braces,
            )
        })
    }
}

impl<'arena, A> Format<'arena, A> for SwitchCase<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, SwitchCase, {
            match self {
                SwitchCase::Expression(c) => c.format(f),
                SwitchCase::Default(c) => c.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for SwitchExpressionCase<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, SwitchExpressionCase, {
            let mut parts = vec_in![f.arena; self.case.format(f), Document::space(), self.expression.format(f), self.separator.format(f)];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hard()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena, A> Format<'arena, A> for SwitchDefaultCase<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, SwitchDefaultCase, {
            let mut parts = vec_in![f.arena; self.default.format(f), self.separator.format(f)];
            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hard()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena, A> Format<'arena, A> for SwitchCaseSeparator
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, SwitchCaseSeparator, {
            match self {
                SwitchCaseSeparator::Colon(_) => Document::String(b":"),
                SwitchCaseSeparator::SemiColon(_) => Document::String(b";"),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for While<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, While, {
            Document::Array(vec_in![f.arena;
                self.r#while.format(f),
                misc::print_condition(
                    f,
                    self.left_parenthesis,
                    self.condition,
                    self.right_parenthesis,
                ),
                self.body.format(f),
            ])
        })
    }
}

impl<'arena, A> Format<'arena, A> for WhileBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, WhileBody, {
            match self {
                WhileBody::Statement(s) => misc::print_clause(f, s, false),
                WhileBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for WhileColonDelimitedBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, WhileColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_while, &self.terminator)
        })
    }
}

impl<'arena, A> Format<'arena, A> for Foreach<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Foreach, {
            Document::Array(vec_in![f.arena;
                self.foreach.format(f),
                Document::space(),
                format_token(f, self.left_parenthesis, b"("),
                self.expression.format(f),
                Document::space(),
                self.r#as.format(f),
                Document::space(),
                self.target.format(f),
                format_token(f, self.right_parenthesis, b")"),
                self.body.format(f),
            ])
        })
    }
}

impl<'arena, A> Format<'arena, A> for ForeachTarget<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ForeachTarget, {
            match self {
                ForeachTarget::Value(t) => t.format(f),
                ForeachTarget::KeyValue(t) => t.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for ForeachValueTarget<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ForeachValueTarget, { self.value.format(f) })
    }
}

impl<'arena, A> Format<'arena, A> for ForeachKeyValueTarget<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ForeachKeyValueTarget, {
            Document::Group(Group::new(vec_in![f.arena;
                self.key.format(f),
                Document::space(),
                Document::String(b"=>"),
                Document::space(),
                self.value.format(f),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for ForeachBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ForeachBody, {
            match self {
                ForeachBody::Statement(s) => misc::print_clause(f, s, false),
                ForeachBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for ForeachColonDelimitedBody<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ForeachColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_foreach, &self.terminator)
        })
    }
}

impl<'arena, A> Format<'arena, A> for Continue<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Continue, {
            Document::Group(Group::new(vec_in![f.arena;
                self.r#continue.format(f),
                if let Some(level) = &self.level {
                    Document::Array(vec_in![f.arena; Document::space(), level.format(f)])
                } else {
                    Document::empty()
                },
                self.terminator.format(f),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for Break<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Break, {
            Document::Group(Group::new(vec_in![f.arena;
                self.r#break.format(f),
                if let Some(level) = &self.level {
                    Document::Array(vec_in![f.arena; Document::space(), level.format(f)])
                } else {
                    Document::empty()
                },
                self.terminator.format(f),
            ]))
        })
    }
}
