use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Language server settings for this workspace.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LanguageServerConfiguration {
    /// Providers to disable. Restart the server after changing this list.
    /// Folder changes that alter the enabled providers also require a restart.
    pub disable: Vec<Provider>,
    /// Hide reference-count code lenses for declarations without references.
    /// When enabled, references are also counted when the editor requests code lenses,
    /// not only when it resolves each lens. Restart the server after changing this.
    pub hide_zero_reference_code_lenses: bool,
}

impl LanguageServerConfiguration {
    #[must_use]
    pub fn is_enabled(&self, provider: Provider) -> bool {
        !self.disable.contains(&provider)
    }
}

/// A provider name accepted in `[language-server].disable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    Formatting,
    RangeFormatting,
    Hover,
    Definition,
    TypeDefinition,
    DocumentHighlight,
    Implementation,
    TypeHierarchy,
    CallHierarchy,
    References,
    Rename,
    Completion,
    FoldingRange,
    SelectionRange,
    CodeAction,
    WorkspaceSymbol,
    DocumentSymbol,
    SignatureHelp,
    InlayHint,
    DocumentLink,
    CodeLens,
}
