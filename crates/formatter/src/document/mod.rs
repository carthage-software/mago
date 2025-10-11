use std::cell::RefCell;

use bumpalo::Bump;
use bumpalo::collections::Vec;
use bumpalo::vec;

use crate::document::group::GroupIdentifier;

pub mod group;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Document<'arena> {
    String(&'arena str),
    Array(Vec<'arena, Document<'arena>>),
    /// Increase the level of indentation.
    Indent(Vec<'arena, Document<'arena>>),
    IndentIfBreak(IndentIfBreak<'arena>),
    /// Mark a group of items which the printer should try to fit on one line.
    /// This is the basic command to tell the printer when to break.
    /// Groups are usually nested, and the printer will try to fit everything on one line,
    /// but if it doesn't fit it will break the outermost group first and try again.
    /// It will continue breaking groups until everything fits (or there are no more groups to break).
    Group(Group<'arena>),
    /// Specify a line break.
    /// If an expression fits on one line, the line break will be replaced with a space.
    /// Line breaks always indent the next line with the current level of indentation.
    Line(Line),
    /// This is used to implement trailing comments.
    /// It's not practical to constantly check where the line ends to avoid accidentally printing some code at the end of a comment.
    /// `lineSuffix` buffers docs passed to it and flushes them before any new line.
    LineSuffix(Vec<'arena, Document<'arena>>),
    LineSuffixBoundary,
    /// Print something if the current `group` or the current element of `fill` breaks and something else if it doesn't.
    IfBreak(IfBreak<'arena>),
    /// This is an alternative type of group which behaves like text layout:
    /// it's going to add a break whenever the next element doesn't fit in the line anymore.
    /// The difference with `group` is that it's not going to break all the separators, just the ones that are at the end of lines.
    Fill(Fill<'arena>),
    /// Include this anywhere to force all parent groups to break.
    BreakParent,
    Align(Align<'arena>),
    /// Trim all newlines from the end of the document.
    Trim(Trim),
    /// Do not perform any trimming before printing the next document.
    DoNotTrim,
    Space(Space),
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Align<'arena> {
    pub alignment: &'arena str,
    pub contents: Vec<'arena, Document<'arena>>,
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

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Group<'arena> {
    pub contents: Vec<'arena, Document<'arena>>,
    pub should_break: RefCell<bool>,
    pub expanded_states: Option<Vec<'arena, Document<'arena>>>,
    pub id: Option<GroupIdentifier>,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct IndentIfBreak<'arena> {
    pub group_id: GroupIdentifier,
    pub contents: Vec<'arena, Document<'arena>>,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Fill<'arena> {
    pub parts: Vec<'arena, Document<'arena>>,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfBreak<'arena> {
    pub break_contents: &'arena Document<'arena>,
    pub flat_content: &'arena Document<'arena>,
    pub group_id: Option<GroupIdentifier>,
}

#[derive(Clone, Copy)]
pub enum Separator {
    #[allow(unused)]
    SoftLine,
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
    pub fn soft() -> Self {
        Self { soft: true, ..Self::default() }
    }

    /// Specify a line break that is **always** included in the output,
    /// no matter if the expression fits on one line or not.
    pub fn hard() -> Self {
        Self { hard: true, ..Self::default() }
    }

    pub fn literal() -> Self {
        Self { hard: true, literal: true, ..Default::default() }
    }
}

impl Space {
    /// Specify a space that is "soft" and will only be printed if the preceding
    /// character is not whitespace or a line break.
    pub fn soft() -> Self {
        Self { soft: true }
    }

    /// Specify a space that is "hard" and will always be printed.
    pub fn hard() -> Self {
        Self { soft: false }
    }
}

impl<'arena> Group<'arena> {
    pub fn new(contents: Vec<'arena, Document<'arena>>) -> Self {
        Self { contents, should_break: RefCell::new(false), id: None, expanded_states: None }
    }

    pub fn conditional(
        contents: Vec<'arena, Document<'arena>>,
        expanded_states: Vec<'arena, Document<'arena>>,
    ) -> Self {
        Self { contents, should_break: RefCell::new(false), id: None, expanded_states: Some(expanded_states) }
    }

    pub fn with_break(mut self, yes: bool) -> Self {
        self.should_break = RefCell::new(yes);
        self
    }

    pub fn with_id(mut self, id: GroupIdentifier) -> Self {
        self.id = Some(id);
        self
    }
}

impl<'arena> IndentIfBreak<'arena> {
    pub fn new(group_id: GroupIdentifier, contents: Vec<'arena, Document<'arena>>) -> Self {
        Self { contents, group_id }
    }
}

impl<'arena> Fill<'arena> {
    pub fn drain_out_pair(&mut self) -> (Option<Document<'arena>>, Option<Document<'arena>>) {
        let content = if !self.parts.is_empty() { Some(self.parts.remove(0)) } else { None };
        let whitespace = if !self.parts.is_empty() { Some(self.parts.remove(0)) } else { None };

        (content, whitespace)
    }

    pub fn dequeue(&mut self) -> Option<Document<'arena>> {
        if !self.parts.is_empty() { Some(self.parts.remove(0)) } else { None }
    }

    pub fn enqueue(&mut self, doc: Document<'arena>) {
        self.parts.insert(0, doc);
    }

    pub fn parts(&self) -> &[Document<'arena>] {
        &self.parts
    }
}

impl<'arena> IfBreak<'arena> {
    pub fn new(arena: &'arena Bump, break_contents: Document<'arena>, flat_content: Document<'arena>) -> Self {
        Self { break_contents: arena.alloc(break_contents), flat_content: arena.alloc(flat_content), group_id: None }
    }

    pub fn then(arena: &'arena Bump, break_contents: Document<'arena>) -> Self {
        Self {
            break_contents: arena.alloc(break_contents),
            flat_content: arena.alloc(Document::empty()),
            group_id: None,
        }
    }

    pub fn with_id(mut self, id: GroupIdentifier) -> Self {
        self.group_id = Some(id);
        self
    }
}

impl<'arena> Document<'arena> {
    #[inline]
    pub fn empty() -> Document<'arena> {
        Document::String("")
    }

    #[inline]
    pub fn space() -> Document<'arena> {
        Document::Space(Space { soft: false })
    }

    #[inline]
    pub fn soft_space() -> Document<'arena> {
        Document::Space(Space { soft: true })
    }

    pub fn can_break(&self) -> bool {
        self.any(|doc| matches!(doc, Document::Line(_)))
    }

    pub fn any<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Document<'arena>) -> bool,
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
            Document::Fill(fill) => fill.parts.iter().any(predicate),
            _ => false,
        }
    }

    pub fn join(
        arena: &'arena Bump,
        documents: impl IntoIterator<Item = Document<'arena>>,
        separator: Separator,
    ) -> Vec<'arena, Document<'arena>> {
        let mut parts = vec![in arena];
        for (i, document) in documents.into_iter().enumerate() {
            if i != 0 {
                parts.push(match separator {
                    Separator::Space => Document::String(" "),
                    Separator::SoftLine => Document::Line(Line::soft()),
                    Separator::HardLine => Document::Line(Line::hard()),
                    Separator::CommaSpace => Document::String(", "),
                    Separator::LiteralLine => {
                        Document::Array(vec![in arena; Document::Line(Line::literal()), Document::BreakParent])
                    }
                    Separator::CommaHardLine => {
                        Document::Array(vec![in arena; Document::String(","), Document::Line(Line::hard())])
                    }
                    Separator::CommaLine => {
                        Document::Array(vec![in arena; Document::String(","), Document::Line(Line::default())])
                    }
                });
            }

            parts.push(document);
        }

        parts
    }
}

/// Recursively clones a `Vec` of `Document`s into the given arena.
pub fn clone_vec_in_arena<'arena>(
    arena: &'arena Bump,
    source_vec: &Vec<'arena, Document<'arena>>,
) -> Vec<'arena, Document<'arena>> {
    let mut new_vec = Vec::with_capacity_in(source_vec.len(), arena);
    new_vec.extend(source_vec.iter().map(|d| clone_in_arena(arena, d)));
    new_vec
}

/// Recursively creates a deep clone of a `Document` within the given arena.
///
/// This function is necessary because `bumpalo::Box` does not implement `Clone`.
/// It manually reconstructs the entire `Document` tree, allocating new `Vec`s and
/// `Box`es in the provided `arena` to create a fully independent copy.
pub fn clone_in_arena<'arena>(arena: &'arena Bump, document: &Document<'arena>) -> Document<'arena> {
    match document {
        // Trivial `Copy` cases
        Document::String(s) => Document::String(s),
        Document::Line(line) => Document::Line(*line),
        Document::Trim(trim) => Document::Trim(*trim),
        Document::Space(space) => Document::Space(*space),
        Document::LineSuffixBoundary => Document::LineSuffixBoundary,
        Document::BreakParent => Document::BreakParent,
        Document::DoNotTrim => Document::DoNotTrim,

        // Variants containing a `Vec`
        Document::Array(docs) => Document::Array(clone_vec_in_arena(arena, docs)),
        Document::Indent(docs) => Document::Indent(clone_vec_in_arena(arena, docs)),
        Document::LineSuffix(docs) => Document::LineSuffix(clone_vec_in_arena(arena, docs)),

        // Variants containing structs that hold collections
        Document::IndentIfBreak(s) => Document::IndentIfBreak(IndentIfBreak {
            contents: clone_vec_in_arena(arena, &s.contents),
            group_id: s.group_id,
        }),
        Document::Group(g) => Document::Group(Group {
            contents: clone_vec_in_arena(arena, &g.contents),
            should_break: g.should_break.clone(),
            expanded_states: g.expanded_states.as_ref().map(|v| clone_vec_in_arena(arena, v)),
            id: g.id,
        }),
        Document::Fill(f) => Document::Fill(Fill { parts: clone_vec_in_arena(arena, &f.parts) }),
        Document::Align(a) => {
            Document::Align(Align { alignment: a.alignment, contents: clone_vec_in_arena(arena, &a.contents) })
        }

        // The special case for `Box`
        Document::IfBreak(ib) => Document::IfBreak(IfBreak {
            break_contents: arena.alloc(clone_in_arena(arena, ib.break_contents)),
            flat_content: arena.alloc(clone_in_arena(arena, ib.flat_content)),
            group_id: ib.group_id,
        }),
    }
}

#[allow(dead_code)]
#[cfg(debug_assertions)]
pub(crate) fn print_document_to_string<'arena>(arena: &'arena Bump, document: &Document<'arena>) -> String {
    use bumpalo::collections::CollectIn;

    fn write_documents_vec_to_buffer<'arena>(
        buffer: &mut String,
        arena: &'arena Bump,
        docs: &Vec<'arena, Document<'arena>>,
    ) {
        let mut printed = docs.iter().map(|d| print_document_to_string(arena, d)).collect_in::<Vec<_>>(arena);

        let length = printed.len();
        if length != 1 {
            buffer.push('[');
        }

        for (i, doc) in printed.iter_mut().enumerate() {
            if i != 0 && i < length {
                buffer.push_str(", ");
            }

            buffer.push_str(doc);
        }

        if length != 1 {
            buffer.push(']');
        }
    }

    let mut buffer = String::new();

    match document {
        Document::String(s) => {
            buffer.push_str(&format!("{s:?}"));
        }
        Document::Array(docs) => {
            write_documents_vec_to_buffer(&mut buffer, arena, docs);
        }
        Document::Indent(docs) => {
            buffer.push_str("indent(");
            write_documents_vec_to_buffer(&mut buffer, arena, docs);
            buffer.push(')');
        }
        Document::IndentIfBreak(IndentIfBreak { contents, group_id }) => {
            buffer.push_str("indentIfBreak(");
            write_documents_vec_to_buffer(&mut buffer, arena, contents);
            buffer.push_str(", { groupId: ");
            buffer.push_str(&format!("{group_id}"));
            buffer.push('}');
            buffer.push(')');
        }
        Document::Group(Group { contents, should_break, expanded_states, id }) => {
            let mut options = vec![in arena];
            if *should_break.borrow() {
                options.push("shouldBreak: true".to_string());
            }

            if let Some(id) = id {
                options.push(format!("id: {id}"));
            }

            let expanded_states_str = if let Some(states) = expanded_states {
                format!(
                    "conditionalGroup([{}]",
                    states.iter().map(|s| print_document_to_string(arena, s)).collect_in::<Vec<_>>(arena).join(", ")
                )
            } else {
                String::new()
            };

            let options_str =
                if options.is_empty() { String::new() } else { format!(", {{ {} }}", options.join(", ")) };

            if expanded_states_str.is_empty() {
                buffer.push_str("group(");
                write_documents_vec_to_buffer(&mut buffer, arena, contents);
                buffer.push_str(&options_str);
                buffer.push(')');
            } else {
                buffer.push_str(&expanded_states_str);
                buffer.push_str(", ");
                write_documents_vec_to_buffer(&mut buffer, arena, contents);
                buffer.push_str(&options_str);
                buffer.push(')');
            }
        }
        Document::Line(line) => {
            if line.literal {
                buffer.push_str("literalLine")
            } else if line.hard {
                buffer.push_str("hardline")
            } else if line.soft {
                buffer.push_str("softline")
            } else {
                buffer.push_str("line")
            }
        }
        Document::LineSuffix(docs) => {
            buffer.push_str("lineSuffix(");
            write_documents_vec_to_buffer(&mut buffer, arena, docs);
            buffer.push(')');
        }
        Document::LineSuffixBoundary => buffer.push_str("lineSuffixBoundary"),
        Document::IfBreak(IfBreak { break_contents, flat_content, group_id }) => {
            let mut options = vec![in arena];
            if let Some(id) = group_id {
                options.push(format!("groupId: {id}"));
            }

            buffer.push_str("ifBreak(");
            buffer.push_str(&print_document_to_string(arena, break_contents));
            buffer.push_str(", ");
            buffer.push_str(&print_document_to_string(arena, flat_content));
            if !options.is_empty() {
                buffer.push_str(", {");
                buffer.push_str(&options.join(", "));
                buffer.push('}');
            }

            buffer.push(')');
        }
        Document::Fill(Fill { parts }) => {
            buffer.push_str("fill(");
            write_documents_vec_to_buffer(&mut buffer, arena, parts);
            buffer.push(')');
        }
        Document::BreakParent => buffer.push_str("breakParent"),
        Document::Align(Align { alignment, contents }) => {
            buffer.push_str("dedentToRoot(align(");
            buffer.push_str(&format!("{:?}, ", alignment));
            write_documents_vec_to_buffer(&mut buffer, arena, contents);
            buffer.push(')');
            buffer.push(')');
        }
        Document::Trim(trim) => match trim {
            Trim::Whitespace => buffer.push_str("trim"),
            Trim::Newlines => buffer.push_str("trimNewlines"),
        },
        Document::DoNotTrim => buffer.push_str("doNotTrim"),
        Document::Space(_) => buffer.push_str("\" \""),
    }

    buffer
}
