use gino_shell::*;
use std::{
    cell::Cell,
    future::Future,
    rc::Rc,
    task::{Context, Poll, Waker},
};

struct Probe(Rc<Cell<usize>>);
impl Command for Probe {
    fn execute<'a>(
        &'a self,
        _: &'a [String],
        _: CommandContext<'a>,
        _: &'a mut dyn CommandIo,
    ) -> LocalFuture<'a, CommandResult> {
        Box::pin(async move {
            self.0.set(self.0.get() + 1);
            Ok(ExitStatus(7))
        })
    }
}
fn metadata(name: &str, aliases: &[&str]) -> CommandMetadata {
    CommandMetadata {
        name: name.into(),
        summary: format!("Describe {name}"),
        usage: format!("{name} [argument]"),
        aliases: aliases.iter().map(|a| (*a).into()).collect(),
    }
}
fn handler() -> Box<dyn Command> {
    Box::new(Probe(Rc::new(Cell::new(0))))
}
fn ready<F: Future>(future: F) -> F::Output {
    match std::pin::pin!(future)
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("unexpected pending test handler"),
    }
}
struct UnusedIo;
impl CommandIo for UnusedIo {
    fn read<'a>(&'a mut self, _: &'a mut [u8]) -> LocalFuture<'a, Result<usize, ShellError>> {
        panic!("unexpected I/O")
    }
    fn write<'a>(
        &'a mut self,
        _: OutputStream,
        _: &'a [u8],
    ) -> LocalFuture<'a, Result<(), ShellError>> {
        panic!("unexpected I/O")
    }
    fn emit(&mut self, _: TerminalEvent) -> LocalFuture<'_, Result<(), ShellError>> {
        panic!("unexpected I/O")
    }
}

#[test]
fn empty_registry_never_advertises_planned_commands() {
    let registry = CommandRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert_eq!(registry.entries().count(), 0);
    assert_eq!(
        registry.help_text(None).unwrap(),
        "No commands registered.\n"
    );
    assert!(registry.get("help").is_none());
}

#[test]
fn canonical_and_alias_lookup_share_one_local_handler() {
    let count = Rc::new(Cell::new(0));
    let mut registry = CommandRegistry::new();
    registry
        .register(metadata("probe", &["p"]), Box::new(Probe(count.clone())))
        .unwrap();
    assert_eq!(registry.len(), 1);
    assert!(std::ptr::eq(
        registry.get("probe").unwrap(),
        registry.get("p").unwrap()
    ));
    let mut session = ShellSession::default();
    let token = CancellationToken::new();
    for name in ["probe", "p"] {
        let result = ready(registry.get(name).unwrap().handler().execute(
            &[],
            CommandContext {
                session: &mut session,
                cancellation: &token,
            },
            &mut UnusedIo,
        ));
        assert_eq!(result, Ok(ExitStatus(7)));
    }
    assert_eq!(count.get(), 2);
}

#[test]
fn duplicate_canonical_name_preserves_original_handler_and_metadata() {
    let mut registry = CommandRegistry::new();
    registry
        .register(metadata("probe", &["p"]), handler())
        .unwrap();
    let before = registry.help("probe").unwrap().clone();
    assert_eq!(
        registry.register(metadata("probe", &["other"]), handler()),
        Err(RegistryError::NameConflict("probe".into()))
    );
    assert_eq!(registry.help("probe"), Some(&before));
    assert!(registry.get("other").is_none());
    assert_eq!(registry.len(), 1);
}

#[test]
fn all_cross_registration_collisions_are_rejected_atomically() {
    for candidate in [
        metadata("p", &[]),
        metadata("second", &["probe"]),
        metadata("second", &["fresh", "p"]),
    ] {
        let mut registry = CommandRegistry::new();
        registry
            .register(metadata("probe", &["p"]), handler())
            .unwrap();
        assert!(matches!(
            registry.register(candidate, handler()),
            Err(RegistryError::NameConflict(_))
        ));
        assert_eq!(registry.len(), 1);
        assert!(registry.get("second").is_none());
        assert!(registry.get("fresh").is_none());
        assert_eq!(registry.help("p").unwrap().name, "probe");
    }
}

#[test]
fn duplicate_aliases_and_self_alias_are_rejected() {
    for aliases in [vec!["probe"], vec!["p", "p"]] {
        let mut registry = CommandRegistry::new();
        assert!(matches!(
            registry.register(metadata("probe", &aliases), handler()),
            Err(RegistryError::NameConflict(_))
        ));
        assert!(registry.is_empty());
        assert!(registry.get("p").is_none());
    }
}

#[test]
fn invalid_names_and_aliases_leave_no_partial_registration() {
    for name in [
        "",
        "Probe",
        "1probe",
        "two words",
        "a;b",
        "a|b",
        "/bin/sh",
        "../x",
        "a\0b",
        "a\nb",
        "écho",
        "-flag",
        "a.b",
    ] {
        let mut registry = CommandRegistry::new();
        let mut invalid_name = metadata("probe", &[]);
        invalid_name.name = name.into();
        assert_eq!(
            registry.register(invalid_name, handler()),
            Err(RegistryError::InvalidName(name.into()))
        );
        assert_eq!(
            registry.register(metadata("probe", &["valid", name]), handler()),
            Err(RegistryError::InvalidName(name.into()))
        );
        assert!(registry.is_empty());
        assert!(registry.get("valid").is_none());
    }
    let mut registry = CommandRegistry::new();
    registry
        .register(metadata("probe-2_test", &[]), handler())
        .unwrap();
}

#[test]
fn invalid_documentation_is_rejected_before_insertion() {
    for value in [
        "",
        "  ",
        "two\nlines",
        "escape\u{1b}[31m",
        "tab\there",
        "delete\u{7f}",
        "separator\u{2028}",
    ] {
        for field in ["summary", "usage"] {
            let mut registry = CommandRegistry::new();
            let mut meta = metadata("probe", &[]);
            if field == "summary" {
                meta.summary = value.into();
            } else {
                meta.usage = value.into();
            }
            assert_eq!(
                registry.register(meta, handler()),
                Err(RegistryError::InvalidMetadata(field))
            );
            assert!(registry.is_empty());
        }
    }
}

#[test]
fn help_is_sorted_deduplicated_and_uses_lookup_metadata_without_executing() {
    let count = Rc::new(Cell::new(0));
    let mut registry = CommandRegistry::new();
    for (name, aliases) in [("zeta", vec![]), ("alpha", vec!["short", "a"])] {
        registry
            .register(metadata(name, &aliases), Box::new(Probe(count.clone())))
            .unwrap();
    }
    assert_eq!(
        registry
            .entries()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "zeta"]
    );
    assert_eq!(
        registry.help_text(None).unwrap(),
        "Available commands:\n  alpha — Describe alpha\n  zeta — Describe zeta\n"
    );
    assert_eq!(
        registry.help_text(Some("short")).unwrap(),
        "alpha — Describe alpha\nUsage: alpha [argument]\nAliases: a, short\n"
    );
    assert_eq!(registry.help("short"), registry.help("alpha"));
    assert_eq!(
        registry.help_text(Some("zeta")).unwrap(),
        "zeta — Describe zeta\nUsage: zeta [argument]\n"
    );
    assert_eq!(count.get(), 0);
}

#[test]
fn lookup_is_exact_and_unknown_help_is_explicit() {
    let mut registry = CommandRegistry::new();
    registry
        .register(metadata("probe", &["p"]), handler())
        .unwrap();
    for name in ["PROBE", " probe", "probe ", "probe arg", "missing", ""] {
        assert!(registry.get(name).is_none());
        assert!(registry.help(name).is_none());
        assert_eq!(
            registry.help_text(Some(name)),
            Err(RegistryError::UnknownCommand(name.into()))
        );
    }
}

#[test]
fn caller_metadata_changes_and_session_aliases_do_not_change_registry() {
    let mut registry = CommandRegistry::new();
    let mut meta = metadata("probe", &["p"]);
    registry.register(meta.clone(), handler()).unwrap();
    meta.name = "changed".into();
    meta.aliases.clear();
    let mut session = ShellSession::default();
    session.aliases.insert("p".into(), "another --flag".into());
    assert_eq!(registry.help("p").unwrap().name, "probe");
    assert!(registry.get("changed").is_none());
    assert!(registry.get("another").is_none());
}
