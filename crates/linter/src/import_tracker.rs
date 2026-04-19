//! File-level tracker that feeds [`LintContext::import_name`] and friends.

use foldhash::HashMap;
use foldhash::HashSet;

use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;
use mago_atom::ascii_lowercase_constant_name_atom;
use mago_atom::atom;
use mago_span::HasSpan;
use mago_syntax::ast::Constant;
use mago_syntax::ast::Declare;
use mago_syntax::ast::EchoTag;
use mago_syntax::ast::Namespace;
use mago_syntax::ast::NamespaceBody;
use mago_syntax::ast::Node;
use mago_syntax::ast::Use;
use mago_syntax::ast::UseItem;
use mago_syntax::ast::UseItems;
use mago_syntax::ast::UseType;
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
    pub local_name: Atom,
    pub use_statement_edit: Option<TextEdit>,
}

impl ImportResolution {
    /// True when no `use` statement needs to be added; either the FQN is
    /// already imported (possibly under an alias) or it resolves through the
    /// current namespace.
    #[inline]
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
    ///
    /// Multiple imports at the same anchor all land at the same offset; the
    /// relaxed `TextRange::overlaps` in `mago_text_edit` lets empty ranges
    /// at the same offset stack in insertion order. Every stacked `use` gets
    /// the full leading, so the formatter only has to squash repeated blank
    /// lines, not fabricate them.
    #[inline]
    fn leading(self) -> &'static str {
        match self {
            InsertionAnchor::AfterUse { .. } => "\n",
            InsertionAnchor::AfterPreamble { .. } => "\n\n",
            InsertionAnchor::BeforeEchoTag { .. } => "",
        }
    }

    /// True when the text must be wrapped in a self-contained `<?php ... ?>`
    /// block rather than emitted as a bare `use` statement.
    #[inline]
    fn wraps_in_php_block(self) -> bool {
        matches!(self, InsertionAnchor::BeforeEchoTag { .. })
    }
}

/// Mutable state scoped to the currently-active namespace block (implicit,
/// braced, or the implicit global scope of a bare-script file).
///
/// Imports, local declarations, and pending-edit bookkeeping are all reset when
/// a new namespace block opens; different blocks don't share use-table entries.
#[derive(Debug, Default)]
struct ScopeState {
    /// Current namespace name (normalised: no leading `\`), or `None` for the
    /// global scope.
    namespace: Option<Atom>,
    /// Current anchor for new `use` statements; `None` means we can't import
    /// right now (e.g. between two braced namespace blocks).
    anchor: Option<InsertionAnchor>,
    /// Case-insensitive short → FQN for class-like imports already in scope.
    class_imports: HashMap<Atom, Atom>,
    /// Case-insensitive short → FQN for function imports already in scope.
    function_imports: HashMap<Atom, Atom>,
    /// Case-sensitive short → FQN for constant imports already in scope.
    constant_imports: HashMap<Atom, Atom>,
    /// Reverse lookup: FQN (lowercased for class/function) → local short name,
    /// so that asking to import the same FQN twice returns the short name
    /// without re-emitting an edit.
    class_fqn_to_local: HashMap<Atom, Atom>,
    function_fqn_to_local: HashMap<Atom, Atom>,
    constant_fqn_to_local: HashMap<Atom, Atom>,
    /// Short names declared in this scope (class-likes, top-level functions,
    /// top-level consts). Compared case-insensitively for classes/functions,
    /// case-sensitively for constants.
    local_classes: HashSet<Atom>,
    local_functions: HashSet<Atom>,
    local_constants: HashSet<Atom>,
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
    /// Whether we've seen a `<?php` tag yet. Before the first open tag there's
    /// no valid place to insert a `use`.
    in_php: bool,
    /// Currently-active scope state. Reset whenever a namespace is entered or
    /// exited; in a "between braced namespaces" limbo the anchor inside this
    /// struct is `None`.
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
                self.scope.local_classes.insert(ascii_lowercase_atom(class.name.value));
            }
            Node::Interface(interface) => {
                self.scope.local_classes.insert(ascii_lowercase_atom(interface.name.value));
            }
            Node::Trait(r#trait) => {
                self.scope.local_classes.insert(ascii_lowercase_atom(r#trait.name.value));
            }
            Node::Enum(r#enum) => {
                self.scope.local_classes.insert(ascii_lowercase_atom(r#enum.name.value));
            }
            Node::Function(function) => {
                self.scope.local_functions.insert(ascii_lowercase_atom(function.name.value));
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
            namespace.name.as_ref().map(|id| id.value().trim_start_matches('\\')).filter(|n| !n.is_empty()).map(atom);

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
            self.scope.local_constants.insert(atom(item.name.value));
        }
    }

    fn enter_use(&mut self, r#use: &Use<'_>) {
        match &r#use.items {
            UseItems::Sequence(seq) => {
                for item in seq.items.iter() {
                    self.register_use_item(item, ImportKind::Name, None);
                }
            }
            UseItems::TypedSequence(typed) => {
                let kind = match &typed.r#type {
                    UseType::Function(_) => ImportKind::Function,
                    UseType::Const(_) => ImportKind::Constant,
                };

                for item in typed.items.iter() {
                    self.register_use_item(item, kind, None);
                }
            }
            UseItems::TypedList(list) => {
                let kind = match &list.r#type {
                    UseType::Function(_) => ImportKind::Function,
                    UseType::Const(_) => ImportKind::Constant,
                };

                let prefix = list.namespace.value().trim_start_matches('\\');
                for item in list.items.iter() {
                    self.register_use_item(item, kind, Some(prefix));
                }
            }
            UseItems::MixedList(list) => {
                let prefix = list.namespace.value().trim_start_matches('\\');
                for item in list.items.iter() {
                    let kind = match &item.r#type {
                        Some(UseType::Function(_)) => ImportKind::Function,
                        Some(UseType::Const(_)) => ImportKind::Constant,
                        None => ImportKind::Name,
                    };

                    self.register_use_item(&item.item, kind, Some(prefix));
                }
            }
        }

        self.scope.record_anchor_after_use(r#use.end_offset());
    }

    fn register_use_item(&mut self, item: &UseItem<'_>, kind: ImportKind, prefix: Option<&str>) {
        let raw = item.name.value().trim_start_matches('\\');
        let fqn = match prefix {
            Some(prefix) => atom(&format!("{}\\{}", prefix, raw)),
            None => atom(raw),
        };

        let local = match &item.alias {
            Some(alias) => atom(alias.identifier.value),
            None => atom(raw.rsplit('\\').next().unwrap_or(raw)),
        };

        match kind {
            ImportKind::Name => {
                let lookup = ascii_lowercase_atom(local.as_str());
                let fqn_key = ascii_lowercase_atom(fqn.as_str());
                self.scope.class_imports.insert(lookup, fqn);
                self.scope.class_fqn_to_local.insert(fqn_key, local);
            }
            ImportKind::Function => {
                let lookup = ascii_lowercase_atom(local.as_str());
                let fqn_key = ascii_lowercase_atom(fqn.as_str());
                self.scope.function_imports.insert(lookup, fqn);
                self.scope.function_fqn_to_local.insert(fqn_key, local);
            }
            ImportKind::Constant => {
                self.scope.constant_imports.insert(local, fqn);
                self.scope.constant_fqn_to_local.insert(ascii_lowercase_constant_name_atom(fqn.as_str()), local);
            }
        }
    }

    pub fn import(&mut self, fqn: &str, kind: ImportKind) -> Option<ImportResolution> {
        let (namespace_part, short_part) = split_fqn(fqn)?;
        if is_reserved_type_name(short_part, kind) {
            return None;
        }

        let short_atom = atom(short_part);
        let short_lookup = ascii_lowercase_atom(short_part);
        let full = match namespace_part.as_deref() {
            Some(ns) => atom(&format!("{}\\{}", ns, short_part)),
            None => atom(short_part),
        };

        let existing_reverse = match kind {
            ImportKind::Name => self.scope.class_fqn_to_local.get(&ascii_lowercase_atom(full.as_str())).copied(),
            ImportKind::Function => self.scope.function_fqn_to_local.get(&ascii_lowercase_atom(full.as_str())).copied(),
            ImportKind::Constant => self.scope.constant_fqn_to_local.get(&ascii_lowercase_constant_name_atom(full.as_str())).copied(),
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
                self.scope.constant_imports.contains_key(&short_atom)
                    || self.scope.local_constants.contains(&short_atom)
            }
        };

        if has_conflict {
            return None;
        }

        if let Some(ns) = namespace_part.as_deref()
            && let Some(current) = self.scope.namespace.as_deref()
            && ns.eq_ignore_ascii_case(current)
        {
            return Some(ImportResolution { local_name: short_atom, use_statement_edit: None });
        }

        if namespace_part.is_none() && self.scope.namespace.is_none() {
            return Some(ImportResolution { local_name: short_atom, use_statement_edit: None });
        }

        let anchor = self.scope.anchor?;
        let offset = anchor.offset();
        let leading = anchor.leading();
        let text = if anchor.wraps_in_php_block() {
            format!("{leading}<?php {}; ?>", render_use_statement(kind, full.as_str()))
        } else {
            format!("{leading}{};", render_use_statement(kind, full.as_str()))
        };

        match kind {
            ImportKind::Name => {
                self.scope.class_imports.insert(short_lookup, full);
                self.scope.class_fqn_to_local.insert(ascii_lowercase_atom(full.as_str()), short_atom);
            }
            ImportKind::Function => {
                self.scope.function_imports.insert(short_lookup, full);
                self.scope.function_fqn_to_local.insert(ascii_lowercase_atom(full.as_str()), short_atom);
            }
            ImportKind::Constant => {
                self.scope.constant_imports.insert(short_atom, full);
                self.scope.constant_fqn_to_local.insert(ascii_lowercase_constant_name_atom(full.as_str()), short_atom);
            }
        }

        Some(ImportResolution { local_name: short_atom, use_statement_edit: Some(TextEdit::insert(offset, text)) })
    }
}

fn render_use_statement(kind: ImportKind, fqn: &str) -> String {
    match kind {
        ImportKind::Name => format!("use {}", fqn),
        ImportKind::Function => format!("use function {}", fqn),
        ImportKind::Constant => format!("use const {}", fqn),
    }
}

/// Normalise an FQN input (`\Foo\Bar\Baz` or `Foo\Bar\Baz`) into (namespace, short).
///
/// Returns `None` for empty input or a lone namespace separator.
fn split_fqn(fqn: &str) -> Option<(Option<String>, &str)> {
    let trimmed = fqn.trim_start_matches('\\');
    if trimmed.is_empty() || trimmed.ends_with('\\') {
        return None;
    }

    match trimmed.rsplit_once('\\') {
        Some((ns, short)) if !ns.is_empty() && !short.is_empty() => Some((Some(ns.to_owned()), short)),
        _ => Some((None, trimmed)),
    }
}

/// PHP reserved type keywords that cannot appear after a `use` statement.
/// These are recognised at grammar level for class and function imports; for
/// constants the only reserved short names are literal identifiers like `true`
/// / `false` / `null` which PHP also refuses to define.
fn is_reserved_type_name(short: &str, kind: ImportKind) -> bool {
    const RESERVED_CLASS_NAMES: &[&str] = &[
        "int", "integer", "float", "double", "bool", "boolean", "string", "array", "object", "callable", "iterable",
        "mixed", "void", "never", "self", "static", "parent", "true", "false", "null",
    ];
    match kind {
        ImportKind::Name => RESERVED_CLASS_NAMES.iter().any(|r| short.eq_ignore_ascii_case(r)),
        ImportKind::Function => false,
        ImportKind::Constant => matches!(short.to_ascii_lowercase().as_str(), "true" | "false" | "null"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_fqn_rejects_empty_and_trailing_separator() {
        assert!(split_fqn("").is_none());
        assert!(split_fqn("\\").is_none());
        assert!(split_fqn("Foo\\").is_none());
    }

    #[test]
    fn split_fqn_single_segment() {
        let (ns, short) = split_fqn("Foo").unwrap();
        assert!(ns.is_none());
        assert_eq!(short, "Foo");

        let (ns, short) = split_fqn("\\Foo").unwrap();
        assert!(ns.is_none());
        assert_eq!(short, "Foo");
    }

    #[test]
    fn split_fqn_multi_segment() {
        let (ns, short) = split_fqn("\\App\\Http\\Controller").unwrap();
        assert_eq!(ns.as_deref(), Some("App\\Http"));
        assert_eq!(short, "Controller");
    }

    #[test]
    fn reserved_names_blocked_for_classes() {
        assert!(is_reserved_type_name("int", ImportKind::Name));
        assert!(is_reserved_type_name("INT", ImportKind::Name));
        assert!(is_reserved_type_name("self", ImportKind::Name));
        assert!(!is_reserved_type_name("User", ImportKind::Name));
    }

    #[test]
    fn reserved_names_allow_functions() {
        assert!(!is_reserved_type_name("int", ImportKind::Function));
        assert!(!is_reserved_type_name("strlen", ImportKind::Function));
    }

    #[test]
    fn reserved_names_for_constants() {
        assert!(is_reserved_type_name("true", ImportKind::Constant));
        assert!(is_reserved_type_name("NULL", ImportKind::Constant));
        assert!(!is_reserved_type_name("FOO", ImportKind::Constant));
    }

    fn tracker_with_anchor(namespace: Option<&str>, offset: u32) -> ImportTracker {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState {
            namespace: namespace.map(atom),
            anchor: Some(InsertionAnchor::AfterPreamble { offset }),
            ..ScopeState::default()
        };
        tracker
    }

    fn do_import(tracker: &mut ImportTracker, fqn: &str, kind: ImportKind) -> Option<ImportResolution> {
        tracker.import(fqn, kind)
    }

    #[test]
    fn import_before_open_tag_returns_none() {
        let mut tracker = ImportTracker::new();
        assert!(do_import(&mut tracker, "Foo\\Bar", ImportKind::Name).is_none());
    }

    #[test]
    fn import_adds_use_and_rewrites_short() {
        let mut tracker = tracker_with_anchor(Some("App"), 42);
        let resolution = tracker.import("Other\\Thing", ImportKind::Name).expect("should import");
        assert_eq!(resolution.local_name.as_str(), "Thing");
        let edit = resolution.use_statement_edit.expect("edit");
        assert!(edit.new_text.contains("use Other\\Thing;"));
    }

    #[test]
    fn import_same_fqn_twice_only_emits_one_edit() {
        let mut tracker = tracker_with_anchor(Some("App"), 42);
        let first = tracker.import("Other\\Thing", ImportKind::Name).unwrap();
        let second = tracker.import("Other\\Thing", ImportKind::Name).unwrap();
        assert!(first.use_statement_edit.is_some());
        assert!(second.use_statement_edit.is_none());
        assert_eq!(first.local_name, second.local_name);
    }

    #[test]
    fn import_fqn_in_current_namespace_no_edit() {
        let mut tracker = tracker_with_anchor(Some("App"), 42);
        let resolution = tracker.import("App\\User", ImportKind::Name).unwrap();
        assert_eq!(resolution.local_name.as_str(), "User");
        assert!(resolution.use_statement_edit.is_none());
    }

    #[test]
    fn import_single_segment_in_global_no_edit() {
        let mut tracker = tracker_with_anchor(None, 10);
        let resolution = tracker.import("Foo", ImportKind::Name).unwrap();
        assert!(resolution.use_statement_edit.is_none());
        assert_eq!(resolution.local_name.as_str(), "Foo");
    }

    #[test]
    fn import_single_segment_in_namespace_still_imports() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let resolution = tracker.import("Foo", ImportKind::Name).unwrap();
        assert!(resolution.use_statement_edit.is_some());
        assert_eq!(resolution.local_name.as_str(), "Foo");
    }

    #[test]
    fn short_name_conflict_with_existing_import_returns_none() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("User"), atom("Other\\User"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("Other\\User"), atom("User"));
        assert!(tracker.import("App\\User", ImportKind::Name).is_none());
    }

    #[test]
    fn short_name_conflict_with_local_class_returns_none() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.local_classes.insert(ascii_lowercase_atom("User"));
        assert!(tracker.import("Other\\User", ImportKind::Name).is_none());
    }

    #[test]
    fn aliased_existing_import_returns_alias_no_edit() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("Baz"), atom("Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("Foo\\Bar"), atom("Baz"));
        let resolution = tracker.import("Foo\\Bar", ImportKind::Name).unwrap();
        assert_eq!(resolution.local_name.as_str(), "Baz");
        assert!(resolution.use_statement_edit.is_none());
    }

    #[test]
    fn reserved_type_name_not_importable() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        assert!(tracker.import("\\int", ImportKind::Name).is_none());
        assert!(tracker.import("\\self", ImportKind::Name).is_none());
    }

    #[test]
    fn function_and_class_imports_do_not_collide() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("foo"), atom("Other\\foo"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("Other\\foo"), atom("foo"));
        let resolution = tracker.import("App\\Util\\foo", ImportKind::Function).expect("import");
        assert_eq!(resolution.local_name.as_str(), "foo");
        assert!(resolution.use_statement_edit.is_some());
    }

    #[test]
    fn constant_imports_are_case_sensitive() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.constant_imports.insert(atom("FOO"), atom("Other\\FOO"));
        tracker.scope.constant_fqn_to_local.insert(ascii_lowercase_constant_name_atom("Other\\FOO"), atom("FOO"));
        let resolution = tracker.import("Other\\foo", ImportKind::Constant).unwrap();
        assert_eq!(resolution.local_name.as_str(), "foo");
        assert!(resolution.use_statement_edit.is_some());
    }

    #[test]
    fn class_imports_are_case_insensitive() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("User"), atom("Other\\User"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("Other\\User"), atom("User"));
        assert!(tracker.import("App\\user", ImportKind::Name).is_none());
    }

    #[test]
    fn no_anchor_returns_none_even_if_otherwise_valid() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState { namespace: Some(atom("App")), anchor: None, ..ScopeState::default() };
        assert!(tracker.import("Other\\Thing", ImportKind::Name).is_none());
    }

    #[test]
    fn edit_text_leading_depends_on_anchor_kind() {
        let mut t1 = tracker_with_anchor(Some("App"), 10);
        t1.scope.anchor = Some(InsertionAnchor::AfterPreamble { offset: 10 });
        let r1 = t1.import("X\\Y", ImportKind::Name).unwrap();
        assert!(r1.use_statement_edit.unwrap().new_text.starts_with("\n\n"));

        let mut t2 = tracker_with_anchor(Some("App"), 10);
        t2.scope.anchor = Some(InsertionAnchor::AfterUse { offset: 10 });
        let r2 = t2.import("X\\Y", ImportKind::Name).unwrap();
        assert!(r2.use_statement_edit.unwrap().new_text.starts_with("\nuse"));
    }

    #[test]
    fn multiple_imports_all_land_at_the_same_anchor_offset() {
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        let r0 = tracker.import("A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import("B\\B", ImportKind::Name).unwrap();
        let r2 = tracker.import("C\\C", ImportKind::Name).unwrap();

        assert_eq!(r0.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(r1.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(r2.use_statement_edit.unwrap().range.start, 5);
    }

    #[test]
    fn stitched_multi_import_contains_all_uses_and_preserves_body() {
        let src = "<?php\n\n$x = 1;";
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        let r0 = tracker.import("A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import("B\\B", ImportKind::Name).unwrap();
        let r2 = tracker.import("C\\C", ImportKind::Name).unwrap();

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
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        let r0 = tracker.import("A\\A", ImportKind::Name).unwrap();
        assert_eq!(r0.use_statement_edit.unwrap().range.start, 5);

        tracker.scope.record_anchor_after_use(13);
        let r1 = tracker.import("B\\B", ImportKind::Name).unwrap();
        assert_eq!(r1.use_statement_edit.unwrap().range.start, 13);
    }

    #[test]
    fn forty_imports_all_stack_at_base_offset() {
        let src = "<?php\n\n$x = 1;";
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        let mut edits = Vec::new();
        for i in 0..40 {
            let fqn = format!("Pkg\\T{:02}", i);
            let r = tracker.import(&fqn, ImportKind::Name).unwrap();
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
        let r = tracker.import("Foo\\Bar", ImportKind::Name).unwrap();
        let edit = r.use_statement_edit.unwrap();
        assert_eq!(edit.range.start, 6);
        assert_eq!(edit.new_text, "<?php use Foo\\Bar; ?>");
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
        let r0 = tracker.import("A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import("B\\B", ImportKind::Name).unwrap();
        let r2 = tracker.import("C\\C", ImportKind::Name).unwrap();
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
        let r0 = tracker.import("A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import("B\\B", ImportKind::Name).unwrap();
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
        let r0 = tracker.import("A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import("B\\B", ImportKind::Name).unwrap();
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
        let r = tracker.import("Foo\\Bar", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(
            tracker.scope.anchor.map(|a| std::mem::discriminant(&a)),
            anchor_before.map(|a| std::mem::discriminant(&a))
        );
    }

    fn stitch(src: &str, edits: &mut [TextEdit]) -> String {
        edits.sort_by_key(|e| e.range.start);
        let mut output = String::new();
        let mut cursor: u32 = 0;
        for edit in edits.iter() {
            output.push_str(&src[cursor as usize..edit.range.start as usize]);
            output.push_str(&edit.new_text);
            cursor = edit.range.end;
        }
        output.push_str(&src[cursor as usize..]);
        output
    }

    #[test]
    fn stitched_output_preserves_unicode_body() {
        let src = "<?php\n\n// héllo\n$x;";
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        let r0 = tracker.import("A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import("B\\B", ImportKind::Name).unwrap();
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
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.local_classes.insert(ascii_lowercase_atom("Foo"));
        assert!(tracker.import("Other\\foo", ImportKind::Function).is_some());
        assert!(tracker.import("Other\\FOO", ImportKind::Constant).is_some());
    }

    #[test]
    fn local_function_does_not_block_class_or_constant_imports() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.local_functions.insert(ascii_lowercase_atom("foo"));
        assert!(tracker.import("Other\\Foo", ImportKind::Name).is_some());
        assert!(tracker.import("Other\\FOO", ImportKind::Constant).is_some());
    }

    #[test]
    fn local_constant_does_not_block_class_or_function_imports() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.local_constants.insert(atom("FOO"));
        assert!(tracker.import("Other\\Foo", ImportKind::Name).is_some());
        assert!(tracker.import("Other\\foo", ImportKind::Function).is_some());
    }

    #[test]
    fn reserved_names_cover_all_scalar_and_compound_types() {
        for name in [
            "int", "integer", "float", "double", "bool", "boolean", "string", "array", "object", "callable",
            "iterable", "mixed", "void", "never", "self", "static", "parent", "true", "false", "null",
        ] {
            assert!(is_reserved_type_name(name, ImportKind::Name), "expected `{name}` reserved");
            assert!(is_reserved_type_name(&name.to_ascii_uppercase(), ImportKind::Name));
        }
    }

    #[test]
    fn fqn_with_leading_backslash_matches_imported_form() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("Bar"), atom("Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("Foo\\Bar"), atom("Bar"));

        let r = tracker.import("\\Foo\\Bar", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_str(), "Bar");
    }

    #[test]
    fn fqn_with_multiple_segments_shortens_to_last() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let r = tracker.import("A\\B\\C\\D\\E", ImportKind::Name).unwrap();
        assert_eq!(r.local_name.as_str(), "E");
        assert!(r.use_statement_edit.as_ref().unwrap().new_text.contains("use A\\B\\C\\D\\E;"));
    }

    #[test]
    fn forty_imports_stitch_cleanly_at_shared_offset() {
        let src = "<?php\n\n$x = 1;";
        let mut tracker = tracker_with_anchor(Some("App"), 5);

        let mut edits: Vec<TextEdit> = Vec::new();
        let mut names: Vec<String> = Vec::new();
        for i in 0..40 {
            let fqn = format!("Pkg\\Class{:02}", i);
            let r = tracker.import(&fqn, ImportKind::Name).unwrap();
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
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        for i in 0..41 {
            let fqn = format!("Pkg\\T{:02}", i);
            let r = tracker.import(&fqn, ImportKind::Name).unwrap();
            assert_eq!(r.use_statement_edit.unwrap().range.start, 5);
        }
    }

    #[test]
    fn function_import_emits_use_function_keyword() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let r = tracker.import("Other\\strlen", ImportKind::Function).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert!(text.contains("use function Other\\strlen;"), "got `{text}`");
        assert!(!text.contains("use const"));
    }

    #[test]
    fn constant_import_emits_use_const_keyword() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let r = tracker.import("Other\\PHP_EOL", ImportKind::Constant).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert!(text.contains("use const Other\\PHP_EOL;"), "got `{text}`");
        assert!(!text.contains("use function"));
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
        let r = tracker.import("Other\\strlen", ImportKind::Function).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert_eq!(text, "<?php use function Other\\strlen; ?>");
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
        let r = tracker.import("Other\\PHP_EOL", ImportKind::Constant).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert_eq!(text, "<?php use const Other\\PHP_EOL; ?>");
    }

    #[test]
    fn two_constant_imports_with_different_cases_coexist() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let a = tracker.import("Other\\FOO", ImportKind::Constant).unwrap();
        let b = tracker.import("Other\\foo", ImportKind::Constant).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_some());
        assert_ne!(a.local_name.as_str(), b.local_name.as_str());
    }

    #[test]
    fn anchor_after_use_is_where_next_import_lands() {
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        tracker.scope.record_anchor_after_use(30);
        let r = tracker.import("X\\Y", ImportKind::Name).unwrap();
        let edit = r.use_statement_edit.unwrap();
        assert_eq!(edit.range.start, 30);
        assert!(edit.new_text.starts_with("\nuse "));
    }

    #[test]
    fn second_import_at_after_use_anchor_stacks_at_same_offset() {
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        tracker.scope.record_anchor_after_use(12);
        let r0 = tracker.import("A\\A", ImportKind::Name).unwrap();
        let r1 = tracker.import("B\\B", ImportKind::Name).unwrap();
        assert_eq!(r0.use_statement_edit.unwrap().range.start, 12);
        assert_eq!(r1.use_statement_edit.unwrap().range.start, 12);
    }

    #[test]
    fn very_long_fqn_preserves_full_path_in_use_text() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let fqn = "A\\B\\C\\D\\E\\F\\G\\H\\I\\J\\K\\Target";
        let r = tracker.import(fqn, ImportKind::Name).unwrap();
        assert_eq!(r.local_name.as_str(), "Target");
        assert!(r.use_statement_edit.unwrap().new_text.contains("use A\\B\\C\\D\\E\\F\\G\\H\\I\\J\\K\\Target;"));
    }

    #[test]
    fn namespace_reset_drops_imports_from_previous_namespace() {
        let mut tracker = tracker_with_anchor(Some("A"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("Foo"), atom("Other\\Foo"));
        tracker.scope = ScopeState {
            namespace: Some(atom("B")),
            anchor: Some(InsertionAnchor::AfterPreamble { offset: 50 }),
            ..ScopeState::default()
        };
        let r = tracker.import("Other\\Foo", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
    }

    #[test]
    fn split_fqn_rejects_only_backslashes() {
        assert!(split_fqn("\\\\\\\\").is_none());
        assert!(split_fqn("\\").is_none());
    }

    #[test]
    fn split_fqn_trailing_separator_rejected() {
        assert!(split_fqn("Foo\\").is_none());
        assert!(split_fqn("A\\B\\C\\").is_none());
        assert!(split_fqn("\\A\\B\\").is_none());
    }

    #[test]
    fn import_empty_fqn_returns_none() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        assert!(tracker.import("", ImportKind::Name).is_none());
        assert!(tracker.import("\\", ImportKind::Name).is_none());
        assert!(tracker.import("Foo\\", ImportKind::Name).is_none());
    }

    #[test]
    fn anchor_offset_past_source_end_does_not_panic() {
        let mut tracker = tracker_with_anchor(Some("App"), 9999);
        let r = tracker.import("Other\\Thing", ImportKind::Name).unwrap();
        assert_eq!(r.use_statement_edit.unwrap().range.start, 9999);
    }

    #[test]
    fn split_fqn_keeps_consecutive_backslashes_in_namespace() {
        let (ns, short) = split_fqn("Foo\\\\Bar").unwrap();
        assert_eq!(ns.as_deref(), Some("Foo\\"));
        assert_eq!(short, "Bar");
    }

    #[test]
    fn extremely_deep_namespace_does_not_panic() {
        let segments: Vec<String> = (0..100).map(|i| format!("N{:03}", i)).collect();
        let fqn = segments.join("\\") + "\\Target";
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let r = tracker.import(&fqn, ImportKind::Name).unwrap();
        assert_eq!(r.local_name.as_str(), "Target");
        let text = r.use_statement_edit.unwrap().new_text;
        assert!(text.contains("N000\\"));
        assert!(text.contains("\\N099\\Target;"));
    }

    #[test]
    fn short_name_matching_current_namespace_is_just_a_short_name() {
        let mut tracker = tracker_with_anchor(Some("Foo"), 10);
        let r = tracker.import("Other\\Foo", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(r.local_name.as_str(), "Foo");
    }

    #[test]
    fn self_namespace_import_is_already_available() {
        let mut tracker = tracker_with_anchor(Some("App\\Sub"), 10);
        let r = tracker.import("App\\Sub\\Thing", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_str(), "Thing");
    }

    #[test]
    fn sub_namespace_of_current_still_needs_use() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let r = tracker.import("App\\Sub\\Thing", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(r.local_name.as_str(), "Thing");
    }

    #[test]
    fn parent_namespace_import_requires_use() {
        let mut tracker = tracker_with_anchor(Some("App\\Sub"), 10);
        let r = tracker.import("App\\Thing", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(r.local_name.as_str(), "Thing");
    }

    #[test]
    fn same_fqn_three_kinds_three_edits() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let a = tracker.import("Other\\foo", ImportKind::Name).unwrap();
        let b = tracker.import("Other\\foo", ImportKind::Function).unwrap();
        let c = tracker.import("Other\\foo", ImportKind::Constant).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_some());
        assert!(c.use_statement_edit.is_some());
    }

    #[test]
    fn case_variant_of_same_class_fqn_deduplicates() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let a = tracker.import("foo\\bar", ImportKind::Name).unwrap();
        let b = tracker.import("Foo\\Bar", ImportKind::Name).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_none());
    }

    #[test]
    fn alias_matching_another_short_name_blocks_future_import() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("Foo"), atom("X\\Y"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("X\\Y"), atom("Foo"));
        assert!(tracker.import("Other\\Foo", ImportKind::Name).is_none());
    }

    #[test]
    fn conflict_uses_alias_short_not_fqn_suffix() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("Bar"), atom("X\\Y\\Z"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("X\\Y\\Z"), atom("Bar"));
        assert!(tracker.import("Other\\Bar", ImportKind::Name).is_none());
        assert!(tracker.import("Other\\Baz", ImportKind::Name).is_some());
    }

    #[test]
    fn imports_follow_anchor_as_it_moves_between_kinds() {
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        let a = tracker.import("A\\A", ImportKind::Name).unwrap();
        let b = tracker.import("B\\B", ImportKind::Name).unwrap();
        assert_eq!(a.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(b.use_statement_edit.unwrap().range.start, 5);
        tracker.scope.record_anchor_after_use(15);
        let c = tracker.import("C\\C", ImportKind::Name).unwrap();
        let d = tracker.import("D\\D", ImportKind::Name).unwrap();
        assert_eq!(c.use_statement_edit.unwrap().range.start, 15);
        assert_eq!(d.use_statement_edit.unwrap().range.start, 15);
    }

    #[test]
    fn conflict_check_runs_before_current_namespace_shortcut() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("User"), atom("Other\\User"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("Other\\User"), atom("User"));
        assert!(tracker.import("App\\User", ImportKind::Name).is_none());
    }

    #[test]
    fn conflict_check_runs_before_global_shortcut() {
        let mut tracker = tracker_with_anchor(None, 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("Bar"), atom("Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("Foo\\Bar"), atom("Bar"));
        assert!(tracker.import("Bar", ImportKind::Name).is_none());
    }

    #[test]
    fn aliased_import_survives_repeated_queries() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("Baz"), atom("Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("Foo\\Bar"), atom("Baz"));
        for _ in 0..5 {
            let r = tracker.import("Foo\\Bar", ImportKind::Name).unwrap();
            assert_eq!(r.local_name.as_str(), "Baz");
            assert!(r.use_statement_edit.is_none());
        }
    }

    #[test]
    fn function_name_collides_with_function_not_with_class() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.function_imports.insert(ascii_lowercase_atom("strlen"), atom("Other\\strlen"));
        tracker.scope.function_fqn_to_local.insert(ascii_lowercase_atom("Other\\strlen"), atom("strlen"));
        assert!(tracker.import("Third\\strlen", ImportKind::Function).is_none());
        assert!(tracker.import("Third\\Strlen", ImportKind::Name).is_some());
    }

    #[test]
    fn reserved_names_do_not_block_function_imports() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        assert!(tracker.import("Other\\int", ImportKind::Function).is_some());
        assert!(tracker.import("Other\\void", ImportKind::Function).is_some());
    }

    #[test]
    fn reserved_constant_names_only_cover_scalar_literals() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        assert!(tracker.import("Other\\int", ImportKind::Constant).is_some());
        assert!(tracker.import("Other\\void", ImportKind::Constant).is_some());
        assert!(tracker.import("Other\\TRUE", ImportKind::Constant).is_none());
        assert!(tracker.import("Other\\False", ImportKind::Constant).is_none());
    }

    #[test]
    fn no_anchor_no_imports_no_panics() {
        let mut tracker = ImportTracker::new();
        tracker.in_php = true;
        tracker.scope = ScopeState { anchor: None, ..ScopeState::default() };
        for kind in [ImportKind::Name, ImportKind::Function, ImportKind::Constant] {
            assert!(tracker.import("X\\Y", kind).is_none());
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
        let r = tracker.import("Foo\\Bar", ImportKind::Name).unwrap();
        let edit = r.use_statement_edit.unwrap();
        assert_eq!(edit.range.start, 0);
        assert_eq!(edit.new_text, "<?php use Foo\\Bar; ?>");
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
        let a = tracker.import("Foo\\Bar", ImportKind::Name).unwrap();
        let b = tracker.import("Baz\\Qux", ImportKind::Name).unwrap();
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
            let r = tracker.import(&fqn, ImportKind::Name).unwrap();
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
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let r = tracker.import("X\\Y", ImportKind::Name).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert!(text.trim_end().ends_with(';'), "got `{text}`");
    }

    #[test]
    fn class_and_function_with_same_short_both_succeed() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let a = tracker.import("Foo\\Bar", ImportKind::Name).unwrap();
        let b = tracker.import("Foo\\bar", ImportKind::Function).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_some());
    }

    #[test]
    fn leading_slash_matches_unslashed_import() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("X"), atom("X"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("X"), atom("X"));
        let r = tracker.import("\\X", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_str(), "X");
    }

    #[test]
    fn idempotent_import_does_not_emit_a_second_edit() {
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        let first = tracker.import("A\\A", ImportKind::Name).unwrap();
        let second = tracker.import("A\\A", ImportKind::Name).unwrap();
        assert!(first.use_statement_edit.is_some());
        assert!(second.use_statement_edit.is_none());
    }

    #[test]
    fn hundred_distinct_imports_all_land_at_same_offset_and_stack() {
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        for i in 0..100 {
            let fqn = format!("Pkg\\T{:03}", i);
            let r = tracker.import(&fqn, ImportKind::Name).unwrap();
            assert_eq!(r.use_statement_edit.unwrap().range.start, 5);
        }
    }

    #[test]
    fn global_single_segment_fqn_no_edit() {
        let mut tracker = tracker_with_anchor(None, 10);
        let r = tracker.import("\\Foo", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_str(), "Foo");
    }

    #[test]
    fn mixed_case_current_namespace_matches_import_case_insensitively() {
        let mut tracker = tracker_with_anchor(Some("APP"), 10);
        let r = tracker.import("app\\User", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_str(), "User");
    }

    #[test]
    fn three_kinds_share_the_same_anchor_offset() {
        let mut tracker = tracker_with_anchor(Some("App"), 5);
        let a = tracker.import("Pkg\\Foo", ImportKind::Name).unwrap();
        let b = tracker.import("Pkg\\foo", ImportKind::Function).unwrap();
        let c = tracker.import("Pkg\\FOO", ImportKind::Constant).unwrap();
        assert_eq!(a.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(b.use_statement_edit.unwrap().range.start, 5);
        assert_eq!(c.use_statement_edit.unwrap().range.start, 5);
    }

    #[test]
    fn global_after_preamble_uses_double_newline_leading() {
        let mut tracker = tracker_with_anchor(None, 5);
        let r = tracker.import("Pkg\\Thing", ImportKind::Name).unwrap();
        let text = r.use_statement_edit.unwrap().new_text;
        assert!(text.starts_with("\n\n"));
        assert!(text.contains("use Pkg\\Thing;"));
    }

    #[test]
    fn aliased_short_is_reported_as_local_name_not_fqn_last_segment() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.class_imports.insert(ascii_lowercase_atom("Alias1"), atom("Foo\\Bar"));
        tracker.scope.class_fqn_to_local.insert(ascii_lowercase_atom("Foo\\Bar"), atom("Alias1"));
        let r = tracker.import("Foo\\Bar", ImportKind::Name).unwrap();
        assert_eq!(r.local_name.as_str(), "Alias1");
        assert_ne!(r.local_name.as_str(), "Bar");
    }

    #[test]
    fn unrelated_local_function_does_not_block_import() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        tracker.scope.local_functions.insert(ascii_lowercase_atom("helper"));
        assert!(tracker.import("Other\\unrelated", ImportKind::Function).is_some());
    }

    #[test]
    fn split_fqn_global_single_segment_ns_is_none() {
        let (ns, short) = split_fqn("\\Foo").unwrap();
        assert!(ns.is_none());
        assert_eq!(short, "Foo");
    }

    #[test]
    fn split_fqn_does_not_allocate_short_but_does_allocate_ns() {
        let fqn_owned = String::from("A\\B\\C");
        let (ns, short) = split_fqn(&fqn_owned).unwrap();
        assert_eq!(ns.as_deref(), Some("A\\B"));
        assert_eq!(short, "C");
        assert!(fqn_owned.as_str().as_ptr() as usize <= short.as_ptr() as usize);
        assert!((short.as_ptr() as usize) < fqn_owned.as_str().as_ptr() as usize + fqn_owned.len());
    }

    #[test]
    fn import_with_leading_slash_and_current_namespace_is_not_same_ns() {
        let mut tracker = tracker_with_anchor(Some("Foo"), 10);
        let r = tracker.import("\\Foo\\Bar", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_str(), "Bar");
    }

    #[test]
    fn deeply_identical_namespace_and_fqn_aliases_correctly() {
        let mut tracker = tracker_with_anchor(Some("Foo\\Bar\\Baz"), 10);
        let r = tracker.import("Foo\\Bar\\Baz\\Qux", ImportKind::Name).unwrap();
        assert!(r.is_already_available());
        assert_eq!(r.local_name.as_str(), "Qux");
    }

    #[test]
    fn near_miss_namespace_still_requires_use() {
        let mut tracker = tracker_with_anchor(Some("Foo\\Bar"), 10);
        let r = tracker.import("Foo\\BarX\\Thing", ImportKind::Name).unwrap();
        assert!(r.use_statement_edit.is_some());
        assert_eq!(r.local_name.as_str(), "Thing");
    }

    #[test]
    fn constant_fqn_lookup_is_namespace_case_insensitive() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let a = tracker.import("App\\Other\\FOO", ImportKind::Constant).unwrap();
        let b = tracker.import("app\\OTHER\\FOO", ImportKind::Constant).unwrap();
        let c = tracker.import("APP\\other\\FOO", ImportKind::Constant).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_none());
        assert!(c.use_statement_edit.is_none());
        assert_eq!(a.local_name, b.local_name);
        assert_eq!(a.local_name, c.local_name);
    }

    #[test]
    fn constant_fqn_lookup_still_respects_short_name_case() {
        let mut tracker = tracker_with_anchor(Some("App"), 10);
        let a = tracker.import("Pkg\\FOO", ImportKind::Constant).unwrap();
        let b = tracker.import("Pkg\\foo", ImportKind::Constant).unwrap();
        assert!(a.use_statement_edit.is_some());
        assert!(b.use_statement_edit.is_some());
        assert_ne!(a.local_name, b.local_name);
    }

}
