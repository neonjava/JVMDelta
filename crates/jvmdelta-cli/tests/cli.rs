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
    assert!(stdout.contains("Inspect JVM GC logs"));
    assert!(stdout.contains("comparison and regression analysis are planned"));
    assert!(stdout.contains("gc"));
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
    assert!(String::from_utf8_lossy(&output.stderr).contains("unrecognized subcommand"));
}

fn inspect(path: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_jvmdelta"))
        .args(["gc", "inspect", path])
        .output()
        .unwrap_or_else(|error| panic!("failed to run CLI: {error}"))
}

#[test]
fn inspect_valid_and_empty_files() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/");
    let output = inspect(&format!("{root}jdk25-g1-real.log"));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Supported events: 2"));
    assert!(stdout.contains("Young: 1"));
    assert!(stdout.contains("Full: 1"));
    assert!(stdout.contains("Unrelated entries skipped: 4"));
    assert!(stdout.contains("All: total=22.970000 min=10.235000 mean=11.485000 max=12.735000"));
    assert!(stdout.contains("Remark: total=0.000000 min=n/a mean=n/a max=n/a"));

    let empty = tempfile_path("empty");
    std::fs::write(&empty, "").expect("write empty file");
    let output = inspect(empty.to_str().expect("UTF-8 path"));
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Supported events: 0"));
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("All: total=0.000000 min=n/a mean=n/a max=n/a")
    );
    std::fs::remove_file(empty).expect("remove temp file");
}

#[test]
fn inspect_rejects_missing_and_malformed_files() {
    let missing = tempfile_path("missing");
    let output = inspect(missing.to_str().expect("UTF-8 path"));
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot open"));
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/malformed-g1-synthetic.log"
    );
    let output = inspect(path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("line 2"));
}

#[test]
fn inspect_real_jdk17_and_21_excerpts() {
    for version in [17, 21] {
        let path = format!(
            "{}/../../tests/fixtures/jdk{version}-g1-real.log",
            env!("CARGO_MANIFEST_DIR")
        );
        let output = inspect(&path);
        assert!(
            output.status.success(),
            "JDK {version}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Supported events: 2"));
        assert!(stdout.contains("Unrelated entries skipped: 3"));
    }
}

fn tempfile_path(suffix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("jvmdelta-cli-{}-{suffix}.log", std::process::id()))
}
