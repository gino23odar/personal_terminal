# Contributing

Read the complete product specification and implementation plan before changes. Work on one roadmap PR at a time and keep implemented behavior distinct from planned features.

Follow README setup, create a focused feature branch, and run `pnpm check`. Commit both lockfiles when dependencies change. Include actual test results and blocked checks in the PR template. Do not merge with failing mandatory checks or mark roadmap work accepted before review.

Keep shell semantics in portable Rust. Do not add host-shell passthrough, host filesystem access or a second TypeScript interpreter. Add behavior tests as features arrive; keep platform I/O in adapters.

PR-01 through PR-08 are merged. PR-02 establishes portable session, command/context, I/O/event, cancellation and adapter contracts with native/WASM checks. Review docs/SHELL_CONTRACTS.md before adding handlers or adapters. PR-03 adds the registry; review docs/COMMAND_REGISTRY.md for registration and alias rules. PR-04 adds the parser; review docs/SHELL_GRAMMAR.md before interpreter integration. PR-05 adds the interpreter and six built-ins; review docs/COMMANDS.md for lifecycle, diagnostics and cancellation rules. PR-06 connects the native CLI; see docs/NATIVE_CLI.md. Linux tests require Python 3 for the standard-library PTY harness.

A public open-source license has not been selected by the owner; this scaffold does not grant one. Resolve licensing before inviting broad public collaboration.

PR-07 adds the WASM bridge. Read docs/WASM_BRIDGE.md for the API and build prerequisites. Generated packages stay uncommitted. Run the actual Node and Chromium WASM tests for bridge changes; browser presentation is connected in PR-08.

For PR-08 browser work, read docs/BROWSER_TERMINAL.md. Build WASM before web type checks or direct Vite commands, then run the production browser suite. Root `pnpm dev`, `pnpm build`, and `pnpm check:web` build WASM automatically. Keep command semantics in Rust.

For PR-09 parity changes, review docs/V0_1_AUDIT.md and tests/parity/fixtures.json. Expected outcomes must be reviewed explicitly; do not regenerate expectations from the implementation. Run `pnpm check:parity` and the native/production-browser adapter suites. Generated native transcripts are ignored and must be rebuilt for current sources.
