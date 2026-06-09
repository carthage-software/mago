use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum WildcardKind {
    Asterisk,
    Underscore,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct WildcardType {
    pub span: Span,
    pub kind: WildcardKind,
}

impl HasSpan for WildcardType {
    fn span(&self) -> Span {
        self.span
    }
}

impl std::fmt::Display for WildcardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            WildcardKind::Asterisk => write!(f, "*"),
            WildcardKind::Underscore => write!(f, "_"),
        }
    }
}
