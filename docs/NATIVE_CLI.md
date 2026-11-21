# Native Linux CLI — PR-06

Run from an ordinary interactive terminal:

```sh
cargo run -p gino-cli --locked
```

The CLI prints a short banner and `visitor@gino.os:~$ `, then calls the shared Rust interpreter for each submitted line. No browser, server or profile data is needed. The current commands are `help`, `echo`, `pwd`, `whoami`, `clear` and `exit`; see [command semantics](COMMANDS.md). This is a virtual shell. Neither unknown commands nor paths such as `/bin/sh` launch host processes.

## Input and shutdown

- Enter submits a line. Left/right, Home/End and Backspace/Delete edit it using Rustyline's default Emacs bindings and Unicode support.
- Up/down recall in-memory history. At most 100 nonblank submitted lines of at most 65,536 bytes are retained; consecutive duplicates are ignored. Failed commands can appear in history. No history files are read or written, and history does not survive restart.
- Ctrl+C while editing discards the line, records status 130 and displays a new prompt. External SIGINT during editing is handled the same way.
- Ctrl+C/SIGINT during execution sets an atomic flag. The native executor forwards it to the current local cancellation token and polls execution to completion. A fresh token is used for the next line.
- Ctrl+D on an empty line ends the session with its last status. On a nonempty line it follows Rustyline's delete-character behavior.
- `exit` or `exit N` closes the virtual session and returns its status to the calling process. Invalid exit arguments leave the prompt active.
- Syntax/lookup/usage errors are already printed by the core and are not printed twice. Terminal transport failures terminate with status 1.
- Rustyline redraws the prompt after terminal resizing. There is no alternate screen or fullscreen mode.

Both stdin and stdout must be terminals. Redirected/piped startup, CLI arguments and `TERM=dumb` are explicitly rejected with status 2; batch execution is not implemented. Stderr can be redirected independently. The host environment is used only for native terminal handling, never for the virtual user's identity or directory.

Rustyline owns raw mode only while reading input, and restores it using its cleanup guard on normal return, EOF, interruption and unwinding. Command execution runs in the restored terminal mode. The CLI returns an exit code normally, allowing destructors to run. Forced termination (for example SIGKILL or process abort) cannot guarantee cleanup; signal-driven suspension and general job control are not implemented.

## Native I/O boundary

The core remains dependency-free and platform-neutral. Only `gino-cli` depends on pinned Rustyline 18.0.1 and signal-hook 0.3.18, with transitive versions in Cargo.lock. Rustyline's file-history features are disabled. Its line-oriented API owns editing; the earlier key-event `TerminalAdapter` contract remains available for future event-driven presentation and is not wrapped in a blocking fake async input implementation here.

The native `CommandIo` uses a single bounded handoff slot. Each write is split into at most 4 KiB chunks and remains pending until the executor renders and acknowledges each chunk. Pending futures register a waker; dropping a request removes queued work. The native executor polls local futures, drains output and parks between wakeups, checking SIGINT at least every 10 ms while waiting. This is terminal delivery only, not the PR-16 pipe runtime. Command stdin is EOF; keyboard editing is separate and no current command consumes stdin.

Stdout/stderr remain distinct. UTF-8 is decoded across chunks; invalid sequences become replacement characters, and incomplete final sequences are flushed at the end of an execution. Control characters other than newline/tab are escaped as visible text, preventing command output from injecting terminal escape sequences. Clear-screen ANSI is emitted only for the structured `ClearScreen` event; it clears the screen and moves the cursor home, without deleting scrollback.

Cancellation is cooperative. It cannot interrupt synchronous handler code or undo rendered output. The final native terminal writes are synchronous and bounded per chunk: if the OS blocks a write to a stalled terminal, cancellation waits for that write to return. A fully nonblocking transport is not claimed. The parser rejects oversized submitted lines, but the line editor can allocate a larger edit buffer before submission; this is a local interactive application, not a public input server.

## Verification

`cargo test -p gino-cli --locked` includes native adapter tests and, on Linux, a Python 3 standard-library PTY harness. Python 3 is required for this test, not for running the application. CI's Rust job runs it through `cargo test --workspace`.

The PTY harness starts the compiled CLI directly, drives command/edit/history/resize/Ctrl+C/D interactions, checks outputs and exit codes, and compares the terminal attributes before/after shutdown. Unit tests cover bounded output, dropped writes, Unicode/control rendering and pending-execution cancellation. This is Linux validation, not macOS/Windows certification. Unix support is compiled conditionally; non-Unix startup reports unsupported status 2.

Manual review:

```text
help
help echo
echo "hello world"
pwd
whoami
clear
missing
exit 7
```

Also try Up/Down, edit a recalled command, Ctrl+C with unfinished input and Ctrl+D at an empty prompt. After exit, the host terminal should behave normally. The caller's `$?` should reflect the final virtual status.

## PR-07 boundary

After PR-06 acceptance, add the planned WASM bridge over the same core API. Browser input/output and JavaScript tests must not reuse native OS dependencies or duplicate the interpreter. Browser terminal UI remains PR-08. No WASM bridge, VFS, new commands or shell composition is added here.
