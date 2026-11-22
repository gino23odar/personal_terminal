# Browser terminal — PR-08

The React/Vite application now loads the PR-07 WASM package and executes the same six Rust commands as the native CLI. xterm.js displays output. A labeled HTML command input supplies native browser editing, selection, IME and clipboard behavior; there is no TypeScript parser or command dispatcher. This is a development shell, not the portfolio release.

## Run and review

Install the pinned toolchains and wasm-pack as described in README. From the repository root:

```sh
pnpm install --frozen-lockfile
pnpm dev
```

The root command builds WASM before starting Vite. Open the URL Vite prints. After Rust changes, restart `pnpm dev` to regenerate the package; Vite does not watch Rust. Direct `pnpm --dir apps/web dev` assumes the WASM package has already been generated.

Try `help`, `help echo`, `echo "hello world"`, `pwd`, `whoami`, an unknown command, an unmatched quote, `clear` and `exit 7`. Expect Rust diagnostics and visible exit status; errors must allow another command. `clear` erases output/scrollback. `exit` closes the virtual session without closing the tab; **New session** resets Rust state, input and history. Each tab has its own in-memory session. Reloading loses state.

| Input                                  | Behavior                                                           |
| -------------------------------------- | ------------------------------------------------------------------ |
| Enter / Run                            | Submit exactly one line to Rust                                    |
| Left/right, Home/End, Backspace/Delete | Native input editing                                               |
| Up/down                                | Recall up to 100 submitted nonblank lines; down restores the draft |
| Ctrl+C with no selection               | Clear idle input or cancel active Rust execution                   |
| Ctrl+C with selection                  | Preserve ordinary browser copying                                  |
| Ctrl+D on empty input                  | Execute Rust `exit`                                                |
| Escape                                 | Clear the editable line                                            |
| Tab                                    | Normal accessible focus navigation, not completion                 |
| Paste                                  | Insert one line only; never automatically execute                  |

Input is limited to 4,096 UTF-16 code units (at most 16 KiB UTF-8), below the core parser cap. Multiline/control-character or oversized paste is rejected explicitly. IME composition does not trigger shell shortcuts. The terminal output area is read-only; enter commands in the labeled input beneath it.

## Lifecycle and safety

Loading disables submission. A failed import/WASM load shows an error and **Reload shell** retries with a fresh component. A synchronous execution guard prevents duplicate submissions even before React renders the busy state. Reset is only offered after execution ends. On unmount/retry, observers and terminal resources are disposed, cancellation is requested and the WASM handle is freed. Stale initialization/results do not update an unmounted component. StrictMode's effect replay is supported.

Output is untrusted. The renderer prints C0/C1 controls, DEL and bidi override/isolate controls visibly, retaining only newline and tab. ESC cannot introduce ANSI, OSC hyperlinks or clipboard operations, including across writes. HTML is never inserted as markup. Only trusted UI code generates clear-screen and stderr-color sequences. No link addon, remote shell, URL command execution or backend is connected. The UI shows core diagnostics once; structured errors supply lifecycle feedback rather than duplicate error lines.

xterm scrollback is bounded to 1,000 lines. A separate plain-text log capped at 32,768 code units exposes recent output to assistive technology while the graphical terminal is hidden from screen readers. The input and status have accessible names; focus moves to input after initialization/execution. ResizeObserver fits the terminal to its container; no animation or external fonts are used. This is basic accessibility support, not a completed accessibility audit.

The bridge still returns bounded batches, not streams. Cancellation is cooperative; keyboard events cannot interrupt synchronous Rust while it occupies the browser thread. Existing commands are short-lived. Tests exercise cancellation before the Rust promise starts; no long-running command or worker scheduler is claimed.

## Build and verification

```sh
pnpm check:web
pnpm exec playwright install --with-deps chromium
pnpm test:web:browser
pnpm check
```

`check:web` builds WASM, formats/lints/type-checks, runs Vitest and builds the application. Vite bundles the generated JS and emits the WASM asset using an explicit URL import. `test:web:browser` serves the production build on 127.0.0.1:4180 and exercises actual React/xterm/WASM integration in Chromium. Run `check:web` first. The existing bridge-only Node/Chromium tests remain separate. CI runs both suites.

Browser tests cover commands/errors/clear/exit/reset with network requests blocked after initialization, editing/history/paste, independent tabs, ignored URL commands, HTML safety, failed WASM loading/retry, resizing and same-turn cancellation/duplicate submissions. Unit tests cover output control neutralization and input bounds. PR-09 adds a shared fixture smoke test and exact native/browser WASM comparisons; see [V0.1 audit](V0_1_AUDIT.md). Firefox, WebKit, mobile keyboard/IME devices and screen-reader software are not yet comprehensively validated.
