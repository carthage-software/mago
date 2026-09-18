//! Transport-agnostic value types the server speaks.
//!
//! These types carry no protocol semantics: positions are byte offsets, never
//! lines/columns or UTF-16. The protocol layer converts to and from its own
//! position model. Capability-specific result types are added here as each
//! capability is ported onto the server.

use foldhash::HashMap;

use mago_database::file::FileId;

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

/// Hover information for the token under the cursor: rendered markdown plus the
/// range the hover applies to.
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// Markdown describing the symbol.
    pub markdown: String,
    /// The byte range of the hovered token.
    pub range: Range,
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

/// A code action (quickfix): a titled set of per-file edits, optionally linked
/// to the diagnostic it resolves.
#[derive(Debug, Clone)]
pub struct CodeActionItem {
    /// The action's title shown to the user.
    pub title: String,
    /// Edits to apply, grouped by file.
    pub edits: HashMap<FileId, Vec<TextReplacement>>,
    /// The diagnostic this action resolves, if any.
    pub diagnostic: Option<DiagnosticData>,
}

/// The semantic-token classification of a source token.
///
/// The variant order is significant: it defines the index the protocol layer
/// reports against its semantic-tokens legend, so the legend must list the same
/// kinds in the same order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticTokenKind {
    Keyword,
    Comment,
    String,
    Number,
    Operator,
    Variable,
    Function,
    Type,
    Namespace,
    Parameter,
    Property,
}

impl SemanticTokenKind {
    /// The legend index for this kind (its position in the variant order).
    #[must_use]
    pub const fn index(self) -> u32 {
        self as u32
    }
}

/// One semantic-highlighting token: a byte offset, a byte length, and a kind.
/// The protocol layer delta-encodes these into the LSP wire format.
#[derive(Debug, Clone, Copy)]
pub struct SemanticTokenItem {
    /// Byte offset of the token's start.
    pub offset: u32,
    /// Byte length of the token.
    pub length: u32,
    /// The token's classification.
    pub kind: SemanticTokenKind,
}

/// The kind of a completion entry (drives the editor's icon).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
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
    /// Documentation (plain text) shown in the detail pane.
    pub documentation: Option<String>,
    /// Text to insert instead of the label (e.g. an FQCN, or a snippet).
    pub insert_text: Option<String>,
    /// Whether `insert_text` is a snippet (uses `$1` tab stops).
    pub snippet: bool,
    /// A byte range to replace with the label (used by variables so the leading
    /// `$` is rewritten); `None` means insert at the cursor.
    pub replace: Option<Range>,
    /// Opaque ordering key reproducing the server's ranking on the client.
    pub sort_text: Option<String>,
    /// Text the editor filters against (defaults to the label).
    pub filter_text: Option<String>,
}

impl CompletionEntry {
    /// A bare entry with just a label and kind; other fields default to empty.
    #[must_use]
    pub fn new(label: String, kind: CompletionKind) -> Self {
        Self {
            label,
            kind,
            detail: None,
            documentation: None,
            insert_text: None,
            snippet: false,
            replace: None,
            sort_text: None,
            filter_text: None,
        }
    }
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

/// The kind of a workspace symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Class,
    Interface,
    Trait,
    Enum,
    Function,
    Constant,
}

/// A workspace-symbol-search match: a name, its kind, and where it's declared.
#[derive(Debug, Clone)]
pub struct WorkspaceSymbolItem {
    /// The symbol's display name.
    pub name: String,
    /// What kind of symbol it is.
    pub kind: SymbolKind,
    /// Where the symbol is declared.
    pub location: SymbolLocation,
}

/// A code lens: a reference count anchored at a byte offset (typically a
/// top-level definition).
#[derive(Debug, Clone, Copy)]
pub struct CodeLensItem {
    /// Byte offset the lens is anchored at.
    pub offset: u32,
    /// Number of references to the symbol declared there.
    pub reference_count: usize,
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

/// A clickable link from a range in the current file to a target file (e.g. a
/// `use Foo\Bar;` pointing at the file declaring `Foo\Bar`).
#[derive(Debug, Clone)]
pub struct DocumentLinkItem {
    /// The byte range of the link's source text in the current file.
    pub range: Range,
    /// The file the link targets.
    pub target: FileId,
    /// A short tooltip (the resolved symbol name).
    pub tooltip: String,
}

/// Signature help for a call site: a rendered signature label, the byte ranges
/// of each parameter *within that label*, and the active parameter index.
#[derive(Debug, Clone)]
pub struct SignatureInfo {
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
