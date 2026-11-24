use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfsError {
    InvalidName,
    NotFound,
    AlreadyExists,
    NotDirectory,
    IsDirectory,
    DirectoryNotEmpty,
    RootProtected,
    InvalidMove,
    IdExhausted,
}

impl fmt::Display for VfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidName => "invalid virtual entry name",
            Self::NotFound => "virtual node or entry not found",
            Self::AlreadyExists => "virtual entry already exists",
            Self::NotDirectory => "virtual node is not a directory",
            Self::IsDirectory => "virtual node is a directory",
            Self::DirectoryNotEmpty => "virtual directory is not empty",
            Self::RootProtected => "virtual root cannot be removed or moved",
            Self::InvalidMove => "cannot move a directory into itself or its descendant",
            Self::IdExhausted => "virtual node identifiers exhausted",
        })
    }
}

impl std::error::Error for VfsError {}
