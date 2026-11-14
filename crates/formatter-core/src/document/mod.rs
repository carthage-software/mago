use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_allocator::vec_in;

mod cloning;
pub mod group;

pub use cloning::{clone_in_arena, clone_vec_in_arena};
use group::GroupIdentifier;
pub use group::{BreakMode, Group, IfBreak, IndentIfBreak};

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Document<'arena, A>
where
    A: Arena,
{
    String(&'arena [u8]),
    /// Zero-width bounds used to return a selected source region.
    RangeStart,
    RangeEnd,
    Array(Vec<'arena, Document<'arena, A>, A>),
    /// Increase the level of indentation.
    Indent(Vec<'arena, Document<'arena, A>, A>),
    IndentIfBreak(IndentIfBreak<'arena, A>),
    /// Mark a group of items which the printer should try to fit on one line.
    /// This is the basic command to tell the printer when to break.
    /// Groups are usually nested, and the printer will try to fit everything on one line,
    /// but if it doesn't fit it will break the outermost group first and try again.
    /// It will continue breaking groups until everything fits (or there are no more groups to break).
    Group(Group<'arena, A>),
    /// Specify a line break.
    /// If an expression fits on one line, the line break will be replaced with a space.
    /// Line breaks always indent the next line with the current level of indentation.
    Line(Line),
    /// This is used to implement trailing comments.
    /// It's not practical to constantly check where the line ends to avoid accidentally printing some code at the end of a comment.
    /// `lineSuffix` buffers docs passed to it and flushes them before any new line.
    LineSuffix(Vec<'arena, Document<'arena, A>, A>),
    LineSuffixBoundary,
    /// Choose content based on whether the current group breaks.
    IfBreak(IfBreak<'arena, A>),
    /// Force parent groups to break.
    BreakParent,
    Align(Align<'arena, A>),
    /// Trim trailing whitespace, optionally including line breaks.
    Trim(Trim),
    /// Do not perform any trimming before printing the next document.
    DoNotTrim,
    Space(Space),
    Column(Column),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Column {
    pub table: GroupIdentifier,
    /// `0` starts a row; higher values are the alignment points of that row, left to right.
    /// [`Column::ANCHOR`] is never padded: it only has to stay on the line of its row.
    pub index: u8,
    /// Width of the text between this point and the position to align.
    pub shift: u8,
    /// Extra width to keep between the widest row and the aligned position.
    pub gutter: u8,
    /// Whether a row only aligns with the row on the line right above it.
    pub adjacent: bool,
}

impl Column {
    pub const ANCHOR: u8 = u8::MAX;
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Align<'arena, A>
where
    A: Arena,
{
    pub alignment: &'arena [u8],
    pub contents: Vec<'arena, Document<'arena, A>, A>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Trim {
    /// Trims trailing whitespace characters (spaces and tabs) from the end of the document.
    Whitespace,
    /// Removes all newline characters from the end of the document.
    Newlines,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Line {
    pub hard: bool,
    pub soft: bool,
    pub literal: bool,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Space {
    /// If `true`, the space is "soft" and will only be printed if the preceding
    /// character is not whitespace. If `false`, the space is "hard" and will always be printed.
    pub soft: bool,
}

#[derive(Clone, Copy)]
pub enum Separator {
    HardLine,
    LiteralLine,
    CommaLine,     // [",", line]
    CommaHardLine, // [",", hardline]
    CommaSpace,    // [",", " "]
    Space,
}

impl Line {
    /// Specify a line break.
    /// The difference from line is that if the expression fits on one line, it will be replaced with nothing.
    #[must_use]
    pub fn soft() -> Self {
        Self { soft: true, ..Self::default() }
    }

    /// Specify a line break that is **always** included in the output,
    /// no matter if the expression fits on one line or not.
    #[must_use]
    pub fn hard() -> Self {
        Self { hard: true, ..Self::default() }
    }

    #[must_use]
    pub fn literal() -> Self {
        Self { hard: true, literal: true, ..Default::default() }
    }
}

impl Space {
    /// Specify a space that is "soft" and will only be printed if the preceding
    /// character is not whitespace or a line break.
    #[must_use]
    pub fn soft() -> Self {
        Self { soft: true }
    }
}

impl<'arena, A> Document<'arena, A>
where
    A: Arena,
{
    #[inline]
    #[must_use]
    pub fn empty() -> Document<'arena, A> {
        Document::String(b"")
    }

    #[inline]
    #[must_use]
    pub fn space() -> Document<'arena, A> {
        Document::Space(Space { soft: false })
    }

    #[inline]
    #[must_use]
    pub fn soft_space() -> Document<'arena, A> {
        Document::Space(Space { soft: true })
    }

    pub fn can_break(&self) -> bool {
        self.any(|doc| matches!(doc, Document::Line(_)))
    }

    pub fn any<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Document<'arena, A>) -> bool,
    {
        if predicate(self) {
            return true;
        }

        match self {
            Document::Array(docs) | Document::LineSuffix(docs) | Document::Indent(docs) => docs.iter().any(predicate),
            Document::IndentIfBreak(IndentIfBreak { contents, .. }) | Document::Group(Group { contents, .. }) => {
                contents.iter().any(predicate)
            }
            Document::IfBreak(IfBreak { break_contents, flat_content, .. }) => {
                predicate(break_contents) || predicate(flat_content)
            }
            _ => false,
        }
    }

    pub fn join(
        arena: &'arena A,
        documents: impl IntoIterator<Item = Document<'arena, A>>,
        separator: Separator,
    ) -> Vec<'arena, Document<'arena, A>, A> {
        let documents = documents.into_iter();
        let capacity = documents.size_hint().0.saturating_mul(2).saturating_sub(1);
        let mut parts = Vec::with_capacity_in(capacity, arena);
        for (i, document) in documents.enumerate() {
            if i != 0 {
                parts.push(match separator {
                    Separator::Space => Document::String(b" "),
                    Separator::HardLine => Document::Line(Line::hard()),
                    Separator::CommaSpace => Document::String(b", "),
                    Separator::LiteralLine => {
                        Document::Array(vec_in![arena; Document::Line(Line::literal()), Document::BreakParent])
                    }
                    Separator::CommaHardLine => {
                        Document::Array(vec_in![arena; Document::String(b","), Document::Line(Line::hard())])
                    }
                    Separator::CommaLine => {
                        Document::Array(vec_in![arena; Document::String(b","), Document::Line(Line::default())])
                    }
                });
            }

            parts.push(document);
        }

        parts
    }
}
