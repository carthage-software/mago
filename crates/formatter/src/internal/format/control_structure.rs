use bumpalo::vec;

use mago_span::HasSpan;
use mago_syntax::ast::*;

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
use crate::settings::*;
use crate::wrap;

impl<'arena> Format<'arena> for If<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, If, {
            Document::Group(Group::new(vec![
                in f.arena;
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

impl<'arena> Format<'arena> for IfBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IfBody, {
            match &self {
                IfBody::Statement(b) => b.format(f),
                IfBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for IfStatementBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IfStatementBody, {
            let mut parts = vec![in f.arena; misc::print_clause(f, self.statement, false)];

            for else_if_clause in self.else_if_clauses.iter() {
                parts.push(else_if_clause.format(f));
            }

            if let Some(else_clause) = &self.else_clause {
                parts.push(else_clause.format(f));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena> Format<'arena> for IfStatementBodyElseClause<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IfStatementBodyElseClause, {
            Document::Group(Group::new(
                vec![in f.arena; self.r#else.format(f), misc::print_clause(f, self.statement, false)],
            ))
        })
    }
}

impl<'arena> Format<'arena> for IfStatementBodyElseIfClause<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IfStatementBodyElseIfClause, {
            Document::Group(Group::new(vec![
                in f.arena;
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

impl<'arena> Format<'arena> for IfColonDelimitedBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IfColonDelimitedBody, {
            let mut parts = vec![in f.arena; Document::String(":")];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                if let Some(Statement::ClosingTag(_)) = self.statements.first() {
                    statements.insert(0, Document::String(" "));
                    parts.push(Document::Array(statements));
                } else {
                    statements.insert(0, Document::Line(Line::hard()));
                    parts.push(Document::Indent(statements));
                }
            }

            if !matches!(self.statements.last(), Some(Statement::OpeningTag(_))) {
                parts.push(Document::Line(Line::hard()));
            } else {
                parts.push(Document::String(" "));
            }

            for else_if_clause in self.else_if_clauses.iter() {
                parts.push(else_if_clause.format(f));
                if !matches!(else_if_clause.statements.last(), Some(Statement::OpeningTag(_))) {
                    parts.push(Document::Line(Line::hard()));
                } else {
                    parts.push(Document::String(" "));
                }
            }

            if let Some(else_clause) = &self.else_clause {
                parts.push(else_clause.format(f));
                if !matches!(else_clause.statements.last(), Some(Statement::OpeningTag(_))) {
                    parts.push(Document::Line(Line::hard()));
                } else {
                    parts.push(Document::String(" "));
                }
            }

            parts.push(self.endif.format(f));
            parts.push(self.terminator.format(f));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena> Format<'arena> for IfColonDelimitedBodyElseIfClause<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IfColonDelimitedBodyElseIfClause, {
            let mut parts = vec![in f.arena; self.elseif.format(f)];

            let condition = misc::print_condition(f, self.left_parenthesis, self.condition, self.right_parenthesis);
            let is_first_stmt_closing_tag = matches!(self.statements.first(), Some(Statement::ClosingTag(_)));
            if is_first_stmt_closing_tag {
                parts.push(Document::Indent(vec![in f.arena; condition, Document::String(":")]));
            } else {
                parts.push(condition);
                parts.push(Document::String(":"));
            }

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                if is_first_stmt_closing_tag {
                    statements.insert(0, Document::String(" "));
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

impl<'arena> Format<'arena> for IfColonDelimitedBodyElseClause<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IfColonDelimitedBodyElseClause, {
            let mut parts = vec![in f.arena; self.r#else.format(f), Document::String(":")];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                if let Some(Statement::ClosingTag(_)) = self.statements.first() {
                    statements.insert(0, Document::String(" "));
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

impl<'arena> Format<'arena> for DoWhile<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, DoWhile, {
            Document::Group(Group::new(vec![
                in f.arena;
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

impl<'arena> Format<'arena> for For<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, For, {
            let mut contents = vec![
                in f.arena;
                self.r#for.format(f),
                Document::space(),
                format_token(f, self.left_parenthesis, "("),
            ];

            let format_expressions = |f: &mut FormatterState<'_, 'arena>, exprs: &'arena [Expression<'arena>]| {
                let Some(first) = exprs.first() else {
                    return Document::empty();
                };

                let first = first.format(f);
                let rest = exprs[1..].iter().map(|expression| expression.format(f)).collect::<Vec<_>>();

                if rest.is_empty() {
                    first
                } else {
                    let mut contents = vec![in f.arena; first, Document::String(",")];
                    for (i, expression) in rest.into_iter().enumerate() {
                        if i != 0 {
                            contents.push(Document::String(","));
                        }

                        contents.push(Document::Indent(vec![in f.arena; Document::Line(Line::default()), expression]));
                    }

                    Document::Group(Group::new(contents))
                }
            };

            contents.push(Document::Group(Group::new(vec![
                in f.arena;
                Document::Indent(vec![
                    in f.arena;
                    Document::Line(Line::soft()),
                    format_expressions(f, self.initializations.as_slice()),
                    Document::String(";"),
                    if self.conditions.is_empty() { Document::empty() } else { Document::Line(Line::default()) },
                    format_expressions(f, self.conditions.as_slice()),
                    Document::String(";"),
                    if self.increments.is_empty() { Document::empty() } else { Document::Line(Line::default()) },
                    format_expressions(f, self.increments.as_slice()),
                ]),
                Document::Line(Line::soft()),
            ])));

            contents.push(format_token(f, self.right_parenthesis, ")"));
            contents.push(self.body.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for ForColonDelimitedBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ForColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_for, &self.terminator)
        })
    }
}

impl<'arena> Format<'arena> for ForBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
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

impl<'arena> Format<'arena> for Switch<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Switch, {
            Document::Array(vec![
                in f.arena;
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

impl<'arena> Format<'arena> for SwitchBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, SwitchBody, {
            match self {
                SwitchBody::BraceDelimited(b) => Document::Array(vec![
                    in f.arena;
                    match f.settings.control_brace_style {
                        BraceStyle::SameLine => Document::space(),
                        BraceStyle::NextLine => {
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

impl<'arena> Format<'arena> for SwitchColonDelimitedBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, SwitchColonDelimitedBody, {
            let mut contents = vec![in f.arena; Document::String(":")];
            for case in self.cases.iter() {
                contents.push(Document::Indent(vec![in f.arena; Document::Line(Line::hard()), case.format(f)]));
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

impl<'arena> Format<'arena> for SwitchBraceDelimitedBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
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

impl<'arena> Format<'arena> for SwitchCase<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, SwitchCase, {
            match self {
                SwitchCase::Expression(c) => c.format(f),
                SwitchCase::Default(c) => c.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for SwitchExpressionCase<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, SwitchExpressionCase, {
            let mut parts = vec![in f.arena; self.case.format(f), Document::space(), self.expression.format(f), self.separator.format(f)];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hard()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena> Format<'arena> for SwitchDefaultCase<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, SwitchDefaultCase, {
            let mut parts = vec![in f.arena; self.default.format(f), self.separator.format(f)];
            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hard()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena> Format<'arena> for SwitchCaseSeparator {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, SwitchCaseSeparator, {
            match self {
                SwitchCaseSeparator::Colon(_) => Document::String(":"),
                SwitchCaseSeparator::SemiColon(_) => Document::String(";"),
            }
        })
    }
}

impl<'arena> Format<'arena> for While<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, While, {
            Document::Array(vec![
                in f.arena;
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

impl<'arena> Format<'arena> for WhileBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, WhileBody, {
            match self {
                WhileBody::Statement(s) => misc::print_clause(f, s, false),
                WhileBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for WhileColonDelimitedBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, WhileColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_while, &self.terminator)
        })
    }
}

impl<'arena> Format<'arena> for Foreach<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Foreach, {
            Document::Array(vec![
                in f.arena;
                self.foreach.format(f),
                Document::space(),
                format_token(f, self.left_parenthesis, "("),
                self.expression.format(f),
                Document::space(),
                self.r#as.format(f),
                Document::space(),
                self.target.format(f),
                format_token(f, self.right_parenthesis, ")"),
                self.body.format(f),
            ])
        })
    }
}

impl<'arena> Format<'arena> for ForeachTarget<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ForeachTarget, {
            match self {
                ForeachTarget::Value(t) => t.format(f),
                ForeachTarget::KeyValue(t) => t.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for ForeachValueTarget<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ForeachValueTarget, { self.value.format(f) })
    }
}

impl<'arena> Format<'arena> for ForeachKeyValueTarget<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ForeachKeyValueTarget, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.key.format(f),
                Document::space(),
                Document::String("=>"),
                Document::space(),
                self.value.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for ForeachBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ForeachBody, {
            match self {
                ForeachBody::Statement(s) => misc::print_clause(f, s, false),
                ForeachBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for ForeachColonDelimitedBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ForeachColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_foreach, &self.terminator)
        })
    }
}

impl<'arena> Format<'arena> for Continue<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Continue, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.r#continue.format(f),
                if let Some(level) = &self.level {
                    Document::Array(vec![in f.arena; Document::space(), level.format(f)])
                } else {
                    Document::empty()
                },
                self.terminator.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for Break<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Break, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.r#break.format(f),
                if let Some(level) = &self.level {
                    Document::Array(vec![in f.arena; Document::space(), level.format(f)])
                } else {
                    Document::empty()
                },
                self.terminator.format(f),
            ]))
        })
    }
}
