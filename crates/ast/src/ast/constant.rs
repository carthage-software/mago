use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::attribute::AttributeList;
use crate::ast::expression::Expression;
use crate::ast::identifier::LocalIdentifier;
use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;
use crate::sequence::Sequence;
use crate::sequence::TokenSeparatedSequence;

/// Represents a constant statement in PHP.
///
/// Example: `const FOO = 1;` or `const BAR = 2, QUX = 3, BAZ = 4;`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Constant {
    pub attribute_lists: Sequence<AttributeList>,
    pub r#const: Keyword,
    pub items: TokenSeparatedSequence<ConstantItem>,
    pub terminator: Terminator,
}

/// Represents a single name-value pair within a constant statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ConstantItem {
    pub name: LocalIdentifier,
    pub equals: Span,
    pub value: Expression,
}

impl HasSpan for Constant {
    fn span(&self) -> Span {
        self.r#const.span().join(self.terminator.span())
    }
}

impl HasSpan for ConstantItem {
    fn span(&self) -> Span {
        self.name.span().join(self.value.span())
    }
}
