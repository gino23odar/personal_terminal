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

- PR-01: implemented on the foundation branch; local baseline checks pass. GitHub CI and review remain acceptance gates. Not yet accepted.
- PR-02 through PR-36: planned, not implemented.
- No V0.1, V0.5 or V1.0 release gate has been met.

PR-02 prerequisites: accept PR-01 with passing baseline checks; install the pinned toolchains and WASM target; read both specification documents and architecture notes. Its scope is portable session, command/context, I/O/events, cancellation and adapter traits, verified natively and for WASM. Registry, parser, commands and runtime integrations remain later work.
