use fennec_ast::*;
use fennec_span::HasSpan;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::format::delimited;
use crate::format::delimited::Delimiter;
use crate::format::misc;
use crate::format::misc::print_colon_delimited_body;
use crate::format::misc::print_token_with_indented_leading_comments;
use crate::format::sequence::TokenSeparatedSequenceFormatter;
use crate::format::statement::print_statement_sequence;
use crate::format::Format;
use crate::format::IfBreak;
use crate::settings::*;
use crate::wrap;
use crate::Formatter;

impl<'a> Format<'a> for If {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for IfBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfBody, {
            match &self {
                IfBody::Statement(b) => b.format(f),
                IfBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for IfStatementBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfStatementBodyElseClause, {
            Document::Group(Group::new(vec![self.r#else.format(f), misc::print_clause(f, &self.statement, false)]))
        })
    }
}

impl<'a> Format<'a> for IfStatementBodyElseIfClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for IfColonDelimitedBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBody, {
            let mut parts = vec![Document::String(":")];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hardline()));

                parts.push(Document::Indent(statements));
            }

            parts.push(Document::Line(Line::hardline()));
            for else_if_clause in self.else_if_clauses.iter() {
                parts.push(else_if_clause.format(f));
                parts.push(Document::Line(Line::hardline()));
            }

            if let Some(else_clause) = &self.else_clause {
                parts.push(else_clause.format(f));
                parts.push(Document::Line(Line::hardline()));
            }

            parts.push(self.endif.format(f));
            parts.push(self.terminator.format(f));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for IfColonDelimitedBodyElseClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBodyElseClause, {
            let mut parts = vec![self.r#else.format(f), Document::String(":")];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hardline()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for IfColonDelimitedBodyElseIfClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBodyElseIfClause, {
            let mut parts = vec![
                self.elseif.format(f),
                Document::space(),
                misc::print_condition(f, &self.condition),
                Document::String(":"),
            ];

            let mut statements = print_statement_sequence(f, &self.statements);
            if !statements.is_empty() {
                statements.insert(0, Document::Line(Line::hardline()));

                parts.push(Document::Indent(statements));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for DoWhile {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for For {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, For, {
            Document::Group(Group::new(vec![
                self.r#for.format(f),
                Document::space(),
                {
                    let delimiter = Delimiter::Parentheses(self.left_parenthesis, self.right_parenthesis);
                    let formatter = |f: &mut Formatter<'a>| {
                        let initializations = TokenSeparatedSequenceFormatter::new(",")
                            .with_trailing_separator(false)
                            .format(f, &self.initializations);
                        let initializations_semicolon = Document::String(";");

                        let conditions = TokenSeparatedSequenceFormatter::new(",")
                            .with_trailing_separator(false)
                            .format(f, &self.conditions);
                        let conditions_semicolon = Document::String(";");

                        let increments = TokenSeparatedSequenceFormatter::new(",")
                            .with_trailing_separator(false)
                            .format(f, &self.increments);

                        let is_empty =
                            self.initializations.is_empty() && self.conditions.is_empty() && self.increments.is_empty();

                        let document = Document::Group(Group::new(vec![
                            initializations,
                            initializations_semicolon,
                            Document::IfBreak(IfBreak::new(Document::Line(Line::hardline()), Document::space())),
                            conditions,
                            conditions_semicolon,
                            Document::IfBreak(IfBreak::new(Document::Line(Line::hardline()), Document::space())),
                            increments,
                        ]));

                        (document, is_empty)
                    };

                    delimited::format_delimited_group(f, delimiter, formatter, false)
                },
                self.body.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for ForColonDelimitedBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ForColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_for, &self.terminator)
        })
    }
}

impl<'a> Format<'a> for ForBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for SwitchBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for SwitchColonDelimitedBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, SwitchColonDelimitedBody, {
            let mut contents = vec![Document::String(":")];
            for case in self.cases.iter() {
                contents.push(Document::Indent(vec![Document::Line(Line::hardline()), case.format(f)]));
            }

            contents.push(Document::Line(Line::hardline()));
            contents.push(self.end_switch.format(f));
            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for SwitchBraceDelimitedBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, SwitchBraceDelimitedBody, {
            let mut contents = vec![Document::String("{")];
            for case in self.cases.iter() {
                contents.push(Document::Indent(vec![Document::Line(Line::hardline()), case.format(f)]));
            }

            contents.push(print_token_with_indented_leading_comments(f, self.right_brace, "}", true));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for SwitchCase {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, SwitchCase, {
            match self {
                SwitchCase::Expression(c) => c.format(f),
                SwitchCase::Default(c) => c.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for SwitchExpressionCase {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for SwitchDefaultCase {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for SwitchCaseSeparator {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, SwitchCaseSeparator, {
            match self {
                SwitchCaseSeparator::Colon(_) => Document::String(":"),
                SwitchCaseSeparator::SemiColon(_) => Document::String(";"),
            }
        })
    }
}

impl<'a> Format<'a> for While {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for WhileBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, WhileBody, {
            match self {
                WhileBody::Statement(s) => misc::print_clause(f, s, false),
                WhileBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for WhileColonDelimitedBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, WhileColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_while, &self.terminator)
        })
    }
}

impl<'a> Format<'a> for Foreach {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

impl<'a> Format<'a> for ForeachTarget {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ForeachTarget, {
            match self {
                ForeachTarget::Value(t) => t.format(f),
                ForeachTarget::KeyValue(t) => t.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for ForeachValueTarget {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ForeachValueTarget, { self.value.format(f) })
    }
}

impl<'a> Format<'a> for ForeachKeyValueTarget {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ForeachBody, {
            match self {
                ForeachBody::Statement(s) => misc::print_clause(f, s, false),
                ForeachBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for ForeachColonDelimitedBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ForeachColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_foreach, &self.terminator)
        })
    }
}

impl<'a> Format<'a> for Continue {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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