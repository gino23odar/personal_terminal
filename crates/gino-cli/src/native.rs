mod output;

use gino_shell::{
    CancellationToken, ExitStatus, MAX_INPUT_BYTES, ShellError, ShellSession, builtin_registry,
    execute_line,
};
use output::{Bridge, Renderer};
use rustyline::{DefaultEditor, config::Configurer, error::ReadlineError};
use std::{
    future::Future,
    io::{self, IsTerminal, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    time::Duration,
};

struct SignalRegistration(signal_hook::SigId);
impl Drop for SignalRegistration {
    fn drop(&mut self) {
        signal_hook::low_level::unregister(self.0);
    }
}
struct ThreadWake(std::thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

// Native-only executor: futures never perform blocking terminal writes themselves.
// A bounded handoff is drained between polls, and SIGINT is observed each iteration.
fn drive<F: Future>(
    future: F,
    bridge: &Bridge,
    renderer: &mut Renderer,
    interrupted: &AtomicBool,
    token: &CancellationToken,
) -> F::Output {
    let mut future = std::pin::pin!(future);
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        if interrupted.swap(false, Ordering::SeqCst) {
            token.cancel();
        }
        if let Poll::Ready(result) = future.as_mut().poll(&mut cx) {
            return result;
        }
        if !bridge.drain(renderer) {
            // Also bounds signal latency while waiting for a future's wakeup.
            std::thread::park_timeout(Duration::from_millis(10));
        }
    }
}

pub fn run() -> Result<u8, Box<dyn std::error::Error>> {
    if std::env::args_os().len() != 1 {
        eprintln!("gino: no CLI arguments supported; run cargo run -p gino-cli");
        return Ok(2);
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        eprintln!(
            "gino: interactive stdin and stdout terminals are required (batch mode is not implemented)"
        );
        return Ok(2);
    }
    if std::env::var("TERM").is_ok_and(|term| term == "dumb") {
        eprintln!("gino: TERM=dumb is not supported by the interactive adapter");
        return Ok(2);
    }
    let interrupted = Arc::new(AtomicBool::new(false));
    let _signal = SignalRegistration(signal_hook::flag::register(
        signal_hook::consts::SIGINT,
        interrupted.clone(),
    )?);
    let mut editor = DefaultEditor::new()?;
    editor.set_max_history_size(100)?;
    editor.set_history_ignore_dups(true)?;
    let registry = builtin_registry();
    let mut session = ShellSession::default();
    let mut renderer = Renderer::default();
    println!("GINO.OS — native virtual shell\nType 'help' for available commands.");
    loop {
        let cwd = if session.cwd() == "/home/visitor" {
            "~"
        } else {
            session.cwd()
        };
        let prompt = format!("visitor@gino.os:{}$ ", cwd.escape_debug());
        match editor.readline(&prompt) {
            Ok(line) => {
                // A signal while editing belongs to that edit, not the next execution.
                interrupted.store(false, Ordering::SeqCst);
                if line.len() <= MAX_INPUT_BYTES && !line.trim().is_empty() {
                    editor.add_history_entry(line.as_str())?;
                }
                let bridge = Bridge::default();
                let mut command_io = bridge.clone();
                let token = CancellationToken::new();
                let result = drive(
                    execute_line(&line, &registry, &mut session, &mut command_io, &token),
                    &bridge,
                    &mut renderer,
                    &interrupted,
                    &token,
                );
                renderer.finish()?;
                match result {
                    Err(ShellError::Cancelled) => {
                        println!("^C");
                    }
                    Err(ShellError::Io(error)) => return Err(io::Error::other(error).into()),
                    _ => {} // Interpreter already emitted ordinary diagnostics.
                }
                if let Some(status) = session.exit_status() {
                    return Ok(status.0);
                }
            }
            Err(
                ReadlineError::Interrupted
                | ReadlineError::Signal(rustyline::error::Signal::Interrupt),
            ) => {
                interrupted.store(false, Ordering::SeqCst);
                session.record_status(ExitStatus::CANCELLED);
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!();
                io::stdout().flush()?;
                return Ok(session.last_status().0);
            }
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gino_shell::{
        Command, CommandContext, CommandIo, CommandMetadata, CommandResult, LocalFuture,
    };
    struct WaitForInterrupt(Arc<AtomicBool>);
    impl Command for WaitForInterrupt {
        fn execute<'a>(
            &'a self,
            _: &'a [String],
            _: CommandContext<'a>,
            _: &'a mut dyn CommandIo,
        ) -> LocalFuture<'a, CommandResult> {
            Box::pin(async move {
                // Simulate the native SIGINT flag being set while a handler waits.
                self.0.store(true, Ordering::SeqCst);
                std::future::pending().await
            })
        }
    }
    #[test]
    fn driver_cancels_pending_handler_then_allows_a_fresh_execution() {
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut registry = builtin_registry();
        registry
            .register(
                CommandMetadata {
                    name: "wait".into(),
                    summary: "Wait".into(),
                    usage: "wait".into(),
                    aliases: vec![],
                },
                Box::new(WaitForInterrupt(interrupted.clone())),
            )
            .unwrap();
        let mut session = ShellSession::default();
        let bridge = Bridge::default();
        let mut command_io = bridge.clone();
        let mut renderer = Renderer::default();
        let token = CancellationToken::new();
        assert_eq!(
            drive(
                execute_line("wait", &registry, &mut session, &mut command_io, &token),
                &bridge,
                &mut renderer,
                &interrupted,
                &token
            ),
            Err(ShellError::Cancelled)
        );
        assert_eq!(session.last_status(), ExitStatus::CANCELLED);
        let token = CancellationToken::new();
        assert_eq!(
            drive(
                execute_line("exit 0", &registry, &mut session, &mut command_io, &token),
                &bridge,
                &mut renderer,
                &interrupted,
                &token
            ),
            Ok(ExitStatus::SUCCESS)
        );
    }
}
