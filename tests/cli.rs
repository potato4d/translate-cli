use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn prints_version() {
    let output = run_cli(&["--version"], &[], "").success();
    assert_eq!(output.stdout, "translate-cli 0.1.4\n");
    assert_eq!(output.stderr, "");
}

#[test]
fn translates_through_fake_codex() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    write_config(&config_path, "codex");

    let path = with_isolated_fake_paths(&["fake-codex"]);
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

    let path = with_isolated_fake_paths(&["fake-claude"]);
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
fn translates_through_fake_ollama() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    write_config(&config_path, "ollama");

    let path = with_isolated_fake_paths(&["fake-ollama"]);
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
fn translates_through_fake_lmstudio() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    write_config(&config_path, "lmstudio");

    let path = with_isolated_fake_paths(&["fake-lmstudio"]);
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
fn translates_through_fake_lms_alias() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    write_config(&config_path, "lmstudio");

    let path = with_isolated_fake_paths(&["fake-lmstudio"]);
    let output = run_cli(
        &["--tool", "lms", "ja", "hello"],
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
fn runs_first_run_setup() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    let path = with_isolated_fake_paths(&["fake-codex"]);

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
    assert!(output.stderr.contains("Translate CLI setup"));
    assert!(output.stderr.contains("Saved config:"));

    let config = fs::read_to_string(config_path).unwrap();
    assert!(config.contains("default_tool = \"codex\""));
    assert!(config.contains("local_lang = "));
}

#[test]
fn setup_recommends_ollama_among_local_llms() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    let path = with_isolated_fake_paths(&["fake-lmstudio", "fake-ollama"]);

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
    assert!(output.stderr.contains("Recommended tool: ollama"));
    assert!(output.stderr.contains("lmstudio"));

    let config = fs::read_to_string(config_path).unwrap();
    assert!(config.contains("default_tool = \"ollama\""));
    assert!(config.contains("model = \"gemma3:latest\""));
}

#[test]
fn setup_recommends_ollama_over_agent_clis_when_no_override_exists() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    let path = with_isolated_fake_paths(&["fake-codex", "fake-ollama"]);

    let output = run_cli(
        &["--setup"],
        &[
            ("TRANSLATE_CLI_CONFIG", config_path.to_str().unwrap()),
            ("PATH", &path),
        ],
        "\n\n",
    )
    .success();
    assert!(output.stderr.contains("Recommended tool: ollama"));

    let config = fs::read_to_string(config_path).unwrap();
    assert!(config.contains("default_tool = \"ollama\""));
}

#[test]
fn setup_guides_ollama_when_no_local_model_is_available() {
    let tmp = temp_dir();
    let config_path = tmp.join("config.toml");
    let path = with_isolated_fake_paths(&["fake-ollama-empty"]);

    let output = run_cli(
        &["--setup"],
        &[
            ("TRANSLATE_CLI_CONFIG", config_path.to_str().unwrap()),
            ("PATH", &path),
        ],
        "",
    );
    assert_eq!(output.status, 4, "stderr:\n{}", output.stderr);
    assert!(output.stderr.contains("no ready translation tool found"));
    assert!(output.stderr.contains("ollama pull gemma3"));
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
        .env_remove("TRANSLATE_CLI_TOOL")
        .env_remove("TRANSLATE_CLI_OLLAMA_MODEL")
        .env_remove("TRANSLATE_CLI_LMSTUDIO_MODEL")
        .env_remove("TRANSLATE_CLI_LMS_MODEL")
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

[tools.ollama]
enabled = true
model = \"gemma3:latest\"

[tools.lmstudio]
enabled = true
model = \"lmstudio-community/gemma-3-4b-it\"
"
        ),
    )
    .unwrap();
}

fn with_isolated_fake_paths(names: &[&str]) -> String {
    let mut paths = names
        .iter()
        .map(|name| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("testdata")
                .join(name)
        })
        .collect::<Vec<_>>();
    paths.extend(
        ["/usr/bin", "/bin", "/usr/sbin", "/sbin"]
            .iter()
            .map(PathBuf::from),
    );
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
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = env::temp_dir().join(format!(
        "translate-cli-test-{}-{nanos}-{counter}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
