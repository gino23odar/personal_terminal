//! Registration and discovery only; no parsing or execution orchestration.
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use crate::Command;

/// Plain-text documentation for a real handler. Validated when registered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandMetadata {
    pub name: String,
    pub summary: String,
    pub usage: String,
    /// Alternate command names, not argument expansions or session aliases.
    pub aliases: Vec<String>,
}

/// Metadata cannot be mutated through the registry after validation.
pub struct RegisteredCommand {
    metadata: CommandMetadata,
    handler: Box<dyn Command>,
}

impl RegisteredCommand {
    pub fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
    pub fn handler(&self) -> &dyn Command {
        self.handler.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    InvalidName(String),
    InvalidMetadata(&'static str),
    NameConflict(String),
    UnknownCommand(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(name) => write!(f, "invalid command name: {name:?}"),
            Self::InvalidMetadata(field) => write!(
                f,
                "{field} must be nonempty single-line text without control characters"
            ),
            Self::NameConflict(name) => write!(f, "command name already used: {name:?}"),
            Self::UnknownCommand(name) => write!(f, "unknown command: {name:?}"),
        }
    }
}
impl std::error::Error for RegistryError {}

/// Owns local handlers; deterministic canonical ordering on native and WASM.
/// Names and aliases share one namespace. There is no host PATH fallback.
#[derive(Default)]
pub struct CommandRegistry {
    commands: BTreeMap<String, RegisteredCommand>,
    aliases: BTreeMap<String, String>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.commands.len()
    }
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Validate the whole registration before insertion. Failure leaves the
    /// registry unchanged; the supplied handler is consumed even on failure.
    pub fn register(
        &mut self,
        mut metadata: CommandMetadata,
        handler: Box<dyn Command>,
    ) -> Result<(), RegistryError> {
        for (field, value) in [("summary", &metadata.summary), ("usage", &metadata.usage)] {
            if value.trim().is_empty()
                || value
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
            {
                return Err(RegistryError::InvalidMetadata(field));
            }
        }
        let mut names = BTreeSet::new();
        for name in std::iter::once(&metadata.name).chain(metadata.aliases.iter()) {
            if !valid_name(name) {
                return Err(RegistryError::InvalidName(name.clone()));
            }
            if !names.insert(name) || self.get(name).is_some() {
                return Err(RegistryError::NameConflict(name.clone()));
            }
        }
        metadata.aliases.sort();
        for alias in &metadata.aliases {
            self.aliases.insert(alias.clone(), metadata.name.clone());
        }
        self.commands.insert(
            metadata.name.clone(),
            RegisteredCommand { metadata, handler },
        );
        Ok(())
    }

    /// Exact, case-sensitive lookup. Aliases always point directly to a canonical entry.
    pub fn get(&self, name: &str) -> Option<&RegisteredCommand> {
        self.commands.get(name).or_else(|| {
            self.aliases
                .get(name)
                .and_then(|canonical| self.commands.get(canonical))
        })
    }

    /// One entry per canonical command, sorted by name; aliases are not extra rows.
    pub fn entries(&self) -> impl Iterator<Item = &CommandMetadata> {
        self.commands.values().map(|entry| entry.metadata())
    }

    /// Same metadata as lookup, including when requested through an alias.
    pub fn help(&self, name: &str) -> Option<&CommandMetadata> {
        self.get(name).map(|entry| entry.metadata())
    }

    /// Plain-text help for future CLI/WASM consumers; never executes handlers.
    /// Used by the runnable `help` built-in and other discovery consumers.
    pub fn help_text(&self, name: Option<&str>) -> Result<String, RegistryError> {
        if let Some(name) = name {
            let metadata = self
                .help(name)
                .ok_or_else(|| RegistryError::UnknownCommand(name.into()))?;
            let mut text = format!(
                "{} — {}\nUsage: {}\n",
                metadata.name, metadata.summary, metadata.usage
            );
            if !metadata.aliases.is_empty() {
                text.push_str(&format!("Aliases: {}\n", metadata.aliases.join(", ")));
            }
            Ok(text)
        } else if self.is_empty() {
            Ok("No commands registered.\n".into())
        } else {
            let mut text = String::from("Available commands:\n");
            for entry in self.entries() {
                text.push_str(&format!("  {} — {}\n", entry.name, entry.summary));
            }
            Ok(text)
        }
    }
}

// Deliberately narrow identifiers; no whitespace, shell operators or host paths.
fn valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z'))
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_'))
}
