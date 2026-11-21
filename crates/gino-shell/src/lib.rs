//! Portable shell parsing, registry, interpreter and commands. Platform adapters are separate.

mod builtins;
mod cancellation;
mod command;
mod interpreter;
mod io;
mod parser;
mod registry;
mod session;

pub use cancellation::{CancellationToken, Cancelled};
pub use command::{Command, CommandContext, CommandResult, ExitStatus, LocalFuture, ShellError};
pub use io::{CommandIo, InputEvent, OutputStream, TerminalAdapter, TerminalEvent, TerminalSize};
pub use parser::{
    MAX_INPUT_BYTES, ParseError, ParseErrorKind, SimpleCommand, Word, WordPart, WordPartKind,
    parse, tokenize,
};
pub use registry::{CommandMetadata, CommandRegistry, RegisteredCommand, RegistryError};
pub use session::{SessionSnapshot, ShellSession};

/// Honest startup status shared with the native scaffold.
pub const DEVELOPMENT_STATUS: &str =
    "GINO.OS — PR-05 core ready. Interactive CLI is not connected yet.";

pub use builtins::builtin_registry;
pub use interpreter::execute_line;
