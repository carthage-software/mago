//! File-level tracker that feeds [`LintContext::import_name`] and friends.

use foldhash::HashMap;
use foldhash::HashSet;

use mago_span::HasSpan;
use mago_span::Span;
use mago_word::Word;
use mago_word::ascii_lowercase_constant_name_word;
use mago_word::ascii_lowercase_word;
use mago_word::concat_word;
use mago_word::word;

use mago_bytes::trim_start_byte;
use mago_syntax::cst::Constant;
use mago_syntax::cst::Declare;
use mago_syntax::cst::EchoTag;
use mago_syntax::cst::Namespace;
use mago_syntax::cst::NamespaceBody;
use mago_syntax::cst::Node;
use mago_syntax::cst::Use;
use mago_syntax::cst::UseItem;
use mago_syntax::cst::UseItems;
use mago_syntax::cst::UseType;
use mago_text_edit::TextEdit;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImportKind {
    Name,
    Function,
    Constant,
}

/// The answer returned to a rule asking the context to import a fully-qualified name.
///
/// - `local_name` is the identifier the rule should replace the FQN span with.
///   When an aliased import already covers the FQN, this is the alias, not the
///   last segment of the FQN; the rule should surface that in its message.
/// - `use_statement_edit` is either `None` when the name is already available
///   (already imported, or the FQN resolves to the current namespace) or an
///   insert edit placing a `use` at the canonical insertion point.
#[derive(Debug, Clone)]
pub struct ImportResolution {
    pub local_name: Word,
    pub use_statement_edit: Option<TextEdit>,
}

impl ImportResolution {
    /// True when no `use` statement needs to be added; either the FQN is
    /// already imported (possibly under an alias) or it resolves through the
    /// current namespace.
    #[inline]
    #[must_use]
    pub const fn is_already_available(&self) -> bool {
        self.use_statement_edit.is_none()
    }
}

/// Where in the file does a newly-synthesised `use` statement get inserted?
///
/// The anchor tells the tracker both the byte offset to insert at and what
/// leading/trailing whitespace reads naturally; back-to-back `use` lines want
/// one newline, but a `use` following a namespace or declare wants a blank line.
#[derive(Debug, Clone, Copy)]
enum InsertionAnchor {
    /// Right after the last `use` statement we've seen in this scope.
    AfterUse { offset: u32 },
    /// After `<?php`, `declare(...);`, `namespace X;`, or `{` of a braced
    /// namespace; all cases where a blank line looks nicer.
    AfterPreamble { offset: u32 },
    /// Right before a `<?=` echo tag at the file's first PHP-relevant
    /// position. `<?=` expects an expression, not a statement, so we can't
    /// insert `use` inside the block; instead we emit a self-contained
    /// `<?php use X; ?>` block just before the echo tag. Stagger walks
    /// backwards through the whitespace that precedes the tag.
    BeforeEchoTag { offset: u32 },
}

impl InsertionAnchor {
    #[inline]
    fn offset(self) -> u32 {
        match self {
            InsertionAnchor::AfterUse { offset }
            | InsertionAnchor::AfterPreamble { offset }
            | InsertionAnchor::BeforeEchoTag { offset } => offset,
        }
    }

    /// Leading whitespace prepended to each `use` emitted at this anchor.
    #[inline]
    fn leading(self) -> &'static str {
        match self {
            InsertionAnchor::AfterUse { .. } => "\n",
            InsertionAnchor::AfterPreamble { .. } => "\n\n",
            InsertionAnchor::BeforeEchoTag { .. } => "",
        }
    }

    #[inline]
    fn wraps_in_php_block(self) -> bool {
        matches!(self, InsertionAnchor::BeforeEchoTag { .. })
    }
}

#[derive(Debug, Default)]
struct ScopeState {
    namespace: Option<Word>,
    anchor: Option<InsertionAnchor>,
    class_imports: HashMap<Word, Word>,
    function_imports: HashMap<Word, Word>,
    constant_imports: HashMap<Word, Word>,
    class_fqn_to_local: HashMap<Word, Word>,
    function_fqn_to_local: HashMap<Word, Word>,
    constant_fqn_to_local: HashMap<Word, Word>,
    local_classes: HashSet<Word>,
    local_functions: HashSet<Word>,
    local_constants: HashSet<Word>,
    sole_function_import_use_span: HashMap<Word, Span>,
}

impl ScopeState {
    fn record_anchor_after_preamble(&mut self, offset: u32) {
        self.anchor = Some(InsertionAnchor::AfterPreamble { offset });
    }

    fn record_anchor_after_use(&mut self, offset: u32) {
        self.anchor = Some(InsertionAnchor::AfterUse { offset });
    }

    fn record_anchor_before_echo_tag(&mut self, offset: u32) {
        self.anchor = Some(InsertionAnchor::BeforeEchoTag { offset });
    }
}

/// Runs alongside the linter walk and answers import questions on behalf of rules.
#[derive(Debug, Default)]
pub struct ImportTracker {
    in_php: bool,
    scope: ScopeState,
}

impl ImportTracker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enter_node(&mut self, node: Node<'_, '_>) {
        match node {
            Node::FullOpeningTag(_) | Node::ShortOpeningTag(_) | Node::OpeningTag(_) if !self.in_php => {
                self.in_php = true;
                self.scope = ScopeState::default();
                self.scope.record_anchor_after_preamble(node.end_offset());
            }
            Node::EchoTag(echo_tag) => self.enter_echo_tag(echo_tag),
            Node::Namespace(namespace) => self.enter_namespace(namespace),
            Node::Use(r#use) => self.enter_use(r#use),
            Node::Declare(declare) => self.after_top_level_declare(declare),
            Node::Class(class) => {
                self.scope.local_classes.insert(ascii_lowercase_word(class.name.value));
            }
            Node::Interface(interface) => {
                self.scope.local_classes.insert(ascii_lowercase_word(interface.name.value));
            }
            Node::Trait(r#trait) => {
                self.scope.local_classes.insert(ascii_lowercase_word(r#trait.name.value));
            }
            Node::Enum(r#enum) => {
                self.scope.local_classes.insert(ascii_lowercase_word(r#enum.name.value));
            }
            Node::Function(function) => {
                self.scope.local_functions.insert(ascii_lowercase_word(function.name.value));
            }
            Node::Constant(constant) => self.after_top_level_const(constant),
            _ => {}
        }
    }

    pub fn exit_node(&mut self, node: Node<'_, '_>) {
        if let Node::Namespace(namespace) = node {
            self.exit_namespace(namespace);
        }
    }

    fn enter_echo_tag(&mut self, echo_tag: &EchoTag<'_>) {
        if self.in_php {
            return;
        }

        self.in_php = true;
        self.scope = ScopeState::default();
        self.scope.record_anchor_before_echo_tag(echo_tag.tag.start.offset);
    }

    fn enter_namespace(&mut self, namespace: &Namespace<'_>) {
        let namespace_name =
            namespace.name.as_ref().map(|id| trim_start_byte(id.value(), b'\\')).filter(|n| !n.is_empty()).map(word);

        self.scope = ScopeState { namespace: namespace_name, ..ScopeState::default() };

        match &namespace.body {
            NamespaceBody::Implicit(body) => {
                self.scope.record_anchor_after_preamble(body.terminator.end_offset());
            }
            NamespaceBody::BraceDelimited(block) => {
                self.scope.record_anchor_after_preamble(block.left_brace.end.offset);
            }
        }
    }

    fn exit_namespace(&mut self, namespace: &Namespace<'_>) {
        if matches!(namespace.body, NamespaceBody::BraceDelimited(_)) {
            self.scope = ScopeState::default();
        }
    }

    fn after_top_level_declare(&mut self, declare: &Declare<'_>) {
        if self.scope.namespace.is_none() && matches!(self.scope.anchor, Some(InsertionAnchor::AfterPreamble { .. })) {
            self.scope.record_anchor_after_preamble(declare.end_offset());
        }
    }

    fn after_top_level_const(&mut self, constant: &Constant<'_>) {
        for item in constant.items.iter() {
            self.scope.local_constants.insert(word(item.name.value));
        }
    }

    fn enter_use(&mut self, r#use: &Use<'_>) {
        let use_span = r#use.span();
        match &r#use.items {
            UseItems::Sequence(seq) => {
                for item in seq.items.iter() {
                    self.register_use_item(item, ImportKind::Name, None, None);
                }
            }
            UseItems::TypedSequence(typed) => {
                let kind = match &typed.r#type {
                    UseType::Function(_) => ImportKind::Function,
                    UseType::Const(_) => ImportKind::Constant,
                };
                let sole_use_span = (typed.items.len() == 1).then_some(use_span);

                for item in typed.items.iter() {
                    self.register_use_item(item, kind, None, sole_use_span);
                }
            }
            UseItems::TypedList(list) => {
                let kind = match &list.r#type {
                    UseType::Function(_) => ImportKind::Function,
                    UseType::Const(_) => ImportKind::Constant,
                };
                let sole_use_span = (list.items.len() == 1).then_some(use_span);

                let prefix = trim_start_byte(list.namespace.value(), b'\\');
                for item in list.items.iter() {
                    self.register_use_item(item, kind, Some(prefix), sole_use_span);
                }
            }
            UseItems::MixedList(list) => {
                let prefix = trim_start_byte(list.namespace.value(), b'\\');
                for item in list.items.iter() {
                    let kind = match &item.r#type {
                        Some(UseType::Function(_)) => ImportKind::Function,
                        Some(UseType::Const(_)) => ImportKind::Constant,
                        None => ImportKind::Name,
                    };

                    self.register_use_item(&item.item, kind, Some(prefix), None);
                }
            }
        }

        self.scope.record_anchor_after_use(r#use.end_offset());
    }

    fn register_use_item(
        &mut self,
        item: &UseItem<'_>,
        kind: ImportKind,
        prefix: Option<&[u8]>,
        sole_use_span: Option<Span>,
    ) {
        let raw = trim_start_byte(item.name.value(), b'\\');
        let fqn = match prefix {
            Some(prefix) => concat_word!(prefix, b"\\", raw),
            None => word(raw),
        };

        let local = match &item.alias {
            Some(alias) => word(alias.identifier.value),
            None => {
                let last = raw.rsplit(|&b| b == b'\\').next().unwrap_or(raw);
                word(last)
            }
        };

        match kind {
            ImportKind::Name => {
                let lookup = ascii_lowercase_word(local.as_bytes());
                let fqn_key = ascii_lowercase_word(fqn.as_bytes());
                self.scope.class_imports.insert(lookup, fqn);
                self.scope.class_fqn_to_local.insert(fqn_key, local);
            }
            ImportKind::Function => {
                let lookup = ascii_lowercase_word(local.as_bytes());
                let fqn_key = ascii_lowercase_word(fqn.as_bytes());
                self.scope.function_imports.insert(lookup, fqn);
                self.scope.function_fqn_to_local.insert(fqn_key, local);

                if let Some(span) = sole_use_span {
                    self.scope.sole_function_import_use_span.insert(lookup, span);
                }
            }
            ImportKind::Constant => {
                self.scope.constant_imports.insert(local, fqn);
                self.scope.constant_fqn_to_local.insert(ascii_lowercase_constant_name_word(fqn.as_bytes()), local);
            }
        }
    }

    #[must_use]
    pub fn sole_function_import_use_span(&self, local: &[u8]) -> Option<Span> {
        let lookup = ascii_lowercase_word(local);
        self.scope.sole_function_import_use_span.get(&lookup).copied()
    }

    /// Like [`sole_function_import_use_span`] but removes the entry after
    /// returning it; subsequent calls for the same `local` return `None`,
    /// preventing duplicate delete edits.
    pub fn take_sole_function_import_use_span(&mut self, local: &[u8]) -> Option<Span> {
        let lookup = ascii_lowercase_word(local);
        self.scope.sole_function_import_use_span.remove(&lookup)
    }

    pub fn import(&mut self, fqn: &[u8], kind: ImportKind) -> Option<ImportResolution> {
        let (namespace_part, short_part) = split_fqn(fqn)?;
        if is_reserved_type_name(short_part, kind) {
            return None;
        }

        let short_word = word(short_part);
        let short_lookup = ascii_lowercase_word(short_part);
        let full = match namespace_part {
            Some(ns) => concat_word!(ns, b"\\", short_part),
            None => word(short_part),
        };

        let existing_reverse = match kind {
            ImportKind::Name => self.scope.class_fqn_to_local.get(&ascii_lowercase_word(full.as_bytes())).copied(),
            ImportKind::Function => {
                self.scope.function_fqn_to_local.get(&ascii_lowercase_word(full.as_bytes())).copied()
            }
            ImportKind::Constant => {
                self.scope.constant_fqn_to_local.get(&ascii_lowercase_constant_name_word(full.as_bytes())).copied()
            }
        };

        if let Some(local) = existing_reverse {
            return Some(ImportResolution { local_name: local, use_statement_edit: None });
        }

        let has_conflict = match kind {
            ImportKind::Name => {
                self.scope.class_imports.contains_key(&short_lookup) || self.scope.local_classes.contains(&short_lookup)
            }
            ImportKind::Function => {
                self.scope.function_imports.contains_key(&short_lookup)
                    || self.scope.local_functions.contains(&short_lookup)
            }
            ImportKind::Constant => {
                self.scope.constant_imports.contains_key(&short_word)
                    || self.scope.local_constants.contains(&short_word)
            }
        };

        if has_conflict {
            return None;
        }

        if let Some(ns) = namespace_part
            && let Some(current) = self.scope.namespace.as_ref()
            && ns.eq_ignore_ascii_case(current.as_bytes())
        {
            return Some(ImportResolution { local_name: short_word, use_statement_edit: None });
        }

        if namespace_part.is_none() && self.scope.namespace.is_none() {
            return Some(ImportResolution { local_name: short_word, use_statement_edit: None });
        }

        let anchor = self.scope.anchor?;
        let offset = anchor.offset();
        let leading = anchor.leading();
        let mut text: Vec<u8> = Vec::new();
        text.extend_from_slice(leading.as_bytes());
        if anchor.wraps_in_php_block() {
            text.extend_from_slice(b"<?php ");
            render_use_statement(kind, full.as_bytes(), &mut text);
            text.extend_from_slice(b"; ?>");
        } else {
            render_use_statement(kind, full.as_bytes(), &mut text);
            text.extend_from_slice(b";");
        }

        match kind {
            ImportKind::Name => {
                self.scope.class_imports.insert(short_lookup, full);
                self.scope.class_fqn_to_local.insert(ascii_lowercase_word(full.as_bytes()), short_word);
            }
            ImportKind::Function => {
                self.scope.function_imports.insert(short_lookup, full);
                self.scope.function_fqn_to_local.insert(ascii_lowercase_word(full.as_bytes()), short_word);
            }
            ImportKind::Constant => {
                self.scope.constant_imports.insert(short_word, full);
                self.scope
                    .constant_fqn_to_local
                    .insert(ascii_lowercase_constant_name_word(full.as_bytes()), short_word);
            }
        }

        Some(ImportResolution { local_name: short_word, use_statement_edit: Some(TextEdit::insert(offset, text)) })
    }
}

fn render_use_statement(kind: ImportKind, fqn: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(match kind {
        ImportKind::Name => b"use ".as_slice(),
        ImportKind::Function => b"use function ".as_slice(),
        ImportKind::Constant => b"use const ".as_slice(),
    });

    out.extend_from_slice(fqn);
}

/// Normalise an FQN input (`\Foo\Bar\Baz` or `Foo\Bar\Baz`) into (namespace, short).
fn split_fqn(fqn: &[u8]) -> Option<(Option<&[u8]>, &[u8])> {
    let trimmed = trim_start_byte(fqn, b'\\');
    if trimmed.is_empty() || trimmed.last().copied() == Some(b'\\') {
        return None;
    }

    match trimmed.iter().rposition(|&b| b == b'\\') {
        Some(idx) => {
            let (ns, rest) = trimmed.split_at(idx);
            let short = &rest[1..];
            if ns.is_empty() || short.is_empty() { Some((None, trimmed)) } else { Some((Some(ns), short)) }
        }
        None => Some((None, trimmed)),
    }
}

fn is_reserved_type_name(short: &[u8], kind: ImportKind) -> bool {
    const RESERVED_CLASS_NAMES: &[&[u8]] = &[
        b"int",
        b"integer",
        b"float",
        b"double",
        b"bool",
        b"boolean",
        b"string",
        b"array",
        b"object",
        b"callable",
        b"iterable",
        b"mixed",
        b"void",
        b"never",
        b"self",
        b"static",
        b"parent",
        b"true",
        b"false",
        b"null",
    ];

    match kind {
        ImportKind::Name => RESERVED_CLASS_NAMES.iter().any(|r| short.eq_ignore_ascii_case(r)),
        ImportKind::Function => false,
        ImportKind::Constant => matches!(short.to_ascii_lowercase().as_slice(), b"true" | b"false" | b"null"),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn split_fqn_rejects_empty_and_trailing_separator() {
        assert!(split_fqn(b"").is_none());
        assert!(split_fqn(b"\\").is_none());
        assert!(split_fqn(b"Foo\\").is_none());
    }

    #[test]
    fn split_fqn_single_segment() {
        let (ns, short) = split_fqn(b"Foo").unwrap();
        assert!(ns.is_none());
        assert_eq!(short, b"Foo");

        let (ns, short) = split_fqn(b"\\Foo").unwrap();
        assert!(ns.is_none());
        assert_eq!(short, b"Foo");
    }

    #[test]
    fn split_fqn_multi_segment() {
        let (ns, short) = split_fqn(b"\\App\\Http\\Controller").unwrap();
        assert_eq!(ns, Some(b"App\\Http".as_slice()));
        assert_eq!(short, b"Controller");
    }

    #[test]
    fn reserved_names_blocked_for_classes() {
        assert!(is_reserved_type_name(b"int", ImportKind::Name));
        assert!(is_reserved_type_name(b"INT", ImportKind::Name));
        assert!(is_reserved_type_name(b"self", ImportKind::Name));
        assert!(!is_reserved_type_name(b"User", ImportKind::Name));
    }

    #[test]
    fn reserved_names_allow_functions() {
        assert!(!is_reserved_type_name(b"int", ImportKind::Function));
        assert!(!is_reserved_type_name(b"strlen", ImportKind::Function));
    }

    #[test]
    fn reserved_names_for_constants() {
        assert!(is_reserved_type_name(b"true", ImportKind::Constant));
        assert!(is_reserved_type_name(b"NULL", ImportKind::Constant));
        assert!(!is_reserved_type_name(b"FOO", ImportKind::Constant));
    }

    fn tracker_with_anchor(namespace: Option<&[u8]>, offset: u32) -> ImportTracker {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: namespace.map(word),
            anchor: Some(InsertionAnchor::AfterPreamble { offset }),
            ..ScopeState::default()
        };
        tracker
    }

    fn do_import(tracker: &mut ImportTracker, fqn: &[u8], kind: ImportKind) -> Option<ImportResolution> {
        tracker.import(fqn, kind)
    }

    #[test]
    fn import_before_open_tag_returns_none() {
        let mut tracker = ImportTracker::new();
        assert!(do_import(&mut tracker, b"Foo\\Bar", ImportKind::Name).is_none());
    }

    #[test]
    fn import_adds_use_and_rewrites_short() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 42);
        let resolution = tracker.import(b"Other\\Thing", ImportKind::Name).expect("should import");
        assert_eq!(resolution.local_name.as_bytes(), b"Thing");
        let edit = resolution.use_statement_edit.expect("edit");
        assert!(contains_bytes(&edit.new_text, b"use Other\\Thing;"));
    }

    #[test]
    fn import_same_fqn_twice_only_emits_one_edit() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 42);
        let first = tracker.import(b"Other\\Thing", ImportKind::Name).unwrap();
        let second = tracker.import(b"Other\\Thing", ImportKind::Name).unwrap();
        assert!(first.use_statement_edit.is_some());
        assert!(second.use_statement_edit.is_none());
        assert_eq!(first.local_name, second.local_name);
    }

    #[test]
    fn import_fqn_in_current_namespace_no_edit() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 42);
        let resolution = tracker.import(b"App\\User", ImportKind::Name).unwrap();
        assert_eq!(resolution.local_name.as_bytes(), b"User");
        assert!(resolution.use_statement_edit.is_none());
    }

    #[test]
    fn import_single_segment_in_global_no_edit() {
        let mut tracker = tracker_with_anchor(None, 10);
        let resolution = tracker.import(b"Foo", ImportKind::Name).unwrap();
        assert!(resolution.use_statement_edit.is_none());
        assert_eq!(resolution.local_name.as_bytes(), b"Foo");
    }

    #[test]
    fn import_single_segment_in_namespace_still_imports() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let resolution = tracker.import(b"Foo", ImportKind::Name).unwrap();
        assert!(resolution.use_statement_edit.is_some());
        assert_eq!(resolution.local_name.as_bytes(), b"Foo");
    }

    #[test]
    fn short_name_conflict_with_existing_import_returns_none() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"User"), word(b"Other\\User"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"Other\\User"), word(b"User"));
        assert!(tracker.import(b"App\\User", ImportKind::Name).is_none());
    }

    #[test]
    fn short_name_conflict_with_local_class_returns_none() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.local_classes.insert(ascii_lowercase_word(b"User"));
        assert!(tracker.import(b"Other\\User", ImportKind::Name).is_none());
    }

    #[test]
    fn aliased_existing_import_returns_alias_no_edit() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"Baz"), word(b"Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"Foo\\Bar"), word(b"Baz"));
        let resolution = tracker.import(b"Foo\\Bar", ImportKind::Name).unwrap();
        assert_eq!(resolution.local_name.as_bytes(), b"Baz");
        assert!(resolution.use_statement_edit.is_none());
    }

    #[test]
    fn reserved_type_name_not_importable() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        assert!(tracker.import(b"\\int", ImportKind::Name).is_none());
        assert!(tracker.import(b"\\self", ImportKind::Name).is_none());
    }

    #[test]
    fn function_and_class_imports_do_not_collide() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"foo"), word(b"Other\\foo"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"Other\\foo"), word(b"foo"));
        let resolution = tracker.import(b"App\\Util\\foo", ImportKind::Function).expect("import");
        assert_eq!(resolution.local_name.as_bytes(), b"foo");
        assert!(resolution.use_statement_edit.is_some());
    }

    #[test]
    fn constant_imports_are_case_sensitive() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.constant_imports.insert(word(b"FOO"), word(b"Other\\FOO"));
        tracker.scope.constant_fqn_to_local.insert(ascii_lowercase_constant_name_word(b"Other\\FOO"), word(b"FOO"));
        let resolution = tracker.import(b"Other\\foo", ImportKind::Constant).unwrap();
        assert_eq!(resolution.local_name.as_bytes(), b"foo");
        assert!(resolution.use_statement_edit.is_some());
    }

    #[test]
    fn class_imports_are_case_insensitive() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"User"), word(b"Other\\User"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"Other\\User"), word(b"User"));
        assert!(tracker.import(b"App\\user", ImportKind::Name).is_none());
    }

    #[test]
    fn no_anchor_returns_none_even_if_otherwise_valid() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState { namespace: Some(word(b"App")), anchor: None, ..ScopeState::default() };
        assert!(tracker.import(b"Other\\Thing", ImportKind::Name).is_none());
    }

    #[test]
    fn edit_text_leading_depends_on_anchor_kind() {
        let mut t1 = tracker_with_anchor(Some(b"App"), 10);
        t1.scope.anchor = Some(InsertionAnchor::AfterPreamble { offset: 10 });
        let r1 = t1.import(b"X\\Y", ImportKind::Name).unwrap();
        assert!(starts_with_bytes(&r1.use_statement_edit.unwrap().new_text, b"\n\n"));

        let mut t2 = tracker_with_anchor(Some(b"App"), 10);
        t2.scope.anchor = Some(InsertionAnchor::AfterUse { offset: 10 });
        let r2 = t2.import(b"X\\Y", ImportKind::Name).unwrap();
        assert!(starts_with_bytes(&r2.use_statement_edit.unwrap().new_text, b"\nuse"));
    }

    #[test]
    fn multiple_imports_all_land_at_the_same_anchor_offset() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        let r0 = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import(b"B\\B", ImportKind::Name).unwrap();
        let r2 = tracker.import(b"C\\C", ImportKind::Name).unwrap();

        assert_eq!(r0.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(r1.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(r2.use_statement_edit.unwrap().range.start, 5);
    }

    #[test]
    fn stitched_multi_import_contains_all_uses_and_preserves_body() {
        let src = "<?php\n\n$x = 1;";
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        let r0 = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import(b"B\\B", ImportKind::Name).unwrap();
        let r2 = tracker.import(b"C\\C", ImportKind::Name).unwrap();

        let mut edits =
            vec![r0.use_statement_edit.unwrap(), r1.use_statement_edit.unwrap(), r2.use_statement_edit.unwrap()];
        let output = stitch(src, &mut edits);

        assert!(output.contains("use A\\A;"));
        assert!(output.contains("use B\\B;"));
        assert!(output.contains("use C\\C;"));
        assert!(output.contains("$x = 1;"));
        assert!(output.starts_with("<?php"));
    }

    #[test]
    fn new_anchor_replaces_old_anchor_offset() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        let r0 = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        assert_eq!(r0.use_statement_edit.unwrap().range.start, 5);

        tracker.scope.record_anchor_after_use(13);
        let r1 = tracker.import(b"B\\B", ImportKind::Name).unwrap();
        assert_eq!(r1.use_statement_edit.unwrap().range.start, 13);
    }

    #[test]
    fn forty_imports_all_stack_at_base_offset() {
        let src = "<?php\n\n$x = 1;";
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        let mut edits = Vec::new();
        for i in 0..40 {
            let fqn = format!("Pkg\\T{:02}", i);
            let r = tracker.import(fqn.as_bytes(), ImportKind::Name).unwrap();
            edits.push(r.use_statement_edit.unwrap());
        }

        assert!(edits.iter().all(|e| e.range.start == 5));
        let out = stitch(src, &mut edits);
        for i in 0..40 {
            let expected = format!("use Pkg\\T{:02};", i);
            assert!(out.contains(&expected), "missing `{expected}`");
        }

        assert!(out.contains("$x = 1;"));
    }

    #[test]
    fn echo_tag_anchor_wraps_first_import_in_php_block() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::BeforeEchoTag { offset: 6 }),
            ..ScopeState::default()
        };
        let r = tracker.import(b"Foo\\Bar", ImportKind::Name).unwrap();
        let edit = r.use_statement_edit.unwrap();
        assert_eq!(edit.range.start, 6);
        assert_eq!(edit.new_text, b"<?php use Foo\\Bar; ?>");
    }

    #[test]
    fn echo_tag_multiple_imports_stack_at_tag_offset() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::BeforeEchoTag { offset: 7 }),
            ..ScopeState::default()
        };
        let r0 = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import(b"B\\B", ImportKind::Name).unwrap();
        let r2 = tracker.import(b"C\\C", ImportKind::Name).unwrap();
        assert_eq!(r0.use_statement_edit.as_ref().unwrap().range.start, 7);
        assert_eq!(r1.use_statement_edit.as_ref().unwrap().range.start, 7);
        assert_eq!(r2.use_statement_edit.as_ref().unwrap().range.start, 7);
    }

    #[test]
    fn echo_tag_stitch_produces_valid_multi_block_output() {
        let src = "hello\n\n<?= $x ?>";
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::BeforeEchoTag { offset: 7 }),
            ..ScopeState::default()
        };
        let r0 = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import(b"B\\B", ImportKind::Name).unwrap();
        let mut edits = vec![r0.use_statement_edit.unwrap(), r1.use_statement_edit.unwrap()];
        let out = stitch(src, &mut edits);
        assert!(out.contains("<?php use A\\A; ?>"));
        assert!(out.contains("<?php use B\\B; ?>"));
        assert!(out.ends_with("<?= $x ?>"));
    }

    #[test]
    fn echo_tag_imports_with_no_preceding_whitespace_stack_at_zero() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::BeforeEchoTag { offset: 0 }),
            ..ScopeState::default()
        };
        let r0 = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import(b"B\\B", ImportKind::Name).unwrap();
        assert_eq!(r0.use_statement_edit.unwrap().range.start, 0);
        assert_eq!(r1.use_statement_edit.unwrap().range.start, 0);
    }

    #[test]
    fn php_opening_tag_takes_precedence_over_later_echo_tag() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::AfterPreamble { offset: 5 }),
            ..ScopeState::default()
        };
        let anchor_before = tracker.scope.anchor;
        let r = tracker.import(b"Foo\\Bar", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(
            tracker.scope.anchor.map(|a| std::mem::discriminant(&a)),
            anchor_before.map(|a| std::mem::discriminant(&a))
        );
    }

    fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
        memchr::memmem::find(haystack, needle).is_some()
    }

    fn starts_with_bytes(haystack: &[u8], prefix: &[u8]) -> bool {
        haystack.starts_with(prefix)
    }

    fn stitch(src: &str, edits: &mut [TextEdit]) -> String {
        edits.sort_by_key(|e| e.range.start);
        let mut output = String::new();
        let mut cursor: u32 = 0;
        for edit in edits.iter() {
            output.push_str(&src[cursor as usize..edit.range.start as usize]);
            output.push_str(&String::from_utf8_lossy(&edit.new_text));
            cursor = edit.range.end;
        }
        output.push_str(&src[cursor as usize..]);
        output
    }

    #[test]
    fn stitched_output_preserves_unicode_body() {
        let src = "<?php\n\n// héllo\n$x;";
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        let r0 = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import(b"B\\B", ImportKind::Name).unwrap();
        let e0 = r0.use_statement_edit.unwrap();
        let e1 = r1.use_statement_edit.unwrap();
        assert!(src.is_char_boundary(e0.range.start as usize));
        assert!(src.is_char_boundary(e1.range.start as usize));

        let mut edits = vec![e0, e1];
        let out = stitch(src, &mut edits);
        assert!(out.contains("// héllo"));
        assert!(out.contains("use A\\A;"));
        assert!(out.contains("use B\\B;"));
    }

    #[test]
    fn local_class_does_not_block_function_or_constant_imports() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.local_classes.insert(ascii_lowercase_word(b"Foo"));
        assert!(tracker.import(b"Other\\foo", ImportKind::Function).is_some());
        assert!(tracker.import(b"Other\\FOO", ImportKind::Constant).is_some());
    }

    #[test]
    fn local_function_does_not_block_class_or_constant_imports() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.local_functions.insert(ascii_lowercase_word(b"foo"));
        assert!(tracker.import(b"Other\\Foo", ImportKind::Name).is_some());
        assert!(tracker.import(b"Other\\FOO", ImportKind::Constant).is_some());
    }

    #[test]
    fn local_constant_does_not_block_class_or_function_imports() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.local_constants.insert(word(b"FOO"));
        assert!(tracker.import(b"Other\\Foo", ImportKind::Name).is_some());
        assert!(tracker.import(b"Other\\foo", ImportKind::Function).is_some());
    }

    #[test]
    fn reserved_names_cover_all_scalar_and_compound_types() {
        for name in [
            b"int".as_slice(),
            b"integer",
            b"float",
            b"double",
            b"bool",
            b"boolean",
            b"string",
            b"array",
            b"object",
            b"callable",
            b"iterable",
            b"mixed",
            b"void",
            b"never",
            b"self",
            b"static",
            b"parent",
            b"true",
            b"false",
            b"null",
        ] {
            assert!(is_reserved_type_name(name, ImportKind::Name));
            let upper: Vec<u8> = name.iter().map(u8::to_ascii_uppercase).collect();
            assert!(is_reserved_type_name(&upper, ImportKind::Name));
        }
    }

    #[test]
    fn fqn_with_leading_backslash_matches_imported_form() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"Bar"), word(b"Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"Foo\\Bar"), word(b"Bar"));

        let r = tracker.import(b"\\Foo\\Bar", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_bytes(), b"Bar");
    }

    #[test]
    fn fqn_with_multiple_segments_shortens_to_last() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let r = tracker.import(b"A\\B\\C\\D\\E", ImportKind::Name).unwrap();
        assert_eq!(r.local_name.as_bytes(), b"E");
        assert!(contains_bytes(&r.use_statement_edit.as_ref().unwrap().new_text, b"use A\\B\\C\\D\\E;"));
    }

    #[test]
    fn forty_imports_stitch_cleanly_at_shared_offset() {
        let src = "<?php\n\n$x = 1;";
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);

        let mut edits: Vec<TextEdit> = Vec::new();
        let mut names: Vec<String> = Vec::new();
        for i in 0..40 {
            let fqn = format!("Pkg\\Class{:02}", i);
            let r = tracker.import(fqn.as_bytes(), ImportKind::Name).unwrap();
            edits.push(r.use_statement_edit.unwrap());
            names.push(format!("use Pkg\\Class{:02};", i));
        }

        assert!(edits.iter().all(|e| e.range.start == 5));

        let out = stitch(src, &mut edits);
        for name in &names {
            assert!(out.contains(name), "missing `{name}` in stitched output");
        }
        assert!(out.contains("$x = 1;"));
    }

    #[test]
    fn forty_one_imports_all_land_at_base() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        for i in 0..41 {
            let fqn = format!("Pkg\\T{:02}", i);
            let r = tracker.import(fqn.as_bytes(), ImportKind::Name).unwrap();
            assert_eq!(r.use_statement_edit.unwrap().range.start, 5);
        }
    }

    #[test]
    fn function_import_emits_use_function_keyword() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let r = tracker.import(b"Other\\strlen", ImportKind::Function).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert!(contains_bytes(&text, b"use function Other\\strlen;"), "got `{}`", String::from_utf8_lossy(&text));
        assert!(!contains_bytes(&text, b"use const"));
    }

    #[test]
    fn constant_import_emits_use_const_keyword() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let r = tracker.import(b"Other\\PHP_EOL", ImportKind::Constant).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert!(contains_bytes(&text, b"use const Other\\PHP_EOL;"), "got `{}`", String::from_utf8_lossy(&text));
        assert!(!contains_bytes(&text, b"use function"));
    }

    #[test]
    fn echo_tag_wraps_function_import_in_php_block() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::BeforeEchoTag { offset: 0 }),
            ..ScopeState::default()
        };
        let r = tracker.import(b"Other\\strlen", ImportKind::Function).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert_eq!(text, b"<?php use function Other\\strlen; ?>");
    }

    #[test]
    fn echo_tag_wraps_constant_import_in_php_block() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::BeforeEchoTag { offset: 0 }),
            ..ScopeState::default()
        };
        let r = tracker.import(b"Other\\PHP_EOL", ImportKind::Constant).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert_eq!(text, b"<?php use const Other\\PHP_EOL; ?>");
    }

    #[test]
    fn two_constant_imports_with_different_cases_coexist() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let a = tracker.import(b"Other\\FOO", ImportKind::Constant).unwrap();
        let b = tracker.import(b"Other\\foo", ImportKind::Constant).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_some());
        assert_ne!(a.local_name.as_bytes(), b.local_name.as_bytes());
    }

    #[test]
    fn anchor_after_use_is_where_next_import_lands() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        tracker.scope.record_anchor_after_use(30);
        let r = tracker.import(b"X\\Y", ImportKind::Name).unwrap();
        let edit = r.use_statement_edit.unwrap();
        assert_eq!(edit.range.start, 30);
        assert!(starts_with_bytes(&edit.new_text, b"\nuse "));
    }

    #[test]
    fn second_import_at_after_use_anchor_stacks_at_same_offset() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        tracker.scope.record_anchor_after_use(12);
        let r0 = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import(b"B\\B", ImportKind::Name).unwrap();
        assert_eq!(r0.use_statement_edit.unwrap().range.start, 12);
        assert_eq!(r1.use_statement_edit.unwrap().range.start, 12);
    }

    #[test]
    fn very_long_fqn_preserves_full_path_in_use_text() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let fqn = b"A\\B\\C\\D\\E\\F\\G\\H\\I\\J\\K\\Target";
        let r = tracker.import(fqn, ImportKind::Name).unwrap();
        assert_eq!(r.local_name.as_bytes(), b"Target");
        assert!(contains_bytes(
            &r.use_statement_edit.unwrap().new_text,
            b"use A\\B\\C\\D\\E\\F\\G\\H\\I\\J\\K\\Target;"
        ));
    }

    #[test]
    fn namespace_reset_drops_imports_from_previous_namespace() {
        let mut tracker = tracker_with_anchor(Some(b"A"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"Foo"), word(b"Other\\Foo"));
        tracker.scope = ScopeState {
            namespace: Some(word(b"B")),
            anchor: Some(InsertionAnchor::AfterPreamble { offset: 50 }),
            ..ScopeState::default()
        };
        let r = tracker.import(b"Other\\Foo", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
    }

    #[test]
    fn split_fqn_rejects_only_backslashes() {
        assert!(split_fqn(b"\\\\\\\\").is_none());
        assert!(split_fqn(b"\\").is_none());
    }

    #[test]
    fn split_fqn_trailing_separator_rejected() {
        assert!(split_fqn(b"Foo\\").is_none());
        assert!(split_fqn(b"A\\B\\C\\").is_none());
        assert!(split_fqn(b"\\A\\B\\").is_none());
    }

    #[test]
    fn import_empty_fqn_returns_none() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        assert!(tracker.import(b"", ImportKind::Name).is_none());
        assert!(tracker.import(b"\\", ImportKind::Name).is_none());
        assert!(tracker.import(b"Foo\\", ImportKind::Name).is_none());
    }

    #[test]
    fn anchor_offset_past_source_end_does_not_panic() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 9999);
        let r = tracker.import(b"Other\\Thing", ImportKind::Name).unwrap();
        assert_eq!(r.use_statement_edit.unwrap().range.start, 9999);
    }

    #[test]
    fn split_fqn_keeps_consecutive_backslashes_in_namespace() {
        let (ns, short) = split_fqn(b"Foo\\\\Bar").unwrap();
        assert_eq!(ns, Some(b"Foo\\".as_slice()));
        assert_eq!(short, b"Bar");
    }

    #[test]
    fn extremely_deep_namespace_does_not_panic() {
        let segments: Vec<String> = (0..100).map(|i| format!("N{i:03}")).collect();
        let fqn = format!("{}\\Target", segments.join("\\"));
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let r = tracker.import(fqn.as_bytes(), ImportKind::Name).unwrap();
        assert_eq!(r.local_name.as_bytes(), b"Target");
        let text = r.use_statement_edit.unwrap().new_text;
        assert!(contains_bytes(&text, b"N000\\"));
        assert!(contains_bytes(&text, b"\\N099\\Target;"));
    }

    #[test]
    fn short_name_matching_current_namespace_is_just_a_short_name() {
        let mut tracker = tracker_with_anchor(Some(b"Foo"), 10);
        let r = tracker.import(b"Other\\Foo", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(r.local_name.as_bytes(), b"Foo");
    }

    #[test]
    fn self_namespace_import_is_already_available() {
        let mut tracker = tracker_with_anchor(Some(b"App\\Sub"), 10);
        let r = tracker.import(b"App\\Sub\\Thing", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_bytes(), b"Thing");
    }

    #[test]
    fn sub_namespace_of_current_still_needs_use() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let r = tracker.import(b"App\\Sub\\Thing", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(r.local_name.as_bytes(), b"Thing");
    }

    #[test]
    fn parent_namespace_import_requires_use() {
        let mut tracker = tracker_with_anchor(Some(b"App\\Sub"), 10);
        let r = tracker.import(b"App\\Thing", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(r.local_name.as_bytes(), b"Thing");
    }

    #[test]
    fn same_fqn_three_kinds_three_edits() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let a = tracker.import(b"Other\\foo", ImportKind::Name).unwrap();
        let b = tracker.import(b"Other\\foo", ImportKind::Function).unwrap();
        let c = tracker.import(b"Other\\foo", ImportKind::Constant).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_some());
        assert!(c.use_statement_edit.is_some());
    }

    #[test]
    fn case_variant_of_same_class_fqn_deduplicates() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let a = tracker.import(b"foo\\bar", ImportKind::Name).unwrap();
        let b = tracker.import(b"Foo\\Bar", ImportKind::Name).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_none());
    }

    #[test]
    fn alias_matching_another_short_name_blocks_future_import() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"Foo"), word(b"X\\Y"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"X\\Y"), word(b"Foo"));
        assert!(tracker.import(b"Other\\Foo", ImportKind::Name).is_none());
    }

    #[test]
    fn conflict_uses_alias_short_not_fqn_suffix() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"Bar"), word(b"X\\Y\\Z"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"X\\Y\\Z"), word(b"Bar"));
        assert!(tracker.import(b"Other\\Bar", ImportKind::Name).is_none());
        assert!(tracker.import(b"Other\\Baz", ImportKind::Name).is_some());
    }

    #[test]
    fn imports_follow_anchor_as_it_moves_between_kinds() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        let a = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        let b = tracker.import(b"B\\B", ImportKind::Name).unwrap();
        assert_eq!(a.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(b.use_statement_edit.unwrap().range.start, 5);
        tracker.scope.record_anchor_after_use(15);
        let c = tracker.import(b"C\\C", ImportKind::Name).unwrap();
        let d = tracker.import(b"D\\D", ImportKind::Name).unwrap();
        assert_eq!(c.use_statement_edit.unwrap().range.start, 15);
        assert_eq!(d.use_statement_edit.unwrap().range.start, 15);
    }

    #[test]
    fn conflict_check_runs_before_current_namespace_shortcut() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"User"), word(b"Other\\User"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"Other\\User"), word(b"User"));
        assert!(tracker.import(b"App\\User", ImportKind::Name).is_none());
    }

    #[test]
    fn conflict_check_runs_before_global_shortcut() {
        let mut tracker = tracker_with_anchor(None, 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"Bar"), word(b"Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"Foo\\Bar"), word(b"Bar"));
        assert!(tracker.import(b"Bar", ImportKind::Name).is_none());
    }

    #[test]
    fn aliased_import_survives_repeated_queries() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"Baz"), word(b"Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"Foo\\Bar"), word(b"Baz"));
        for _ in 0..5 {
            let r = tracker.import(b"Foo\\Bar", ImportKind::Name).unwrap();
            assert_eq!(r.local_name.as_bytes(), b"Baz");
            assert!(r.use_statement_edit.is_none());
        }
    }

    #[test]
    fn function_name_collides_with_function_not_with_class() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.function_imports.insert(ascii_lowercase_word(b"strlen"), word(b"Other\\strlen"));
        tracker.scope.function_fqn_to_local.insert(ascii_lowercase_word(b"Other\\strlen"), word(b"strlen"));
        assert!(tracker.import(b"Third\\strlen", ImportKind::Function).is_none());
        assert!(tracker.import(b"Third\\Strlen", ImportKind::Name).is_some());
    }

    #[test]
    fn reserved_names_do_not_block_function_imports() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        assert!(tracker.import(b"Other\\int", ImportKind::Function).is_some());
        assert!(tracker.import(b"Other\\void", ImportKind::Function).is_some());
    }

    #[test]
    fn reserved_constant_names_only_cover_scalar_literals() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        assert!(tracker.import(b"Other\\int", ImportKind::Constant).is_some());
        assert!(tracker.import(b"Other\\void", ImportKind::Constant).is_some());
        assert!(tracker.import(b"Other\\TRUE", ImportKind::Constant).is_none());
        assert!(tracker.import(b"Other\\False", ImportKind::Constant).is_none());
    }

    #[test]
    fn no_anchor_no_imports_no_panics() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState { anchor: None, ..ScopeState::default() };
        for kind in [ImportKind::Name, ImportKind::Function, ImportKind::Constant] {
            assert!(tracker.import(b"X\\Y", kind).is_none());
        }
    }

    #[test]
    fn echo_tag_no_preceding_whitespace_inserts_at_zero() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::BeforeEchoTag { offset: 0 }),
            ..ScopeState::default()
        };
        let r = tracker.import(b"Foo\\Bar", ImportKind::Name).unwrap();
        let edit = r.use_statement_edit.unwrap();
        assert_eq!(edit.range.start, 0);
        assert_eq!(edit.new_text, b"<?php use Foo\\Bar; ?>");
    }

    #[test]
    fn echo_tag_after_html_text_stacks_both_imports_at_tag_offset() {
        let src = "<p>Hello world</p>\n<?= $x ?>";
        let tag_offset = src.find("<?=").unwrap() as u32;
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::BeforeEchoTag { offset: tag_offset }),
            ..ScopeState::default()
        };
        let a = tracker.import(b"Foo\\Bar", ImportKind::Name).unwrap();
        let b = tracker.import(b"Baz\\Qux", ImportKind::Name).unwrap();
        let e_a = a.use_statement_edit.unwrap();
        let e_b = b.use_statement_edit.unwrap();
        assert_eq!(e_a.range.start, tag_offset);
        assert_eq!(e_b.range.start, tag_offset);

        let mut edits = vec![e_a, e_b];
        let out = stitch(src, &mut edits);
        assert!(out.contains("<p>Hello world</p>"));
        assert!(out.contains("<?php use Foo\\Bar; ?>"));
        assert!(out.contains("<?php use Baz\\Qux; ?>"));
        assert!(out.contains("<?= $x ?>"));
    }

    #[test]
    fn echo_tag_five_imports_all_land_at_tag_offset() {
        let src = "<!DOCTYPE html>\n<html>\n<body>\n<h1>Hi</h1>\n\n\n\n\n<?= 'x' ?>";
        let tag_offset = src.find("<?=").unwrap() as u32;
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: None,
            anchor: Some(InsertionAnchor::BeforeEchoTag { offset: tag_offset }),
            ..ScopeState::default()
        };
        let mut edits = Vec::new();
        for i in 0..5 {
            let fqn = format!("Pkg\\Class{}", i);
            let r = tracker.import(fqn.as_bytes(), ImportKind::Name).unwrap();
            edits.push(r.use_statement_edit.unwrap());
        }
        assert!(edits.iter().all(|e| e.range.start == tag_offset));

        let out = stitch(src, &mut edits);
        for i in 0..5 {
            assert!(out.contains(&format!("<?php use Pkg\\Class{}; ?>", i)));
        }
        assert!(out.contains("<!DOCTYPE html>"));
        assert!(out.contains("<h1>Hi</h1>"));
        assert!(out.contains("<?= 'x' ?>"));
    }

    #[test]
    fn scope_state_default_is_empty() {
        let s = ScopeState::default();
        assert!(s.namespace.is_none());
        assert!(s.anchor.is_none());
        assert!(s.class_imports.is_empty());
        assert!(s.function_imports.is_empty());
        assert!(s.constant_imports.is_empty());
        assert!(s.local_classes.is_empty());
        assert!(s.local_functions.is_empty());
        assert!(s.local_constants.is_empty());
    }

    #[test]
    fn fresh_tracker_is_not_in_php() {
        let tracker = ImportTracker::new();
        assert!(!tracker.in_php);
    }

    #[test]
    fn use_statement_text_has_trailing_semicolon() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let r = tracker.import(b"X\\Y", ImportKind::Name).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        let trimmed_end = text.iter().rposition(|&b| !b.is_ascii_whitespace()).map_or(0, |i| i + 1);
        assert!(text[..trimmed_end].ends_with(b";"), "got `{}`", String::from_utf8_lossy(&text));
    }

    #[test]
    fn class_and_function_with_same_short_both_succeed() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let a = tracker.import(b"Foo\\Bar", ImportKind::Name).unwrap();
        let b = tracker.import(b"Foo\\bar", ImportKind::Function).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_some());
    }

    #[test]
    fn leading_slash_matches_unslashed_import() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"X"), word(b"X"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"X"), word(b"X"));
        let r = tracker.import(b"\\X", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_bytes(), b"X");
    }

    #[test]
    fn idempotent_import_does_not_emit_a_second_edit() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        let first = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        let second = tracker.import(b"A\\A", ImportKind::Name).unwrap();
        assert!(first.use_statement_edit.is_some());
        assert!(second.use_statement_edit.is_none());
    }

    #[test]
    fn hundred_distinct_imports_all_land_at_same_offset_and_stack() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        for i in 0..100 {
            let fqn = format!("Pkg\\T{:03}", i);
            let r = tracker.import(fqn.as_bytes(), ImportKind::Name).unwrap();
            assert_eq!(r.use_statement_edit.unwrap().range.start, 5);
        }
    }

    #[test]
    fn global_single_segment_fqn_no_edit() {
        let mut tracker = tracker_with_anchor(None, 10);
        let r = tracker.import(b"\\Foo", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_bytes(), b"Foo");
    }

    #[test]
    fn mixed_case_current_namespace_matches_import_case_insensitively() {
        let mut tracker = tracker_with_anchor(Some(b"APP"), 10);
        let r = tracker.import(b"app\\User", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_bytes(), b"User");
    }

    #[test]
    fn three_kinds_share_the_same_anchor_offset() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 5);
        let a = tracker.import(b"Pkg\\Foo", ImportKind::Name).unwrap();
        let b = tracker.import(b"Pkg\\foo", ImportKind::Function).unwrap();
        let c = tracker.import(b"Pkg\\FOO", ImportKind::Constant).unwrap();
        assert_eq!(a.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(b.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(c.use_statement_edit.unwrap().range.start, 5);
    }

    #[test]
    fn global_after_preamble_uses_double_newline_leading() {
        let mut tracker = tracker_with_anchor(None, 5);
        let r = tracker.import(b"Pkg\\Thing", ImportKind::Name).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert!(starts_with_bytes(&text, b"\n\n"));
        assert!(contains_bytes(&text, b"use Pkg\\Thing;"));
    }

    #[test]
    fn aliased_short_is_reported_as_local_name_not_fqn_last_segment() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_word(b"Alias1"), word(b"Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_word(b"Foo\\Bar"), word(b"Alias1"));
        let r = tracker.import(b"Foo\\Bar", ImportKind::Name).unwrap();
        assert_eq!(r.local_name.as_bytes(), b"Alias1");
        assert_ne!(r.local_name.as_bytes(), b"Bar");
    }

    #[test]
    fn unrelated_local_function_does_not_block_import() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        tracker.scope.local_functions.insert(ascii_lowercase_word(b"helper"));
        assert!(tracker.import(b"Other\\unrelated", ImportKind::Function).is_some());
    }

    #[test]
    fn split_fqn_global_single_segment_ns_is_none() {
        let (ns, short) = split_fqn(b"\\Foo").unwrap();
        assert!(ns.is_none());
        assert_eq!(short, b"Foo");
    }

    #[test]
    fn import_with_leading_slash_and_current_namespace_is_not_same_ns() {
        let mut tracker = tracker_with_anchor(Some(b"Foo"), 10);
        let r = tracker.import(b"\\Foo\\Bar", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_bytes(), b"Bar");
    }

    #[test]
    fn deeply_identical_namespace_and_fqn_aliases_correctly() {
        let mut tracker = tracker_with_anchor(Some(b"Foo\\Bar\\Baz"), 10);
        let r = tracker.import(b"Foo\\Bar\\Baz\\Qux", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_bytes(), b"Qux");
    }

    #[test]
    fn near_miss_namespace_still_requires_use() {
        let mut tracker = tracker_with_anchor(Some(b"Foo\\Bar"), 10);
        let r = tracker.import(b"Foo\\BarX\\Thing", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(r.local_name.as_bytes(), b"Thing");
    }

    #[test]
    fn constant_fqn_lookup_is_namespace_case_insensitive() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let a = tracker.import(b"App\\Other\\FOO", ImportKind::Constant).unwrap();
        let b = tracker.import(b"app\\OTHER\\FOO", ImportKind::Constant).unwrap();
        let c = tracker.import(b"APP\\other\\FOO", ImportKind::Constant).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_none());
        assert!(c.use_statement_edit.is_none());
        assert_eq!(a.local_name, b.local_name);
        assert_eq!(a.local_name, c.local_name);
    }

    #[test]
    fn constant_fqn_lookup_still_respects_short_name_case() {
        let mut tracker = tracker_with_anchor(Some(b"App"), 10);
        let a = tracker.import(b"Pkg\\FOO", ImportKind::Constant).unwrap();
        let b = tracker.import(b"Pkg\\foo", ImportKind::Constant).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_some());
        assert_ne!(a.local_name, b.local_name);
    }
}
