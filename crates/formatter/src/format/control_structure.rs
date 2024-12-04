use fennec_ast::*;
use fennec_span::HasSpan;

use crate::array;
use crate::default_line;
use crate::document::Document;
use crate::empty_string;
use crate::format::delimited;
use crate::format::delimited::Delimiter;
use crate::format::misc;
use crate::format::misc::print_colon_delimited_body;
use crate::format::misc::print_token_with_indented_leading_comments;
use crate::format::sequence::TokenSeparatedSequenceFormatter;
use crate::format::statement::print_statement_sequence;
use crate::format::Format;
use crate::format::Group;
use crate::format::IfBreak;
use crate::format::Line;
use crate::group;
use crate::hardline;
use crate::indent;
use crate::settings::*;
use crate::space;
use crate::token;
use crate::wrap;
use crate::Formatter;

impl<'a> Format<'a> for If {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, If, {
            group!(self.r#if.format(f), space!(), misc::print_condition(f, &self.condition), self.body.format(f))
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

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for IfStatementBodyElseClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfStatementBodyElseClause, {
            group!(self.r#else.format(f), misc::print_clause(f, &self.statement, false))
        })
    }
}

impl<'a> Format<'a> for IfStatementBodyElseIfClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfStatementBodyElseIfClause, {
            group!(
                self.elseif.format(f),
                space!(),
                misc::print_condition(f, &self.condition),
                misc::print_clause(f, &self.statement, false),
            )
        })
    }
}

impl<'a> Format<'a> for IfColonDelimitedBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBody, {
            let mut parts = vec![token!(f, self.colon, ":")];

            let statements = print_statement_sequence(f, &self.statements);
            let has_statements = !statements.is_empty();
            if has_statements {
                parts.push(indent!(@hardline!()));
            }

            for stmt in statements {
                parts.push(indent!(stmt));
            }

            parts.extend(hardline!());

            for else_if_clause in self.else_if_clauses.iter() {
                parts.push(else_if_clause.format(f));
                parts.extend(hardline!());
            }

            if let Some(else_clause) = &self.else_clause {
                parts.push(else_clause.format(f));
                parts.extend(hardline!());
            }

            parts.push(self.endif.format(f));
            parts.push(self.terminator.format(f));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for IfColonDelimitedBodyElseClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBodyElseClause, {
            let mut parts = vec![self.r#else.format(f), token!(f, self.colon, ":")];

            let statements = print_statement_sequence(f, &self.statements);
            let has_statements = !statements.is_empty();
            if has_statements {
                parts.push(indent!(@hardline!()));
            }

            for stmt in statements {
                parts.push(indent!(stmt));
            }

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for IfColonDelimitedBodyElseIfClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IfColonDelimitedBodyElseIfClause, {
            let mut parts = vec![
                self.elseif.format(f),
                space!(),
                misc::print_condition(f, &self.condition),
                token!(f, self.colon, ":"),
            ];

            let statements = print_statement_sequence(f, &self.statements);
            let has_statements = !statements.is_empty();
            if has_statements {
                parts.push(indent!(@hardline!()));
            }

            for stmt in statements {
                parts.push(indent!(stmt));
            }

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for DoWhile {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, DoWhile, {
            group!(
                self.r#do.format(f),
                misc::print_clause(f, &self.statement, false),
                self.r#while.format(f),
                space!(),
                misc::print_condition(f, &self.condition),
                self.terminator.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for For {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, For, {
            group!(
                self.r#for.format(f),
                space!(),
                {
                    let delimiter = Delimiter::Parentheses(self.left_parenthesis, self.right_parenthesis);
                    let formatter = |f: &mut Formatter<'a>| {
                        let initializations = TokenSeparatedSequenceFormatter::new(",")
                            .with_trailing_separator(false)
                            .format(f, &self.initializations);
                        let initializations_semicolon = token!(f, self.initializations_semicolon, ";");

                        let conditions = TokenSeparatedSequenceFormatter::new(",")
                            .with_trailing_separator(false)
                            .format(f, &self.conditions);
                        let conditions_semicolon = token!(f, self.conditions_semicolon, ";");

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
                self.body.format(f)
            )
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

                    misc::adjust_clause(f, &s, stmt, false)
                }
                ForBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for Switch {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Switch, {
            let mut parts = vec![];

            parts.push(self.switch.format(f));
            parts.push(space!());
            parts.push(token!(f, self.left_parenthesis, "("));
            parts.push(self.expression.format(f));
            parts.push(token!(f, self.right_parenthesis, ")"));

            parts.push(self.body.format(f));

            array!(@parts)
        })
    }
}

impl<'a> Format<'a> for SwitchBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, SwitchBody, {
            match self {
                SwitchBody::BraceDelimited(b) => array!(
                    match f.settings.control_brace_style {
                        BraceStyle::SameLine => {
                            space!()
                        }
                        BraceStyle::NextLine => {
                            default_line!()
                        }
                    },
                    b.format(f)
                ),
                SwitchBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for SwitchColonDelimitedBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, SwitchColonDelimitedBody, {
            let mut parts = vec![token!(f, self.colon, ":")];
            for case in self.cases.iter() {
                parts.push(indent!(default_line!(), case.format(f)));
            }

            parts.push(default_line!());
            parts.push(self.end_switch.format(f));
            parts.push(self.terminator.format(f));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for SwitchBraceDelimitedBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, SwitchBraceDelimitedBody, {
            let mut parts = vec![token!(f, self.left_brace, "{")];

            for case in self.cases.iter() {
                parts.push(indent!(default_line!(), case.format(f)));
            }

            parts.push(print_token_with_indented_leading_comments(f, self.right_brace, "}", true));

            group!(@parts)
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
            let mut parts = vec![self.case.format(f), space!(), self.expression.format(f), self.separator.format(f)];
            let statements = print_statement_sequence(f, &self.statements);
            let has_statements = !statements.is_empty();
            if has_statements {
                parts.push(indent!(@hardline!()));
            }

            for stmt in statements {
                parts.push(indent!(stmt));
            }

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for SwitchDefaultCase {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, SwitchDefaultCase, {
            let mut parts = vec![self.default.format(f), self.separator.format(f)];
            let statements = print_statement_sequence(f, &self.statements);
            let has_statements = !statements.is_empty();
            if has_statements {
                parts.push(indent!(@hardline!()));
            }

            for stmt in statements {
                parts.push(indent!(stmt));
            }

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for SwitchCaseSeparator {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, SwitchCaseSeparator, {
            match self {
                SwitchCaseSeparator::Colon(span) => token!(f, *span, ":"),
                SwitchCaseSeparator::SemiColon(span) => token!(f, *span, ";"),
            }
        })
    }
}

impl<'a> Format<'a> for While {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, While, {
            let mut parts = vec![];

            parts.push(self.r#while.format(f));
            parts.push(space!());
            parts.push(token!(f, self.left_parenthesis, "("));
            parts.push(self.condition.format(f));
            parts.push(token!(f, self.right_parenthesis, ")"));

            parts.push(self.body.format(f));

            array!(@parts)
        })
    }
}

impl<'a> Format<'a> for WhileBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, WhileBody, {
            match self {
                WhileBody::Statement(s) => misc::print_clause(f, &s, false),
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
            let mut parts = vec![];

            parts.push(self.foreach.format(f));
            parts.push(space!());
            parts.push(token!(f, self.left_parenthesis, "("));
            parts.push(self.expression.format(f));
            parts.push(space!());
            parts.push(self.r#as.format(f));
            parts.push(space!());
            parts.push(self.target.format(f));
            parts.push(token!(f, self.right_parenthesis, ")"));

            parts.push(self.body.format(f));

            array!(@parts)
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
            group!(self.key.format(f), space!(), token!(f, self.double_arrow, "=>"), space!(), self.value.format(f))
        })
    }
}

impl<'a> Format<'a> for ForeachBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ForeachBody, {
            match self {
                ForeachBody::Statement(s) => misc::print_clause(f, &s, false),
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
            group!(
                self.r#continue.format(f),
                if let Some(level) = &self.level { array!(space!(), level.format(f)) } else { empty_string!() },
                self.terminator.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for Break {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Break, {
            group!(
                self.r#break.format(f),
                if let Some(level) = &self.level { array!(space!(), level.format(f)) } else { empty_string!() },
                self.terminator.format(f)
            )
        })
    }
}
