use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ArgumentList;
use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum TestArguments<'arena> {
    None,
    Parenthesised(ArgumentList<'arena>),
    Bare(&'arena Expression<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Test<'arena> {
    pub operand: &'arena Expression<'arena>,
    pub is_keyword: Keyword<'arena>,
    pub not_keyword: Option<Keyword<'arena>>,
    pub name: Identifier<'arena>,
    pub second_word: Option<Keyword<'arena>>,
    pub arguments: TestArguments<'arena>,
}

impl HasSpan for Test<'_> {
    fn span(&self) -> Span {
        let end = match &self.arguments {
            TestArguments::Parenthesised(list) => list.right_parenthesis,
            TestArguments::Bare(e) => e.span(),
            TestArguments::None => self.second_word.map(|k| k.span).unwrap_or(self.name.span),
        };
        self.operand.span().join(end)
    }
}
