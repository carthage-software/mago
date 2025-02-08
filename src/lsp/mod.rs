#![allow(dead_code)]

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use ahash::HashMap;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Error as ServerError;
use tower_lsp::jsonrpc::ErrorCode;
use tower_lsp::jsonrpc::Result as ServerResult;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tower_lsp::LanguageServer;

use mago_interner::ThreadedInterner;

use crate::consts::BIN;
use crate::consts::VERSION;
use crate::lsp::workspace::MagoWorkspace;

mod workspace;

#[derive(Debug, Clone)]
pub struct MagoLanguageServer {
    client: Client,
    interner: ThreadedInterner,
    workspaces: Arc<RwLock<HashMap<String, MagoWorkspace>>>,
    current_workspace: Arc<RwLock<Option<String>>>,
}

impl MagoLanguageServer {
    pub fn new(interner: ThreadedInterner, client: Client) -> Self {
        MagoLanguageServer {
            interner,
            client,
            workspaces: Arc::new(RwLock::new(HashMap::default())),
            current_workspace: Arc::new(RwLock::new(None)),
        }
    }

    async fn get_current_workspace_name(&self) -> Option<String> {
        self.current_workspace.read().await.clone()
    }

    async fn select_workspace(&self, name: String) -> ServerResult<()> {
        if self.workspaces.read().await.contains_key(&name) {
            self.current_workspace.write().await.replace(name);
            Ok(())
        } else {
            Err(ServerError {
                code: ErrorCode::InvalidRequest,
                message: Cow::Owned(format!("workspace not found: {}", name)),
                data: None,
            })
        }
    }

    async fn initialize_workspace(&self, name: String, root_uri: Url) -> ServerResult<()> {
        let root: PathBuf = match root_uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                return Err(ServerError {
                    code: ErrorCode::InvalidRequest,
                    message: Cow::Owned(format!("workspace root URI is not a file URI: {}", root_uri)),
                    data: None,
                })
            }
        };

        let workspace = match MagoWorkspace::initialize(&self.interner, root).await {
            Ok(workspace) => workspace,
            Err(error) => {
                return Err(ServerError {
                    code: ErrorCode::InternalError,
                    message: Cow::Owned(format!("failed to initialize workspace: {}", error)),
                    data: None,
                })
            }
        };

        self.workspaces.write().await.insert(name, workspace);

        Ok(())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for MagoLanguageServer {
    async fn initialize(&self, parameters: InitializeParams) -> ServerResult<InitializeResult> {
        match parameters.workspace_folders {
            Some(workspaces) => {
                for workspace in workspaces {
                    self.initialize_workspace(workspace.name.clone(), workspace.uri).await?;
                    self.select_workspace(workspace.name).await?;
                }
            }
            None => {
                if let Some(root_uri) = parameters.root_uri {
                    let name = root_uri.to_string();
                    self.initialize_workspace(name.clone(), root_uri).await?;
                    self.select_workspace(name).await?;
                }
            }
        };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                // hover_provider: Some(HoverProviderCapability::Simple(true)),
                // definition_provider: Some(OneOf::Left(true)),
                // type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                // implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
                // references_provider: Some(OneOf::Left(true)),
                // document_formatting_provider: Some(OneOf::Left(true)),
                // declaration_provider: Some(DeclarationCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                    identifier: Some(BIN.to_string()),
                    inter_file_dependencies: true,
                    workspace_diagnostics: true,
                    ..DiagnosticOptions::default()
                })),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "Mago Language Server".to_string(),
                version: Some(VERSION.to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        eprintln!("Mago Language Server initialized");
    }

    async fn workspace_diagnostic(
        &self,
        _: WorkspaceDiagnosticParams,
    ) -> ServerResult<WorkspaceDiagnosticReportResult> {
        let Some(workspace_name) = self.get_current_workspace_name().await else {
            return Err(ServerError {
                code: ErrorCode::InvalidRequest,
                message: Cow::Owned("no workspace selected".to_string()),
                data: None,
            });
        };

        let workspaces = self.workspaces.read().await;
        let workspace = workspaces.get(&workspace_name).unwrap();

        match workspace.get_workspace_diagnostic_report().await {
            Ok(report) => Ok(WorkspaceDiagnosticReportResult::Report(report)),
            Err(error) => Err(ServerError {
                code: ErrorCode::InternalError,
                message: Cow::Owned(format!("failed to get workspace diagnostic report: {}", error)),
                data: None,
            }),
        }
    }

    async fn diagnostic(&self, params: DocumentDiagnosticParams) -> ServerResult<DocumentDiagnosticReportResult> {
        let file = params.text_document.uri.to_file_path().map_err(|_| ServerError {
            code: ErrorCode::InvalidRequest,
            message: Cow::Owned(format!("invalid URI: {}", params.text_document.uri)),
            data: None,
        })?;

        let workspace_name = self.get_current_workspace_name().await.ok_or_else(|| ServerError {
            code: ErrorCode::InvalidRequest,
            message: Cow::Owned("no workspace selected".to_string()),
            data: None,
        })?;

        let workspaces = self.workspaces.read().await;
        let workspace = workspaces.get(&workspace_name).unwrap();

        let diagnostics = match workspace.get_document_diagnostic(&params.text_document.uri, file).await {
            Ok(report) => report,
            Err(error) => {
                return Err(ServerError {
                    code: ErrorCode::InternalError,
                    message: Cow::Owned(format!("failed to get document diagnostic report: {}", error)),
                    data: None,
                })
            }
        };

        Ok(DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(diagnostics)))
    }

    async fn shutdown(&self) -> ServerResult<()> {
        eprintln!("Mago LSP Server: Shutdown requested");
        Ok(())
    }
}
