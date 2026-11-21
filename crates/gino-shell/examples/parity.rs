//! Test-only native transcript producer. Not a CLI mode or production API.
use gino_shell::*;
use serde_json::{Value, json};
use std::{
    future::Future,
    task::{Context, Poll, Waker},
};

#[derive(Default)]
struct Capture(Vec<Value>);
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
            self.0.push(json!({"type": match stream { OutputStream::Stdout => "stdout", OutputStream::Stderr => "stderr" }, "text": std::str::from_utf8(bytes).unwrap()}));
            Ok(())
        })
    }
    fn emit(&mut self, event: TerminalEvent) -> LocalFuture<'_, Result<(), ShellError>> {
        Box::pin(async move {
            match event {
                TerminalEvent::ClearScreen => self.0.push(json!({"type":"clearScreen"})),
            }
            Ok(())
        })
    }
}
fn ready<F: Future>(future: F) -> F::Output {
    match std::pin::pin!(future)
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("parity built-ins must finish with ready capture I/O"),
    }
}
fn snapshot(session: &ShellSession) -> Value {
    json!({"cwd":session.cwd(),"lastStatus":session.last_status().0,"exitStatus":session.exit_status().map(|s|s.0)})
}
fn failure(error: &ShellError) -> Value {
    let (kind, span) = match error {
        ShellError::Syntax(e) => ("syntax", Some([e.span.start, e.span.end])),
        ShellError::UnknownCommand(_) => ("unknownCommand", None),
        ShellError::Cancelled => ("cancelled", None),
        ShellError::InvalidInput(_) => ("invalidInput", None),
        ShellError::Io(_) => ("io", None),
        ShellError::Unsupported(_) => ("unsupported", None),
    };
    json!({"kind":kind,"span":span,"message":error.to_string()})
}
fn main() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("../../../tests/parity/fixtures.json")).unwrap();
    assert!(!fixtures.is_empty(), "fixture suite must not be empty");
    let registry = builtin_registry();
    let mut transcript = Vec::new();
    for fixture in fixtures {
        let mut session = ShellSession::default();
        let mut steps = Vec::new();
        for (index, step) in fixture["steps"].as_array().unwrap().iter().enumerate() {
            let actual = if step["reset"] == true {
                session.reset();
                json!({"session":snapshot(&session)})
            } else {
                let input = match step["input"].as_str() {
                    Some(input) => input.to_owned(),
                    None => step["repeat"]
                        .as_str()
                        .unwrap()
                        .repeat(step["count"].as_u64().unwrap().try_into().unwrap()),
                };
                let token = CancellationToken::new();
                if step["cancel"] == true {
                    token.cancel();
                }
                let mut capture = Capture::default();
                let result = ready(execute_line(
                    &input,
                    &registry,
                    &mut session,
                    &mut capture,
                    &token,
                ));
                json!({"status":match &result { Ok(status)=>status.0,Err(error)=>error.exit_status().0 },
                    "events":capture.0,"error":result.as_ref().err().map(failure),"session":snapshot(&session)})
            };
            assert_eq!(
                actual, step["expected"],
                "fixture {} step {index}",
                fixture["name"]
            );
            steps.push(actual);
        }
        transcript.push(json!({"name":fixture["name"],"steps":steps}));
    }
    println!("{}", serde_json::to_string(&transcript).unwrap());
}
