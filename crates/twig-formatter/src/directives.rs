use std::ops::Range;

use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_twig_syntax::cst::Template;
use mago_twig_syntax::cst::TriviaKind;

use crate::collect::TagCollection;

pub(super) struct Directives<'arena, A: Arena> {
    pub ignores_file: bool,
    next: Vec<'arena, u32, A>,
    ranges: Vec<'arena, Range<u32>, A>,
}

impl<'arena, A: Arena> Directives<'arena, A> {
    pub fn collect(arena: &'arena A, template: &Template<'_>) -> Self {
        let mut directives = Self { ignores_file: false, next: Vec::new_in(arena), ranges: Vec::new_in(arena) };
        let mut start = None;
        for trivia in template.trivia.iter().filter(|trivia| trivia.kind.is_comment()) {
            match read(trivia.value) {
                Some(Directive::File) => {
                    directives.ignores_file = true;
                    return directives;
                }
                _ if trivia.kind != TriviaKind::Comment => {}
                Some(Directive::Next) => directives.next.push(trivia.span.end.offset),
                Some(Directive::Start) => {
                    start.get_or_insert(trivia.span.start.offset);
                }
                Some(Directive::End) => {
                    if let Some(start) = start.take() {
                        directives.ranges.push(start..trivia.span.end.offset);
                    }
                }
                None => {}
            }
        }

        if let Some(start) = start {
            directives.ranges.push(start..template.source_text.len() as u32);
        }

        directives
    }

    pub fn needs_statements(&self) -> bool {
        !self.next.is_empty()
    }

    pub fn resolve(mut self, collected: &TagCollection<'arena, A>) -> IgnoredRanges<'arena, A> {
        for offset in &self.next {
            let statement = collected.statements.partition_point(|span| span.start.offset < *offset);
            let tag = collected.tags.partition_point(|tag| tag.span().start.offset < *offset);
            if let Some(span) = collected.statements.get(statement)
                && collected.tags.get(tag).is_some_and(|tag| tag.span().start == span.start)
            {
                self.ranges.push(span.start.offset..span.end.offset);
            }
        }

        self.ranges.sort_unstable_by_key(|range| range.start);
        self.ranges.dedup_by(|next, previous| {
            if next.start > previous.end {
                return false;
            }

            previous.end = previous.end.max(next.end);
            true
        });

        IgnoredRanges { ranges: self.ranges, next: 0 }
    }
}

pub(super) struct IgnoredRanges<'arena, A: Arena> {
    ranges: Vec<'arena, Range<u32>, A>,
    next: usize,
}

impl<A: Arena> IgnoredRanges<'_, A> {
    pub fn advance_to(&mut self, offset: u32) -> bool {
        while self.ranges.get(self.next).is_some_and(|range| range.end <= offset) {
            self.next += 1;
        }

        self.ranges.get(self.next).is_some_and(|range| range.contains(&offset))
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum Directive {
    File,
    Start,
    End,
    Next,
}

fn read(comment: &[u8]) -> Option<Directive> {
    memchr::memmem::find_iter(comment, b"@mago-format").filter_map(|offset| parse(&comment[offset..])).min()
}

fn parse(candidate: &[u8]) -> Option<Directive> {
    let suffix =
        candidate.strip_prefix(b"@mago-formatter-ignore").or_else(|| candidate.strip_prefix(b"@mago-format-ignore"))?;

    [
        (b"".as_slice(), Directive::File),
        (b"-start".as_slice(), Directive::Start),
        (b"-end".as_slice(), Directive::End),
        (b"-next".as_slice(), Directive::Next),
    ]
    .into_iter()
    .find_map(|(name, directive)| {
        let rest = suffix.strip_prefix(name)?;
        rest.first()
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && !matches!(byte, b'-' | b'_'))
            .then_some(directive)
    })
}
