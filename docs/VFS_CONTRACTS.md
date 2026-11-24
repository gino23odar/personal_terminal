# Portable VFS storage contracts — PR-10

## Implemented boundary

`gino-vfs` is a dependency-free Rust library with an object-safe `VfsBackend`
contract and a working `MemoryFs`. It stores a node-addressed tree in memory and
compiles natively and for `wasm32-unknown-unknown`. It does not depend on the
shell, native terminal, DOM, host filesystem, environment, clock or executor.

This PR implements storage primitives, not shell filesystem commands. The CLI
and browser still expose only the six existing commands. `ShellSession::cwd`
remains a validated string, not a VFS lookup. The crate is exercised directly by
behavior tests and a runnable Rustdoc example; no placeholder command/context
capability or unused adapter wiring is added.

## Types and identity

- `EntryName::new` validates one UTF-8 component: empty names, `.`/`..`, slash,
  backslash and NUL are rejected. Spaces, Unicode, leading dashes, dot names,
  literal `~`/`$HOME` and other control characters are preserved. There is no
  expansion, Unicode normalization or case folding. These names remain untrusted
  display text; future commands must use the adapters' existing sanitization.
- `NodeId` is opaque and scoped to one filesystem instance. It is cloneable but
  not a path, persistent identifier or transferable authority to another
  filesystem. A private `Rc` identity distinguishes instances without global
  atomics, randomness, thread constraints or host facilities. Equality checks
  instance identity and a monotonically allocated numeric index. Debug output
  omits the identity pointer and must not be used as an ID serialization.
- `NodeKind` is `File` or `Directory`. `Metadata` is detached data with kind and
  `byte_len`: exact logical byte length for files, zero for directories. It does
  not fabricate timestamps, mode bits, users or host inode numbers.
- `DirectoryEntry` contains an ID, validated name and detached metadata. Listing
  results cannot mutate the backend. `read_file` borrows exact bytes, including
  invalid UTF-8 and embedded NUL, while preventing mutable access during the borrow.

A fresh `MemoryFs` has only an empty directory root. It owns all writable nodes
and bytes. The backend deliberately does not implement `Clone`; handle cloning
never shares writable filesystem state with a different backend. Dropping it
releases its nodes; retained handles keep only the small identity allocation.
Creating a new backend is a fresh tree and old handles are rejected.

## Service contract

Methods are synchronous and require no `Send`, `Sync` or async runtime. They
compose through `dyn VfsBackend`. Existing shell commands remain asynchronous;
in-memory storage does not need fake async wrappers. Future persistence can
save/load validated snapshots at adapter boundaries rather than make each
lookup perform platform I/O.

| Operation                    | Semantics                                                                                                                                                                    |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `root`                       | Returns this instance's root handle.                                                                                                                                         |
| `metadata`                   | Detached kind/length; reads do not mutate metadata.                                                                                                                          |
| `lookup`                     | Exact direct-child lookup by validated name.                                                                                                                                 |
| `read_dir`                   | Owned entries sorted by case-sensitive UTF-8 name, without synthetic `.`/`..`.                                                                                               |
| `read_file`                  | Borrowed exact bytes; a directory is an error.                                                                                                                               |
| `create_dir` / `create_file` | Parent must exist and be a directory; duplicate name is an error, never overwrite. File input is copied.                                                                     |
| `replace_file`               | Complete byte replacement of an existing file; no append or offset semantics.                                                                                                |
| `remove_empty`               | Remove a file or empty directory; reject nonempty directories and root.                                                                                                      |
| `rename`                     | Move/rename within this tree, preserving ID, contents and descendants. No overwrite; identical parent/name is a successful no-op. Reject root movement and directory cycles. |

A moved directory's children remain addressable with their original handles.
Parent links are updated so later moves/removals still enforce tree integrity.
Deleted IDs are never reused; re-creating the same name yields a new ID. The
allocator uses checked addition and returns `IdExhausted` without changing the
tree when its counter cannot advance.

## Errors and atomicity

Typed `VfsError` variants are `InvalidName`, `NotFound`, `AlreadyExists`,
`NotDirectory`, `IsDirectory`, `DirectoryNotEmpty`, `RootProtected`,
`InvalidMove` and `IdExhausted`. Diagnostics are fixed virtual-storage messages;
they expose neither supplied names nor host paths. Shell error/status mapping
will be defined when commands consume the service.

Missing, deleted and foreign-instance handles return `NotFound`. Methods
validate source identity/existence before kind checks; creation validates parent
and collision before allocating an ID. Rename checks source/root, destination
kind, same-entry no-op, collision, then ancestry before detaching the source.
Callers must not infer an arbitrary global error precedence beyond these rules
when more than one argument is invalid.

Every returned mutation error leaves the tree, file bytes and existing IDs
unchanged. Validation precedes commit; no fallible storage operation or await
occurs halfway through commit. Standard allocator failure/panic is not modeled
as a recoverable transaction error. The library does not promise rollback of
process termination or allocator failure.

There is no recursive copy/remove, streaming, persistence, access policy or
resource quota in PR-10. Operations allocate in memory; callers are trusted
library consumers. This backend is not an untrusted-code sandbox and must not
be exposed directly to shell commands once policy enforcement is introduced.

## Evidence and reproduction

```sh
cargo test -p gino-vfs --locked
cargo build -p gino-vfs --target wasm32-unknown-unknown --locked
cargo check -p gino-vfs --tests --target wasm32-unknown-unknown --locked
pnpm check:rust
```

Tests cover the trait-object contract, empty root, nested nodes, exact binary
bytes, Unicode byte lengths/names, deterministic ordering, detached metadata,
wrong-kind/missing/duplicate failures, root/nonempty protection, subtree moves,
no-op rename, cycles, collisions, stale/foreign handles, independent instances
and identifier exhaustion. Rejected mutation tests compare complete reachable
state including IDs, metadata and bytes.

`check-rust.sh` runs native tests, formatting, Clippy and builds, plus production
and test compilation for both portable crates on WASM. WASM compilation is not
browser execution of these new VFS operations. Existing bridge/UI/parity suites
continue to exercise the unchanged shell. Full VFS runtime parity arrives in
PR-14 after command and adapter integration.

## Next boundary: PR-11

After PR-10 acceptance, add component-aware virtual path resolution, read-only
mounts, permissions/capabilities, quotas and isolation regression tests around
this storage model. Normal command consumers must use the future policy service
rather than an unrestricted backend handle. Shell filesystem commands remain
PR-12, persistence PR-13 and VFS parity/state audit PR-14. No portfolio facts,
seed mount layout or future flags are advertised by this PR.
