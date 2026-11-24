use std::{fmt, rc::Rc};

use crate::VfsError;

/// Opaque, instance-scoped handle. Cloning a handle does not clone a filesystem.
/// Handles from another backend (including its root) are rejected as NotFound.
#[derive(Clone)]
pub struct NodeId {
    pub(crate) owner: Rc<()>,
    pub(crate) index: u64,
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.owner, &other.owner) && self.index == other.index
    }
}
impl Eq for NodeId {}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Do not expose memory addresses or a reusable serialized identity.
        f.debug_tuple("NodeId").field(&self.index).finish()
    }
}

/// One UTF-8 directory-entry name, not a path. No normalization or expansion.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryName(String);

impl EntryName {
    pub fn new(name: impl Into<String>) -> Result<Self, VfsError> {
        let name = name.into();
        if matches!(name.as_str(), "" | "." | "..") || name.contains(['/', '\\', '\0']) {
            return Err(VfsError::InvalidName);
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Directory,
}

/// Detached metadata: byte length for files, zero for directories.
/// No host timestamps, permissions, ownership or inode values are fabricated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub kind: NodeKind,
    pub byte_len: u64,
}

/// A detached listing entry; changing it cannot mutate the backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub id: NodeId,
    pub name: EntryName,
    pub metadata: Metadata,
}
