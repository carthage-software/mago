use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::text::Text;
use crate::cst::r#type::Type;
use crate::cst::variable::Variable;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub variable: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

impl HasSpan for PropertyTagValue<'_> {
    fn span(&self) -> Span {
        let end = self.description.as_ref().map_or_else(|| self.variable.span(), HasSpan::span);

        self.r#type.span().join(end)
    }
}
