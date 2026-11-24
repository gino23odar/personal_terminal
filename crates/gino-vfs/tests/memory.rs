use gino_vfs::{EntryName, MemoryFs, Metadata, NodeId, NodeKind, VfsBackend, VfsError};

fn name(value: &str) -> EntryName {
    EntryName::new(value).unwrap()
}

// Observe complete reachable state, including identity, metadata and exact bytes.
// Rejected mutations must preserve this, not merely return the expected error.
fn state(fs: &dyn VfsBackend) -> Vec<(String, NodeId, Metadata, Vec<u8>)> {
    fn visit(
        fs: &dyn VfsBackend,
        id: NodeId,
        path: String,
        rows: &mut Vec<(String, NodeId, Metadata, Vec<u8>)>,
    ) {
        let metadata = fs.metadata(&id).unwrap();
        let bytes = if metadata.kind == NodeKind::File {
            fs.read_file(&id).unwrap().to_vec()
        } else {
            vec![]
        };
        rows.push((path.clone(), id.clone(), metadata.clone(), bytes));
        if metadata.kind == NodeKind::Directory {
            for entry in fs.read_dir(&id).unwrap() {
                visit(
                    fs,
                    entry.id,
                    format!("{path}/{}", entry.name.as_str()),
                    rows,
                );
            }
        }
    }
    let mut rows = vec![];
    visit(fs, fs.root(), String::new(), &mut rows);
    rows
}

#[test]
fn empty_root_and_object_safe_contract() {
    let mut backend: Box<dyn VfsBackend> = Box::new(MemoryFs::default());
    let root = backend.root();
    assert_eq!(
        backend.metadata(&root).unwrap(),
        Metadata {
            kind: NodeKind::Directory,
            byte_len: 0
        }
    );
    assert!(backend.read_dir(&root).unwrap().is_empty());
    let dir = backend.create_dir(&root, name("notes")).unwrap();
    let file = backend.create_file(&dir, name("note"), b"hello").unwrap();
    assert_eq!(backend.lookup(&root, &name("notes")).unwrap(), dir);
    assert_eq!(backend.lookup(&dir, &name("note")).unwrap(), file);
    assert_eq!(backend.read_file(&file).unwrap(), b"hello");
}

#[test]
fn names_are_single_components_without_normalization() {
    for invalid in ["", ".", "..", "a/b", "/", "a\\b", "a\0b"] {
        assert_eq!(EntryName::new(invalid), Err(VfsError::InvalidName));
    }
    for valid in [
        " spaced name ",
        "日本語",
        "é",
        "e\u{301}",
        "-flag",
        "~",
        "$HOME",
        ".hidden",
        "line\nbreak",
    ] {
        assert_eq!(name(valid).as_str(), valid);
    }
    assert_ne!(name("é"), name("e\u{301}"));
}

#[test]
fn binary_contents_and_metadata_are_exact_and_detached() {
    let mut fs = MemoryFs::new();
    let root = fs.root();
    let mut original = vec![0, 255, 128, b'a'];
    let file = fs.create_file(&root, name("binary"), &original).unwrap();
    original[0] = 42;
    let old = fs.metadata(&file).unwrap();
    let mut listing = fs.read_dir(&root).unwrap();
    listing[0].name = name("different");
    assert_eq!(fs.read_file(&file).unwrap(), &[0, 255, 128, b'a']);
    assert_eq!(
        old,
        Metadata {
            kind: NodeKind::File,
            byte_len: 4
        }
    );
    fs.replace_file(&file, "日本語".as_bytes()).unwrap();
    assert_eq!(fs.metadata(&file).unwrap().byte_len, 9);
    assert_eq!(old.byte_len, 4);
    assert_eq!(fs.lookup(&root, &name("binary")).unwrap(), file);
    fs.replace_file(&file, b"").unwrap();
    assert!(fs.read_file(&file).unwrap().is_empty());
    assert_eq!(fs.metadata(&file).unwrap().byte_len, 0);
}

#[test]
fn listings_are_sorted_and_case_sensitive() {
    let mut fs = MemoryFs::new();
    let root = fs.root();
    for entry in ["z", "a", "é", "A", ".hidden", "日本語"] {
        fs.create_file(&root, name(entry), b"").unwrap();
    }
    let listing = fs.read_dir(&root).unwrap();
    assert_eq!(
        listing.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
        vec![".hidden", "A", "a", "z", "é", "日本語"]
    );
    assert_eq!(listing, fs.read_dir(&root).unwrap());
}

#[test]
fn duplicate_wrong_kind_missing_and_root_errors_preserve_state() {
    let mut fs = MemoryFs::new();
    let root = fs.root();
    let dir = fs.create_dir(&root, name("dir")).unwrap();
    let file = fs.create_file(&dir, name("file"), b"safe").unwrap();
    let before = state(&fs);
    assert_eq!(
        fs.create_file(&root, name("dir"), b"bad"),
        Err(VfsError::AlreadyExists)
    );
    assert_eq!(
        fs.create_dir(&dir, name("file")),
        Err(VfsError::AlreadyExists)
    );
    assert_eq!(
        fs.create_file(&file, name("child"), b"bad"),
        Err(VfsError::NotDirectory)
    );
    assert_eq!(
        fs.create_dir(&file, name("child")),
        Err(VfsError::NotDirectory)
    );
    assert_eq!(fs.read_dir(&file), Err(VfsError::NotDirectory));
    assert_eq!(
        fs.lookup(&file, &name("child")),
        Err(VfsError::NotDirectory)
    );
    assert_eq!(fs.lookup(&dir, &name("missing")), Err(VfsError::NotFound));
    assert_eq!(fs.read_file(&dir), Err(VfsError::IsDirectory));
    assert_eq!(fs.replace_file(&dir, b"bad"), Err(VfsError::IsDirectory));
    assert_eq!(fs.remove_empty(&dir), Err(VfsError::DirectoryNotEmpty));
    assert_eq!(fs.remove_empty(&root), Err(VfsError::RootProtected));
    assert_eq!(
        fs.rename(&root, &dir, name("bad")),
        Err(VfsError::RootProtected)
    );
    assert_eq!(state(&fs), before);
}

#[test]
fn moving_subtree_preserves_identity_contents_and_parent_links() {
    let mut fs = MemoryFs::new();
    let root = fs.root();
    let a = fs.create_dir(&root, name("a")).unwrap();
    let b = fs.create_dir(&root, name("b")).unwrap();
    let nested = fs.create_dir(&a, name("nested")).unwrap();
    let file = fs.create_file(&nested, name("file"), b"bytes").unwrap();
    fs.rename(&nested, &b, name("moved")).unwrap();
    assert!(fs.read_dir(&a).unwrap().is_empty());
    assert_eq!(fs.lookup(&b, &name("moved")).unwrap(), nested);
    assert_eq!(fs.lookup(&nested, &name("file")).unwrap(), file);
    assert_eq!(fs.read_file(&file).unwrap(), b"bytes");
    fs.rename(&file, &nested, name("renamed")).unwrap();
    assert_eq!(fs.lookup(&nested, &name("file")), Err(VfsError::NotFound));
    assert_eq!(fs.lookup(&nested, &name("renamed")).unwrap(), file);
    let before = state(&fs);
    fs.rename(&file, &nested, name("renamed")).unwrap();
    assert_eq!(state(&fs), before);
    assert_eq!(
        fs.rename(&b, &nested, name("cycle")),
        Err(VfsError::InvalidMove)
    );
    assert_eq!(state(&fs), before);
    fs.remove_empty(&file).unwrap();
    fs.remove_empty(&nested).unwrap();
    assert!(fs.read_dir(&b).unwrap().is_empty());
}

#[test]
fn invalid_moves_do_not_detach_source_or_overwrite_destination() {
    let mut fs = MemoryFs::new();
    let root = fs.root();
    let dir = fs.create_dir(&root, name("dir")).unwrap();
    let child = fs.create_dir(&dir, name("child")).unwrap();
    let file = fs.create_file(&root, name("file"), b"keep").unwrap();
    let before = state(&fs);
    assert_eq!(
        fs.rename(&dir, &dir, name("self")),
        Err(VfsError::InvalidMove)
    );
    assert_eq!(
        fs.rename(&dir, &child, name("cycle")),
        Err(VfsError::InvalidMove)
    );
    assert_eq!(
        fs.rename(&dir, &root, name("file")),
        Err(VfsError::AlreadyExists)
    );
    assert_eq!(
        fs.rename(&file, &root, name("dir")),
        Err(VfsError::AlreadyExists)
    );
    assert_eq!(
        fs.rename(&child, &file, name("bad")),
        Err(VfsError::NotDirectory)
    );
    assert_eq!(state(&fs), before);
}

#[test]
fn removed_handles_never_alias_recreated_nodes() {
    let mut fs = MemoryFs::new();
    let root = fs.root();
    let removed = fs.create_file(&root, name("same"), b"old").unwrap();
    fs.remove_empty(&removed).unwrap();
    let replacement = fs.create_file(&root, name("same"), b"new").unwrap();
    assert_ne!(removed, replacement);
    let before = state(&fs);
    assert_eq!(fs.metadata(&removed), Err(VfsError::NotFound));
    assert_eq!(fs.read_file(&removed), Err(VfsError::NotFound));
    assert_eq!(fs.replace_file(&removed, b"bad"), Err(VfsError::NotFound));
    assert_eq!(fs.remove_empty(&removed), Err(VfsError::NotFound));
    assert_eq!(
        fs.rename(&removed, &root, name("bad")),
        Err(VfsError::NotFound)
    );
    assert_eq!(
        fs.rename(&replacement, &removed, name("bad")),
        Err(VfsError::NotFound)
    );
    assert_eq!(
        fs.create_dir(&removed, name("bad")),
        Err(VfsError::NotFound)
    );
    assert_eq!(state(&fs), before);
}

#[test]
fn foreign_handles_cannot_read_or_mutate_another_instance() {
    let mut a = MemoryFs::new();
    let mut b = MemoryFs::new();
    let ar = a.root();
    let br = b.root();
    let af = a.create_file(&ar, name("same"), b"a").unwrap();
    let bf = b.create_file(&br, name("same"), b"b").unwrap();
    assert_ne!(ar, br);
    assert_ne!(af, bf);
    let before_a = state(&a);
    let before_b = state(&b);
    assert_eq!(a.metadata(&bf), Err(VfsError::NotFound));
    assert_eq!(a.read_file(&bf), Err(VfsError::NotFound));
    assert_eq!(a.read_dir(&br), Err(VfsError::NotFound));
    assert_eq!(a.lookup(&br, &name("same")), Err(VfsError::NotFound));
    assert_eq!(a.create_dir(&br, name("bad")), Err(VfsError::NotFound));
    assert_eq!(
        a.create_file(&br, name("bad"), b"bad"),
        Err(VfsError::NotFound)
    );
    assert_eq!(a.replace_file(&bf, b"bad"), Err(VfsError::NotFound));
    assert_eq!(a.remove_empty(&bf), Err(VfsError::NotFound));
    assert_eq!(a.rename(&af, &br, name("bad")), Err(VfsError::NotFound));
    assert_eq!(a.rename(&bf, &ar, name("bad")), Err(VfsError::NotFound));
    assert_eq!(state(&a), before_a);
    assert_eq!(state(&b), before_b);
    drop(b);
    let fresh = MemoryFs::new();
    assert_eq!(fresh.metadata(&br), Err(VfsError::NotFound));
}
