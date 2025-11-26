//! Provides fundamental types for source code location tracking.
//!
//! This crate defines the core primitives [`Position`] and [`Span`] used throughout
//! mago to identify specific locations in source files. It also provides
//! the generic traits [`HasPosition`] and [`HasSpan`] to abstract over any syntax
//! tree node or token that has a location.

use std::ops::Range;

use serde::Deserialize;
use serde::Serialize;

use mago_database::file::FileId;
use mago_database::file::HasFileId;

/// Represents a specific byte offset within a single source file.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Position {
    pub offset: u32,
}

/// Represents a contiguous range of source code within a single file.
///
/// A `Span` is defined by a `start` and `end` [`Position`], marking the beginning
/// (inclusive) and end (exclusive) of a source code segment.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Span {
    /// The unique identifier of the file this span belongs to.
    pub file_id: FileId,
    /// The start position is inclusive, meaning it includes the byte at this position.
    pub start: Position,
    /// The end position is exclusive, meaning it does not include the byte at this position.
    pub end: Position,
}

/// A trait for types that have a single, defined source position.
pub trait HasPosition {
    /// Returns the source position.
    fn position(&self) -> Position;

    /// A convenience method to get the byte offset of the position.
    #[inline]
    fn offset(&self) -> u32 {
        self.position().offset
    }
}

/// A trait for types that cover a span of source code.
pub trait HasSpan {
    /// Returns the source span.
    fn span(&self) -> Span;

    /// A convenience method to get the starting position of the span.
    fn start_position(&self) -> Position {
        self.span().start
    }

    /// A convenience method to get the starting byte offset of the span.
    fn start_offset(&self) -> u32 {
        self.start_position().offset
    }

    /// A convenience method to get the ending position of the span.
    fn end_position(&self) -> Position {
        self.span().end
    }

    /// A convenience method to get the ending byte offset of the span.
    fn end_offset(&self) -> u32 {
        self.end_position().offset
    }
}

impl Position {
    /// Creates a new `Position` from a byte offset.
    pub const fn new(offset: u32) -> Self {
        Self { offset }
    }

    /// Creates a new `Position` with an offset of zero.
    pub const fn zero() -> Self {
        Self { offset: 0 }
    }

    /// Checks if this position is at the start of a file.
    pub const fn is_zero(&self) -> bool {
        self.offset == 0
    }

    /// Returns a new position moved forward by the given offset.
    ///
    /// Uses saturating arithmetic to prevent overflow.
    pub const fn forward(&self, offset: u32) -> Self {
        Self { offset: self.offset.saturating_add(offset) }
    }

    /// Returns a new position moved backward by the given offset.
    ///
    /// Uses saturating arithmetic to prevent underflow.
    pub const fn backward(&self, offset: u32) -> Self {
        Self { offset: self.offset.saturating_sub(offset) }
    }

    /// Creates a `Range<u32>` starting at this position's offset with a given length.
    pub const fn range_for(&self, length: u32) -> Range<u32> {
        self.offset..self.offset.saturating_add(length)
    }
}

impl Span {
    /// Creates a new `Span` from a start and end position.
    ///
    /// # Panics
    ///
    /// In debug builds, this will panic if the start and end positions are not
    /// from the same file (unless one is a dummy position).
    pub const fn new(file_id: FileId, start: Position, end: Position) -> Self {
        Self { file_id, start, end }
    }

    /// Creates a new `Span` with a zero-length, starting and ending at the same position.
    pub const fn zero() -> Self {
        Self { file_id: FileId::zero(), start: Position::zero(), end: Position::zero() }
    }

    /// Creates a "dummy" span with a null file ID.
    pub fn dummy(start_offset: u32, end_offset: u32) -> Self {
        Self::new(FileId::zero(), Position::new(start_offset), Position::new(end_offset))
    }

    /// Creates a new span that starts at the beginning of the first span
    /// and ends at the conclusion of the second span.
    pub fn between(start: Span, end: Span) -> Self {
        start.join(end)
    }

    /// Checks if this span is a zero-length span, meaning it starts and ends at the same position.
    pub const fn is_zero(&self) -> bool {
        self.start.is_zero() && self.end.is_zero()
    }

    /// Creates a new span that encompasses both `self` and `other`.
    /// The new span starts at `self.start` and ends at `other.end`.
    pub fn join(self, other: Span) -> Span {
        Span::new(self.file_id, self.start, other.end)
    }

    /// Creates a new span that starts at the beginning of this span
    /// and ends at the specified position.
    pub fn to_end(&self, end: Position) -> Span {
        Span::new(self.file_id, self.start, end)
    }

    /// Creates a new span that starts at the specified position
    /// and ends at the end of this span.
    pub fn from_start(&self, start: Position) -> Span {
        Span::new(self.file_id, start, self.end)
    }

    /// Creates a new span that is a subspan of this span, defined by the given byte offsets.
    /// The `start` and `end` parameters are relative to the start of this span.
    pub fn subspan(&self, start: u32, end: u32) -> Span {
        Span::new(self.file_id, self.start.forward(start), self.start.forward(end))
    }

    /// Checks if a position is contained within this span's byte offsets.
    pub fn contains(&self, position: &impl HasPosition) -> bool {
        self.has_offset(position.offset())
    }

    /// Checks if a raw byte offset is contained within this span.
    pub fn has_offset(&self, offset: u32) -> bool {
        self.start.offset <= offset && offset <= self.end.offset
    }

    /// Converts the span to a `Range<u32>` of its byte offsets.
    pub fn to_range(&self) -> Range<u32> {
        self.start.offset..self.end.offset
    }

    /// Converts the span to a `Range<usize>` of its byte offsets.
    pub fn to_range_usize(&self) -> Range<usize> {
        let start = self.start.offset as usize;
        let end = self.end.offset as usize;

        start..end
    }

    /// Converts the span to a tuple of byte offsets.
    pub fn to_offset_tuple(&self) -> (u32, u32) {
        (self.start.offset, self.end.offset)
    }

    /// Returns the length of the span in bytes.
    pub fn length(&self) -> u32 {
        self.end.offset.saturating_sub(self.start.offset)
    }

    pub fn is_before(&self, other: impl HasPosition) -> bool {
        self.end.offset <= other.position().offset
    }

    pub fn is_after(&self, other: impl HasPosition) -> bool {
        self.start.offset >= other.position().offset
    }
}

impl HasPosition for Position {
    fn position(&self) -> Position {
        *self
    }
}

impl HasSpan for Span {
    fn span(&self) -> Span {
        *self
    }
}

/// A blanket implementation that allows any `HasSpan` type to also be treated
/// as a `HasPosition` type, using the span's start as its position.
impl<T: HasSpan> HasPosition for T {
    fn position(&self) -> Position {
        self.start_position()
    }
}

impl HasFileId for Span {
    fn file_id(&self) -> FileId {
        self.file_id
    }
}

/// Ergonomic blanket impl for references.
impl<T: HasSpan> HasSpan for &T {
    fn span(&self) -> Span {
        (*self).span()
    }
}

/// Ergonomic blanket impl for boxed values.
impl<T: HasSpan> HasSpan for Box<T> {
    fn span(&self) -> Span {
        self.as_ref().span()
    }
}

impl From<Span> for Range<u32> {
    fn from(span: Span) -> Range<u32> {
        span.to_range()
    }
}

impl From<&Span> for Range<u32> {
    fn from(span: &Span) -> Range<u32> {
        span.to_range()
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Range<usize> {
        let start = span.start.offset as usize;
        let end = span.end.offset as usize;

        start..end
    }
}

impl From<&Span> for Range<usize> {
    fn from(span: &Span) -> Range<usize> {
        let start = span.start.offset as usize;
        let end = span.end.offset as usize;

        start..end
    }
}

impl From<Position> for u32 {
    fn from(position: Position) -> u32 {
        position.offset
    }
}

impl From<&Position> for u32 {
    fn from(position: &Position) -> u32 {
        position.offset
    }
}

impl From<u32> for Position {
    fn from(offset: u32) -> Self {
        Position { offset }
    }
}

impl std::ops::Add<u32> for Position {
    type Output = Position;

    fn add(self, rhs: u32) -> Self::Output {
        self.forward(rhs)
    }
}

impl std::ops::Sub<u32> for Position {
    type Output = Position;

    fn sub(self, rhs: u32) -> Self::Output {
        self.backward(rhs)
    }
}

impl std::ops::AddAssign<u32> for Position {
    fn add_assign(&mut self, rhs: u32) {
        self.offset = self.offset.saturating_add(rhs);
    }
}

impl std::ops::SubAssign<u32> for Position {
    /// Moves the position backward in-place.
    fn sub_assign(&mut self, rhs: u32) {
        self.offset = self.offset.saturating_sub(rhs);
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.offset)
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start.offset, self.end.offset)
    }
}
