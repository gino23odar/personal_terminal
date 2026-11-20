# Contributing

Read the complete product specification and implementation plan before changes. Work on one roadmap PR at a time and keep implemented behavior distinct from planned features.

Follow README setup, create a focused feature branch, and run `pnpm check`. Commit both lockfiles when dependencies change. Include actual test results and blocked checks in the PR template. Do not merge with failing mandatory checks or mark roadmap work accepted before review.

Keep shell semantics in portable Rust. Do not add host-shell passthrough, host filesystem access or a second TypeScript interpreter. Add behavior tests as features arrive; keep platform I/O in adapters.

PR-01 is merged. PR-02 establishes portable session, command/context, I/O/event, cancellation and adapter contracts with native/WASM checks. Review docs/SHELL_CONTRACTS.md before adding handlers or adapters. The registry, parser and commands remain later PRs.

A public open-source license has not been selected by the owner; this scaffold does not grant one. Resolve licensing before inviting broad public collaboration.
