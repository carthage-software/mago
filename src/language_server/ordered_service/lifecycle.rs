//! Check session requests at arrival and track initialization through completion.

use std::io;
use std::sync::Arc;
use std::sync::Mutex;

use tower_lsp_server::jsonrpc::Error;
use tower_lsp_server::jsonrpc::ErrorCode;
use tower_lsp_server::jsonrpc::Request;
use tower_lsp_server::jsonrpc::Response;

use super::Output;

#[derive(Default)]
pub(in crate::language_server) enum SessionState {
    #[default]
    Starting,
    Initializing,
    Running,
    Shutdown,
    Exited,
    Failed(String),
}

pub(super) enum Admission {
    Forward,
    Reply(Option<Response>),
}

impl SessionState {
    pub(in crate::language_server) fn result(&self) -> io::Result<()> {
        match self {
            Self::Exited => Ok(()),
            Self::Failed(message) => Err(io::Error::other(message.clone())),
            _ => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "language server input closed before exit")),
        }
    }

    pub(super) fn admit(&mut self, request: &Request, duplicate_id: bool, overloaded: bool) -> Admission {
        let error = match request.method() {
            "initialize" if !matches!(self, Self::Starting) => Error::invalid_request(),
            "initialize" if request.id().is_none() => return Admission::Reply(None),
            "initialize" if duplicate_id => Error::invalid_request(),
            "initialize" if overloaded => busy_error(),
            // Tower changes its state before it checks shutdown's shape and request ID.
            "shutdown" if matches!(self, Self::Running) => {
                if request.id().is_none() {
                    return Admission::Reply(None);
                }
                if !no_params(request) {
                    Error::invalid_params("shutdown takes no parameters")
                } else if duplicate_id {
                    Error::invalid_request()
                } else if overloaded {
                    busy_error()
                } else {
                    *self = Self::Shutdown;
                    return Admission::Forward;
                }
            }
            "shutdown" if matches!(self, Self::Starting | Self::Initializing) => {
                Error { code: ErrorCode::ServerError(-32002), message: "Server not initialized".into(), data: None }
            }
            "shutdown" => Error::invalid_request(),
            "exit" => {
                self.exit(request);
                return Admission::Forward;
            }
            _ if overloaded => match self {
                Self::Starting | Self::Initializing => {
                    Error { code: ErrorCode::ServerError(-32002), message: "Server not initialized".into(), data: None }
                }
                Self::Running if !duplicate_id => busy_error(),
                _ => Error::invalid_request(),
            },
            _ => return Admission::Forward,
        };
        Admission::Reply(request.id().cloned().map(|id| Response::from_error(id, error)))
    }

    fn exit(&mut self, request: &Request) {
        if matches!(self, Self::Failed(_)) {
            return;
        }
        *self = if request.id().is_some() || !no_params(request) {
            Self::Failed("invalid exit notification".into())
        } else if matches!(self, Self::Shutdown) {
            Self::Exited
        } else {
            Self::Failed("exit received before shutdown".into())
        };
    }
}

fn busy_error() -> Error {
    Error { code: ErrorCode::ServerError(-32000), message: "Server busy; retry request".into(), data: None }
}

fn no_params(request: &Request) -> bool {
    request.params().is_none_or(serde_json::Value::is_null)
}

/// Keep the session pending until the admitted initialize handler completes or drops.
pub(super) struct Initialization {
    session: Arc<Mutex<SessionState>>,
    completed: bool,
}

impl Initialization {
    pub(super) fn start(session: Arc<Mutex<SessionState>>) -> Self {
        let mut state = session.lock().unwrap();
        if matches!(*state, SessionState::Starting) {
            *state = SessionState::Initializing;
        }
        drop(state);
        Self { session, completed: false }
    }

    pub(super) fn complete(mut self, result: &Output) {
        let mut state = self.session.lock().unwrap();
        if matches!(*state, SessionState::Initializing) {
            *state = if matches!(result, Ok(Some(response)) if response.is_ok()) {
                SessionState::Running
            } else {
                SessionState::Starting
            };
        }
        self.completed = true;
    }
}

impl Drop for Initialization {
    fn drop(&mut self) {
        if self.completed {
            return;
        }
        let mut state = self.session.lock().unwrap();
        if matches!(*state, SessionState::Initializing) {
            *state = SessionState::Starting;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_initialization_allows_retry() {
        let session = Arc::new(Mutex::new(SessionState::Starting));
        let initialize = Initialization::start(Arc::clone(&session));
        assert!(matches!(*session.lock().unwrap(), SessionState::Initializing));
        drop(initialize);
        assert!(matches!(*session.lock().unwrap(), SessionState::Starting));
    }

    #[test]
    fn initialization_cannot_change_a_terminal_state() {
        for complete in [false, true] {
            for terminal in [SessionState::Shutdown, SessionState::Exited, SessionState::Failed("failed".into())] {
                let session = Arc::new(Mutex::new(SessionState::Starting));
                let initialize = Initialization::start(Arc::clone(&session));
                *session.lock().unwrap() = terminal;
                let before = std::mem::discriminant(&*session.lock().unwrap());
                if complete {
                    initialize.complete(&Ok(Some(Response::from_ok(1.into(), serde_json::Value::Null))));
                } else {
                    drop(initialize);
                }
                assert_eq!(std::mem::discriminant(&*session.lock().unwrap()), before);
            }
        }
    }

    #[test]
    fn shutdown_before_initialization_cannot_advance_the_session() {
        for mut session in [SessionState::Starting, SessionState::Initializing] {
            let before = std::mem::discriminant(&session);
            let request = Request::build("shutdown").id(1).finish();
            let Admission::Reply(Some(response)) = session.admit(&request, false, false) else {
                panic!("shutdown before initialization must return an error");
            };
            assert_eq!(response.error().unwrap().code, ErrorCode::ServerError(-32002));
            assert_eq!(std::mem::discriminant(&session), before);
        }
    }
}
