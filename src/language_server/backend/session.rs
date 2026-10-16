//! Own the state that exists after a successful initialize request.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tower_lsp_server::Client;
use tower_lsp_server::gen_lsp_types::{
    InitializeParams, InitializeResult, MessageType, ServerCapabilities, ServerInfo,
};
use tower_lsp_server::jsonrpc::{Error, Result as JsonRpcResult};

use crate::config::language_server::LanguageServerConfiguration;
use crate::language_server::ServerConfig;
use crate::language_server::registry::WorkspaceRegistry;
use crate::language_server::workspace::workspace_roots;

use super::Backend;
use super::client::ClientPreferences;
use super::debounce::Debounced;
use super::restarts::Restarts;
use super::watchers::Watchers;

const CODE_LENS_QUIET_PERIOD: Duration = Duration::from_millis(150);

/// Publish these fields together after workspace loading succeeds.
pub(super) struct Session {
    preferences: ClientPreferences,
    capabilities: ServerCapabilities,
    restart_notified: AtomicBool,
    registry: Mutex<WorkspaceRegistry>,
    watchers: Mutex<Option<Watchers>>,
    code_lenses: Mutex<Option<Debounced>>,
    pub(super) restarts: Mutex<Restarts>,
}

impl Session {
    fn new(preferences: ClientPreferences, registry: WorkspaceRegistry, capabilities: ServerCapabilities) -> Self {
        Self {
            preferences,
            capabilities,
            restart_notified: AtomicBool::new(false),
            registry: Mutex::new(registry),
            watchers: Mutex::new(None),
            code_lenses: Mutex::new(None),
            restarts: Mutex::default(),
        }
    }

    pub(super) fn with_registry<R>(&self, read: impl FnOnce(&WorkspaceRegistry) -> R) -> R {
        read(&self.registry.lock().unwrap())
    }

    pub(super) fn with_registry_mut<R>(&self, write: impl FnOnce(&mut WorkspaceRegistry) -> R) -> R {
        write(&mut self.registry.lock().unwrap())
    }

    pub(super) fn refresh_watchers(&self, client: &Client, patterns: Vec<String>) {
        let mut watchers = self.watchers.lock().unwrap();
        match watchers.as_ref() {
            Some(watchers) => watchers.update(patterns),
            None => *watchers = Some(Watchers::start(client.clone(), patterns)),
        }
    }

    pub(super) fn refresh_code_lenses(&self, client: &Client) {
        if self.preferences.code_lens_refresh {
            self.code_lenses
                .lock()
                .unwrap()
                .get_or_insert_with(|| {
                    let client = client.clone();
                    Debounced::start(CODE_LENS_QUIET_PERIOD, move |_| {
                        let client = client.clone();
                        async move {
                            if let Err(error) = client.code_lens_refresh().await {
                                tracing::debug!(%error, "code-lens refresh failed");
                            }
                        }
                    })
                })
                .schedule(0);
        }
    }

    async fn stop(&self) {
        let watchers = self.watchers.lock().unwrap().take();
        let code_lenses = self.code_lenses.lock().unwrap().take();
        let restarts = self.restarts.lock().unwrap().stop();
        for restart in restarts {
            restart.stop().await;
        }

        if let Some(code_lenses) = code_lenses {
            code_lenses.stop().await;
        }

        if let Some(watchers) = watchers {
            watchers.stop().await;
        }
    }
}

impl Backend {
    pub(super) fn client_preferences(&self) -> ClientPreferences {
        self.session.get().map(|session| session.preferences).unwrap_or_default()
    }

    pub(super) async fn start_session(&self, params: InitializeParams) -> JsonRpcResult<InitializeResult> {
        let preferences = ClientPreferences::from(&params);
        let roots = match &self.config.launch.workspace {
            Some(root) => vec![root.clone()],
            None => workspace_roots(&params),
        };

        if roots.is_empty() {
            self.client.log_message(MessageType::Warning, "mago-server: no workspace root provided").await;
        } else {
            for root in &roots {
                self.client
                    .log_message(MessageType::Info, format!("mago-server: workspace root = {}", root.display()))
                    .await;
            }
        }

        let registry = self.bootstrap(roots).await;
        // Observe cancellation after loading, before making the session visible.
        tokio::task::yield_now().await;
        let mut config = (*self.config).clone();
        config.configuration.language_server = providers(&registry, &self.config);
        let capabilities = preferences.server_capabilities(&config);
        self.session
            .set(Session::new(preferences, registry, capabilities.clone()))
            .map_err(|_| Error::invalid_request())?;

        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "mago-server".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    pub(super) async fn notify_capability_change(&self) {
        let Some(session) = self.session.get() else { return };
        if session.restart_notified.load(Ordering::Relaxed) {
            return;
        }

        let mut config = (*self.config).clone();
        config.configuration.language_server = self.providers();
        if session.preferences.server_capabilities(&config) != session.capabilities
            && !session.restart_notified.swap(true, Ordering::Relaxed)
        {
            self.client
                .show_message(
                    MessageType::Warning,
                    "The enabled Mago features changed. Restart the Mago language server to update the editor.",
                )
                .await;
        }
    }

    pub(super) async fn stop_session(&self) -> JsonRpcResult<()> {
        if let Some(session) = self.session.get() {
            session.stop().await;
        }

        Ok(())
    }
}

pub(super) fn providers(registry: &WorkspaceRegistry, config: &ServerConfig) -> LanguageServerConfiguration {
    let Some(workspace) = registry.iter().next() else { return config.configuration.language_server.clone() };
    let mut providers = workspace.configuration.language_server.clone();
    providers.disable.retain(|provider| {
        registry.iter().all(|workspace| !workspace.configuration.language_server.is_enabled(*provider))
    });

    providers
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::Arc;
    use std::task::Poll;

    use tower_lsp_server::LspService;

    use super::*;
    use crate::language_server::ServerConfig;

    #[tokio::test]
    async fn cancelled_session_setup_does_not_install_partial_state() {
        let (service, socket) = LspService::new(|client| {
            Backend::new(
                client,
                Arc::new(ServerConfig::default()),
                crate::language_server::ordered_service::Tasks::default(),
            )
        });
        let backend = service.inner();
        // Let logging finish at once so the first pause follows registry construction.
        drop(socket);
        let mut initialize = Box::pin(backend.start_session(InitializeParams::default()));
        std::future::poll_fn(|context| {
            assert!(initialize.as_mut().poll(context).is_pending());
            Poll::Ready(())
        })
        .await;
        drop(initialize);
        assert!(backend.session.get().is_none());
        backend.start_session(InitializeParams::default()).await.unwrap();
        assert!(backend.session.get().is_some());
    }
}

#[cfg(test)]
#[path = "session/recovery_tests.rs"]
mod recovery_tests;
