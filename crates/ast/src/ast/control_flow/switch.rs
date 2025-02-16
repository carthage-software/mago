use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::statement::Statement;
use crate::ast::terminator::Terminator;
use crate::sequence::Sequence;

/// Represents a `switch` statement in PHP.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Switch<'a> {
    pub switch: Keyword,
    pub left_parenthesis: Span,
    pub expression: Box<'a, Expression<'a>>,
    pub right_parenthesis: Span,
    pub body: SwitchBody<'a>,
}

/// Represents the body of a switch statement.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum SwitchBody<'a> {
    BraceDelimited(SwitchBraceDelimitedBody<'a>),
    ColonDelimited(SwitchColonDelimitedBody<'a>),
}

/// Represents a brace-delimited body of a switch statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct SwitchBraceDelimitedBody<'a> {
    pub left_brace: Span,
    pub optional_terminator: Option<Terminator>,
    pub cases: Sequence<'a, SwitchCase<'a>>,
    pub right_brace: Span,
}

/// Represents a colon-delimited body of a switch statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct SwitchColonDelimitedBody<'a> {
    pub colon: Span,
    pub optional_terminator: Option<Terminator>,
    pub cases: Sequence<'a, SwitchCase<'a>>,
    pub end_switch: Keyword,
    pub terminator: Terminator,
}

/// Represents a single case within a switch statement.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum SwitchCase<'a> {
    Expression(SwitchExpressionCase<'a>),
    Default(SwitchDefaultCase<'a>),
}

/// Represents a single case within a switch statement.
///
/// Example: `case 1: echo "One";`
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct SwitchExpressionCase<'a> {
    pub case: Keyword,
    pub expression: Box<'a, Expression<'a>>,
    pub separator: SwitchCaseSeparator,
    pub statements: Sequence<'a, Statement<'a>>,
}

/// Represents the default case within a switch statement.
///
/// Example: `default: echo "Default";`
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct SwitchDefaultCase<'a> {
    pub default: Keyword,
    pub separator: SwitchCaseSeparator,
    pub statements: Sequence<'a, Statement<'a>>,
}

/// Represents the separator between a case and its statements.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum SwitchCaseSeparator {
    Colon(Span),
    SemiColon(Span),
}

impl<'a> SwitchBody<'a> {
    pub fn cases(&self) -> &[SwitchCase<'a>] {
        match self {
            SwitchBody::BraceDelimited(body) => body.cases.as_slice(),
            SwitchBody::ColonDelimited(body) => body.cases.as_slice(),
        }
    }
}

impl<'a> SwitchCase<'a> {
    /// Returns the statements within the case.
    pub fn statements(&self) -> &[Statement<'a>] {
        match self {
            SwitchCase::Expression(case) => case.statements.as_slice(),
            SwitchCase::Default(case) => case.statements.as_slice(),
        }
    }

    /// Returns `true` if the case is a default case.
    pub fn is_default(&self) -> bool {
        match self {
            SwitchCase::Expression(_) => false,
            SwitchCase::Default(_) => true,
        }
    }

    /// Returns `true` if the case is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            SwitchCase::Expression(case) => case.statements.is_empty(),
            SwitchCase::Default(case) => case.statements.is_empty(),
        }
    }

    /// Returns the case is fall-through.
    ///
    /// A case is considered fall-through if it is not empty and
    /// does not end with a `break` statement.
    pub fn is_fall_through(&self) -> bool {
        let Some(last_statement) = self.statements().last() else {
            return false;
        };

        !matches!(last_statement, Statement::Break(_))
    }
}

impl HasSpan for Switch<'_> {
    fn span(&self) -> Span {
        Span::between(self.switch.span(), self.body.span())
    }
}

impl HasSpan for SwitchBody<'_> {
    fn span(&self) -> Span {
        match self {
            SwitchBody::BraceDelimited(body) => body.span(),
            SwitchBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for SwitchBraceDelimitedBody<'_> {
    fn span(&self) -> Span {
        Span::between(self.left_brace, self.right_brace)
    }
}

impl HasSpan for SwitchColonDelimitedBody<'_> {
    fn span(&self) -> Span {
        Span::between(self.colon, self.terminator.span())
    }
}

impl HasSpan for SwitchCase<'_> {
    fn span(&self) -> Span {
        match self {
            SwitchCase::Expression(case) => case.span(),
            SwitchCase::Default(case) => case.span(),
        }
    }
}

impl HasSpan for SwitchExpressionCase<'_> {
    fn span(&self) -> Span {
        Span::between(
            self.case.span(),
            self.statements.last().map(|statement| statement.span()).unwrap_or(self.separator.span()),
        )
    }
}

impl HasSpan for SwitchDefaultCase<'_> {
    fn span(&self) -> Span {
        Span::between(
            self.default.span(),
            self.statements.last().map(|statement| statement.span()).unwrap_or(self.separator.span()),
        )
    }
}

impl HasSpan for SwitchCaseSeparator {
    fn span(&self) -> Span {
        match self {
            SwitchCaseSeparator::Colon(span) => *span,
            SwitchCaseSeparator::SemiColon(span) => *span,
        }
    }
}
