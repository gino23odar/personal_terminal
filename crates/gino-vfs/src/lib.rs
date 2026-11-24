//! Portable, node-addressed virtual storage. No paths, host I/O or access policy.
//!
//! ```
//! use gino_vfs::{EntryName, MemoryFs, VfsBackend};
//! let mut fs = MemoryFs::new();
//! let root = fs.root();
//! let file = fs.create_file(&root, EntryName::new("hello.txt")?, b"hello")?;
//! assert_eq!(fs.read_file(&file)?, b"hello");
//! # Ok::<(), gino_vfs::VfsError>(())
//! ```

mod backend;
mod error;
mod memory;
mod types;

pub use backend::VfsBackend;
pub use error::VfsError;
pub use memory::MemoryFs;
pub use types::{DirectoryEntry, EntryName, Metadata, NodeId, NodeKind};
