use mago_ast::*;
use mago_span::HasSpan;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::block::print_block_of_nodes;
use crate::internal::format::misc;
use crate::internal::format::misc::print_colon_delimited_body;
use crate::internal::format::statement::print_statement_sequence;
use crate::settings::*;
use crate::wrap;

impl<'a> Format<'a> for If {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, If, {
            Document::Group(Group::new(vec![
                self.r#if.format(f),
                misc::print_condition(
                    f,
                    &self.condition,
                    f.settings.space_before_if_parenthesis,
                    f.settings.space_within_if_parenthesis,
                ),
                self.body.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for IfBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, IfBody, {
            match &self {
                IfBody::Statement(b) => b.format(f),
                IfBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for IfStatementBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for IfStatementBodyElseClause {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, IfStatementBodyElseClause, {
            Document::Group(Group::new(vec![self.r#else.format(f), misc::print_clause(f, &self.statement, false)]))
        })
    }
}

impl<'a> Format<'a> for IfStatementBodyElseIfClause {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, IfStatementBodyElseIfClause, {
            Document::Group(Group::new(vec![
                self.elseif.format(f),
                misc::print_condition(
                    f,
                    &self.condition,
                    f.settings.space_before_if_parenthesis,
                    f.settings.space_within_if_parenthesis,
                ),
                misc::print_clause(f, &self.statement, false),
            ]))
        })
    }
}

impl<'a> Format<'a> for IfColonDelimitedBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBody, {
            let mut parts = vec![Document::String(":")];

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

impl<'a> Format<'a> for IfColonDelimitedBodyElseIfClause {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBodyElseIfClause, {
            let mut parts = vec![self.elseif.format(f)];

            let condition = misc::print_condition(
                f,
                &self.condition,
                f.settings.space_before_if_parenthesis,
                f.settings.space_within_if_parenthesis,
            );
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
                    statements.insert(0, Document::Line(Line::hard()));
                    parts.push(Document::Indent(statements));
                }
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for IfColonDelimitedBodyElseClause {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBodyElseClause, {
            let mut parts = vec![self.r#else.format(f), Document::String(":")];

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

impl<'a> Format<'a> for DoWhile {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, DoWhile, {
            Document::Group(Group::new(vec![
                self.r#do.format(f),
                misc::print_clause(f, &self.statement, false),
                self.r#while.format(f),
                misc::print_condition(
                    f,
                    &self.condition,
                    f.settings.space_before_while_parenthesis,
                    f.settings.space_within_while_parenthesis,
                ),
                self.terminator.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for For {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, For, {
            let mut contents = vec![
                self.r#for.format(f),
                if f.settings.space_before_for_parenthesis { Document::space() } else { Document::empty() },
                Document::String("("),
                if f.settings.space_within_for_parenthesis { Document::space() } else { Document::empty() },
            ];

            let format_expressions = |f: &mut FormatterState<'a>, expressions: &'a [Expression]| {
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
                    Document::Line(Line::soft()),
                    format_expressions(f, self.initializations.as_slice()),
                    if f.settings.space_before_for_semicolon { Document::space() } else { Document::empty() },
                    Document::String(";"),
                    if self.conditions.is_empty() {
                        Document::empty()
                    } else if f.settings.space_after_for_semicolon {
                        Document::Line(Line::default())
                    } else {
                        Document::Line(Line::soft())
                    },
                    format_expressions(f, self.conditions.as_slice()),
                    if f.settings.space_before_for_semicolon { Document::space() } else { Document::empty() },
                    Document::String(";"),
                    if self.increments.is_empty() {
                        Document::empty()
                    } else if f.settings.space_after_for_semicolon {
                        Document::Line(Line::default())
                    } else {
                        Document::Line(Line::soft())
                    },
                    format_expressions(f, self.increments.as_slice()),
                ]),
                Document::Line(Line::soft()),
            ])));

            if f.settings.space_within_for_parenthesis {
                contents.push(Document::space());
            }

            contents.push(Document::String(")"));
            contents.push(self.body.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for ForColonDelimitedBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ForColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_for, &self.terminator)
        })
    }
}

impl<'a> Format<'a> for ForBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for Switch {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Switch, {
            Document::Array(vec![
                self.switch.format(f),
                misc::print_condition(
                    f,
                    &self.expression,
                    f.settings.space_before_switch_parenthesis,
                    f.settings.space_within_switch_parenthesis,
                ),
                self.body.format(f),
            ])
        })
    }
}

impl<'a> Format<'a> for SwitchBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, SwitchBody, {
            match self {
                SwitchBody::BraceDelimited(b) => Document::Array(vec![
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

impl<'a> Format<'a> for SwitchColonDelimitedBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, SwitchColonDelimitedBody, {
            let mut contents = vec![Document::String(":")];
            for case in self.cases.iter() {
                contents.push(Document::Indent(vec![Document::Line(Line::hard()), case.format(f)]));
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

impl<'a> Format<'a> for SwitchBraceDelimitedBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for SwitchCase {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, SwitchCase, {
            match self {
                SwitchCase::Expression(c) => c.format(f),
                SwitchCase::Default(c) => c.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for SwitchExpressionCase {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, SwitchExpressionCase, {
            let mut parts =
                vec![self.case.format(f), Document::space(), self.expression.format(f), self.separator.format(f)];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hard()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for SwitchDefaultCase {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, SwitchDefaultCase, {
            let mut parts = vec![self.default.format(f), self.separator.format(f)];
            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hard()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for SwitchCaseSeparator {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, SwitchCaseSeparator, {
            match self {
                SwitchCaseSeparator::Colon(_) => Document::String(":"),
                SwitchCaseSeparator::SemiColon(_) => Document::String(";"),
            }
        })
    }
}

impl<'a> Format<'a> for While {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, While, {
            Document::Array(vec![
                self.r#while.format(f),
                misc::print_condition(
                    f,
                    &self.condition,
                    f.settings.space_before_while_parenthesis,
                    f.settings.space_within_while_parenthesis,
                ),
                self.body.format(f),
            ])
        })
    }
}

impl<'a> Format<'a> for WhileBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, WhileBody, {
            match self {
                WhileBody::Statement(s) => misc::print_clause(f, s, false),
                WhileBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for WhileColonDelimitedBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, WhileColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_while, &self.terminator)
        })
    }
}

impl<'a> Format<'a> for Foreach {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Foreach, {
            Document::Array(vec![
                self.foreach.format(f),
                if f.settings.space_before_foreach_parenthesis { Document::space() } else { Document::empty() },
                Document::String("("),
                if f.settings.space_within_foreach_parenthesis { Document::space() } else { Document::empty() },
                self.expression.format(f),
                Document::space(),
                self.r#as.format(f),
                Document::space(),
                self.target.format(f),
                if f.settings.space_within_foreach_parenthesis { Document::space() } else { Document::empty() },
                Document::String(")"),
                self.body.format(f),
            ])
        })
    }
}

impl<'a> Format<'a> for ForeachTarget {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ForeachTarget, {
            match self {
                ForeachTarget::Value(t) => t.format(f),
                ForeachTarget::KeyValue(t) => t.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for ForeachValueTarget {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ForeachValueTarget, { self.value.format(f) })
    }
}

impl<'a> Format<'a> for ForeachKeyValueTarget {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for ForeachBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ForeachBody, {
            match self {
                ForeachBody::Statement(s) => misc::print_clause(f, s, false),
                ForeachBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for ForeachColonDelimitedBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ForeachColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_foreach, &self.terminator)
        })
    }
}

impl<'a> Format<'a> for Continue {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for Break {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
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
