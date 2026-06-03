#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ResolutionKind {
    Default,
    Function,
    Constant,
}

impl ResolutionKind {
    #[inline]
    #[must_use]
    pub const fn is_case_sensitive(self) -> bool {
        matches!(self, ResolutionKind::Constant)
    }
}
