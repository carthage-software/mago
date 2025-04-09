use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::attribute::AttributeList;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::function_like::parameter::FunctionLikeParameterList;
use crate::ast::ast::function_like::r#return::FunctionLikeReturnTypeHint;
use crate::ast::ast::keyword::Keyword;
use crate::ast::sequence::Sequence;

/// Represents an arrow function in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// $fn = fn($x) => $x * 2;
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ArrowFunction {
    pub attribute_lists: Sequence<AttributeList>,
    pub r#static: Option<Keyword>,
    pub r#fn: Keyword,
    pub ampersand: Option<Span>,
    pub parameter_list: FunctionLikeParameterList,
    pub return_type_hint: Option<FunctionLikeReturnTypeHint>,
    pub arrow: Span,
    pub expression: Box<Expression>,
}

impl HasSpan for ArrowFunction {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.expression.span());
        }

        if let Some(r#static) = &self.r#static {
            return r#static.span().join(self.expression.span());
        }

        self.r#fn.span().join(self.expression.span())
    }
}
