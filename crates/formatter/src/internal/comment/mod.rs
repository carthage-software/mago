use mago_database::file::File;
use mago_syntax::ast::Trivia;
use mago_syntax::ast::TriviaKind;

pub mod format;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CommentFlags(u8);

impl CommentFlags {
    pub const LEADING: CommentFlags = CommentFlags(1 << 0); // Check comment is a leading comment
    pub const TRAILING: CommentFlags = CommentFlags(1 << 1); // Check comment is a trailing comment
    pub const DANGLING: CommentFlags = CommentFlags(1 << 2); // Check comment is a dangling comment
    pub const BLOCK: CommentFlags = CommentFlags(1 << 3); // Check comment is a block comment
    pub const LINE: CommentFlags = CommentFlags(1 << 4); // Check comment is a line comment
    pub const FIRST: CommentFlags = CommentFlags(1 << 5); // Check comment is the first attached comment
    pub const LAST: CommentFlags = CommentFlags(1 << 6); // Check comment is the last attached comment
}

#[derive(Debug, Clone, Copy)]
pub struct Comment {
    pub start: u32,
    pub end: u32,
    pub is_block: bool,
    pub is_shell_comment: bool,
    pub is_single_line: bool,
    pub has_line_suffix: bool,
}

impl CommentFlags {
    #[inline]
    #[must_use]
    pub const fn all() -> Self {
        CommentFlags(
            Self::LEADING.0
                | Self::TRAILING.0
                | Self::DANGLING.0
                | Self::BLOCK.0
                | Self::LINE.0
                | Self::FIRST.0
                | Self::LAST.0,
        )
    }

    #[inline]
    pub const fn contains(self, other: CommentFlags) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl Comment {
    pub fn new(start: u32, end: u32, is_block: bool, is_shell_comment: bool, is_single_line: bool) -> Self {
        Self { start, end, is_block, is_shell_comment, is_single_line, has_line_suffix: false }
    }

    pub fn from_trivia<'arena>(file: &File, trivia: &'arena Trivia<'arena>) -> Self {
        debug_assert!(trivia.kind.is_comment());

        let is_block = trivia.kind.is_block_comment();
        let is_single_line =
            !is_block || (file.line_number(trivia.span.start.offset) == file.line_number(trivia.span.end.offset));
        let is_shell_comment = matches!(trivia.kind, TriviaKind::HashComment);

        Self::new(trivia.span.start.offset, trivia.span.end.offset, is_block, is_shell_comment, is_single_line)
    }

    pub fn with_line_suffix(mut self, yes: bool) -> Self {
        self.has_line_suffix = yes;
        self
    }

    pub fn matches_flags(self, flags: CommentFlags) -> bool {
        if flags.contains(CommentFlags::BLOCK) && !self.is_block {
            return false;
        }

        if flags.contains(CommentFlags::LINE) && self.is_block {
            return false;
        }

        true
    }

    pub fn is_inline_comment(&self) -> bool {
        !self.is_block || self.is_single_line
    }
}

impl std::ops::BitOr for CommentFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        CommentFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for CommentFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        CommentFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for CommentFlags {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        CommentFlags(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for CommentFlags {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        CommentFlags(!self.0)
    }
}

impl std::ops::BitOrAssign for CommentFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAndAssign for CommentFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitXorAssign for CommentFlags {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
