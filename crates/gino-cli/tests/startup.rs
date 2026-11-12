use std::process::{Command, Stdio};

#[test]
fn startup_reports_scaffold_status_and_exits_without_input() {
    // Launch only our compiled executable, never user input or a host shell.
    let output = Command::new(env!("CARGO_BIN_EXE_gino-cli"))
        .stdin(Stdio::null())
        .output()
        .expect("CLI should launch");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 startup output");
    assert!(stdout.contains("PR-01 scaffold"));
    assert!(stdout.contains("Shell commands are not implemented yet."));
}
