# GINO.OS — 36-PR Implementation Roadmap

**This is the timeline. Use PR sequence and dependency gates, not dates, story points, person-hours, or estimated durations.** `PR-XX` is the *planned sequence*, not a claim that a GitHub PR already exists. Feature branches should use `feat/pr-XX-short-name` or equivalent. Each PR must be independently reviewable, pass relevant checks, add tests for behavior, and update documentation where needed.

### Phase 1 — Rust shell foundation (PR-01 → PR-09) · V0.1 gate

| PR | Scope | Definition of done |
|---|---|---|
| **PR-01** | Initialize Rust workspace, minimal CLI crate and web scaffold, toolchains, formatting/lint/test scripts, PR template, CI baseline | Repo installs/builds in documented environment; baseline checks execute; architecture boundaries and PR plan recorded |
| **PR-02** | Portable shell contracts: session, command/context, I/O/events, cancellation, adapter traits | Contracts compile natively and for WASM target; unit/compile checks demonstrate boundaries |
| **PR-03** | Rust command registry, metadata, aliases, lookup and help integration | Register/lookup/alias tests pass; no monolithic dispatcher |
| **PR-04** | Rust tokenizer, minimal AST, quotes/escapes, malformed syntax errors | Parser tests cover valid/invalid input; unsupported operators rejected |
| **PR-05** | Rust interpreter, session lifecycle and `help`, `echo`, `pwd`, `whoami`, `clear`, `exit` | Built-ins execute through registry with stable output/exit codes |
| **PR-06** | Native Rust Linux CLI adapter with history, input, Ctrl+C/D behavior, clean shutdown | Interactive Linux command sequence works without browser/server; terminal state restored |
| **PR-07** | Rust WASM adapter/bridge exposing same shell engine | WASM build passes; JS-facing tests prove commands come from Rust core |
| **PR-08** | React/Vite/xterm.js browser terminal, keyboard handling and Rust WASM binding | Browser executes shared commands locally; no duplicate TS interpreter |
| **PR-09** | Cross-runtime parity fixtures, CLI/browser smoke tests, docs and V0.1 audit | Native + WASM outputs equivalent for fixtures; checks green; V0.1 criteria evidenced |

### Phase 2 — Virtual filesystem (PR-10 → PR-14)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-10** | VFS Rust types, in-memory backend, filesystem metadata and service contracts | Portable operations compile for native/WASM; unit tests pass |
| **PR-11** | Path resolution, read-only mounts, permissions/capabilities, quotas | Traversal, mount and isolation regression tests pass |
| **PR-12** | `ls`, `cd`, `cat`, `mkdir`, `touch`, `cp`, `mv`, `rm`, `tree` plus appropriate utilities | Consistent command behavior and explicit unsupported-option errors |
| **PR-13** | Per-session persistence adapters: browser IndexedDB and native app-data storage | Data survives restart when enabled, reset works, no host path leak |
| **PR-14** | VFS parity, state-invariant tests and documentation | Same file-operation fixtures pass natively and in browser |

### Phase 3 — Shell composition (PR-15 → PR-19)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-15** | Variable/home expansion, aliases, virtual `env`/`export`/`printenv` | Expansion order documented; quoting and state tests pass |
| **PR-16** | Portable asynchronous pipes/stream forwarding, backpressure, cancellation | Multi-stage pipeline tests pass in CLI and WASM browser |
| **PR-17** | Input/output redirection, append, `;`, `&&`, `||` | AST/operator precedence and exit-code semantics tested |
| **PR-18** | Text utilities `grep`, `head`, `tail`, `wc`, `sort`, `uniq` and documented flags | Utilities compose through pipes and redirects; errors meaningful |
| **PR-19** | Advanced shell conformance fixtures and hardening | Full documented examples work across runtimes; regression suite passes |

### Phase 4 — Public portfolio (PR-20 → PR-24) · V0.5 gate

| PR | Scope | Definition of done |
|---|---|---|
| **PR-20** | Canonical approved portfolio schema, import/validation tools | Facts provenance and statuses preserved; no fabricated data |
| **PR-21** | Read-only virtual portfolio tree generated from canonical data | Data appears correctly under virtual files; writes blocked |
| **PR-22** | `profile`, `experience`, `projects`, `stack`, `ai`, `architecture`, `contact` commands | Output consistent with virtual files and public links |
| **PR-23** | Recruiter shortcuts, command palette, safe deep links, mobile/accessibility improvements | Nontechnical visitor can explore; URL commands allowlisted |
| **PR-24** | Portfolio E2E, content accuracy audit, deployable preview, V0.5 documentation | Hosted preview works without AI; V0.5 criteria evidenced |

### Phase 5 — Animations and extension SDK (PR-25 → PR-28)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-25** | Portable animation runtime, frame lifecycle, reduced motion, cancellation | Animation works in both adapters and restores terminal on interruption |
| **PR-26** | Curated independent ASCII/fun commands with original assets | New animations use shared runtime; no scrollback flooding |
| **PR-27** | One interactive game (e.g., Snake) with temporary keyboard ownership | Controls, resizing, pause/exit and cleanup tested |
| **PR-28** | Extensibility documentation/SDK example and plugin boundary tests | New custom command/game added without interpreter modification |

### Phase 6 — Optional AI service (PR-29 → PR-33)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-29** | FastAPI skeleton, typed schemas, health/error endpoints, config | API checks and tests pass independently of CLI/web |
| **PR-30** | Retrieval from approved portfolio records with evidence links | Grounded retrieval tested; unknown facts handled gracefully |
| **PR-31** | Provider abstraction and grounded answer-generation/streaming | Swappable providers; no model keys in client; mock tests pass |
| **PR-32** | Rate/quota/spend budgets, timeouts, prompt-injection defenses, safe logging | Abuse/security tests pass; bounded failure modes documented |
| **PR-33** | Rust `ask` command through optional HTTP client capability, CLI/web UX and offline errors | Same command works across runtimes; cancellation and fallback tested |

Phases 5 and 6 can proceed on separate branches after Phase 4 if contracts are stable. Merge conflict-sensitive integration work should be sequenced; PR numbers are review-order identifiers, **not elapsed-time estimates**.

### Phase 7 — Production and V1.0 (PR-34 → PR-36)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-34** | Production Docker/API deployment and static frontend hosting, TLS/config/health docs | Reproducible staging deployment; frontend and API independently configurable |
| **PR-35** | CI/CD integration, E2E/accessibility/security/load/failure validation | Release gates green; known issues documented and blocking ones fixed |
| **PR-36** | Installation, architecture, command catalog, contributing/extension docs, release notes, GitHub profile links | Public demo and native install verified; V1.0 checklist complete |

### PR workflow and merge gates

For **every PR**:

1. Read `docs/PRODUCT_SPEC.md` and `docs/IMPLEMENTATION_PLAN.md`; inspect existing code and previous PR outcomes.
2. State **scope, non-goals, dependencies, architecture impact, acceptance criteria**.
3. Implement the smallest complete vertical slice; avoid speculative code.
4. Add or update automated tests and run applicable native/WASM/web/API checks.
5. Review security, portability, cancellation, docs, and regressions.
6. Submit a clear PR description: summary, screenshots/CLI output where useful, verification commands/results, limitations, follow-ups.
7. Update roadmap status only after acceptance; do not mark a PR complete before checks pass.

Use a PR template with checkboxes for scoped change, user-visible behavior, test evidence, Rust native/WASM parity where applicable, security review, documentation, and unresolved concerns. No PR merges with known failing mandatory checks. If an external service blocks validation, describe the limitation rather than fabricate green results.

## Repository implementation status

- PR-01: accepted and merged as GitHub PR #1; owner reports local verification.
- PR-02: accepted and merged as GitHub PR #2; owner reports verification.
- PR-03: accepted and merged as GitHub PR #3; owner reports verification.
- PR-04: accepted and merged as GitHub PR #4; owner reports verification and hosted CI passed.
- PR-05: accepted and merged as GitHub PR #5; owner reports verification and hosted CI passed.
- PR-06: accepted and merged as GitHub PR #6; owner reports verification and hosted CI passed.
- PR-07: accepted and merged as GitHub PR #7; hosted Rust, web and WASM CI passed.
- PR-08: accepted and merged as GitHub PR #8; owner reports verification and hosted CI passed.
- PR-09: accepted and merged as GitHub PR #9 with hosted Rust, web and WASM CI passing.
- PR-10: portable VFS types, in-memory backend, metadata and service contract implemented on feature branch; acceptance pending verification and owner review.
- PR-11 through PR-36: planned, not implemented.
- V0.1 technical evidence is documented in V0_1_AUDIT.md; the architectural gate was accepted with PR-09; no release tag is implied. V0.5 and V1.0 remain future work.

PR-02 prerequisites: accept PR-01 with passing baseline checks; install the pinned toolchains and WASM target; read both specification documents and architecture notes. Its scope is portable session, command/context, I/O/events, cancellation and adapter traits, verified natively and for WASM. Registry, parser, commands and runtime integrations remain later work.

PR-03 prerequisites: accept PR-02 with native and WASM compile checks passing; review the contract semantics. Implement the Rust command registry, metadata, aliases, lookup and help integration with register/lookup/alias tests. Do not add the parser or interpreter ahead of their roadmap PRs.

PR-04 prerequisites: accept PR-03 after verification/review; preserve the registry and command contracts. Implement Rust tokenization and a minimal AST with quotes, escapes and informative malformed-input errors. Recognize and reject unsupported operators. Do not add interpreter execution or built-ins before PR-05.

PR-05 prerequisites: accept PR-04 after native tests, WASM compile checks and review; review SHELL_GRAMMAR.md, COMMAND_REGISTRY.md and SHELL_CONTRACTS.md. Connect parsing to registry execution and session lifecycle, then implement `help`, `echo`, `pwd`, `whoami`, `clear` and `exit` with stable output/exit codes. Handle blank input and syntax failures without dispatch. Interactive terminal input remains PR-06.

PR-06 prerequisites: accept PR-05 with passing checks; review COMMANDS.md lifecycle and diagnostic rules. Implement a native Linux input/history and I/O adapter with Ctrl+C/D handling and clean terminal shutdown, using the shared interpreter without host-shell passthrough.

PR-07 prerequisites: accept PR-06 with native/PTY and core WASM checks passing. Add the WASM adapter/bridge over the shared interpreter, with JS-facing execution tests. Keep native dependencies out of WASM and browser UI work in PR-08.

PR-08 prerequisites: accept PR-07 after native, web, generated TypeScript and Node/Chromium WASM checks pass. Review WASM_BRIDGE.md, build the generated web package, then connect React/Vite/xterm.js with keyboard handling and safe output rendering. Keep shell semantics in Rust and preserve busy/cancellation/session behavior. V0.1 acceptance remains PR-09.

PR-09 prerequisites: accept PR-08 with native, bridge and production browser UI checks passing. Review BROWSER_TERMINAL.md and WASM_BRIDGE.md; introduce cross-runtime parity fixtures, CLI/browser smoke coverage and the V0.1 audit. Do not begin VFS or claim V0.1 until its criteria are evidenced.

PR-10 prerequisites: accept PR-09 with passing native, WASM, parity and production browser checks; review V0_1_AUDIT.md and portable contracts. Introduce VFS types, in-memory backend, metadata and service contracts with native/WASM tests. Path resolution, filesystem commands and persistence remain PR-11 through PR-14.

PR-11 prerequisites: accept PR-10 with native tests and VFS WASM production/test compilation passing; review VFS_CONTRACTS.md. Add virtual path resolution, read-only mounts, permissions/capabilities and quotas with traversal, mount and isolation regressions. Filesystem shell commands and persistence remain PR-12 and PR-13.
