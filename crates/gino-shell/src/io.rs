use crate::{LocalFuture, ShellError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// Presentation controls travel separately from bytes; never encode them as HTML.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalEvent {
    ClearScreen,
}

/// Async byte-oriented boundary. Implementations must bound internal buffers and
/// keep writes pending until capacity is available. No pipe/channel runtime yet.
/// Pending futures must register a waker, be drop-safe, and never block a thread.
/// The caller races operations with the execution's cancellation token.
pub trait CommandIo {
    /// Read at most buffer.len() bytes. Zero means EOF for a nonempty buffer;
    /// an empty buffer returns zero immediately without consuming input.
    fn read<'a>(&'a mut self, buffer: &'a mut [u8]) -> LocalFuture<'a, Result<usize, ShellError>>;

    /// Accept the complete slice or return an error (partial side effects may exist).
    /// Do not retain or queue unbounded copies after acknowledging completion.
    fn write<'a>(
        &'a mut self,
        stream: OutputStream,
        bytes: &'a [u8],
    ) -> LocalFuture<'a, Result<(), ShellError>>;

    /// Unsupported controls return ShellError::Unsupported rather than being ignored.
    fn emit(&mut self, event: TerminalEvent) -> LocalFuture<'_, Result<(), ShellError>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalSize {
    pub columns: u16,
    pub rows: u16,
}

/// Adapter-normalized input, not native keycodes or DOM events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    Text(String),
    Enter,
    Backspace,
    Interrupt,
    EndOfInput,
    Resize(TerminalSize),
}

/// Presentation/input contract. Concrete adapters own raw-mode setup and cleanup,
/// including drop/error paths. They do not interpret shell commands.
pub trait TerminalAdapter {
    /// None means dimensions are unavailable; zero dimensions must not be reported.
    fn size(&self) -> Option<TerminalSize>;
    fn next_input(&mut self) -> LocalFuture<'_, Result<InputEvent, ShellError>>;
    fn render<'a>(
        &'a mut self,
        stream: OutputStream,
        bytes: &'a [u8],
    ) -> LocalFuture<'a, Result<(), ShellError>>;
    fn control(&mut self, event: TerminalEvent) -> LocalFuture<'_, Result<(), ShellError>>;
}
