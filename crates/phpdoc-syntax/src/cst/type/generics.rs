use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax_core::cst::Sequence;

use crate::cst::keyword::Keyword;
use crate::cst::r#type::Type;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum GenericParameterVariance<'arena> {
    Covariant(Keyword<'arena>),
    Contravariant(Keyword<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GenericParameterEntry<'arena> {
    pub variance: Option<GenericParameterVariance<'arena>>,
    pub inner: Type<'arena>,
    pub comma: Option<Span>,
}

impl<'arena> GenericParameterVariance<'arena> {
    #[must_use]
    pub fn keyword(&self) -> &Keyword<'arena> {
        match self {
            GenericParameterVariance::Covariant(keyword) | GenericParameterVariance::Contravariant(keyword) => keyword,
        }
    }
}

impl HasSpan for GenericParameterVariance<'_> {
    fn span(&self) -> Span {
        self.keyword().span()
    }
}

impl std::fmt::Display for GenericParameterVariance<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericParameterVariance::Covariant(_) => f.write_str("covariant"),
            GenericParameterVariance::Contravariant(_) => f.write_str("contravariant"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GenericParameters<'arena> {
    pub less_than: Span,
    pub entries: Sequence<'arena, GenericParameterEntry<'arena>>,
    pub greater_than: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SingleGenericParameter<'arena> {
    pub less_than: Span,
    pub entry: &'arena GenericParameterEntry<'arena>,
    pub greater_than: Span,
}

impl HasSpan for GenericParameterEntry<'_> {
    fn span(&self) -> Span {
        let start = match &self.variance {
            Some(variance) => variance.span(),
            None => self.inner.span(),
        };

        match &self.comma {
            Some(comma) => start.join(*comma),
            None => start.join(self.inner.span()),
        }
    }
}

impl HasSpan for GenericParameters<'_> {
    fn span(&self) -> Span {
        self.less_than.join(self.greater_than)
    }
}

impl HasSpan for SingleGenericParameter<'_> {
    fn span(&self) -> Span {
        self.less_than.join(self.greater_than)
    }
}

impl std::fmt::Display for GenericParameterEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(variance) = &self.variance {
            write!(f, "{variance} ")?;
        }

        write!(f, "{}", self.inner)
    }
}

impl std::fmt::Display for SingleGenericParameter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.entry)
    }
}

impl std::fmt::Display for GenericParameters<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<")?;
        for (i, entry) in self.entries.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{entry}")?;
        }
        write!(f, ">")
    }
}
