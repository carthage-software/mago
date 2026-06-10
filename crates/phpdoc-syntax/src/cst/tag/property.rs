use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::text::Text;
use crate::cst::r#type::Type;
use crate::cst::variable::Variable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PropertyTagValue<'arena> {
    pub r#type: Option<&'arena Type<'arena>>,
    pub variable: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

impl HasSpan for PropertyTagValue<'_> {
    fn span(&self) -> Span {
        let start = self.r#type.map_or_else(|| self.variable.span(), |r#type| r#type.span());
        let end = self.description.as_ref().map_or_else(|| self.variable.span(), HasSpan::span);

        start.join(end)
    }
}
