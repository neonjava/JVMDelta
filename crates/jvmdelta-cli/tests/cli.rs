use std::process::Command;

fn run(argument: &str) -> std::process::Output {
    let executable = env!("CARGO_BIN_EXE_jvmdelta");
    Command::new(executable)
        .arg(argument)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {executable}: {error}"))
}

#[test]
fn help_describes_current_scope() {
    let output = run("--help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("JVM performance regression analysis"));
    assert!(stdout.contains("analysis commands are planned"));
    assert!(stdout.contains("--version"));
}

#[test]
fn version_reports_package_version() {
    let output = run("--version");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        concat!("jvmdelta ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unsupported_command_fails() {
    let output = run("compare");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
}
