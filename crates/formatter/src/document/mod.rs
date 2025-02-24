use std::fmt;

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
    LineSuffixBoundary,
    /// Print something if the current `group` or the current element of `fill` breaks and something else if it doesn't.
    IfBreak(IfBreak<'a>),
    /// This is an alternative type of group which behaves like text layout:
    /// it's going to add a break whenever the next element doesn't fit in the line anymore.
    /// The difference with `group` is that it's not going to break all the separators, just the ones that are at the end of lines.
    Fill(Fill<'a>),
    /// Include this anywhere to force all parent groups to break.
    BreakParent,
    Align(Align<'a>),
    /// Trim all newlines from the end of the document.
    Trim(Trim),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum Trim {
    /// Trims trailing whitespace characters (spaces and tabs) from the end of the document.
    Whitespace,
    /// Removes all newline characters from the end of the document.
    Newlines,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Align<'a> {
    pub alignment: &'a str,
    pub contents: Vec<Document<'a>>,
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

    pub fn conditional(contents: Vec<Document<'a>>, expanded_states: Vec<Document<'a>>) -> Self {
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
    pub fn drain_out_pair(&mut self) -> (Option<Document<'a>>, Option<Document<'a>>) {
        let content = if !self.parts.is_empty() { Some(self.parts.remove(0)) } else { None };
        let whitespace = if !self.parts.is_empty() { Some(self.parts.remove(0)) } else { None };

        (content, whitespace)
    }

    pub fn dequeue(&mut self) -> Option<Document<'a>> {
        if !self.parts.is_empty() { Some(self.parts.remove(0)) } else { None }
    }

    pub fn enqueue(&mut self, doc: Document<'a>) {
        self.parts.insert(0, doc);
    }

    pub fn parts(&self) -> &[Document<'a>] {
        &self.parts
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

#[derive(Clone, Copy)]
pub enum Separator {
    #[allow(unused)]
    SoftLine,
    HardLine,
    LiteralLine,
    CommaLine, // [",", line]
    Space,
}

impl<'a> Document<'a> {
    #[inline]
    pub fn empty() -> Document<'a> {
        Document::String("")
    }

    #[inline]
    pub fn space() -> Document<'a> {
        Document::String(" ")
    }

    pub fn can_break(&self) -> bool {
        self.any(|doc| matches!(doc, Document::Line(_)))
    }

    pub fn any<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Document<'a>) -> bool,
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

    pub fn join(documents: Vec<Document<'a>>, separator: Separator) -> Vec<Document<'a>> {
        let mut parts = vec![];
        for (i, document) in documents.into_iter().enumerate() {
            if i != 0 {
                parts.push(match separator {
                    Separator::Space => Document::String(" "),
                    Separator::SoftLine => Document::Line(Line::soft()),
                    Separator::HardLine => Document::Line(Line::hard()),
                    Separator::LiteralLine => {
                        Document::Array(vec![Document::Line(Line::literal()), Document::BreakParent])
                    }
                    Separator::CommaLine => {
                        Document::Array(vec![Document::String(","), Document::Line(Line::default())])
                    }
                });
            }

            parts.push(document);
        }
        parts
    }
}

impl fmt::Display for Document<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", print_doc_to_debug(self))
    }
}

fn print_doc_to_debug(doc: &Document) -> String {
    match doc {
        Document::String(s) => format!("{:?}", s),
        Document::Array(docs) => {
            let printed: Vec<String> = docs.iter().map(|d| print_doc_to_debug(d)).collect();
            if printed.len() == 1 { printed[0].clone() } else { format!("[{}]", printed.join(", ")) }
        }
        Document::Indent(docs) => {
            format!("indent({})", print_doc_to_debug(&Document::Array(docs.clone())))
        }
        Document::IndentIfBreak(IndentIfBreak { contents, group_id }) => {
            let mut options = vec![];
            if let Some(id) = group_id {
                options.push(format!("groupId: {:?}", id));
            }
            let options_str =
                if options.is_empty() { String::new() } else { format!(", {{ {} }}", options.join(", ")) };
            format!("indentIfBreak({}{})", print_doc_to_debug(&Document::Array(contents.clone())), options_str)
        }
        Document::Group(Group { contents, should_break, expanded_states, id }) => {
            let mut options = vec![];
            if *should_break {
                options.push("shouldBreak: true".to_string());
            }
            if let Some(id) = id {
                options.push(format!("id: {:?}", id));
            }
            let expanded_states_str = if let Some(states) = expanded_states {
                format!(
                    "conditionalGroup([{}]",
                    states.iter().map(|s| print_doc_to_debug(s)).collect::<Vec<_>>().join(", ")
                )
            } else {
                String::new()
            };
            let options_str =
                if options.is_empty() { String::new() } else { format!(", {{ {} }}", options.join(", ")) };

            if expanded_states_str.is_empty() {
                format!("group({}{})", print_doc_to_debug(&Document::Array(contents.clone())), options_str)
            } else {
                format!(
                    "{}, {}{})",
                    expanded_states_str,
                    print_doc_to_debug(&Document::Array(contents.clone())),
                    options_str,
                )
            }
        }
        Document::Line(line) => {
            if line.literal {
                "literalline".to_string()
            } else if line.hard {
                "hardline".to_string()
            } else if line.soft {
                "softline".to_string()
            } else {
                "line".to_string()
            }
        }
        Document::LineSuffix(docs) => {
            format!("lineSuffix({})", print_doc_to_debug(&Document::Array(docs.clone())))
        }
        Document::LineSuffixBoundary => "lineSuffixBoundary".to_string(),
        Document::IfBreak(IfBreak { break_contents, flat_content, group_id }) => {
            let mut options = vec![];
            if let Some(id) = group_id {
                options.push(format!("groupId: {:?}", id));
            }
            let options_str =
                if options.is_empty() { String::new() } else { format!(", {{ {} }}", options.join(", ")) };
            format!(
                "ifBreak({}, {}{})",
                print_doc_to_debug(break_contents),
                print_doc_to_debug(flat_content),
                options_str
            )
        }
        Document::Fill(Fill { parts }) => {
            format!("fill([{}])", parts.iter().map(|p| print_doc_to_debug(p)).collect::<Vec<_>>().join(", "))
        }
        Document::BreakParent => "breakParent".to_string(),
        Document::Align(Align { alignment, contents }) => {
            format!("dedentToRoot(align({:?}, {}))", alignment, print_doc_to_debug(&Document::Array(contents.clone())))
        }
        Document::Trim(_) => "trim".to_string(),
    }
}
