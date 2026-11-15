//! Apply legacy code actions through the client.

use foldhash::HashMap;
use serde_json::Value;
use tower_lsp_server::gen_lsp_types::LspErrorCodes;
use tower_lsp_server::gen_lsp_types::{ExecuteCommandParams, Uri, WorkspaceEdit};
use tower_lsp_server::jsonrpc::{Error, ErrorCode, Result};

use crate::config::language_server::Provider;
use crate::language_server::codec::{APPLY_CODE_ACTION, ActionDocument};

use super::Backend;

impl Backend {
    pub(super) async fn execute_code_action(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        if params.command != APPLY_CODE_ACTION {
            return Err(Error::invalid_params("Unknown code action command"));
        }

        let preferences = self.client_preferences();
        if preferences.code_action_literals || !preferences.apply_edits {
            return Err(failed("Client does not support legacy code action edits".into()));
        }

        let (source, edit, snapshots): (Uri, WorkspaceEdit, Vec<ActionDocument>) =
            serde_json::from_value(Value::Array(params.arguments.unwrap_or_default()))
                .map_err(|_| Error::invalid_params("Expected a source URI, workspace edit, and document snapshots"))?;
        let Some(changes) = &edit.changes else {
            return Err(Error::invalid_params("Expected text edits"));
        };

        if changes.is_empty()
            || changes.values().any(Vec::is_empty)
            || edit.document_changes.is_some()
            || edit.change_annotations.is_some()
        {
            return Err(Error::invalid_params("Expected plain text edits"));
        }

        let mut documents = HashMap::default();
        for snapshot in snapshots {
            if (snapshot.uri != source && !changes.contains_key(&snapshot.uri))
                || documents.insert(snapshot.uri.clone(), snapshot).is_some()
            {
                return Err(Error::invalid_params("Expected one snapshot per code action document"));
            }
        }

        for uri in std::iter::once(&source).chain(changes.keys()) {
            let snapshot = documents.get(uri).ok_or_else(|| Error::invalid_params("Missing document snapshot"))?;
            let Some(unchanged) =
                self.query(Provider::CodeAction, uri, |document| Some(snapshot.matches(&document.file)))?
            else {
                return Err(failed("Code actions are unavailable for this document".into()));
            };
            if !unchanged {
                return Err(Error {
                    code: ErrorCode::ServerError(i32::from(LspErrorCodes::ContentModified).into()),
                    message: "Code action document changed".into(),
                    data: None,
                });
            }
        }

        let result = self.client.apply_edit(self.workspace_edit(edit)).await?;
        if !result.applied {
            return Err(failed(result.failure_reason.unwrap_or_else(|| "Client did not apply the code action".into())));
        }

        Ok(None)
    }
}

fn failed(message: String) -> Error {
    Error {
        code: ErrorCode::ServerError(i32::from(LspErrorCodes::RequestFailed).into()),
        message: message.into(),
        data: None,
    }
}
