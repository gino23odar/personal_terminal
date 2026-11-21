# WASM bridge — PR-07

`gino-wasm` adapts the existing `gino-shell` registry, interpreter and session to JavaScript. It contains no command implementations. The native CLI still uses the same dependency-free core. Neither adapter provides host-shell passthrough. PR-08 now connects React/xterm to this API; see [browser terminal](BROWSER_TERMINAL.md).

## Build and verify

Use the pinned Rust/Node/pnpm versions from the repository and a native C compiler/linker. Install:

```sh
rustup show
cargo install wasm-pack --version 0.15.1 --locked
pnpm install --frozen-lockfile
pnpm exec playwright install --with-deps chromium
pnpm check:wasm
```

`check:wasm` builds a release package using `wasm-pack --target web --locked`, checks the generated TypeScript API, and runs the same five JS fixture groups in Node and headless Chromium. The browser tests serve the generated module and WASM with the proper MIME type from a small allowlisted test server, then initialize it with the browser loader. They do not mock shell execution. Node loads the same artifact using `initSync`. `pnpm check` also runs Rust and web checks.

If the pinned browser download is unavailable locally, `PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH=/usr/bin/chromium pnpm test:wasm:browser` explicitly selects an installed Chromium. Record its version separately; CI uses Playwright’s pinned browser.

Generated JS, declarations and WASM live in `crates/gino-wasm/pkg/` and are ignored by Git, formatting and linting. Rebuild after Rust changes. wasm-opt is disabled explicitly: this baseline uses Rust release optimization without requiring another binary optimizer. Dependency versions and lockfiles are committed. macOS, Windows, Firefox and WebKit have not been validated by this suite.

## JavaScript API

After building, an ES module consumer can use:

```js
import init, { WasmShell } from "./crates/gino-wasm/pkg/gino_wasm.js";
await init();
const shell = new WasmShell();
try {
  const result = await shell.execute('echo "hello from Rust"');
  console.log(result.events); // [{ type: "stdout", text: "hello from Rust\n" }]
  console.log(shell.snapshot());
} finally {
  shell.free();
}
```

Serve the package over HTTP for browsers; this is not a file-URL or connected application example. Generated declarations export `ExecutionResult`, `OutputEvent`, `ShellFailure` and `SessionSnapshot`. Consumers must include TypeScript’s `ESNext.Disposable` library for wasm-bindgen’s generated `Symbol.dispose` declaration (explicit `free()` also works). Type assertions in `tests/wasm/types.ts` verify this public surface after generation.

| Method            | Contract                                                                                    |
| ----------------- | ------------------------------------------------------------------------------------------- |
| `new WasmShell()` | Independent registry and fresh virtual session at `/home/visitor`                           |
| `execute(input)`  | Returns `Promise<ExecutionResult>`; one active execution per shell                          |
| `cancel()`        | Cancels the active execution's token; returns false when idle                               |
| `snapshot()`      | Returns a detached `{ cwd, lastStatus, exitStatus }` value                                  |
| `reset()`         | Restores a fresh session, including reopening after exit                                    |
| `free()`          | Releases the JS handle; an already returned promise retains its Rust state until completion |

Overlapping `execute`, `snapshot` and `reset` calls throw a synchronous JavaScript Error with a busy message. Await execution before using them. Ordinary shell failures resolve normally: `status`, ordered `events`, nullable `error` and final `session` are returned. Failures include a typed `kind`, message and nullable UTF-8 byte span `[start, end]`. Optional values serialize as `null`, not `undefined`. Unexpected boundary serialization failures reject the promise.

Events are `stdout` or `stderr` with text, or `clearScreen` without text. Core diagnostics already appear in stderr; clients must not print the structured error a second time. `exit` closes only that virtual session. Further commands on a closed session return its status without dispatch, until reset. Blank input preserves the previous status. Status and command semantics are documented in [COMMANDS.md](COMMANDS.md).

## Lifetime, cancellation and limits

Execution temporarily owns the session through an RAII lease. No mutable WASM export borrow or RefCell borrow crosses an await. Dropping the lease restores the session and clears the active token, including on serialization failure. Separate instances remain independent; snapshots cannot mutate Rust state.

A fresh token is created for each execution. Calling `cancel()` immediately after `execute()` can cancel before its future starts; tests verify status 130, no exit mutation and successful subsequent execution. Cancellation remains cooperative: it cannot preempt synchronous Rust code, and JavaScript timers/key handlers cannot run while a synchronous computation occupies the browser thread. The six current built-ins complete quickly. There is no worker scheduler or long-running command claim in this PR.

Output is a bounded batch, returned when execution settles: at most 256 KiB of UTF-8 bytes and 1,024 events per execution. Exceeding either limit or writing invalid UTF-8 returns an I/O failure without appending the rejected write. Existing accepted output is retained. Each write must be valid UTF-8; arbitrary fragmented binary streams are unsupported. stdin immediately returns EOF. Streaming/backpressure and pipelines remain PR-16; no streaming API is implied by the Promise.

The core's 64 KiB parser input limit still applies, but JavaScript strings are copied before parsing; this is not a pre-allocation memory cap. The PR-08 UI applies a tighter 4,096-code-unit input bound. Output text is untrusted data, not HTML or executable terminal control. The PR-08 renderer neutralizes control sequences and renders `clearScreen` explicitly. No DOM rendering, filesystem access, persistence, network service or portfolio data is introduced here.
