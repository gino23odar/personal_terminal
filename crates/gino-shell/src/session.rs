use std::collections::BTreeMap;

use crate::{ExitStatus, ShellError};

/// Session-owned virtual state, never initialized from the host environment.
/// Alias values are stored only; expansion and history storage are later work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellSession {
    cwd: String,
    pub environment: BTreeMap<String, String>,
    pub aliases: BTreeMap<String, String>,
    last_status: ExitStatus,
}

impl Default for ShellSession {
    fn default() -> Self {
        Self {
            cwd: "/home/visitor".into(),
            environment: BTreeMap::new(),
            aliases: BTreeMap::new(),
            last_status: ExitStatus::SUCCESS,
        }
    }
}

impl ShellSession {
    pub fn cwd(&self) -> &str {
        &self.cwd
    }

    /// Accept only canonical absolute virtual paths. No filesystem lookup occurs.
    /// The future VFS must resolve paths and verify directory access before this call.
    pub fn set_cwd(&mut self, path: String) -> Result<(), ShellError> {
        if !path.starts_with('/')
            || path.contains(['\0', '\\'])
            || (path != "/" && path[1..].split('/').any(|p| matches!(p, "" | "." | "..")))
        {
            return Err(ShellError::InvalidInput(
                "expected a canonical absolute virtual path".into(),
            ));
        }
        self.cwd = path;
        Ok(())
    }

    pub fn last_status(&self) -> ExitStatus {
        self.last_status
    }

    /// Called by the future interpreter once an execution finishes.
    pub fn record_status(&mut self, status: ExitStatus) {
        self.last_status = status;
    }

    /// An owned execution snapshot stays unchanged when the session is mutated.
    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            cwd: self.cwd.clone(),
            environment: self.environment.clone(),
            aliases: self.aliases.clone(),
            last_status: self.last_status,
        }
    }
}

/// Detached data for read-only command inspection; never a host environment handle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionSnapshot {
    pub cwd: String,
    pub environment: BTreeMap<String, String>,
    pub aliases: BTreeMap<String, String>,
    pub last_status: ExitStatus,
}
