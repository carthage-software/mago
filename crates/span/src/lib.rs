use std::ops::Range;

use fennec_source::HasSource;
use serde::Deserialize;
use serde::Serialize;

use fennec_source::SourceIdentifier;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Position {
    pub source: SourceIdentifier,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

pub trait HasPosition {
    fn position(&self) -> Position;

    fn offset(&self) -> usize {
        self.position().offset
    }
}

pub trait HasSpan {
    fn span(&self) -> Span;

    fn start_position(&self) -> Position {
        self.span().start
    }

    fn end_position(&self) -> Position {
        self.span().end
    }
}

impl Position {
    pub fn new(source: SourceIdentifier, offset: usize) -> Self {
        Self { source, offset }
    }

    pub fn dummy(offset: usize) -> Self {
        Self::new(SourceIdentifier::dummy(), offset)
    }

    pub fn start_of(source: SourceIdentifier) -> Self {
        Self::new(source, 0)
    }

    /// Return the position moved by the given offset.
    ///
    /// # Parameters
    ///
    /// - `offset`: The offset to move the position by.
    ///
    /// # Returns
    ///
    /// The position moved by the given offset.
    pub fn forward(&self, offset: usize) -> Self {
        Self { source: self.source, offset: self.offset + offset }
    }

    /// Return the position moved back by the given offset.
    ///
    /// # Parameters
    ///
    /// - `offset`: The offset to move the position back by.
    ///
    /// # Returns
    ///
    /// The position moved back by the given offset.
    pub fn backward(&self, offset: usize) -> Self {
        Self { source: self.source, offset: self.offset - offset }
    }

    pub fn range_for(&self, length: usize) -> Range<usize> {
        self.offset..self.offset + length
    }
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        debug_assert!(start.source == end.source, "span start and end must be in the same file");

        Self { start, end }
    }

    pub fn between(start: Span, end: Span) -> Self {
        start.join(end)
    }

    pub fn join(self, other: Span) -> Span {
        Span::new(self.start, other.end)
    }

    pub fn contains(&self, position: &impl HasPosition) -> bool {
        self.has_offset(position.offset())
    }

    pub fn has_offset(&self, offset: usize) -> bool {
        self.start.offset <= offset && offset <= self.end.offset
    }

    pub fn to_range(&self) -> Range<usize> {
        self.start.offset..self.end.offset
    }

    pub fn length(&self) -> usize {
        self.end.offset - self.start.offset
    }

    pub fn subspan(&self, start: usize, end: usize) -> Span {
        Span::new(self.start.forward(start), self.start.forward(end))
    }
}

impl HasPosition for Position {
    fn position(&self) -> Position {
        *self
    }
}

impl HasSource for Position {
    fn source(&self) -> SourceIdentifier {
        self.source
    }
}

impl HasSpan for Span {
    fn span(&self) -> Span {
        *self
    }
}

impl<T: HasSpan> HasPosition for T {
    fn position(&self) -> Position {
        self.start_position()
    }
}

impl HasSource for Span {
    fn source(&self) -> SourceIdentifier {
        self.start.source
    }
}

impl HasSource for dyn HasPosition {
    fn source(&self) -> SourceIdentifier {
        self.position().source()
    }
}

impl HasSource for dyn HasSpan {
    fn source(&self) -> SourceIdentifier {
        self.span().source()
    }
}

impl<T: HasSpan> HasSpan for Box<T> {
    fn span(&self) -> Span {
        self.as_ref().span()
    }
}

impl Into<usize> for Position {
    fn into(self) -> usize {
        self.offset
    }
}

impl Into<usize> for &Position {
    fn into(self) -> usize {
        self.offset
    }
}

impl Into<Range<usize>> for Span {
    fn into(self) -> Range<usize> {
        Range { start: self.start.into(), end: self.end.into() }
    }
}
impl Into<Range<usize>> for &Span {
    fn into(self) -> Range<usize> {
        Range { start: self.start.into(), end: self.end.into() }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.offset)
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl Into<SourceIdentifier> for Position {
    fn into(self) -> SourceIdentifier {
        self.source
    }
}

impl Into<SourceIdentifier> for &Position {
    fn into(self) -> SourceIdentifier {
        self.source
    }
}

impl Into<SourceIdentifier> for Span {
    fn into(self) -> SourceIdentifier {
        self.start.source
    }
}

impl Into<SourceIdentifier> for &Span {
    fn into(self) -> SourceIdentifier {
        self.start.source
    }
}
