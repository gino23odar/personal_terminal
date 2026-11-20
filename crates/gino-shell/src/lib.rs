//! Portable shell contracts. No interpreter or platform adapter is implemented.

mod cancellation;
mod command;
mod io;
mod registry;
mod session;

pub use cancellation::{CancellationToken, Cancelled};
pub use command::{Command, CommandContext, CommandResult, ExitStatus, LocalFuture, ShellError};
pub use io::{CommandIo, InputEvent, OutputStream, TerminalAdapter, TerminalEvent, TerminalSize};
pub use registry::{CommandMetadata, CommandRegistry, RegisteredCommand, RegistryError};
pub use session::{SessionSnapshot, ShellSession};

/// Honest startup status shared with the native scaffold.
pub const DEVELOPMENT_STATUS: &str =
    "GINO.OS — PR-01 scaffold. Shell commands are not implemented yet.";
