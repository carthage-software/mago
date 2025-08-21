use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::attribute::AttributeList;
use crate::ast::ast::block::Block;
use crate::ast::ast::function_like::parameter::FunctionLikeParameterList;
use crate::ast::ast::function_like::r#return::FunctionLikeReturnTypeHint;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::sequence::Sequence;

/// Represents a `function` declaration in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// function foo(): string {
///    return 'bar';
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Function<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub function: Keyword<'arena>,
    pub ampersand: Option<Span>,
    pub name: LocalIdentifier<'arena>,
    pub parameter_list: FunctionLikeParameterList<'arena>,
    pub return_type_hint: Option<FunctionLikeReturnTypeHint<'arena>>,
    pub body: Block<'arena>,
}

impl HasSpan for Function<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.body.span());
        }

        self.function.span().join(self.body.span())
    }
}
