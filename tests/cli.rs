use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn prints_version() {
    let output = run_cli(&["--version"], &[], "").success();
    assert_eq!(output.stdout, "translate-cli 0.1.0\n");
    assert_eq!(output.stderr, "");
}

#[test]
fn translates_through_fake_codex() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    write_config(&config_path, "codex");

    let path = with_fake_path("fake-codex");
    let output = run_cli(
        &["ja", "hello"],
        &[
            ("TRANSLATE_CLI_CONFIG", config_path.to_str().unwrap()),
            ("PATH", &path),
        ],
        "",
    )
    .success();
    assert_eq!(output.stdout, "こんにちは\n");
    assert_eq!(output.stderr, "");
}

#[test]
fn translates_stdin_through_fake_claude() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    write_config(&config_path, "claude");

    let path = with_fake_path("fake-claude");
    let output = run_cli(
        &["--tool", "claude", "ja"],
        &[
            ("TRANSLATE_CLI_CONFIG", config_path.to_str().unwrap()),
            ("PATH", &path),
        ],
        "hello",
    )
    .success();
    assert_eq!(output.stdout, "こんにちは\n");
    assert_eq!(output.stderr, "");
}

#[test]
fn runs_first_run_setup() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    let path = with_fake_path("fake-codex");

    let output = run_cli(
        &["--setup"],
        &[
            ("TRANSLATE_CLI_CONFIG", config_path.to_str().unwrap()),
            ("PATH", &path),
        ],
        "\n\n",
    )
    .success();
    assert_eq!(output.stdout, "");
    assert!(output.stderr.contains("translate CLI setup"));
    assert!(output.stderr.contains("Saved config:"));

    let config = fs::read_to_string(config_path).unwrap();
    assert!(config.contains("default_tool = \"codex\""));
    assert!(config.contains("local_lang = "));
}

struct CliResult {
    status: i32,
    stdout: String,
    stderr: String,
}

impl CliResult {
    fn success(self) -> Self {
        assert_eq!(self.status, 0, "stderr:\n{}", self.stderr);
        self
    }
}

fn run_cli(args: &[&str], envs: &[(&str, &str)], input: &str) -> CliResult {
    let mut child = Command::new(env!("CARGO_BIN_EXE_t"))
        .args(args)
        .envs(envs.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if !input.is_empty() {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    CliResult {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

fn write_config(config_path: &Path, tool: &str) {
    fs::write(
        config_path,
        format!(
            "version = 1
default_tool = \"{tool}\"
local_lang = \"ja\"
timeout_ms = 60000

[tools.codex]
enabled = true

[tools.claude]
enabled = true
"
        ),
    )
    .unwrap();
}

fn with_fake_path(name: &str) -> String {
    let fake = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join(name);
    let mut paths = vec![fake];
    if let Some(existing) = env::var_os("PATH") {
        paths.extend(env::split_paths(&existing));
    }
    env::join_paths(paths)
        .unwrap()
        .to_string_lossy()
        .to_string()
}

fn temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!("translate-cli-test-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
