use gino_shell::*;
use std::{
    cell::RefCell,
    future::{Future, pending},
    pin::Pin,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

fn ready<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("test operation unexpectedly pending"),
    }
}

#[derive(Default)]
struct WakeCount(AtomicUsize);
impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn session_state_is_virtual_isolated_and_snapshots_are_detached() {
    let mut session = ShellSession::default();
    assert_eq!(session.cwd(), "/home/visitor");
    assert!(session.environment.is_empty());
    let snapshot = session.snapshot();
    session.set_cwd("/tmp".into()).unwrap();
    session.environment.insert("NAME".into(), "visitor".into());
    session
        .aliases
        .insert("example".into(), "future-command".into());
    session.record_status(ExitStatus(7));
    assert_eq!(snapshot, ShellSession::default().snapshot());
    assert_eq!(session.last_status(), ExitStatus(7));
    assert_ne!(session.snapshot(), snapshot);
}

#[test]
fn invalid_virtual_paths_leave_session_unchanged() {
    let mut session = ShellSession::default();
    for path in [
        "", "relative", "/..", "/a/../b", "//a", "/a/", "/a/./b", "/a\0b", "C:\\host",
    ] {
        assert!(matches!(
            session.set_cwd(path.into()),
            Err(ShellError::InvalidInput(_))
        ));
        assert_eq!(session.cwd(), "/home/visitor");
    }
    session.set_cwd("/".into()).unwrap();
    session.set_cwd("/home/訪客".into()).unwrap();
}

#[test]
fn cancellation_is_shared_idempotent_and_execution_scoped() {
    let token = CancellationToken::new();
    let clone = token.clone();
    let independent = CancellationToken::new();
    token.cancel();
    token.cancel();
    assert!(clone.is_cancelled());
    assert!(!independent.is_cancelled());
    ready(clone.cancelled());
}

#[test]
fn cancellation_wakes_all_live_waiters_and_unregisters_dropped_waiters() {
    let token = CancellationToken::new();
    let count = Arc::new(WakeCount::default());
    let waker = Waker::from(count.clone());
    let mut cx = Context::from_waker(&waker);
    let mut first = Box::pin(token.cancelled());
    let mut second = Box::pin(token.cancelled());
    let mut dropped = Box::pin(token.cancelled());
    assert!(first.as_mut().poll(&mut cx).is_pending());
    assert!(second.as_mut().poll(&mut cx).is_pending());
    assert!(dropped.as_mut().poll(&mut cx).is_pending());
    drop(dropped);
    token.cancel();
    token.cancel();
    assert_eq!(count.0.load(Ordering::SeqCst), 2);
    assert!(first.as_mut().poll(&mut cx).is_ready());
    assert!(second.as_mut().poll(&mut cx).is_ready());
}

#[test]
fn cancellation_updates_the_registered_waker() {
    let token = CancellationToken::new();
    let old = Arc::new(WakeCount::default());
    let new = Arc::new(WakeCount::default());
    let mut future = Box::pin(token.cancelled());
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(&Waker::from(old.clone())))
            .is_pending()
    );
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(&Waker::from(new.clone())))
            .is_pending()
    );
    token.cancel();
    assert_eq!(old.0.load(Ordering::SeqCst), 0);
    assert_eq!(new.0.load(Ordering::SeqCst), 1);
}

struct PendingOperation(Rc<RefCell<bool>>);
impl Future for PendingOperation {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }
}
impl Drop for PendingOperation {
    fn drop(&mut self) {
        *self.0.borrow_mut() = true;
    }
}

#[test]
fn cancelling_a_pending_operation_wakes_executor_and_drops_operation() {
    let token = CancellationToken::new();
    let dropped = Rc::new(RefCell::new(false));
    let mut operation = Box::pin(token.run_until_cancelled(PendingOperation(dropped.clone())));
    let count = Arc::new(WakeCount::default());
    let waker = Waker::from(count.clone());
    let mut cx = Context::from_waker(&waker);
    assert!(operation.as_mut().poll(&mut cx).is_pending());
    token.cancel();
    assert_eq!(count.0.load(Ordering::SeqCst), 1);
    assert_eq!(
        operation.as_mut().poll(&mut cx),
        Poll::Ready(Err(ShellError::Cancelled))
    );
    assert!(*dropped.borrow());
}

#[test]
fn cancellation_precedes_ready_work_but_success_preserves_result() {
    let token = CancellationToken::new();
    assert_eq!(ready(token.run_until_cancelled(async { 42 })), Ok(42));
    token.cancel();
    assert_eq!(
        ready(token.run_until_cancelled(async { panic!("must not poll") })),
        Err(ShellError::Cancelled)
    );
}

type CapturedOutput = Rc<RefCell<Vec<(OutputStream, Vec<u8>)>>>;

// Deliberately !Send fake: this must also compile under the WASM target check.
#[derive(Default)]
struct MemoryIo {
    output: CapturedOutput,
    events: Vec<TerminalEvent>,
    input: Vec<u8>,
    blocked: bool,
    fail: bool,
}
impl CommandIo for MemoryIo {
    fn read<'a>(&'a mut self, buffer: &'a mut [u8]) -> LocalFuture<'a, Result<usize, ShellError>> {
        Box::pin(async move {
            let n = buffer.len().min(self.input.len());
            buffer[..n].copy_from_slice(&self.input[..n]);
            self.input.drain(..n);
            Ok(n)
        })
    }
    fn write<'a>(
        &'a mut self,
        stream: OutputStream,
        bytes: &'a [u8],
    ) -> LocalFuture<'a, Result<(), ShellError>> {
        Box::pin(async move {
            if self.fail {
                return Err(ShellError::Io("closed".into()));
            }
            // Models unavailable capacity, released here only by cancellation.
            if self.blocked {
                pending::<()>().await;
            }
            self.output.borrow_mut().push((stream, bytes.to_vec()));
            Ok(())
        })
    }
    fn emit(&mut self, event: TerminalEvent) -> LocalFuture<'_, Result<(), ShellError>> {
        Box::pin(async move {
            self.events.push(event);
            Ok(())
        })
    }
}
impl TerminalAdapter for MemoryIo {
    fn size(&self) -> Option<TerminalSize> {
        None
    }
    fn next_input(&mut self) -> LocalFuture<'_, Result<InputEvent, ShellError>> {
        Box::pin(async { Ok(InputEvent::EndOfInput) })
    }
    fn render<'a>(
        &'a mut self,
        stream: OutputStream,
        bytes: &'a [u8],
    ) -> LocalFuture<'a, Result<(), ShellError>> {
        self.write(stream, bytes)
    }
    fn control(&mut self, event: TerminalEvent) -> LocalFuture<'_, Result<(), ShellError>> {
        self.emit(event)
    }
}

// Test-only handler, never registered or exposed as a product command.
struct Probe;
impl Command for Probe {
    fn execute<'a>(
        &'a self,
        args: &'a [String],
        context: CommandContext<'a>,
        io: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult> {
        Box::pin(async move {
            context
                .cancellation
                .run_until_cancelled(async {
                    let snapshot = context.session.snapshot();
                    io.write(OutputStream::Stdout, snapshot.cwd.as_bytes())
                        .await?;
                    io.write(OutputStream::Stderr, args[0].as_bytes()).await?;
                    io.emit(TerminalEvent::ClearScreen).await?;
                    context
                        .session
                        .environment
                        .insert("PROBE".into(), "ok".into());
                    Ok(ExitStatus::SUCCESS)
                })
                .await?
        })
    }
}

#[test]
fn dynamic_handler_and_local_io_preserve_streams_events_and_session_mutation() {
    let command: Box<dyn Command> = Box::new(Probe);
    let mut session = ShellSession::default();
    let token = CancellationToken::new();
    let mut io = MemoryIo::default();
    let result = ready(command.execute(
        &["diagnostic".into()],
        CommandContext {
            session: &mut session,
            cancellation: &token,
        },
        &mut io,
    ));
    assert_eq!(result, Ok(ExitStatus::SUCCESS));
    assert_eq!(
        *io.output.borrow(),
        vec![
            (OutputStream::Stdout, b"/home/visitor".to_vec()),
            (OutputStream::Stderr, b"diagnostic".to_vec())
        ]
    );
    assert_eq!(io.events, vec![TerminalEvent::ClearScreen]);
    assert_eq!(
        session.environment.get("PROBE").map(String::as_str),
        Some("ok")
    );
    let adapter: &mut dyn TerminalAdapter = &mut io;
    assert_eq!(adapter.size(), None);
    assert_eq!(ready(adapter.next_input()), Ok(InputEvent::EndOfInput));
    ready(adapter.render(OutputStream::Stdout, b"bytes")).unwrap();
    ready(adapter.control(TerminalEvent::ClearScreen)).unwrap();
}

#[test]
fn byte_input_supports_bounded_reads_empty_buffers_and_eof() {
    let mut io = MemoryIo {
        input: vec![0, 255, 7],
        ..Default::default()
    };
    assert_eq!(ready(io.read(&mut [])), Ok(0));
    let mut buffer = [0; 2];
    assert_eq!(ready(io.read(&mut buffer)), Ok(2));
    assert_eq!(buffer, [0, 255]);
    assert_eq!(ready(io.read(&mut buffer)), Ok(1));
    assert_eq!(buffer[0], 7);
    assert_eq!(ready(io.read(&mut buffer)), Ok(0));
}

#[test]
fn pending_output_can_be_cancelled_without_acknowledging_or_buffering_bytes() {
    let token = CancellationToken::new();
    let mut io = MemoryIo {
        blocked: true,
        ..Default::default()
    };
    {
        let mut write =
            Box::pin(token.run_until_cancelled(io.write(OutputStream::Stdout, b"data")));
        assert!(
            write
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        token.cancel();
        assert_eq!(ready(write), Err(ShellError::Cancelled));
    }
    assert!(io.output.borrow().is_empty());
}

#[test]
fn io_errors_propagate_without_mutating_session_and_have_stable_exit_statuses() {
    let mut session = ShellSession::default();
    let snapshot = session.snapshot();
    let token = CancellationToken::new();
    let mut io = MemoryIo {
        fail: true,
        ..Default::default()
    };
    let error = ready(Probe.execute(
        &["unused".into()],
        CommandContext {
            session: &mut session,
            cancellation: &token,
        },
        &mut io,
    ))
    .unwrap_err();
    assert_eq!(error, ShellError::Io("closed".into()));
    assert_eq!(session.snapshot(), snapshot);
    assert_eq!(error.exit_status(), ExitStatus::FAILURE);
    assert_eq!(ShellError::Cancelled.exit_status(), ExitStatus(130));
    assert_eq!(
        ShellError::InvalidInput("bad".into()).exit_status(),
        ExitStatus(2)
    );
    assert_eq!(
        ShellError::Unsupported("clear".into()).exit_status(),
        ExitStatus(1)
    );
    assert!(!ExitStatus(7).is_success());
    assert!(ExitStatus::SUCCESS.is_success());
    assert_eq!(error.to_string(), "I/O error: closed");
}

#[test]
fn cancelled_handler_does_not_emit_output_or_change_session() {
    let mut session = ShellSession::default();
    let snapshot = session.snapshot();
    let token = CancellationToken::new();
    token.cancel();
    let mut io = MemoryIo::default();
    assert_eq!(
        ready(Probe.execute(
            &["unused".into()],
            CommandContext {
                session: &mut session,
                cancellation: &token
            },
            &mut io
        )),
        Err(ShellError::Cancelled)
    );
    assert_eq!(session.snapshot(), snapshot);
    assert!(io.output.borrow().is_empty());
    assert!(io.events.is_empty());
}
