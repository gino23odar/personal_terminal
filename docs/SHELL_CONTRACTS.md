# Portable shell contracts (PR-02)

## Scope and acceptance

PR-02 builds on merged PR-01. It establishes types and object-safe interfaces inside `gino-shell` and tests their composition. It adds no registry, parser, dispatcher, built-ins, native input loop, browser bridge or API. CLI and browser output intentionally remain unchanged.

Acceptance requires native formatting, lint, tests and builds; a WASM library build; and a WASM compile check of the contract tests. Runtime parity cannot be claimed until adapters exist. All test handlers and in-memory adapters live under `tests/` and are not product commands.

## Session ownership

`ShellSession` owns a virtual current directory, environment map, alias map and last exit status. A new session starts at `/home/visitor`, with empty maps and success status; it never reads host environment variables or paths. Alias storage does not implement alias expansion. History storage and execution lifecycle orchestration are deferred.

`set_cwd` accepts a canonical absolute virtual path, rejecting empty/relative paths, empty components, dot/parent components, NUL and backslashes. It performs no normalization, permission checks or filesystem lookup. The VFS must eventually resolve paths and authorize access before calling it; this small validation is not a filesystem sandbox.

`snapshot()` returns detached owned data. Changing a snapshot cannot change the session, and subsequent session changes cannot alter the snapshot. `CommandContext` explicitly borrows the mutable session; exclusive Rust borrowing prevents simultaneous mutation through that context. Session edits take effect immediately and are not transactional. The future interpreter must record the final status and manage execution lifetime.

## Commands and results

`Command::execute` accepts already-separated arguments, context and a dynamic `CommandIo`. It returns a boxed `LocalFuture<CommandResult>` that can borrow all inputs. No `Send`, `Sync`, thread, runtime, parser or registry dependency is imposed. This permits `Rc` and future browser-owned values and supports `dyn Command` registration in PR-03.

`Ok(ExitStatus(n))` is a normal command outcome, including ordinary nonzero results. `Err(ShellError)` represents cancellation, invalid input, I/O failure or an unsupported operation. Error status mapping is 130, 2, 1 and 1 respectively. Adapters must supply safe error messages rather than embedding secrets or host details. The status type is an eight-bit shell code, not a host process status.

Context currently grants only session mutation and cancellation. VFS, network, portfolio and animation capabilities must be added explicitly in their owning PRs. This is an architectural boundary for trusted Rust code, not a sandbox for arbitrary third-party Rust plugins.

## Async I/O and terminal presentation

`CommandIo` is byte-oriented so it can later support pipelines without requiring UTF-8. Reads use caller-sized buffers; for nonempty buffers zero means EOF. An empty buffer returns zero without consuming input. Writes acknowledge a complete slice or return an error; errors or cancellation may follow partial side effects.

Implementations must bound their internal queues and leave a write pending while capacity is unavailable. A pending operation registers a waker; it must not spin or block the executor. PR-02 defines this obligation and tests cancellation of a simulated blocked write; it does not implement transport queues or the PR-16 pipeline runtime. Separate stdout/stderr tags preserve stream identity.

`TerminalEvent::ClearScreen` is a structured presentation control, not an escape string mixed into command bytes. Unsupported controls must produce an explicit error. `TerminalAdapter` abstracts dimensions, normalized text/control input, byte rendering and terminal controls. It does not parse commands. The event vocabulary is deliberately minimal and can grow when concrete input editing arrives in PR-06/08. Adapters must avoid interpreting output as HTML and will own terminal escape handling.

Native raw-mode setup, resizing and cleanup belong to a concrete adapter, including error/drop paths. No raw-mode resource is acquired in PR-02. Keyboard events and terminal input are separate from command stdin, which will allow redirected input later.

## Cancellation and executor rules

Create a new `CancellationToken` for every execution. Clones share state, cancellation is permanent/idempotent, and independent tokens are isolated. Tokens and futures are local to one executor thread. A native event loop must forward interrupt events to that executor rather than move these tokens across threads.

`cancelled()` is awaitable and independently registers a waker per waiter. Repolling replaces the waiter’s waker; dropping it removes its registration. `cancel()` marks the token first, removes registrations, then wakes outside internal borrows. Cancellation before the first poll is immediately observable.

`run_until_cancelled(operation)` polls cancellation before the operation and drops pending work when cancelled. If both are ready on the same poll, cancellation wins. Its outer error represents cancellation; the operation's own result is preserved inside `Ok`. Commands can wrap their entire async body, and future adapters may wrap individual I/O operations with the same token.

Cancellation is cooperative: it cannot preempt synchronous CPU loops, roll back session mutations or undo emitted bytes. CPU-heavy handlers must periodically yield/check the token. I/O futures must be drop-safe and release subscriptions/resources on drop. Concrete adapter tests must enforce those responsibilities when the adapters arrive.

## Verification and next boundary

`cargo test --workspace --locked` executes native behavior tests. `cargo check -p gino-shell --tests --target wasm32-unknown-unknown --locked` type-checks the same contract fixtures, including non-Send fake handlers/adapters, for WASM. It does not execute tests in a browser. Both checks are in `scripts/check-rust.sh`, so existing CI picks them up.

PR-03 now consumes `dyn Command` for metadata, registration, aliases and lookup/help discovery; see [registry rules](COMMAND_REGISTRY.md). The registry preserves these ownership and cancellation rules. The CLI will only execute commands once the later interpreter and native adapter work is implemented.

## PR-05 integration update

The historical PR-02 scope above describes the original contracts. PR-05 adds read-only registry access to CommandContext, typed syntax/unknown-command errors (statuses 2/127), and session close/reset/exit snapshots. The interpreter now owns dispatch, error reporting, final status recording and cancellation orchestration. See [current execution rules](COMMANDS.md). Concrete adapters and history remain deferred.

## PR-06 integration update

The native CLI now supplies line editing, local future polling, cancellation forwarding and bounded CommandIo delivery. See [native adapter semantics and limits](NATIVE_CLI.md). Native history is session-only; browser adapters and persistence remain deferred.
