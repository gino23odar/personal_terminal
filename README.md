# GINO.OS · personal_terminal

Rust-first virtual shell and interactive engineering portfolio, under development.

**Current state: PR-05 interpreter and six Rust built-ins; CLI/web remain scaffolds.** The native executable prints a status message and exits. The browser shows a static development page. The Rust library executes commands through its async API. There is no interactive prompt, WASM bridge, virtual filesystem, portfolio data or API yet.

## Development setup

Use Rust via rustup (the pinned toolchain, rustfmt, clippy and WASM target are in `rust-toolchain.toml`), a native C linker/compiler, Node **24.22.1**, and pnpm **11.25.0**. Linux is the initial verified native platform.

```sh
npm install --global pnpm@11.25.0
rustup show
pnpm install --frozen-lockfile
cargo run -p gino-cli --locked
pnpm dev
```

Vite prints the local development URL. No backend, environment variables, secrets or external portfolio data are needed. The CLI reports:

```text
GINO.OS — PR-05 core ready. Interactive CLI is not connected yet.
```

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

The core now exposes session, command/context, asynchronous I/O, terminal adapter and cancellation contracts, with test-only adapters. See [contract semantics](docs/SHELL_CONTRACTS.md). The [command registry](docs/COMMAND_REGISTRY.md) now provides validated registration, aliases, exact lookup and metadata-derived help. `builtin_registry()` registers the six implemented [core commands](docs/COMMANDS.md). The [single-command parser](docs/SHELL_GRAMMAR.md) provides quoted/escaped words, a minimal AST and typed syntax errors; the interpreter now consumes its AST for registry dispatch.

The web test checks rendered markup in Node, not an actual browser. WASM compilation checks portability only; browser execution and parity tests start when the bridge exists. CI runs the same baseline checks on pull requests and pushes to main.

## Project documents

- [Product specification](docs/PRODUCT_SPEC.md): full target design; future features are not current capabilities.
- [36-PR implementation plan](docs/IMPLEMENTATION_PLAN.md): scope and acceptance gates.
- [Architecture](docs/ARCHITECTURE.md): present dependencies and future Rust-to-WASM boundary.
- [Contributing](CONTRIBUTING.md): scoped workflow and checks.

The specification and roadmap were imported from the owner-supplied attachments because the initial repository contained only a README. Their original contents are preserved; implementation status is recorded separately in the plan. No profile repository content is imported in PR-01.
