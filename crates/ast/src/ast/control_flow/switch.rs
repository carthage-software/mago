use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_span::HasSpan;
use fennec_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::statement::Statement;
use crate::ast::terminator::Terminator;
use crate::sequence::Sequence;

/// Represents a `switch` statement in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Switch {
    pub switch: Keyword,
    pub left_parenthesis: Span,
    pub expression: Expression,
    pub right_parenthesis: Span,
    pub body: SwitchBody,
}

/// Represents the body of a switch statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
pub enum SwitchBody {
    BraceDelimited(SwitchBraceDelimitedBody),
    ColonDelimited(SwitchColonDelimitedBody),
}

/// Represents a brace-delimited body of a switch statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SwitchBraceDelimitedBody {
    pub left_brace: Span,
    pub optional_terminator: Option<Terminator>,
    pub cases: Sequence<SwitchCase>,
    pub right_brace: Span,
}

/// Represents a colon-delimited body of a switch statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SwitchColonDelimitedBody {
    pub colon: Span,
    pub optional_terminator: Option<Terminator>,
    pub cases: Sequence<SwitchCase>,
    pub end_switch: Keyword,
    pub terminator: Terminator,
}

/// Represents a single case within a switch statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
pub enum SwitchCase {
    Expression(SwitchExpressionCase),
    Default(SwitchDefaultCase),
}

/// Represents a single case within a switch statement.
///
/// Example: `case 1: echo "One";`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SwitchExpressionCase {
    pub case: Keyword,
    pub expression: Expression,
    pub separator: SwitchCaseSeparator,
    pub statements: Sequence<Statement>,
}

/// Represents the default case within a switch statement.
///
/// Example: `default: echo "Default";`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SwitchDefaultCase {
    pub default: Keyword,
    pub separator: SwitchCaseSeparator,
    pub statements: Sequence<Statement>,
}

/// Represents the separator between a case and its statements.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
pub enum SwitchCaseSeparator {
    Colon(Span),
    SemiColon(Span),
}

impl SwitchBody {
    pub fn cases(&self) -> &[SwitchCase] {
        match self {
            SwitchBody::BraceDelimited(body) => body.cases.as_slice(),
            SwitchBody::ColonDelimited(body) => body.cases.as_slice(),
        }
    }
}

impl SwitchCase {
    /// Returns the statements within the case.
    pub fn statements(&self) -> &[Statement] {
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

impl HasSpan for Switch {
    fn span(&self) -> Span {
        Span::between(self.switch.span(), self.body.span())
    }
}

impl HasSpan for SwitchBody {
    fn span(&self) -> Span {
        match self {
            SwitchBody::BraceDelimited(body) => body.span(),
            SwitchBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for SwitchBraceDelimitedBody {
    fn span(&self) -> Span {
        Span::between(self.left_brace, self.right_brace)
    }
}

impl HasSpan for SwitchColonDelimitedBody {
    fn span(&self) -> Span {
        Span::between(self.colon, self.terminator.span())
    }
}

impl HasSpan for SwitchCase {
    fn span(&self) -> Span {
        match self {
            SwitchCase::Expression(case) => case.span(),
            SwitchCase::Default(case) => case.span(),
        }
    }
}

impl HasSpan for SwitchExpressionCase {
    fn span(&self) -> Span {
        Span::between(
            self.case.span(),
            self.statements.last().map(|statement| statement.span()).unwrap_or(self.separator.span()),
        )
    }
}

impl HasSpan for SwitchDefaultCase {
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
