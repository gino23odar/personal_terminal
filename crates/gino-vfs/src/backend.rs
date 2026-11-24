use crate::{DirectoryEntry, EntryName, Metadata, NodeId, VfsError};

/// Object-safe synchronous storage contract for local native/WASM execution.
///
/// Mutations returning Err leave state unchanged. Allocation failure/panic is not
/// a recoverable storage error. No Send/Sync or executor requirement is imposed.
/// This low-level interface grants unrestricted access to its in-memory tree;
/// future path/mount policy must wrap it rather than expose it to commands.
pub trait VfsBackend {
    fn root(&self) -> NodeId;
    fn metadata(&self, node: &NodeId) -> Result<Metadata, VfsError>;
    fn lookup(&self, directory: &NodeId, name: &EntryName) -> Result<NodeId, VfsError>;
    /// Entries sorted by case-sensitive UTF-8 name; excludes synthetic dot entries.
    fn read_dir(&self, directory: &NodeId) -> Result<Vec<DirectoryEntry>, VfsError>;
    /// Borrowed, exact bytes, including invalid UTF-8. Never modifies read metadata.
    fn read_file(&self, file: &NodeId) -> Result<&[u8], VfsError>;
    fn create_dir(&mut self, parent: &NodeId, name: EntryName) -> Result<NodeId, VfsError>;
    fn create_file(
        &mut self,
        parent: &NodeId,
        name: EntryName,
        bytes: &[u8],
    ) -> Result<NodeId, VfsError>;
    /// Full replacement, not append or an offset write.
    fn replace_file(&mut self, file: &NodeId, bytes: &[u8]) -> Result<(), VfsError>;
    /// Removes a file or empty directory; never recursive and never the root.
    fn remove_empty(&mut self, node: &NodeId) -> Result<(), VfsError>;
    /// Preserves identity and contents. No overwrite; identical source/destination
    /// is a successful no-op. Root moves and directory cycles are rejected.
    fn rename(
        &mut self,
        node: &NodeId,
        new_parent: &NodeId,
        new_name: EntryName,
    ) -> Result<(), VfsError>;
}
