use bitflags::bitflags;

use mago_database::file::File;
use mago_syntax::ast::Trivia;
use mago_syntax::ast::TriviaKind;

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
    pub start: u32,
    pub end: u32,
    pub is_block: bool,
    pub is_shell_comment: bool,
    pub is_single_line: bool,
    pub has_line_suffix: bool,
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
        if flags.contains(CommentFlags::Block) && !self.is_block {
            return false;
        }

        if flags.contains(CommentFlags::Line) && self.is_block {
            return false;
        }

        true
    }

    pub fn is_inline_comment(&self) -> bool {
        !self.is_block || self.is_single_line
    }
}
