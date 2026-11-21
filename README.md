# GINO.OS · personal_terminal

Rust-first virtual shell and interactive engineering portfolio, under development.

**Current state: PR-06 interactive native CLI with six shared Rust commands.** Run it in a Linux terminal to use `help`, `echo`, `pwd`, `whoami`, `clear` and `exit`. Input editing, in-memory history and Ctrl+C/D are implemented. The browser remains a static development page; WASM binding, VFS, portfolio data and API are still planned.

## Development setup

Use Rust via rustup (the pinned toolchain, rustfmt, clippy and WASM target are in `rust-toolchain.toml`), a native C linker/compiler, Node **24.22.1**, and pnpm **11.25.0**. Linux is the initial verified native platform.

```sh
npm install --global pnpm@11.25.0
rustup show
pnpm install --frozen-lockfile
cargo run -p gino-cli --locked
pnpm dev
```

Vite prints the local browser development URL; that page is still a scaffold. The native CLI runs directly in your terminal, without a browser or backend. It starts with:

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
```

`pnpm check:web` runs Prettier, ESLint, TypeScript, Vitest and the production build. `pnpm check:rust` runs the listed Rust commands using Bash; `pnpm check` runs both groups. For intentional formatting changes use `cargo fmt --all` and `pnpm format`.

The core now exposes session, command/context, asynchronous I/O, terminal adapter and cancellation contracts, with a native CLI adapter and test fixtures. See [contract semantics](docs/SHELL_CONTRACTS.md). The [command registry](docs/COMMAND_REGISTRY.md) now provides validated registration, aliases, exact lookup and metadata-derived help. `builtin_registry()` registers the six implemented [core commands](docs/COMMANDS.md). The [single-command parser](docs/SHELL_GRAMMAR.md) provides quoted/escaped words, a minimal AST and typed syntax errors; the interpreter now consumes its AST for registry dispatch.

The web test checks rendered markup in Node, not an actual browser. WASM compilation checks portability only; browser execution and parity tests start when the bridge exists. CI runs the same baseline checks on pull requests and pushes to main.

## Project documents

- [Product specification](docs/PRODUCT_SPEC.md): full target design; future features are not current capabilities.
- [36-PR implementation plan](docs/IMPLEMENTATION_PLAN.md): scope and acceptance gates.
- [Architecture](docs/ARCHITECTURE.md): present dependencies and future Rust-to-WASM boundary.
- [Contributing](CONTRIBUTING.md): scoped workflow and checks.

The specification and roadmap were imported from the owner-supplied attachments because the initial repository contained only a README. Their original contents are preserved; implementation status is recorded separately in the plan. No profile repository content is imported in PR-01.
