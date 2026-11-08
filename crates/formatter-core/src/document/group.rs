use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;
use core::ops::AddAssign;
use std::cell::RefCell;

use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::document::Document;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GroupIdentifier(pub usize);

#[derive(Debug)]
pub struct GroupIdentifierBuilder {
    id: GroupIdentifier,
}

impl Default for GroupIdentifierBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupIdentifierBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { id: GroupIdentifier(0) }
    }

    #[inline]
    pub fn next_id(&mut self) -> GroupIdentifier {
        self.id += 1;
        self.id
    }
}

impl AddAssign<usize> for GroupIdentifier {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Display for GroupIdentifier {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}

/// The break mode of a `Group`, determining when it breaks and whether
/// the break propagates to parent groups via `will_break()`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum BreakMode {
    /// Break only if the group doesn't fit on a single line.
    Auto,
    /// Always break. Propagates to parent groups.
    Force,
    /// Always break, but does not propagate to parent groups.
    Preserve,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Group<'arena, A>
where
    A: Arena,
{
    pub contents: Vec<'arena, Document<'arena, A>, A>,
    pub break_mode: RefCell<BreakMode>,
    pub expanded_states: Option<Vec<'arena, Document<'arena, A>, A>>,
    pub id: Option<GroupIdentifier>,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct IndentIfBreak<'arena, A>
where
    A: Arena,
{
    pub group_id: GroupIdentifier,
    pub contents: Vec<'arena, Document<'arena, A>, A>,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Copy, Clone)]
pub struct IfBreak<'arena, A>
where
    A: Arena,
{
    pub break_contents: &'arena Document<'arena, A>,
    pub flat_content: &'arena Document<'arena, A>,
    pub group_id: Option<GroupIdentifier>,
}

impl<'arena, A> Group<'arena, A>
where
    A: Arena,
{
    #[must_use]
    pub fn new(contents: Vec<'arena, Document<'arena, A>, A>) -> Self {
        Self { contents, break_mode: RefCell::new(BreakMode::Auto), id: None, expanded_states: None }
    }

    #[must_use]
    pub fn conditional(
        contents: Vec<'arena, Document<'arena, A>, A>,
        expanded_states: Vec<'arena, Document<'arena, A>, A>,
    ) -> Self {
        Self { contents, break_mode: RefCell::new(BreakMode::Auto), id: None, expanded_states: Some(expanded_states) }
    }

    #[must_use]
    pub fn with_break_mode(mut self, mode: BreakMode) -> Self {
        self.break_mode = RefCell::new(mode);
        self
    }

    #[must_use]
    pub fn with_id(mut self, id: GroupIdentifier) -> Self {
        self.id = Some(id);
        self
    }
}

impl<'arena, A> IndentIfBreak<'arena, A>
where
    A: Arena,
{
    #[must_use]
    pub fn new(group_id: GroupIdentifier, contents: Vec<'arena, Document<'arena, A>, A>) -> Self {
        Self { group_id, contents }
    }
}

impl<'arena, A> IfBreak<'arena, A>
where
    A: Arena,
{
    pub fn new(arena: &'arena A, break_contents: Document<'arena, A>, flat_content: Document<'arena, A>) -> Self {
        Self { break_contents: arena.alloc(break_contents), flat_content: arena.alloc(flat_content), group_id: None }
    }

    pub fn then(arena: &'arena A, break_contents: Document<'arena, A>) -> Self {
        Self {
            break_contents: arena.alloc(break_contents),
            flat_content: arena.alloc(Document::empty()),
            group_id: None,
        }
    }

    #[must_use]
    pub fn with_id(mut self, id: GroupIdentifier) -> Self {
        self.group_id = Some(id);
        self
    }
}
