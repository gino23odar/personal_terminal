//! JavaScript boundary only; all shell semantics belong to gino-shell.
use gino_shell::{
    CancellationToken, CommandIo, CommandRegistry, LocalFuture, OutputStream, ShellError,
    ShellSession, TerminalEvent, builtin_registry, execute_line,
};
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

const MAX_OUTPUT_BYTES: usize = 262_144;
const MAX_EVENTS: usize = 1024;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Event {
    Stdout { text: String },
    Stderr { text: String },
    ClearScreen,
}
#[derive(Default)]
struct Capture {
    events: Vec<Event>,
    bytes: usize,
}
impl CommandIo for Capture {
    fn read<'a>(&'a mut self, _: &'a mut [u8]) -> LocalFuture<'a, Result<usize, ShellError>> {
        Box::pin(async { Ok(0) })
    }
    fn write<'a>(
        &'a mut self,
        stream: OutputStream,
        bytes: &'a [u8],
    ) -> LocalFuture<'a, Result<(), ShellError>> {
        Box::pin(async move {
            // Current built-ins write UTF-8; reject invalid text instead of corrupting it.
            let text = std::str::from_utf8(bytes)
                .map_err(|_| ShellError::Io("bridge output is not UTF-8".into()))?;
            if bytes.len() > MAX_OUTPUT_BYTES - self.bytes || self.events.len() >= MAX_EVENTS {
                return Err(ShellError::Io("bridge output limit exceeded".into()));
            }
            self.bytes += bytes.len();
            self.events.push(match stream {
                OutputStream::Stdout => Event::Stdout { text: text.into() },
                OutputStream::Stderr => Event::Stderr { text: text.into() },
            });
            Ok(())
        })
    }
    fn emit(&mut self, _: TerminalEvent) -> LocalFuture<'_, Result<(), ShellError>> {
        Box::pin(async move {
            if self.events.len() >= MAX_EVENTS {
                return Err(ShellError::Io("bridge output limit exceeded".into()));
            }
            self.events.push(Event::ClearScreen);
            Ok(())
        })
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    cwd: String,
    last_status: u8,
    exit_status: Option<u8>,
}
impl From<&ShellSession> for Snapshot {
    fn from(session: &ShellSession) -> Self {
        Self {
            cwd: session.cwd().into(),
            last_status: session.last_status().0,
            exit_status: session.exit_status().map(|s| s.0),
        }
    }
}
#[derive(Serialize)]
struct ErrorInfo {
    kind: &'static str,
    message: String,
    span: Option<[usize; 2]>,
}
impl From<&ShellError> for ErrorInfo {
    fn from(error: &ShellError) -> Self {
        let (kind, span) = match error {
            ShellError::Syntax(error) => ("syntax", Some([error.span.start, error.span.end])),
            ShellError::UnknownCommand(_) => ("unknownCommand", None),
            ShellError::Cancelled => ("cancelled", None),
            ShellError::InvalidInput(_) => ("invalidInput", None),
            ShellError::Io(_) => ("io", None),
            ShellError::Unsupported(_) => ("unsupported", None),
        };
        Self {
            kind,
            message: error.to_string(),
            span,
        }
    }
}
#[derive(Serialize)]
struct ExecutionResult {
    status: u8,
    events: Vec<Event>,
    error: Option<ErrorInfo>,
    session: Snapshot,
}

struct Inner {
    registry: CommandRegistry,
    session: RefCell<Option<ShellSession>>,
    active: RefCell<Option<CancellationToken>>,
}
/// A detached session lease avoids holding RefCell or wasm-bindgen mutable borrows
/// across await. Drop restores the session even if result serialization fails.
struct Lease {
    inner: Rc<Inner>,
    session: Option<ShellSession>,
}
impl Drop for Lease {
    fn drop(&mut self) {
        *self.inner.session.borrow_mut() = self.session.take();
        self.inner.active.borrow_mut().take();
    }
}

#[wasm_bindgen]
pub struct WasmShell {
    inner: Rc<Inner>,
}
impl Default for WasmShell {
    fn default() -> Self {
        Self::new()
    }
}
#[wasm_bindgen]
impl WasmShell {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Rc::new(Inner {
                registry: builtin_registry(),
                session: RefCell::new(Some(ShellSession::default())),
                active: RefCell::new(None),
            }),
        }
    }

    /// Busy calls throw synchronously; shell outcomes resolve as structured results.
    #[wasm_bindgen(unchecked_return_type = "Promise<ExecutionResult>")]
    pub fn execute(&self, input: String) -> Result<js_sys::Promise, JsValue> {
        let session = self.inner.session.borrow_mut().take().ok_or_else(busy)?;
        let token = CancellationToken::new();
        *self.inner.active.borrow_mut() = Some(token.clone());
        let mut lease = Lease {
            inner: self.inner.clone(),
            session: Some(session),
        };
        Ok(wasm_bindgen_futures::future_to_promise(async move {
            let session = lease.session.as_mut().expect("execution owns its session");
            let mut capture = Capture::default();
            let result =
                execute_line(&input, &lease.inner.registry, session, &mut capture, &token).await;
            let response = ExecutionResult {
                status: match &result {
                    Ok(status) => status.0,
                    Err(error) => error.exit_status().0,
                },
                events: capture.events,
                error: result.as_ref().err().map(ErrorInfo::from),
                session: Snapshot::from(&*session),
            };
            encode(&response)
        }))
    }

    /// Returns false when idle. Cancellation is permanent for this execution only.
    pub fn cancel(&self) -> bool {
        if let Some(token) = self.inner.active.borrow().as_ref() {
            token.cancel();
            true
        } else {
            false
        }
    }

    #[wasm_bindgen(unchecked_return_type = "SessionSnapshot")]
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        let session = self.inner.session.borrow();
        encode(&Snapshot::from(session.as_ref().ok_or_else(busy)?))
    }

    pub fn reset(&self) -> Result<(), JsValue> {
        self.inner
            .session
            .borrow_mut()
            .as_mut()
            .ok_or_else(busy)?
            .reset();
        Ok(())
    }
}
fn busy() -> JsValue {
    js_sys::Error::new("shell is busy; await execute() before continuing").into()
}
fn encode<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true))
        .map_err(|error| js_sys::Error::new(&error.to_string()).into())
}

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &str = r#"
export interface SessionSnapshot { cwd: string; lastStatus: number; exitStatus: number | null; }
export type OutputEvent = { type: "stdout" | "stderr"; text: string } | { type: "clearScreen" };
export interface ShellFailure {
  kind: "syntax" | "unknownCommand" | "cancelled" | "invalidInput" | "io" | "unsupported";
  message: string;
  span: [number, number] | null;
}
export interface ExecutionResult {
  status: number;
  events: OutputEvent[];
  error: ShellFailure | null;
  session: SessionSnapshot;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future,
        task::{Context, Poll, Waker},
    };
    fn ready<F: Future>(f: F) -> F::Output {
        match std::pin::pin!(f)
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("capture must be ready"),
        }
    }
    #[test]
    fn output_limits_fail_without_appending() {
        let mut capture = Capture::default();
        ready(capture.write(OutputStream::Stdout, &vec![b'a'; MAX_OUTPUT_BYTES])).unwrap();
        assert!(ready(capture.write(OutputStream::Stderr, b"x")).is_err());
        assert_eq!(capture.events.len(), 1);
        assert_eq!(capture.bytes, MAX_OUTPUT_BYTES);
        assert!(ready(Capture::default().write(OutputStream::Stdout, &[255])).is_err());
    }
    #[test]
    fn event_limit_includes_clear_controls() {
        let mut capture = Capture::default();
        for _ in 0..MAX_EVENTS {
            ready(capture.emit(TerminalEvent::ClearScreen)).unwrap();
        }
        assert!(ready(capture.emit(TerminalEvent::ClearScreen)).is_err());
        assert!(ready(capture.write(OutputStream::Stdout, b"")).is_err());
        assert_eq!(capture.events.len(), MAX_EVENTS);
    }
}
