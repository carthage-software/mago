use bitflags::bitflags;

use fennec_ast::Trivia;

pub mod format;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct CommentFlags: u8 {
        const Leading        = 1 << 0; // Check comment is a leading comment
        const Trailing       = 1 << 1; // Check comment is a trailing comment
        const Dangling       = 1 << 2; // Check comment is a dangling comment
        const Block          = 1 << 3; // Check comment is a block comment
        const Line           = 1 << 4; // Check comment is a line comment
        const First          = 1 << 5; // Check comment is the first attached comment
        const Last           = 1 << 6; // Check comment is the last attached comment
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Comment {
    pub start: usize,
    pub end: usize,
    pub is_block: bool,
    pub has_line_suffix: bool,
}

impl Comment {
    pub fn new(start: usize, end: usize, is_block: bool) -> Self {
        Self { start, end, is_block, has_line_suffix: false }
    }

    pub fn from_trivia(trivia: &Trivia) -> Self {
        debug_assert!(trivia.kind.is_comment());

        Self::new(trivia.span.start.offset, trivia.span.end.offset, trivia.kind.is_block_comment())
    }

    pub fn with_line_suffix(mut self, yes: bool) -> Self {
        self.has_line_suffix = yes;
        self
    }

    pub fn matches_flags(self, flags: CommentFlags) -> bool {
        if flags.contains(CommentFlags::Block) && !self.is_block {
            return false;
        }

        if flags.contains(CommentFlags::Line) && self.is_block {
            return false;
        }

        true
    }
}
