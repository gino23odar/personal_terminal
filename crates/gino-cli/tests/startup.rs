#![cfg(unix)]

use std::process::{Command, Stdio};

#[test]
fn noninteractive_startup_fails_explicitly_without_waiting_for_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_gino-cli"))
        .stdin(Stdio::null())
        .output()
        .expect("CLI should launch");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("interactive"));
}

#[test]
fn arguments_are_not_treated_as_host_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_gino-cli"))
        .args(["-c", "echo unsafe"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn linux_pty_interaction_and_terminal_restoration() {
    let output = Command::new("python3")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/native_pty.py"))
        .arg(env!("CARGO_BIN_EXE_gino-cli"))
        .output()
        .expect("Python 3 is required for Linux PTY integration tests");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
