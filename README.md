# GINO.OS · personal_terminal

Rust-first virtual shell and interactive engineering portfolio, under development.

**Current state: native CLI and browser terminal, with PR-09 cross-runtime parity verification and a V0.1 technical audit.** Use `help`, `echo`, `pwd`, `whoami`, `clear` and `exit` in either runtime. The browser executes locally through WASM; VFS, portfolio data and API remain planned.

## Development setup

Use Rust via rustup (the pinned toolchain, rustfmt, clippy and WASM target are in `rust-toolchain.toml`), a native C linker/compiler, Node **24.22.1**, and pnpm **11.25.0**. Linux is the initial verified native platform.

```sh
npm install --global pnpm@11.25.0
rustup show
pnpm install --frozen-lockfile
cargo install wasm-pack --version 0.15.1 --locked
cargo run -p gino-cli --locked
pnpm dev
```

`pnpm dev` builds WASM, then Vite prints the browser terminal URL. See [browser usage, keyboard controls and verification](docs/BROWSER_TERMINAL.md). The native CLI runs directly in your terminal, without a browser or backend. It starts with:

```text
GINO.OS — native virtual shell
Type 'help' for available commands.
visitor@gino.os:~$
```

See [native input, history, shutdown and limitations](docs/NATIVE_CLI.md). Python 3 is required for Linux PTY tests, not for running the CLI. No environment secrets or external portfolio data are needed.

## Verification

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
cargo build -p gino-shell --target wasm32-unknown-unknown --locked
cargo check -p gino-shell --tests --target wasm32-unknown-unknown --locked
pnpm check:web
pnpm check:wasm
pnpm test:web:browser
pnpm check:parity
```

`pnpm check:web` builds WASM then runs Prettier, ESLint, TypeScript, Vitest and the production build. `pnpm check:rust` runs the listed Rust commands using Bash; `pnpm check` runs Rust, web, WASM, browser UI and native/WASM parity checks. For intentional formatting changes use `cargo fmt --all` and `pnpm format`.

The core now exposes session, command/context, asynchronous I/O, terminal adapter and cancellation contracts, with a native CLI adapter and test fixtures. See [contract semantics](docs/SHELL_CONTRACTS.md). The [command registry](docs/COMMAND_REGISTRY.md) now provides validated registration, aliases, exact lookup and metadata-derived help. `builtin_registry()` registers the six implemented [core commands](docs/COMMANDS.md). The [single-command parser](docs/SHELL_GRAMMAR.md) provides quoted/escaped words, a minimal AST and typed syntax errors; the interpreter now consumes its AST for registry dispatch.

Web unit tests check loading markup, input bounds and control sanitization; browser UI tests exercise the production React/xterm application with real WASM. The WASM suite executes the generated Rust module in Node and headless Chromium, including all six commands, errors, session isolation/reset and cancellation. CI runs Rust, web and WASM jobs on pull requests and pushes to main. Shared fixtures now compare exact native Rust, Node WASM and Chromium WASM results, and exercise a common scenario through the CLI and production browser. See the [V0.1 audit](docs/V0_1_AUDIT.md) for evidence, intentional adapter differences and remaining limits.

Before `pnpm check:wasm` or `pnpm check`, install `cargo install wasm-pack --version 0.15.1 --locked` and `pnpm exec playwright install --with-deps chromium`. Use `pnpm build:wasm` to generate the uncommitted web-target package. See [bridge API, tests and limitations](docs/WASM_BRIDGE.md).

## Project documents

- [Product specification](docs/PRODUCT_SPEC.md): full target design; future features are not current capabilities.
- [36-PR implementation plan](docs/IMPLEMENTATION_PLAN.md): scope and acceptance gates.
- [Architecture](docs/ARCHITECTURE.md): present dependencies and future Rust-to-WASM boundary.
- [Contributing](CONTRIBUTING.md): scoped workflow and checks.

The specification and roadmap were imported from the owner-supplied attachments because the initial repository contained only a README. Their original contents are preserved; implementation status is recorded separately in the plan. No profile repository content is imported in PR-01.
