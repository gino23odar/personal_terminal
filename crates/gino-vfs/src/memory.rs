use std::{collections::BTreeMap, rc::Rc};

use crate::{DirectoryEntry, EntryName, Metadata, NodeId, NodeKind, VfsBackend, VfsError};

#[derive(Debug)]
enum Content {
    File(Vec<u8>),
    Directory(BTreeMap<EntryName, u64>),
}

#[derive(Debug)]
struct Node {
    parent: Option<(u64, EntryName)>,
    content: Content,
}

impl Node {
    fn metadata(&self) -> Metadata {
        match &self.content {
            Content::File(bytes) => Metadata {
                kind: NodeKind::File,
                byte_len: bytes.len() as u64,
            },
            Content::Directory(_) => Metadata {
                kind: NodeKind::Directory,
                byte_len: 0,
            },
        }
    }
}

/// An independent, initially empty tree. No shared writable state or host access.
/// IDs are never reused, including after deletion. This type is deliberately not
/// Clone: copying a handle must not silently share or fork a mutable filesystem.
#[derive(Debug)]
pub struct MemoryFs {
    owner: Rc<()>,
    nodes: BTreeMap<u64, Node>,
    next_id: u64,
}

impl Default for MemoryFs {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryFs {
    pub fn new() -> Self {
        Self {
            owner: Rc::new(()),
            nodes: BTreeMap::from([(
                0,
                Node {
                    parent: None,
                    content: Content::Directory(BTreeMap::new()),
                },
            )]),
            next_id: 1,
        }
    }

    fn id(&self, index: u64) -> NodeId {
        NodeId {
            owner: Rc::clone(&self.owner),
            index,
        }
    }

    fn node(&self, id: &NodeId) -> Result<&Node, VfsError> {
        if !Rc::ptr_eq(&self.owner, &id.owner) {
            return Err(VfsError::NotFound);
        }
        self.nodes.get(&id.index).ok_or(VfsError::NotFound)
    }

    fn directory(&self, id: &NodeId) -> Result<&BTreeMap<EntryName, u64>, VfsError> {
        match &self.node(id)?.content {
            Content::Directory(entries) => Ok(entries),
            Content::File(_) => Err(VfsError::NotDirectory),
        }
    }

    // Private commit helpers are called only after all fallible checks complete.
    fn directory_mut(&mut self, index: u64) -> &mut BTreeMap<EntryName, u64> {
        match &mut self
            .nodes
            .get_mut(&index)
            .expect("validated directory")
            .content
        {
            Content::Directory(entries) => entries,
            Content::File(_) => unreachable!("validated directory"),
        }
    }

    fn create(
        &mut self,
        parent: &NodeId,
        name: EntryName,
        content: Content,
    ) -> Result<NodeId, VfsError> {
        if self.directory(parent)?.contains_key(&name) {
            return Err(VfsError::AlreadyExists);
        }
        let index = self.next_id;
        let next = index.checked_add(1).ok_or(VfsError::IdExhausted)?;
        self.nodes.insert(
            index,
            Node {
                parent: Some((parent.index, name.clone())),
                content,
            },
        );
        self.directory_mut(parent.index).insert(name, index);
        self.next_id = next;
        Ok(self.id(index))
    }
}

impl VfsBackend for MemoryFs {
    fn root(&self) -> NodeId {
        self.id(0)
    }

    fn metadata(&self, node: &NodeId) -> Result<Metadata, VfsError> {
        Ok(self.node(node)?.metadata())
    }

    fn lookup(&self, directory: &NodeId, name: &EntryName) -> Result<NodeId, VfsError> {
        self.directory(directory)?
            .get(name)
            .map(|index| self.id(*index))
            .ok_or(VfsError::NotFound)
    }

    fn read_dir(&self, directory: &NodeId) -> Result<Vec<DirectoryEntry>, VfsError> {
        Ok(self
            .directory(directory)?
            .iter()
            .map(|(name, index)| DirectoryEntry {
                id: self.id(*index),
                name: name.clone(),
                metadata: self.nodes[index].metadata(),
            })
            .collect())
    }

    fn read_file(&self, file: &NodeId) -> Result<&[u8], VfsError> {
        match &self.node(file)?.content {
            Content::File(bytes) => Ok(bytes),
            Content::Directory(_) => Err(VfsError::IsDirectory),
        }
    }

    fn create_dir(&mut self, parent: &NodeId, name: EntryName) -> Result<NodeId, VfsError> {
        self.create(parent, name, Content::Directory(BTreeMap::new()))
    }

    fn create_file(
        &mut self,
        parent: &NodeId,
        name: EntryName,
        bytes: &[u8],
    ) -> Result<NodeId, VfsError> {
        self.create(parent, name, Content::File(bytes.to_vec()))
    }

    fn replace_file(&mut self, file: &NodeId, bytes: &[u8]) -> Result<(), VfsError> {
        self.read_file(file)?;
        let content = Content::File(bytes.to_vec());
        self.nodes
            .get_mut(&file.index)
            .expect("validated file")
            .content = content;
        Ok(())
    }

    fn remove_empty(&mut self, node: &NodeId) -> Result<(), VfsError> {
        let entry = self.node(node)?;
        let (parent, name) = entry.parent.clone().ok_or(VfsError::RootProtected)?;
        if matches!(&entry.content, Content::Directory(entries) if !entries.is_empty()) {
            return Err(VfsError::DirectoryNotEmpty);
        }
        self.directory_mut(parent).remove(&name);
        self.nodes.remove(&node.index);
        Ok(())
    }

    fn rename(
        &mut self,
        node: &NodeId,
        new_parent: &NodeId,
        new_name: EntryName,
    ) -> Result<(), VfsError> {
        let (old_parent, old_name) = self
            .node(node)?
            .parent
            .clone()
            .ok_or(VfsError::RootProtected)?;
        let destination = self.directory(new_parent)?;
        if old_parent == new_parent.index && old_name == new_name {
            return Ok(());
        }
        if destination.contains_key(&new_name) {
            return Err(VfsError::AlreadyExists);
        }
        let mut ancestor = Some(new_parent.index);
        while let Some(index) = ancestor {
            if index == node.index {
                return Err(VfsError::InvalidMove);
            }
            ancestor = self.nodes[&index]
                .parent
                .as_ref()
                .map(|(parent, _)| *parent);
        }
        self.directory_mut(old_parent).remove(&old_name);
        self.directory_mut(new_parent.index)
            .insert(new_name.clone(), node.index);
        self.nodes
            .get_mut(&node.index)
            .expect("validated node")
            .parent = Some((new_parent.index, new_name));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_exhaustion_does_not_mutate_tree() {
        let mut fs = MemoryFs::new();
        fs.next_id = u64::MAX;
        let root = fs.root();
        assert_eq!(
            fs.create_dir(&root, EntryName::new("full").unwrap()),
            Err(VfsError::IdExhausted)
        );
        assert!(fs.read_dir(&root).unwrap().is_empty());
        assert_eq!(fs.nodes.len(), 1);
        assert_eq!(fs.next_id, u64::MAX);
    }
}
