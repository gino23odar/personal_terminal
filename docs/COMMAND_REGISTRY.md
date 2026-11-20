# Command registry (PR-03)

## Scope

The registry owns metadata and handlers using the PR-02 `Command` contract. It registers, resolves and describes commands. It does not tokenize input, dispatch a parsed program, create sessions or change cancellation behavior. No built-ins are registered in the CLI or browser; the executable `help` command is planned for PR-05.

## Registration and names

Create a `CommandRegistry` and pass `CommandMetadata` with a `Box<dyn Command>` to `register`. Metadata includes a canonical name, one-line summary, usage and a list of aliases. Registered metadata is available only by immutable reference, preventing name/index drift. The registry owns handlers for its lifetime, without requiring `Send`/`Sync`; browser-compatible `Rc` state remains supported.

Names are case-sensitive ASCII identifiers matching `[a-z][a-z0-9_-]*`. Spaces, paths, punctuation/operators, non-ASCII names and leading digits are rejected, not normalized. This is a registry identifier policy, not shell grammar. Summaries and usage must be nonempty single-line text without control characters or Unicode line/paragraph separators. Rendering adapters remain responsible for safe text presentation.

Canonical names and aliases share one namespace. Duplicate commands, aliases that collide with an existing command or alias, self-aliases and repeated aliases within one registration return `RegistryError::NameConflict`. The full metadata and all names are validated before mutation. Failure preserves existing entries and does not reserve any new names. The supplied handler is consumed/dropped even on error; metadata can be cloned before registration if needed for retry.

No replace/remove API is needed in this PR. A future plugin lifecycle must define those semantics before adding one. The registry does not inspect the host PATH or load external executables.

## Alias semantics

Registry aliases are direct alternate names for the same canonical entry and handler. They do not form chains, accept arguments, or expand strings, so alias cycles are impossible in this representation. Aliases are sorted on registration for stable help output.

`ShellSession::aliases` is separate storage reserved for future user-defined expansion in PR-15. The registry does not consult it. Precedence and expansion rules must be specified in that PR, rather than silently treating session strings as registered handlers now.

## Lookup and help integration

- `get(name)` returns the canonical `RegisteredCommand` for an exact command name or alias, or `None`. The caller can access its metadata and handler. It never invokes the handler.
- `entries()` yields one metadata entry per canonical command, sorted lexicographically; aliases do not create duplicate rows.
- `help(name)` uses the same lookup and metadata, so aliases receive canonical documentation.
- `help_text(None)` formats the discovered names and summaries, or explicitly reports that no commands are registered.
- `help_text(Some(name))` formats the canonical name, summary, usage and aliases. Unknown names produce `RegistryError::UnknownCommand`, distinct from successful empty discovery.

Help has no separate command catalog and cannot advertise future features unless a caller explicitly registers a handler for them. It is library-level formatting, available for the future Rust `help` built-in and WASM consumers. No TypeScript copy exists. `RegistryError` describes registration/discovery failures, not shell execution exit status; interpreter mapping is deferred to PR-05.

## Verification and next step

`cargo test -p gino-shell --test registry --locked` tests registrations, collisions, atomic rejection, invalid metadata, deterministic discovery/help, exact lookup, alias identity, session-alias separation and invocation of a test-only non-Send handler retrieved from the registry.

The existing full Rust script also builds the core and compile-checks all core tests for `wasm32-unknown-unknown`. These are compilation checks, not browser execution or runtime parity. Existing CLI/web scaffolds are unchanged. No new dependencies or crates are introduced.

PR-04 adds the tokenizer and minimal AST, with malformed-input errors and unsupported-operator rejection. Interpreter dispatch and actual `help` execution arrive in PR-05.
