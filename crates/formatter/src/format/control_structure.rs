use mago_ast::*;
use mago_span::HasSpan;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::format::block::print_block_of_nodes;
use crate::format::misc;
use crate::format::misc::print_colon_delimited_body;
use crate::format::statement::print_statement_sequence;
use crate::format::Format;
use crate::settings::*;
use crate::wrap;
use crate::Formatter;

impl<'a, 'alloc> Format<'a, 'alloc> for If<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, If, {
            Document::Group(Group::new(vec![
                self.r#if.format(f),
                Document::space(),
                misc::print_condition(f, &self.condition),
                self.body.format(f),
            ]))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for IfBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, IfBody, {
            match &self {
                IfBody::Statement(b) => b.format(f),
                IfBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for IfStatementBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, IfStatementBody, {
            let mut parts = vec![misc::print_clause(f, &self.statement, false)];

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

impl<'a, 'alloc> Format<'a, 'alloc> for IfStatementBodyElseClause<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, IfStatementBodyElseClause, {
            Document::Group(Group::new(vec![self.r#else.format(f), misc::print_clause(f, &self.statement, false)]))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for IfStatementBodyElseIfClause<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, IfStatementBodyElseIfClause, {
            Document::Group(Group::new(vec![
                self.elseif.format(f),
                Document::space(),
                misc::print_condition(f, &self.condition),
                misc::print_clause(f, &self.statement, false),
            ]))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for IfColonDelimitedBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBody, {
            let mut parts = vec![Document::String(":")];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                if let Some(Statement::ClosingTag(_)) = self.statements.first() {
                    statements.insert(0, Document::String(" "));
                    parts.push(Document::Array(statements));
                } else {
                    statements.insert(0, Document::Line(Line::hardline()));
                    parts.push(Document::Indent(statements));
                }
            }

            if !matches!(self.statements.last(), Some(Statement::OpeningTag(_))) {
                parts.push(Document::Line(Line::hardline()));
            } else {
                parts.push(Document::String(" "));
            }

            for else_if_clause in self.else_if_clauses.iter() {
                parts.push(else_if_clause.format(f));
                if !matches!(else_if_clause.statements.last(), Some(Statement::OpeningTag(_))) {
                    parts.push(Document::Line(Line::hardline()));
                } else {
                    parts.push(Document::String(" "));
                }
            }

            if let Some(else_clause) = &self.else_clause {
                parts.push(else_clause.format(f));
                if !matches!(else_clause.statements.last(), Some(Statement::OpeningTag(_))) {
                    parts.push(Document::Line(Line::hardline()));
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

impl<'a, 'alloc> Format<'a, 'alloc> for IfColonDelimitedBodyElseIfClause<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBodyElseIfClause, {
            let mut parts = vec![self.elseif.format(f), Document::space()];

            let condition = misc::print_condition(f, &self.condition);
            let is_first_stmt_closing_tag = matches!(self.statements.first(), Some(Statement::ClosingTag(_)));
            if is_first_stmt_closing_tag {
                parts.push(Document::Indent(vec![condition, Document::String(":")]));
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
                    statements.insert(0, Document::Line(Line::hardline()));
                    parts.push(Document::Indent(statements));
                }
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for IfColonDelimitedBodyElseClause<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBodyElseClause, {
            let mut parts = vec![self.r#else.format(f), Document::String(":")];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                if let Some(Statement::ClosingTag(_)) = self.statements.first() {
                    statements.insert(0, Document::String(" "));
                    parts.push(Document::Array(statements));
                } else {
                    statements.insert(0, Document::Line(Line::hardline()));
                    parts.push(Document::Indent(statements));
                }
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for DoWhile<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, DoWhile, {
            Document::Group(Group::new(vec![
                self.r#do.format(f),
                misc::print_clause(f, &self.statement, false),
                self.r#while.format(f),
                Document::space(),
                misc::print_condition(f, &self.condition),
                self.terminator.format(f),
            ]))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for For<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, For, {
            let mut contents = vec![self.r#for.format(f), Document::String(" (")];

            let format_expressions = |f: &mut Formatter<'a, 'alloc>, expressions: &'a [Expression<'alloc>]| {
                let Some(first) = expressions.first() else {
                    return Document::empty();
                };

                let first = first.format(f);
                let rest = expressions[1..].iter().map(|expression| expression.format(f)).collect::<Vec<_>>();

                if rest.is_empty() {
                    first
                } else {
                    let mut contents = vec![first, Document::String(",")];
                    for (i, expression) in rest.into_iter().enumerate() {
                        if i != 0 {
                            contents.push(Document::String(","));
                        }

                        contents.push(Document::Indent(vec![Document::Line(Line::default()), expression]));
                    }

                    Document::Group(Group::new(contents))
                }
            };

            contents.push(Document::Group(Group::new(vec![
                Document::Indent(vec![
                    Document::Line(Line::softline()),
                    format_expressions(f, self.initializations.as_slice()),
                    Document::String(";"),
                    if self.conditions.is_empty() { Document::empty() } else { Document::Line(Line::default()) },
                    format_expressions(f, self.conditions.as_slice()),
                    Document::String(";"),
                    if self.increments.is_empty() { Document::empty() } else { Document::Line(Line::default()) },
                    format_expressions(f, self.increments.as_slice()),
                ]),
                Document::Line(Line::softline()),
            ])));

            contents.push(Document::String(")"));
            contents.push(self.body.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for ForColonDelimitedBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, ForColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_for, &self.terminator)
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for ForBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
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

impl<'a, 'alloc> Format<'a, 'alloc> for Switch<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, Switch, {
            Document::Array(vec![
                self.switch.format(f),
                Document::space(),
                Document::String("("),
                self.expression.format(f),
                Document::String(")"),
                self.body.format(f),
            ])
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for SwitchBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, SwitchBody, {
            match self {
                SwitchBody::BraceDelimited(b) => Document::Array(vec![
                    match f.settings.control_brace_style {
                        BraceStyle::SameLine => Document::space(),
                        BraceStyle::NextLine => Document::Line(Line::hardline()),
                    },
                    b.format(f),
                ]),
                SwitchBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for SwitchColonDelimitedBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, SwitchColonDelimitedBody, {
            let mut contents = vec![Document::String(":")];
            for case in self.cases.iter() {
                contents.push(Document::Indent(vec![Document::Line(Line::hardline()), case.format(f)]));
            }

            if let Some(comment) = f.print_dangling_comments(self.colon.join(self.end_switch.span), true) {
                contents.push(comment);
            } else {
                contents.push(Document::Line(Line::hardline()));
            }

            contents.push(self.end_switch.format(f));
            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for SwitchBraceDelimitedBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, SwitchBraceDelimitedBody, {
            print_block_of_nodes(f, &self.left_brace, &self.cases, &self.right_brace, false)
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for SwitchCase<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, SwitchCase, {
            match self {
                SwitchCase::Expression(c) => c.format(f),
                SwitchCase::Default(c) => c.format(f),
            }
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for SwitchExpressionCase<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, SwitchExpressionCase, {
            let mut parts =
                vec![self.case.format(f), Document::space(), self.expression.format(f), self.separator.format(f)];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hardline()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for SwitchDefaultCase<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, SwitchDefaultCase, {
            let mut parts = vec![self.default.format(f), self.separator.format(f)];
            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hardline()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for SwitchCaseSeparator {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, SwitchCaseSeparator, {
            match self {
                SwitchCaseSeparator::Colon(_) => Document::String(":"),
                SwitchCaseSeparator::SemiColon(_) => Document::String(";"),
            }
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for While<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, While, {
            Document::Array(vec![
                self.r#while.format(f),
                Document::space(),
                Document::String("("),
                self.condition.format(f),
                Document::String(")"),
                self.body.format(f),
            ])
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for WhileBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, WhileBody, {
            match self {
                WhileBody::Statement(s) => misc::print_clause(f, s, false),
                WhileBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for WhileColonDelimitedBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, WhileColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_while, &self.terminator)
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for Foreach<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, Foreach, {
            Document::Array(vec![
                self.foreach.format(f),
                Document::space(),
                Document::String("("),
                self.expression.format(f),
                Document::space(),
                self.r#as.format(f),
                Document::space(),
                self.target.format(f),
                Document::String(")"),
                self.body.format(f),
            ])
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for ForeachTarget<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, ForeachTarget, {
            match self {
                ForeachTarget::Value(t) => t.format(f),
                ForeachTarget::KeyValue(t) => t.format(f),
            }
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for ForeachValueTarget<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, ForeachValueTarget, { self.value.format(f) })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for ForeachKeyValueTarget<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, ForeachKeyValueTarget, {
            Document::Group(Group::new(vec![
                self.key.format(f),
                Document::space(),
                Document::String("=>"),
                Document::space(),
                self.value.format(f),
            ]))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for ForeachBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, ForeachBody, {
            match self {
                ForeachBody::Statement(s) => misc::print_clause(f, s, false),
                ForeachBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for ForeachColonDelimitedBody<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, ForeachColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_foreach, &self.terminator)
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for Continue<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, Continue, {
            Document::Group(Group::new(vec![
                self.r#continue.format(f),
                if let Some(level) = &self.level {
                    Document::Array(vec![Document::space(), level.format(f)])
                } else {
                    Document::empty()
                },
                self.terminator.format(f),
            ]))
        })
    }
}

impl<'a, 'alloc> Format<'a, 'alloc> for Break<'alloc> {
    fn format(&'a self, f: &mut Formatter<'a, 'alloc>) -> Document<'a> {
        wrap!(f, self, Break, {
            Document::Group(Group::new(vec![
                self.r#break.format(f),
                if let Some(level) = &self.level {
                    Document::Array(vec![Document::space(), level.format(f)])
                } else {
                    Document::empty()
                },
                self.terminator.format(f),
            ]))
        })
    }
}
