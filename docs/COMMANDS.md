# Core commands — PR-05

These commands execute through the portable Rust `execute_line` API and `builtin_registry()`. They are tested with in-memory I/O. The native executable and web page are still scaffolds; interactive native input starts in PR-06, the WASM bridge in PR-07 and browser terminal in PR-08.

| Command                | Behavior                                                                                                                                             |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `help [command]`       | List registered commands in sorted order, or show usage for one name/registry alias. Reads the live registry, including extensions.                  |
| `echo [--] [text ...]` | Join decoded arguments with one space, append a newline. No variable or backslash expansion beyond parser decoding.                                  |
| `pwd`                  | Print the session's virtual directory plus newline; initially `/home/visitor`. No host or VFS lookup.                                                |
| `whoami`               | Print `visitor` plus newline, independent of host identity or session environment entries.                                                           |
| `clear`                | Emit `TerminalEvent::ClearScreen`, without output bytes. Unsupported adapters report an error.                                                       |
| `exit [status]`        | Close the virtual session with an ASCII decimal status from 0 through 255, or the previous status when omitted. Does not terminate the host process. |

No flags are supported. `pwd`, `whoami` and `clear` reject all arguments. `help` rejects multiple arguments or a leading hyphen in its argument; use `help echo`, not `echo --help`. `echo` rejects a leading option-like argument (except a lone `-`); use `echo -- -n` to print `-n`. After its first text argument, hyphen-prefixed words are ordinary text. Quotes do not change option handling. `exit` rejects signs, empty/nondecimal values, overflow and extra arguments, leaving the session active. Leading zeros are accepted. This is a documented subset, not full POSIX/GNU compatibility.

Examples through the Rust API:

| Input                   | Output / outcome                                               |
| ----------------------- | -------------------------------------------------------------- |
| `echo 'hello world' ""` | `hello world ` followed by newline, status 0                   |
| `echo "$HOME"`          | Literal `$HOME` followed by newline, status 0                  |
| `pwd`                   | `/home/visitor` followed by newline in a new session, status 0 |
| `echo -n`               | Usage diagnostic on stderr, status 2                           |
| `missing`               | Command-not-found diagnostic on stderr, status 127             |
| `echo hello \| world`   | `hello                                                         | world` followed by newline, status 0                        |
| `echo hello             | other`                                                         | Syntax diagnostic on stderr, status 2; neither command runs |
| `exit 7`                | No output, status 7; session closes                            |

## Execution and lifecycle contract

Call `execute_line(input, &registry, &mut session, &mut io, &token).await` with one logical line without its terminator. Create a fresh `CancellationToken` per call. The adapter owns polling and interrupt delivery. A mutable session borrow serializes execution; different sessions can share an immutable registry. No host shell fallback exists. Commands are independent `Command` handlers, not an interpreter switch. `CommandContext` now includes a read-only registry for discovery.

A new session is active, starts at `/home/visitor`, and has empty environment/alias maps and status 0. Blank input emits nothing and preserves status. Completed execution records its success or error status. Closing is idempotent; calls on a closed session return its exit status without parsing, I/O or cancellation checks. `reset()` restores the default session state and clears maps/status/exit state, without changing registry or adapter resources. Snapshots include the exit state. History and persistence remain deferred.

| Result                                         | Status                          |
| ---------------------------------------------- | ------------------------------- |
| Successful built-in                            | 0 (or the selected exit status) |
| Syntax / invalid arguments                     | 2                               |
| Unknown command, including unknown help target | 127                             |
| I/O failure / unsupported terminal operation   | 1                               |
| Cancellation                                   | 130                             |

`execute_line` returns `CommandResult`. Ordinary handler nonzero statuses remain `Ok(status)`; syntax, lookup, usage, unsupported controls, transport errors and cancellation remain typed `Err(ShellError)`. The session records `error.exit_status()` for errors. Syntax errors retain the parser's typed kind and UTF-8 span.

The interpreter writes one `gino: ...` diagnostic line to stderr for syntax, lookup, usage and unsupported-operation errors. Control characters in diagnostic text are escaped. It returns cancellation and I/O errors silently for adapter handling; adapters should not print already-rendered diagnostics again. If diagnostic delivery fails, that transport error replaces the original result and records status 1. If delivery is cancelled, status 130 wins. No recursive error reporting occurs.

Cancellation races parsing, dispatch and diagnostic delivery as one async operation. Before the first poll it prevents execution entirely; while awaiting I/O it drops pending work. Synchronous code is cooperative and cannot be preempted. Partial output or session mutations are not rolled back. Directly dropping the execution future is not a completed execution and does not record a final status; adapters should cancel and poll to completion. No runtime, thread or external dependency is added.

## Verification and PR-06 handoff

Run `cargo test -p gino-shell --test interpreter --locked`, or `pnpm check` for the complete baseline. Tests cover registry extensions and aliases, all built-ins, status/diagnostic behavior, malformed input without dispatch, session isolation/reset, unsupported controls, transport failure and cancellation while output is pending. WASM checks compile these tests but do not run them in a browser.

PR-06 requires PR-05 acceptance and passing checks. It should add native input/history, polling and I/O adapters, Ctrl+C/D handling, lifecycle-driven shutdown and terminal cleanup. It must preserve this core behavior and keep host commands disabled. Pipes, redirects, expansion, VFS and browser execution remain at their roadmap PRs.
