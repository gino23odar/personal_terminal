use gino_shell::*;
use std::{
    cell::Cell,
    future::Future,
    rc::Rc,
    task::{Context, Poll, Waker},
};

fn ready<F: Future>(future: F) -> F::Output {
    match std::pin::pin!(future)
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("unexpected pending operation"),
    }
}
#[derive(Default)]
struct MemoryIo {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    events: Vec<TerminalEvent>,
    fail: bool,
    unsupported: bool,
    pending: bool,
}
impl CommandIo for MemoryIo {
    fn read<'a>(&'a mut self, _: &'a mut [u8]) -> LocalFuture<'a, Result<usize, ShellError>> {
        panic!("these commands must not read stdin")
    }
    fn write<'a>(
        &'a mut self,
        stream: OutputStream,
        bytes: &'a [u8],
    ) -> LocalFuture<'a, Result<(), ShellError>> {
        Box::pin(async move {
            if self.pending {
                std::future::pending::<()>().await;
            }
            if self.fail {
                return Err(ShellError::Io("closed".into()));
            }
            match stream {
                OutputStream::Stdout => &mut self.stdout,
                OutputStream::Stderr => &mut self.stderr,
            }
            .extend_from_slice(bytes);
            Ok(())
        })
    }
    fn emit(&mut self, event: TerminalEvent) -> LocalFuture<'_, Result<(), ShellError>> {
        Box::pin(async move {
            if self.unsupported {
                return Err(ShellError::Unsupported("clear screen".into()));
            }
            if self.pending {
                std::future::pending::<()>().await;
            }
            self.events.push(event);
            Ok(())
        })
    }
}
fn run(input: &str, session: &mut ShellSession, io: &mut MemoryIo) -> CommandResult {
    ready(execute_line(
        input,
        &builtin_registry(),
        session,
        io,
        &CancellationToken::new(),
    ))
}
#[test]
fn basic_commands_use_virtual_state_and_literal_parsed_arguments() {
    for (input, expected) in [
        ("echo", "\n"),
        ("echo 'hello world' '' 🦀", "hello world  🦀\n"),
        (r#"echo "$HOME" ~ "\n""#, "$HOME ~ \\n\n"),
        ("echo -- -n", "-n\n"),
        ("echo -", "-\n"),
        ("echo text -n", "text -n\n"),
        ("pwd", "/home/visitor\n"),
        ("whoami", "visitor\n"),
    ] {
        let mut session = ShellSession::default();
        session
            .environment
            .insert("USER".into(), "host-impostor".into());
        let mut io = MemoryIo::default();
        assert_eq!(
            run(input, &mut session, &mut io),
            Ok(ExitStatus::SUCCESS),
            "{input}"
        );
        assert_eq!(io.stdout, expected.as_bytes(), "{input}");
        assert!(io.stderr.is_empty());
        assert_eq!(session.last_status(), ExitStatus::SUCCESS);
    }
    let mut session = ShellSession::default();
    session.set_cwd("/tmp".into()).unwrap();
    let mut io = MemoryIo::default();
    run("pwd", &mut session, &mut io).unwrap();
    assert_eq!(io.stdout, b"/tmp\n");
}
#[test]
fn help_uses_current_registry_including_extension_aliases() {
    let mut registry = builtin_registry();
    let calls = Rc::new(Cell::new(0));
    registry
        .register(metadata(), Box::new(Probe(calls.clone())))
        .unwrap();
    for (input, expected) in [
        ("help", registry.help_text(None).unwrap()),
        ("help p", registry.help_text(Some("probe")).unwrap()),
    ] {
        let mut io = MemoryIo::default();
        assert_eq!(
            ready(execute_line(
                input,
                &registry,
                &mut ShellSession::default(),
                &mut io,
                &CancellationToken::new()
            )),
            Ok(ExitStatus::SUCCESS)
        );
        assert_eq!(io.stdout, expected.as_bytes());
    }
    assert_eq!(calls.get(), 0);
    assert_eq!(
        builtin_registry()
            .entries()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["clear", "echo", "exit", "help", "pwd", "whoami"]
    );
}
struct Probe(Rc<Cell<usize>>);
impl Command for Probe {
    fn execute<'a>(
        &'a self,
        args: &'a [String],
        context: CommandContext<'a>,
        _: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult> {
        Box::pin(async move {
            assert_eq!(args, &["two words"]);
            self.0.set(self.0.get() + 1);
            context.session.set_cwd("/tmp".into())?;
            Ok(ExitStatus(7))
        })
    }
}
fn metadata() -> CommandMetadata {
    CommandMetadata {
        name: "probe".into(),
        summary: "Test handler".into(),
        usage: "probe value".into(),
        aliases: vec!["p".into()],
    }
}
#[test]
fn registry_extensions_dispatch_without_interpreter_changes() {
    let calls = Rc::new(Cell::new(0));
    let mut registry = builtin_registry();
    registry
        .register(metadata(), Box::new(Probe(calls.clone())))
        .unwrap();
    let mut session = ShellSession::default();
    let mut io = MemoryIo::default();
    assert_eq!(
        ready(execute_line(
            "p 'two words'",
            &registry,
            &mut session,
            &mut io,
            &CancellationToken::new()
        )),
        Ok(ExitStatus(7))
    );
    assert_eq!(session.last_status(), ExitStatus(7));
    assert_eq!(session.cwd(), "/tmp");
    assert_eq!(calls.get(), 1);
}
#[test]
fn failures_have_stable_statuses_and_do_not_execute_partial_input() {
    for (input, status) in [
        ("missing", 127),
        ("/bin/sh", 127),
        ("help missing", 127),
        ("echo before | echo after", 2),
        ("echo 'unclosed", 2),
        ("echo -n", 2),
        ("pwd --help", 2),
        ("whoami extra", 2),
        ("clear extra", 2),
        ("help a b", 2),
        ("help --help", 2),
        ("exit 256", 2),
        ("exit -1", 2),
        ("exit +1", 2),
        ("exit ''", 2),
        ("exit 1 2", 2),
        ("exit nope", 2),
    ] {
        let mut session = ShellSession::default();
        let mut io = MemoryIo::default();
        assert_eq!(
            run(input, &mut session, &mut io).unwrap_err().exit_status(),
            ExitStatus(status),
            "{input}"
        );
        assert_eq!(session.last_status(), ExitStatus(status));
        assert!(session.exit_status().is_none());
        assert!(io.stdout.is_empty());
        assert!(io.events.is_empty());
        assert!(io.stderr.starts_with(b"gino: "));
        assert!(io.stderr.ends_with(b"\n"));
    }
}
#[test]
fn blank_input_preserves_status_and_errors_recover_on_next_command() {
    let mut session = ShellSession::default();
    let mut io = MemoryIo::default();
    run("missing", &mut session, &mut io).unwrap_err();
    io.stderr.clear();
    assert_eq!(run(" \t", &mut session, &mut io), Ok(ExitStatus(127)));
    assert!(io.stderr.is_empty());
    assert_eq!(
        run("echo recovered", &mut session, &mut io),
        Ok(ExitStatus::SUCCESS)
    );
    assert_eq!(session.last_status(), ExitStatus::SUCCESS);
}
#[test]
fn clear_emits_a_structured_event_and_reports_unsupported_adapters() {
    let mut session = ShellSession::default();
    let mut io = MemoryIo::default();
    run("clear", &mut session, &mut io).unwrap();
    assert_eq!(io.events, [TerminalEvent::ClearScreen]);
    assert!(io.stdout.is_empty());
    io.unsupported = true;
    assert!(matches!(
        run("clear", &mut session, &mut io),
        Err(ShellError::Unsupported(_))
    ));
    assert_eq!(session.last_status(), ExitStatus::FAILURE);
}
#[test]
fn exit_closes_only_its_session_and_reset_starts_fresh() {
    for input in ["exit", "exit 7", "exit 255"] {
        let mut session = ShellSession::default();
        session.record_status(ExitStatus(9));
        session.environment.insert("X".into(), "y".into());
        session.aliases.insert("x".into(), "echo".into());
        let mut io = MemoryIo::default();
        let expected = if input == "exit" {
            9
        } else if input == "exit 7" {
            7
        } else {
            255
        };
        assert_eq!(run(input, &mut session, &mut io), Ok(ExitStatus(expected)));
        assert_eq!(session.snapshot().exit_status, Some(ExitStatus(expected)));
        assert_eq!(
            run("echo ignored", &mut session, &mut io),
            Ok(ExitStatus(expected))
        );
        assert!(io.stdout.is_empty());
        assert_eq!(ShellSession::default().exit_status(), None);
        session.reset();
        assert_eq!(session, ShellSession::default());
        run("echo new", &mut session, &mut io).unwrap();
        assert_eq!(io.stdout, b"new\n");
    }
}
#[test]
fn pre_cancelled_execution_has_no_output_or_exit_side_effect() {
    let token = CancellationToken::new();
    token.cancel();
    let mut session = ShellSession::default();
    let mut io = MemoryIo::default();
    assert_eq!(
        ready(execute_line(
            "exit",
            &builtin_registry(),
            &mut session,
            &mut io,
            &token
        )),
        Err(ShellError::Cancelled)
    );
    assert_eq!(session.last_status(), ExitStatus::CANCELLED);
    assert_eq!(session.exit_status(), None);
    assert!(io.stderr.is_empty());
    run("echo next", &mut session, &mut io).unwrap();
    assert_eq!(io.stdout, b"next\n");
}
#[test]
fn cancellation_interrupts_pending_output_events_and_error_reporting() {
    for input in ["echo blocked", "clear", "missing"] {
        let registry = builtin_registry();
        let mut session = ShellSession::default();
        let mut io = MemoryIo {
            pending: true,
            ..MemoryIo::default()
        };
        let token = CancellationToken::new();
        {
            let mut future = Box::pin(execute_line(
                input,
                &registry,
                &mut session,
                &mut io,
                &token,
            ));
            let mut cx = Context::from_waker(Waker::noop());
            assert!(future.as_mut().poll(&mut cx).is_pending());
            token.cancel();
            assert_eq!(
                future.as_mut().poll(&mut cx),
                Poll::Ready(Err(ShellError::Cancelled))
            );
        }
        assert_eq!(session.last_status(), ExitStatus::CANCELLED);
        assert!(io.stdout.is_empty() && io.stderr.is_empty() && io.events.is_empty());
    }
}
#[test]
fn failed_output_and_failed_diagnostics_return_transport_error() {
    for input in ["echo text", "missing"] {
        let mut session = ShellSession::default();
        let mut io = MemoryIo {
            fail: true,
            ..MemoryIo::default()
        };
        assert_eq!(
            run(input, &mut session, &mut io),
            Err(ShellError::Io("closed".into()))
        );
        assert_eq!(session.last_status(), ExitStatus::FAILURE);
        assert!(io.stderr.is_empty());
    }
}
#[test]
fn session_aliases_are_not_expanded_and_sessions_are_isolated() {
    let mut a = ShellSession::default();
    let mut b = ShellSession::default();
    a.aliases.insert("x".into(), "echo surprise".into());
    a.set_cwd("/tmp".into()).unwrap();
    let mut io = MemoryIo::default();
    assert!(matches!(
        run("x", &mut a, &mut io),
        Err(ShellError::UnknownCommand(_))
    ));
    run("pwd", &mut b, &mut io).unwrap();
    assert_eq!(io.stdout, b"/home/visitor\n");
    assert_eq!(b.last_status(), ExitStatus::SUCCESS);
}
