use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::statement::Statement;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::Sequence;

/// Represents a `switch` statement in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Switch<'arena> {
    pub switch: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub expression: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
    pub body: SwitchBody<'arena>,
}

/// Represents the body of a switch statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum SwitchBody<'arena> {
    BraceDelimited(SwitchBraceDelimitedBody<'arena>),
    ColonDelimited(SwitchColonDelimitedBody<'arena>),
}

/// Represents a brace-delimited body of a switch statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SwitchBraceDelimitedBody<'arena> {
    pub left_brace: Span,
    pub optional_terminator: Option<Terminator<'arena>>,
    pub cases: Sequence<'arena, SwitchCase<'arena>>,
    pub right_brace: Span,
}

/// Represents a colon-delimited body of a switch statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SwitchColonDelimitedBody<'arena> {
    pub colon: Span,
    pub optional_terminator: Option<Terminator<'arena>>,
    pub cases: Sequence<'arena, SwitchCase<'arena>>,
    pub end_switch: Keyword<'arena>,
    pub terminator: Terminator<'arena>,
}

/// Represents a single case within a switch statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum SwitchCase<'arena> {
    Expression(SwitchExpressionCase<'arena>),
    Default(SwitchDefaultCase<'arena>),
}

/// Represents a single case within a switch statement.
///
/// Example: `case 1: echo "One";`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SwitchExpressionCase<'arena> {
    pub case: Keyword<'arena>,
    pub expression: &'arena Expression<'arena>,
    pub separator: SwitchCaseSeparator,
    pub statements: Sequence<'arena, Statement<'arena>>,
}

/// Represents the default case within a switch statement.
///
/// Example: `default: echo "Default";`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SwitchDefaultCase<'arena> {
    pub default: Keyword<'arena>,
    pub separator: SwitchCaseSeparator,
    pub statements: Sequence<'arena, Statement<'arena>>,
}

/// Represents the separator between a case and its statements.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum SwitchCaseSeparator {
    Colon(Span),
    SemiColon(Span),
}

impl<'arena> SwitchBody<'arena> {
    pub fn has_default_case(&self) -> bool {
        self.cases().iter().any(SwitchCase::is_default)
    }

    pub fn cases(&self) -> &[SwitchCase<'arena>] {
        match self {
            SwitchBody::BraceDelimited(body) => body.cases.as_slice(),
            SwitchBody::ColonDelimited(body) => body.cases.as_slice(),
        }
    }
}

impl<'arena> SwitchCase<'arena> {
    /// Returns the case expression if it exists.
    pub fn expression(&self) -> Option<&Expression<'arena>> {
        match self {
            SwitchCase::Expression(case) => Some(case.expression),
            SwitchCase::Default(_) => None,
        }
    }

    /// Returns the statements within the case.
    pub fn statements(&self) -> &[Statement<'arena>] {
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
