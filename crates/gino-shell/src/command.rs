use std::{fmt, future::Future, pin::Pin};

use crate::{CancellationToken, CommandIo, ShellSession};

/// Object-safe, borrowed, executor-neutral future. Browser values need not be Send.
pub type LocalFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExitStatus(pub u8);

impl ExitStatus {
    pub const SUCCESS: Self = Self(0);
    pub const FAILURE: Self = Self(1);
    pub const INVALID_INPUT: Self = Self(2);
    pub const CANCELLED: Self = Self(130);

    pub fn is_success(self) -> bool {
        self == Self::SUCCESS
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellError {
    Cancelled,
    InvalidInput(String),
    Io(String),
    Unsupported(String),
}

impl ShellError {
    pub fn exit_status(&self) -> ExitStatus {
        match self {
            Self::Cancelled => ExitStatus::CANCELLED,
            Self::InvalidInput(_) => ExitStatus::INVALID_INPUT,
            Self::Io(_) | Self::Unsupported(_) => ExitStatus::FAILURE,
        }
    }
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => f.write_str("execution cancelled"),
            Self::InvalidInput(message) => write!(f, "invalid input: {message}"),
            Self::Io(message) => write!(f, "I/O error: {message}"),
            Self::Unsupported(message) => write!(f, "unsupported: {message}"),
        }
    }
}

impl std::error::Error for ShellError {}

/// Ordinary nonzero command outcomes are Ok(status); infrastructure failures are Err.
pub type CommandResult = Result<ExitStatus, ShellError>;

/// Explicit session mutation plus per-execution cancellation. Future capabilities
/// (VFS, portfolio, AI) will be added when implemented, never as host handles.
pub struct CommandContext<'a> {
    pub session: &'a mut ShellSession,
    pub cancellation: &'a CancellationToken,
}

/// Handler contract only: no lookup, parsing, dispatch, or built-in commands yet.
pub trait Command {
    fn execute<'a>(
        &'a self,
        args: &'a [String],
        context: CommandContext<'a>,
        io: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult>;
}
