//! Resolve a document and check its provider before running an editor query.

use std::borrow::Cow;
use std::sync::Arc;

use tower_lsp_server::gen_lsp_types::LspErrorCodes;
use tower_lsp_server::gen_lsp_types::{Position, Range, Uri};
use tower_lsp_server::jsonrpc::{Error, ErrorCode, Result};

use mago_database::file::File;

use crate::config::language_server::Provider;
use crate::language_server::position::{line_end_offset, offset_at_position, position_at_offset};
use crate::language_server::state::WorkspaceState;
use crate::language_server::uri::UriFilePath;

use super::Backend;
use super::client::ClientPreferences;
use super::routing::{file_for_uri, traced};

pub(super) struct DocumentQuery<'a> {
    pub file: Arc<File>,
    pub workspace: &'a mut WorkspaceState,
    pub preferences: ClientPreferences,
}

impl DocumentQuery<'_> {
    pub fn offset(&self, position: Position) -> u32 {
        offset_at_position(&self.file, position)
    }

    /// Check edit ranges without rejecting the protocol's line-end column clamp.
    pub fn checked_range(&self, range: Range) -> Option<mago_server::Range> {
        let start = self.checked_offset(range.start)?;
        let end = self.checked_offset(range.end)?;
        (start <= end).then_some(mago_server::Range::new(start, end))
    }

    pub fn checked_offset(&self, position: Position) -> Option<u32> {
        self.file.get_line_start_offset(position.line)?;
        let offset = self.offset(position);
        (offset == line_end_offset(&self.file, position.line) || position_at_offset(&self.file, offset) == position)
            .then_some(offset)
    }

    pub fn checked_symbol_offset(&self, position: Position) -> Option<u32> {
        Some(self.adjust_symbol_offset(self.checked_offset(position)?))
    }

    fn adjust_symbol_offset(&self, offset: u32) -> u32 {
        self.workspace.core.server.symbol_offset(&self.file, offset)
    }
}

impl Backend {
    pub(super) fn query<R>(
        &self,
        provider: Provider,
        uri: &Uri,
        run: impl FnOnce(DocumentQuery<'_>) -> Option<R>,
    ) -> Result<Option<R>> {
        self.workspace_query(provider, uri, |workspace| {
            let file = file_for_uri(workspace, uri)?;
            // These edits have no target URI; the client applies them to the requested document.
            if matches!(provider, Provider::Formatting | Provider::RangeFormatting | Provider::Completion)
                && workspace.core.opened_key(file.id).is_some_and(|opened| opened != uri)
            {
                return None;
            }
            run(DocumentQuery { file, workspace, preferences: self.client_preferences() })
        })
    }

    pub(super) fn workspace_query<R>(
        &self,
        provider: Provider,
        uri: &Uri,
        run: impl FnOnce(&mut WorkspaceState) -> Option<R>,
    ) -> Result<Option<R>> {
        traced(provider, || {
            if !self.config.supports_provider(provider) {
                return Ok(None);
            }
            let Some(path) = uri.to_file_path() else { return Ok(None) };
            self.with_registry_mut(|registry| {
                let Some(workspace) = registry.for_path_mut(path.as_ref()) else { return Ok(None) };
                if !workspace.configuration.language_server.is_enabled(provider) {
                    return Ok(None);
                }
                if !matches!(
                    provider,
                    Provider::Formatting
                        | Provider::RangeFormatting
                        | Provider::FoldingRange
                        | Provider::SelectionRange
                        | Provider::DocumentSymbol
                ) {
                    require_analysis(workspace)?;
                }
                Ok(run(workspace))
            })
            .unwrap_or(Ok(None))
        })
    }
}

pub(super) fn require_analysis(workspace: &WorkspaceState) -> Result<()> {
    if workspace.core.server.is_initialized() {
        Ok(())
    } else {
        Err(request_failed("Workspace analysis failed; edit or save a file to retry"))
    }
}

pub(super) fn request_failed(message: impl Into<Cow<'static, str>>) -> Error {
    Error {
        code: ErrorCode::ServerError(i32::from(LspErrorCodes::RequestFailed).into()),
        message: message.into(),
        data: None,
    }
}
