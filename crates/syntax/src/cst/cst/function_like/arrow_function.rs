use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::cst::attribute::AttributeList;
use crate::cst::cst::expression::Expression;
use crate::cst::cst::function_like::parameter::FunctionLikeParameterList;
use crate::cst::cst::function_like::r#return::FunctionLikeReturnTypeHint;
use crate::cst::cst::keyword::Keyword;
use crate::cst::sequence::Sequence;

/// Represents an arrow function in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// $fn = fn($x) => $x * 2;
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArrowFunction<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub r#static: Option<Keyword<'arena>>,
    pub r#fn: Keyword<'arena>,
    pub ampersand: Option<Span>,
    pub parameter_list: FunctionLikeParameterList<'arena>,
    pub return_type_hint: Option<FunctionLikeReturnTypeHint<'arena>>,
    pub arrow: Span,
    pub expression: &'arena Expression<'arena>,
}

impl HasSpan for ArrowFunction<'_> {
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
