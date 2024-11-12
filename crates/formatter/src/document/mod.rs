use std::fmt;

use serde;
use serde::Serialize;

use crate::document::group::GroupIdentifier;

pub mod group;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum Document<'a> {
    String(&'a str),
    Array(Vec<Document<'a>>),
    /// Increase the level of indentation.
    Indent(Vec<Document<'a>>),
    IndentIfBreak(IndentIfBreak<'a>),
    /// Mark a group of items which the printer should try to fit on one line.
    /// This is the basic command to tell the printer when to break.
    /// Groups are usually nested, and the printer will try to fit everything on one line,
    /// but if it doesn't fit it will break the outermost group first and try again.
    /// It will continue breaking groups until everything fits (or there are no more groups to break).
    Group(Group<'a>),
    /// Specify a line break.
    /// If an expression fits on one line, the line break will be replaced with a space.
    /// Line breaks always indent the next line with the current level of indentation.
    Line(Line),
    /// This is used to implement trailing comments.
    /// It's not practical to constantly check where the line ends to avoid accidentally printing some code at the end of a comment.
    /// `lineSuffix` buffers docs passed to it and flushes them before any new line.
    LineSuffix(Vec<Document<'a>>),
    /// Print something if the current `group` or the current element of `fill` breaks and something else if it doesn't.
    IfBreak(IfBreak<'a>),
    /// This is an alternative type of group which behaves like text layout:
    /// it's going to add a break whenever the next element doesn't fit in the line anymore.
    /// The difference with `group` is that it's not going to break all the separators, just the ones that are at the end of lines.
    Fill(Fill<'a>),
    /// Include this anywhere to force all parent groups to break.
    BreakParent,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Line {
    pub hard: bool,
    pub soft: bool,
    pub literal: bool,
}

impl Line {
    /// Specify a line break.
    /// The difference from line is that if the expression fits on one line, it will be replaced with nothing.
    pub fn softline() -> Self {
        Self { soft: true, ..Self::default() }
    }

    /// Specify a line break that is **always** included in the output,
    /// no matter if the expression fits on one line or not.
    pub fn hardline() -> Self {
        Self { hard: true, ..Self::default() }
    }

    pub fn literal_line() -> Self {
        Self { literal: true, ..Self::default() }
    }

    pub fn hardline_without_break_parent() -> Self {
        Self { hard: true, ..Self::default() }
    }

    pub fn literal_line_without_break_parent() -> Self {
        Self { hard: true, literal: true, ..Self::default() }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Group<'a> {
    pub contents: Vec<Document<'a>>,
    pub should_break: bool,
    pub expanded_states: Option<Vec<Document<'a>>>,
    pub id: Option<GroupIdentifier>,
}

impl<'a> Group<'a> {
    pub fn new(contents: Vec<Document<'a>>) -> Self {
        Self { contents, should_break: false, id: None, expanded_states: None }
    }

    pub fn new_conditional_group(contents: Vec<Document<'a>>, expanded_states: Vec<Document<'a>>) -> Self {
        Self { contents, should_break: false, id: None, expanded_states: Some(expanded_states) }
    }

    pub fn with_break(mut self, yes: bool) -> Self {
        self.should_break = yes;
        self
    }

    pub fn with_id(mut self, id: GroupIdentifier) -> Self {
        self.id = Some(id);
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IndentIfBreak<'a> {
    pub contents: Vec<Document<'a>>,
    pub group_id: Option<GroupIdentifier>,
}

impl<'a> IndentIfBreak<'a> {
    pub fn new(contents: Vec<Document<'a>>) -> Self {
        Self { contents, group_id: None }
    }

    pub fn with_id(mut self, id: GroupIdentifier) -> Self {
        self.group_id = Some(id);
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Fill<'a> {
    pub parts: Vec<Document<'a>>,
}

impl<'a> Fill<'a> {
    pub fn new(documents: Vec<Document<'a>>) -> Self {
        Self { parts: documents }
    }

    pub fn drain_out_pair(&mut self) -> (Option<Document<'a>>, Option<Document<'a>>) {
        let content = if self.parts.len() > 0 { Some(self.parts.remove(0)) } else { None };
        let whitespace = if self.parts.len() > 0 { Some(self.parts.remove(0)) } else { None };

        (content, whitespace)
    }

    pub fn dequeue(&mut self) -> Option<Document<'a>> {
        if self.parts.len() > 0 {
            Some(self.parts.remove(0))
        } else {
            None
        }
    }

    pub fn enqueue(&mut self, doc: Document<'a>) {
        self.parts.insert(0, doc);
    }

    pub fn parts(&self) -> &[Document<'a>] {
        &self.parts
    }

    pub fn take_parts(self) -> Vec<Document<'a>> {
        self.parts
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IfBreak<'a> {
    pub break_contents: Box<Document<'a>>,
    pub flat_content: Box<Document<'a>>,
    pub group_id: Option<GroupIdentifier>,
}

impl<'a> IfBreak<'a> {
    pub fn new(break_contents: Document<'a>, flat_content: Document<'a>) -> Self {
        Self { break_contents: Box::new(break_contents), flat_content: Box::new(flat_content), group_id: None }
    }

    pub fn then(break_contents: Document<'a>) -> Self {
        Self { break_contents: Box::new(break_contents), flat_content: Box::new(Document::empty()), group_id: None }
    }

    pub fn with_id(mut self, id: GroupIdentifier) -> Self {
        self.group_id = Some(id);
        self
    }
}

/// Doc Builder
impl<'a> Document<'a> {
    #[inline]
    pub fn string(s: &'a str) -> Document<'a> {
        Document::String(s)
    }

    #[inline]
    pub fn empty() -> Document<'a> {
        Document::String("")
    }

    #[inline]
    pub fn space() -> Document<'a> {
        Document::String(" ")
    }

    #[inline]
    pub fn boxed(self) -> Box<Document<'a>> {
        Box::new(self)
    }
}

impl fmt::Display for Document<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", print_doc_to_debug(self, 0))
    }
}

fn print_doc_to_debug(doc: &Document, indent: usize) -> String {
    let mut string = String::new();
    match doc {
        Document::String(s) => {
            string.push('"');
            string.push_str(&s.to_string());
            string.push('"');
        }
        Document::Array(contents) => {
            string.push_str("array(");
            for (idx, doc) in contents.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc, indent + 1));
                if idx != contents.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str(")");
        }
        Document::Indent(contents) => {
            string.push_str("indent(");
            for (idx, doc) in contents.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc, indent + 1));
                if idx != contents.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str(")");
        }
        Document::IndentIfBreak(indent_if_break) => {
            string.push_str("indent_if_break(");
            for (idx, doc) in indent_if_break.contents.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc, indent + 1));
                if idx != indent_if_break.contents.len() - 1 {
                    string.push_str(", ");
                }
            }

            if let Some(id) = indent_if_break.group_id {
                string.push_str(&format!(", {{ group_id: {} }}", id.0));
            }

            string.push_str(")");
        }
        Document::Group(group) => {
            if group.expanded_states.is_some() {
                string.push_str("conditional_group(");
            }
            string.push_str("group(");
            for (idx, doc) in group.contents.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc, indent + 1));
                if idx != group.contents.len() - 1 {
                    string.push_str(", ");
                }
            }

            if let Some(expanded_states) = &group.expanded_states {
                string.push_str(", expanded_states(");
                for (idx, doc) in expanded_states.iter().enumerate() {
                    string.push_str(&print_doc_to_debug(doc, indent + 1));
                    if idx != expanded_states.len() - 1 {
                        string.push_str(", ");
                    }
                }
                string.push_str(")");
            }

            string.push_str(&format!(", {{ should_break: {} }}", group.should_break));
            if let Some(id) = group.id {
                string.push_str(&format!(", {{ id: {} }}", id.0));
            }

            string.push_str(")");
            if group.expanded_states.is_some() {
                string.push_str(")");
            }
        }
        Document::Line(Line { soft, hard, .. }) => {
            if *soft {
                string.push_str("softline");
            } else if *hard {
                string.push_str("hardline");
            } else {
                string.push_str("line");
            }
        }
        Document::IfBreak(if_break) => {
            string.push_str(&format!(
                "if_break({}, {}",
                print_doc_to_debug(&if_break.break_contents, indent + 1),
                print_doc_to_debug(&if_break.flat_content, indent + 1)
            ));

            if let Some(group_id) = if_break.group_id {
                string.push_str(&format!(",\n {{ id: {} }}", group_id.0));
            }

            string.push(')');
        }
        Document::Fill(fill) => {
            string.push_str("fill(");
            let parts = fill.parts();
            for (idx, doc) in parts.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc, indent + 1));
                if idx != parts.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str(")");
        }
        Document::LineSuffix(docs) => {
            string.push_str("line_suffix(");
            for (idx, doc) in docs.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc, indent + 1));
                if idx != docs.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str(")");
        }
        Document::BreakParent => {
            string.push_str("break_parent");
        }
    }

    string
}
