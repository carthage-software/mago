//! Transport-agnostic value types the server speaks.
//!
//! These types carry no protocol semantics: positions are byte offsets, never
//! lines/columns or UTF-16. The protocol layer converts to and from its own
//! position model. Capability-specific result types are added here as each
//! capability is ported onto the server.

use foldhash::HashMap;

use mago_bytes::BytesDisplay;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_database::file::FileId;
use mago_span::Span;
use mago_word::Word;

use crate::matcher::Match;

/// A half-open byte range `[start, end)` into a file's raw contents.
///
/// Both bounds are zero-based byte offsets (`u32`) into the file's bytes; not
/// character indices and not UTF-16 code units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Range {
    /// Inclusive start byte offset.
    pub start: u32,
    /// Exclusive end byte offset.
    pub end: u32,
}

impl Range {
    /// Create a range from a half-open `[start, end)` byte range.
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// The length of the range in bytes (`0` if the bounds are inverted).
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Whether the range covers no bytes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Whether `offset` falls within the half-open range `[start, end)`.
    #[must_use]
    pub const fn contains(&self, offset: u32) -> bool {
        self.start <= offset && offset < self.end
    }
}

/// What a [`FoldRange`] folds, when the editor distinguishes kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoldKind {
    /// A comment block.
    Comment,
    /// A generic collapsible region (the default for code blocks).
    Region,
}

/// A symbol's location: the file it lives in and its byte range. Used for
/// go-to-definition results and reference sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolLocation {
    /// The file containing the symbol.
    pub file: FileId,
    /// The byte range of the symbol within that file.
    pub range: Range,
}

/// Hover descriptions and the range they apply to.
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// One entry per distinct symbol under the cursor.
    pub entries: Vec<HoverEntry>,
    /// The byte range of the hovered token.
    pub range: Range,
}

/// A symbol's scope and declaration, without client markup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverEntry {
    /// Namespace for a symbol, or fully qualified class for a member.
    pub scope: Option<String>,
    pub declaration: String,
    /// Source PHPDoc and its parsed tag headers.
    pub documentation: Option<SourceDocumentation>,
}

impl HoverEntry {
    /// Describe a symbol under its namespace, if any.
    #[must_use]
    pub fn for_symbol(name: &[u8], declaration: String) -> Self {
        let scope = name.iter().rposition(|byte| *byte == b'\\').map(|end| BytesDisplay(&name[..end]).to_string());
        Self { scope, declaration, documentation: None }
    }

    /// Describe a member under the class that declares or exposes it.
    #[must_use]
    pub fn in_class(class: &[u8], declaration: String) -> Self {
        Self { scope: Some(BytesDisplay(class).to_string()), declaration, documentation: None }
    }
}

/// Source documentation with parsed tag headers; all other text stays unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDocumentation {
    pub text: String,
    pub tags: Vec<DocumentationTag>,
}

/// A parsed tag header within [`SourceDocumentation::text`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentationTag {
    pub range: Range,
    pub kind: DocumentationTagKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentationTagKind {
    Parameter { parameter: Option<String>, type_name: Option<String> },
    Returns { type_name: String },
    Throws { type_name: String },
    Deprecated,
}

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

/// A diagnostic: a located, severity-tagged message. Used both for published
/// diagnostics and for the diagnostic a code action resolves.
#[derive(Debug, Clone)]
pub struct DiagnosticData {
    /// The file the diagnostic is anchored in.
    pub file: FileId,
    /// The byte range of the diagnostic.
    pub range: Range,
    /// The diagnostic's severity.
    pub severity: Severity,
    /// An optional machine-readable code.
    pub code: Option<String>,
    /// The human-readable message.
    pub message: String,
}

/// A single text replacement: new text for a byte range.
#[derive(Debug, Clone)]
pub struct TextReplacement {
    /// The byte range to replace.
    pub range: Range,
    /// The replacement text.
    pub new_text: String,
}

/// A code action: a titled set of per-file edits, optionally linked
/// to the diagnostic it resolves.
#[derive(Debug, Clone)]
pub struct CodeActionItem {
    /// The action's title shown to the user.
    pub title: String,
    /// The kind of change this action makes.
    pub kind: CodeActionKind,
    /// Edits to apply, grouped by file.
    pub edits: HashMap<FileId, Vec<TextReplacement>>,
    /// The diagnostic this action resolves, if any.
    pub diagnostic: Option<DiagnosticData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeActionKind {
    QuickFix,
    RefactorRewrite,
    RefactorExtract,
}

impl CodeActionKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::QuickFix => "quickfix",
            Self::RefactorRewrite => "refactor.rewrite",
            Self::RefactorExtract => "refactor.extract",
        }
    }

    pub(crate) fn requested(self, only: Option<&[&str]>) -> bool {
        only.is_none_or(|kinds| {
            kinds.iter().any(|kind| {
                kind.is_empty()
                    || self.as_str() == *kind
                    || self.as_str().strip_prefix(*kind).is_some_and(|tail| tail.starts_with('.'))
            })
        })
    }
}

/// The kind of a completion entry (drives the editor's icon).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Snippet,
    Keyword,
    Variable,
    Function,
    Constant,
    Class,
    Interface,
    Enum,
    EnumMember,
    Field,
    Method,
}

/// A name to insert, with optional call arguments supplied by the editor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompletionInsert {
    Text(String),
    Call(String),
    /// A declaration with LSP snippet placeholders; omit it for plain-text clients.
    Snippet(String),
}

/// One completion candidate. Positions are byte offsets; the protocol layer
/// turns `replace` into a text edit and maps the other fields to its item type.
#[derive(Debug, Clone)]
pub struct CompletionEntry {
    /// The label shown in the completion list (and inserted, unless overridden).
    pub label: String,
    /// The kind of symbol.
    pub kind: CompletionKind,
    /// Extra detail shown beside the label (e.g. a signature).
    pub detail: Option<String>,
    /// Namespace shown beside a class-like name; `\` means the global namespace.
    pub namespace: Option<String>,
    /// Source PHPDoc shown in the detail pane.
    pub documentation: Option<SourceDocumentation>,
    /// Read source docs after ranking and limiting candidates.
    pub(crate) documentation_source: Option<CompletionDocumentation>,
    /// Distinct declarations of one member across possible receiver classes.
    pub member_alternatives: Option<Box<[CompletionAlternative]>>,
    /// Override the label or offer a call insertion.
    pub insert: Option<CompletionInsert>,
    /// A byte range to replace with the label (used by variables so the leading
    /// `$` is rewritten); `None` means insert at the cursor.
    pub replace: Option<Range>,
    /// Text the editor filters against (defaults to the label).
    pub filter_text: Option<String>,
    /// Select this entry by default; set on a unique exact match.
    pub preselect: bool,
    /// The symbol is deprecated.
    pub deprecated: bool,
}

impl CompletionEntry {
    /// A bare entry with just a label and kind; other fields default to empty.
    #[must_use]
    pub fn new(label: String, kind: CompletionKind) -> Self {
        Self {
            label,
            kind,
            detail: None,
            namespace: None,
            documentation: None,
            documentation_source: None,
            member_alternatives: None,
            insert: None,
            replace: None,
            filter_text: None,
            preselect: false,
            deprecated: false,
        }
    }
}

/// One declaration behind a shared member completion.
#[derive(Debug, Clone)]
pub struct CompletionAlternative {
    /// The declaring class, signature, and source docs.
    pub description: HoverEntry,
    pub(crate) documentation_source: Option<CompletionDocumentation>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum CompletionDocumentation {
    Callable(FunctionLikeIdentifier),
    Declaration(Span),
    DeclarationName(Span),
    Parameter { callable: FunctionLikeIdentifier, name: Word },
    Property { class: Word, name: Word },
    MagicProperty { class: Word, name: Word },
}

/// A ranked list of completion candidates.
#[derive(Debug, Clone, Default)]
pub struct CompletionList {
    /// The candidates, already ordered.
    pub items: Vec<CompletionEntry>,
    /// Whether the list is incomplete (the editor should re-query as the user types).
    pub is_incomplete: bool,
}

/// A reformatted whole document. The protocol layer turns this into a single
/// full-file text edit.
#[derive(Debug, Clone)]
pub struct FormattedDocument {
    /// The complete reformatted file contents.
    pub new_text: String,
}

/// The kind of a document or workspace symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Class,
    Interface,
    Trait,
    Enum,
    Function,
    Constant,
    Namespace,
    Method,
    Constructor,
    Property,
    EnumMember,
}

/// A declaration in a document outline, with its nested declarations.
#[derive(Debug, Clone)]
pub struct DocumentSymbolItem {
    /// Name shown in the outline.
    pub name: String,
    pub kind: SymbolKind,
    /// Full declaration span, including its body.
    pub range: Range,
    /// Name span to select when navigating to the declaration.
    pub selection_range: Range,
    pub children: Vec<Self>,
}

/// A workspace-symbol-search match: a name, its kind, and where it's declared.
#[derive(Debug, Clone)]
pub struct WorkspaceSymbolItem {
    /// The symbol's display name; members read `Class::member`.
    pub name: String,
    /// What kind of symbol it is.
    pub kind: SymbolKind,
    /// Where the symbol is declared.
    pub location: SymbolLocation,
    /// The class-like that declares a member.
    pub container: Option<String>,
    /// How well the symbol matched the query, comparable across workspaces.
    pub rank: SymbolRank,
}

/// Workspace symbol relevance; lower ranks sort first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolRank {
    pub(crate) weak: bool,
    pub(crate) library: bool,
    pub(crate) test: bool,
    pub(crate) partial_qualifier: bool,
    pub(crate) matched: Match,
    pub(crate) kind: u8,
}

/// A declaration whose code lens can resolve its references on demand.
#[derive(Debug, Clone, Copy)]
pub struct CodeLensItem {
    /// Byte offset the lens is anchored at.
    pub offset: u32,
}

/// What an [`InlayHintItem`] annotates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintKind {
    /// A parameter-name hint before an argument.
    Parameter,
    /// An inferred-type hint.
    Type,
}

/// An inlay hint: a short label rendered at a byte offset in the file.
#[derive(Debug, Clone)]
pub struct InlayHintItem {
    /// Byte offset the hint is anchored at.
    pub offset: u32,
    /// The hint text (e.g. `name:`).
    pub label: String,
    /// What kind of hint this is.
    pub kind: HintKind,
}

/// A selection-range chain for one cursor: nested ranges ordered innermost
/// (smallest) to outermost (largest).
#[derive(Debug, Clone)]
pub struct SelectionRangeItem {
    /// Ranges from innermost to outermost.
    pub ranges: Vec<Range>,
}

/// A clickable link from an import to its declaration.
#[derive(Debug, Clone)]
pub struct DocumentLinkItem {
    /// The byte range of the link's source text in the current file.
    pub range: Range,
    /// The declaration the link targets.
    pub target: SymbolLocation,
    /// A short tooltip (the resolved symbol name).
    pub tooltip: String,
}

/// Signature help for a call site: a rendered signature label, the byte ranges
/// of each parameter *within that label*, and the active parameter index.
#[derive(Debug, Clone)]
pub struct SignatureInfo {
    pub documentation: Option<SourceDocumentation>,
    /// The full signature label, e.g. `function f(int $a, string $b): void`.
    pub label: String,
    /// `(start, end)` byte ranges of each parameter within `label`.
    pub parameters: Vec<Range>,
    /// Index of the parameter the cursor is currently on.
    pub active_parameter: u32,
}

/// A collapsible source region, as a half-open byte range `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FoldRange {
    /// Byte range of the foldable region.
    pub range: Range,
    /// The kind of fold, if the region has a distinguished one.
    pub kind: Option<FoldKind>,
}

/// Why a rename was declined, and where the blocking construct is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenameRefusal {
    pub reason: RefusalReason,
    /// The construct that blocks the rename, when it has a source location.
    pub location: Option<SymbolLocation>,
}

impl RenameRefusal {
    #[must_use]
    pub const fn new(reason: RefusalReason) -> Self {
        Self { reason, location: None }
    }

    #[must_use]
    pub const fn at(reason: RefusalReason, file: FileId, range: Range) -> Self {
        Self { reason, location: Some(SymbolLocation { file, range }) }
    }
}

/// The check that declined a rename, without client wording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalReason {
    /// Nothing renameable is under the cursor.
    NoSymbol,
    /// Namespace names and namespace imports keep their names.
    Namespace {
        import: bool,
    },
    /// The cursor is on the namespace part of a qualified name.
    NamespaceSegment,
    InvalidName(NameProblem),
    /// The edit would change a source outside the project.
    ReadOnly,
    /// The symbol is declared in PHP's builtin stubs.
    Builtin,
    /// PHP generates the method for an enum that the rename would affect.
    EnumMethod,
    /// The symbol is declared outside the project sources.
    External,
    /// The symbol has no declaration.
    Undeclared,
    /// The selected method is a magic method.
    MagicMethod,
    /// A use goes through a magic method or virtual property.
    Magic,
    /// A use selects its member dynamically.
    Dynamic,
    /// A use's target cannot be resolved.
    Unresolved,
    /// A use may also reach a member outside the renamed family.
    SharedTarget,
    /// An affected class depends on this undeclared name, spelled as written for a direct ancestor.
    IncompleteHierarchy(Word),
    /// A declaration already uses the new name.
    NameTaken,
    /// A variable in an affected scope already uses the new name.
    BindingTaken,
    /// An import or imported name in the namespace already uses the new name.
    AliasTaken,
    /// An affected call already passes a named argument with the new name.
    NamedArgTaken,
    /// The new name would change what another reference resolves to.
    Capture,
    /// A string may name the symbol at runtime; `proposed` marks a match of the new name.
    StringReference {
        proposed: bool,
    },
    /// The import is also used as a namespace prefix.
    ImportPrefix,
    /// A construct such as `compact()`, `extract()` or `$$name` uses variables by name at runtime.
    RuntimeVariables,
    /// The variable is the implicit `$value` parameter of a `set` hook.
    ImplicitParameter,
    /// A runtime call such as `define()` or `class_alias()` declares the name from a string.
    RuntimeDeclaration,
    /// A trait adaptation refers to the method.
    TraitAdaptation,
    Doc(DocProblem),
    /// Member uses in this file have no analysis results.
    AnalysisDisabled,
    /// Derived data no longer matches the source.
    Stale,
}

/// Why a proposed name is not accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameProblem {
    Empty,
    NotIdentifier,
    Keyword,
    ReservedType,
    ReservedVariable,
    ReservedMethodName,
    ReservedConstant,
}

/// Why a PHPDoc name blocks a rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocProblem {
    /// The comment does not parse, and it mentions an affected name.
    Unparsed,
    /// The name may be bound by a template or type alias.
    Template,
    /// The edit would change which class the name resolves to.
    Capture,
}
