//! List reference lenses and resolve them against the current workspace.

use tower_lsp_server::gen_lsp_types::LspErrorCodes;
use tower_lsp_server::gen_lsp_types::{CodeLens, Location, Uri};
use tower_lsp_server::jsonrpc::{Error, ErrorCode, Result};

use crate::config::language_server::Provider;
use crate::language_server::codec;
use crate::language_server::codec::CodeLensData;

use super::Backend;
use super::query::DocumentQuery;

impl Backend {
    pub(super) fn resolve_lens(&self, lens: CodeLens) -> Result<CodeLens> {
        let data: CodeLensData =
            serde_json::from_value(lens.data.clone().ok_or_else(|| Error::invalid_params("Missing code-lens data"))?)
                .map_err(|_| Error::invalid_params("Invalid code-lens data"))?;
        self.query(Provider::CodeLens, &data.uri, |document| Some(document.resolve_lens(lens, &data)))?.unwrap_or_else(
            || {
                Err(Error {
                    code: ErrorCode::ServerError(i32::from(LspErrorCodes::RequestFailed).into()),
                    message: "Code lenses are unavailable for this document".into(),
                    data: None,
                })
            },
        )
    }
}

impl DocumentQuery<'_> {
    pub fn lenses(self, uri: &Uri) -> Vec<CodeLens> {
        let server = &mut self.workspace.core.server;
        if !self.workspace.configuration.language_server.hide_zero_reference_code_lenses {
            return codec::code_lens(&self.file, uri, server.get_lens(self.file.id));
        }
        let counted = server.get_lens_counts(self.file.id);
        codec::code_lens(&self.file, uri, counted.into_iter().filter(|(_, count)| *count > 0).map(|(item, _)| item))
    }

    fn resolve_lens(mut self, mut lens: CodeLens, data: &CodeLensData) -> Result<CodeLens> {
        if !data.matches(&self.file) {
            return Err(Error {
                code: ErrorCode::ServerError(i32::from(LspErrorCodes::ContentModified).into()),
                message: "Code-lens document changed".into(),
                data: None,
            });
        }
        let Some(offset) = self.checked_offset(lens.range.start).filter(|offset| {
            lens.range.start == lens.range.end
                && self.workspace.core.server.get_lens(self.file.id).iter().any(|item| item.offset == *offset)
        }) else {
            return Err(Error::invalid_params("Code lens must point to a declaration"));
        };
        let locations = self.lens_references(offset);
        lens.command = Some(codec::reference_command(&data.uri, lens.range.start, locations));
        Ok(lens)
    }

    fn lens_references(&mut self, offset: u32) -> Vec<Location> {
        let references = self.workspace.core.server.get_references(self.file.id, offset, false);
        self.workspace.locations(&references)
    }
}
