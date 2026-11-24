# V0.1 technical audit — PR-09

V0.1's scope is architectural proof: an independently runnable native Linux CLI and a browser terminal execute the same basic Rust shell through platform adapters. This audit covers PR-01 through PR-09. PR-09 was accepted and merged with [hosted CI passing](https://github.com/gino23odar/personal_terminal/actions/runs/44957638820). This audit records the PR-09 boundary; later work is tracked in the implementation plan. It does not publish a release or claim V0.5/V1.0.

## Acceptance evidence

| Criterion                                                    | Implementation                                                          | Verification                                                                                                                |
| ------------------------------------------------------------ | ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Rust owns parsing, registry, interpretation and six commands | `gino-shell` with no production dependencies                            | Core contract/registry/parser/interpreter tests; native and WASM target builds                                              |
| Native Linux CLI works without browser/backend               | `gino-cli` input loop and bounded native adapter                        | CLI startup tests; Python PTY commands, editing, history, Ctrl+C/D, resize, exit and termios restoration                    |
| Browser uses the same compiled engine                        | `gino-wasm` plus React/Vite/xterm presentation                          | Node and Chromium bridge tests; production browser tests, including commands after network access is blocked                |
| Native and WASM outcomes agree                               | Shared reviewed fixture JSON and independent native transcript producer | Exact equality against expectations and between native Rust, Node WASM and Chromium WASM                                    |
| Both real adapters execute a shared scenario                 | The `adapter-smoke` fixture                                             | CLI PTY output/process status and production browser transcript/status checks                                               |
| Errors do not silently dispatch unsupported syntax           | Typed syntax/usage/lookup errors                                        | Error fixtures, byte spans, recovery, invalid exit values and unsupported operators                                         |
| Session lifecycle and cancellation remain portable           | Core sessions and per-execution token                                   | Exit/reset, ignored input after close, previous-status exit and pre-start cancellation fixtures; existing pending-I/O tests |
| No host-shell passthrough or duplicate TS interpreter        | Core handlers; native I/O; browser input/rendering only                 | Code review; `/bin/sh` rejected; CLI `-c` rejected; no command execution from URLs                                          |
| Verification and architecture are reproducible               | Pinned toolchains, lockfiles, generated WASM excluded from Git          | Rust/web/WASM CI jobs, now including parity checks; documented root `pnpm check`                                            |

## Exact parity boundary

`tests/parity/fixtures.json` is the reviewed source of inputs and expected results. Five independent scenarios contain 34 steps and cover all six commands, help metadata, Unicode, quoted/empty/escaped arguments, literal unexpanded values, errors and exact diagnostics, UTF-8 spans, blank input, exit/reset, cancellation, newline/control rejection and the 64 KiB parser boundary. A compact repeat descriptor avoids committing a 65 KiB string.

`crates/gino-shell/examples/parity.rs` is a test-only native executable. It invokes the real `execute_line` and captures ordered output events; it never interprets commands itself. It validates every step against the fixture and emits JSON. `serde_json` is a dev-dependency only; the portable library still has no normal dependencies. The production CLI has no new batch mode.

`tests/parity/node.mjs` runs that executable, checks its transcript against the same fixture, executes the generated WASM API and compares full results. Only after success does it write ignored `tests/parity/generated/native.json`. It removes any older baseline first. Chromium then executes the same fixtures with the browser WASM loader and compares against both the fresh native transcript and explicit expectations. Missing generated data fails rather than silently skipping tests.

Equality includes ordered stdout/stderr/clear events, exact text, status, structured error kind/message/byte span, and session cwd/lastStatus/exitStatus. Object key order is irrelevant; text and event order are exact. No output is normalized at this engine boundary. Each scenario uses a fresh session, and reset is verified within a scenario. Independently authored expected results prevent two runtimes agreeing on a regression from passing unnoticed. Changes to expected behavior require explicit fixture review; there is no snapshot update command.

The same `adapter-smoke` scenario traverses the real CLI and production browser. The PTY combines stdout/stderr and strips presentation ANSI, prompt and CRLF/edge newlines before comparing text; clear is checked as an actual terminal control and final exit as the process status. The browser compares its full plain-text output log and displayed statuses. Existing browser tests additionally assert actual xterm rendered content and clear behavior. These adapter tests complement, rather than replace, exact engine comparisons.

## Intentional differences and remaining limits

- Native uses Rustyline; browser uses an HTML command input and xterm output. Input editing and terminal rendering are adapter responsibilities, not byte-identical terminal protocols.
- Idle Ctrl+C in the native CLI records status 130; the browser currently clears unsubmitted text while retaining the preceding execution status. Active/pre-start command cancellation returns 130 in both. This UI difference is recorded, not hidden by the engine parity claim.
- Native Ctrl+D on nonempty input deletes at the cursor; the browser only binds empty-input Ctrl+D to exit. Browser Tab follows normal focus order; completion is not implemented.
- Browser input is capped at 4,096 UTF-16 code units, below the core's 64 KiB UTF-8 parser limit. Direct bridge fixtures test larger/core-only input; it is not asserted that the UI admits those strings.
- Clear is a portable event. Browser additionally clears scrollback and its accessible log; native emits its documented screen-clear sequence. No assertion compares their raw escape sequences.
- WASM captures batches limited to 256 KiB / 1,024 events; native uses bounded handoff. Streaming/pipes remain PR-16. The parity producer only supports the ready-I/O built-ins; a future pending command fails explicitly rather than pretending to schedule it.
- Cancellation is cooperative and cannot preempt synchronous Rust. Browser tests do not demonstrate interruption of long-running computations. Existing core/native tests cover cancellation during pending operations.
- Linux, Node and Chromium are the verified platforms. macOS/Windows, Firefox/WebKit, real mobile IME and assistive software are not comprehensively tested. Basic accessibility support is not an accessibility certification.
- History/session are memory-only. VFS, expansion, composition, portfolio facts, games, AI, public deployment and release packaging remain later roadmap work. Licensing still requires the owner's decision before broad public collaboration.

## Reproduce the gate

Install the pinned Rust/Node/pnpm toolchains, a C linker/compiler, Python 3, wasm-pack 0.15.1 and Playwright Chromium. Then:

```sh
pnpm install --frozen-lockfile
pnpm exec playwright install --with-deps chromium
pnpm check
```

For the new exact-comparison suite alone:

```sh
pnpm check:parity
```

For the adapter smoke tests: `pnpm check:rust`, then `pnpm check:web` and `pnpm test:web:browser`. `pnpm test:parity:browser` requires a successful `pnpm test:parity:node` since the last source/fixture change; `check:parity` enforces the order. CI does this on every PR. Actual run results and environment limitations are recorded in the PR description, not inferred from a checklist.

After owner acceptance of PR-09 with all mandatory jobs green, the V0.1 architectural gate is satisfied for this documented scope. PR-10 may then introduce portable VFS types, in-memory backend and service contracts only; paths, commands and persistence remain PR-11 through PR-14.
