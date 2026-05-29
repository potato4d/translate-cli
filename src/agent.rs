use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::cli::TranslationMode;
use crate::config::default_timeout_ms;
use crate::error::{agent_run_error, AppError, AppResult, EXIT_TIMEOUT};
use crate::lang::Language;
use crate::output::normalize_allow_raw;
use crate::prompt::{build_plain_text_prompt, build_prompt};

const JSON_SCHEMA: &str = r#"{"type":"object","properties":{"translated_text":{"type":"string"}},"required":["translated_text"],"additionalProperties":false}"#;
const MAX_CODEX_PROMPT_ARG_BYTES: usize = 16 * 1024;
const MAX_LOCAL_PROMPT_ARG_BYTES: usize = 16 * 1024;
const SUPPORTED_TOOLS: &[&str] = &["codex", "claude", "ollama", "lmstudio"];

#[derive(Clone, Debug)]
pub struct DetectionRuntime {
    pub existing_default: String,
    pub env_tool: String,
    pub skip_auth: bool,
    pub tool_enabled: HashMap<String, bool>,
    pub tool_models: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct DetectionResult {
    pub id: String,
    pub found: bool,
    pub authenticated: bool,
    pub non_interactive: bool,
    pub score: i32,
    pub status: String,
    pub model: String,
}

#[derive(Clone, Debug)]
pub struct TranslationRequest {
    pub text: String,
    pub target_lang: Option<Language>,
    pub local_lang: Language,
    pub mode: TranslationMode,
}

#[derive(Clone, Debug, Default)]
pub struct RunOptions {
    pub model: String,
}

#[derive(Clone, Debug)]
struct RuntimeContext {
    timeout_ms: u64,
    work_dir: PathBuf,
    schema_path: PathBuf,
    last_message_path: PathBuf,
}

#[derive(Clone, Debug)]
struct ExecSpec {
    command: String,
    args: Vec<String>,
    stdin: String,
    work_dir: PathBuf,
    stream_json: bool,
}

#[derive(Clone, Debug)]
struct ExecResult {
    stdout: String,
    last_message_text: String,
}

pub fn by_id(id: &str) -> Option<&'static str> {
    match id {
        "codex" => Some("codex"),
        "claude" => Some("claude"),
        "ollama" => Some("ollama"),
        "lmstudio" | "lms" => Some("lmstudio"),
        _ => None,
    }
}

pub fn available_tools_hint() -> String {
    format!(
        "available tools: {} (alias: lms)",
        SUPPORTED_TOOLS.join(", ")
    )
}

pub fn detect_all(runtime: &DetectionRuntime) -> Vec<DetectionResult> {
    vec![
        detect("codex", runtime),
        detect("claude", runtime),
        detect("ollama", runtime),
        detect("lmstudio", runtime),
    ]
}

pub fn detect(id: &str, runtime: &DetectionRuntime) -> DetectionResult {
    let canonical = by_id(id).unwrap_or(id);
    if !tool_enabled(canonical, runtime) {
        return DetectionResult {
            id: canonical.to_string(),
            found: false,
            authenticated: false,
            non_interactive: false,
            score: 0,
            status: "disabled".to_string(),
            model: String::new(),
        };
    }
    match canonical {
        "codex" => detect_codex(runtime),
        "claude" => detect_claude(runtime),
        "ollama" => detect_ollama(runtime),
        "lmstudio" => detect_lmstudio(runtime),
        _ => DetectionResult {
            id: id.to_string(),
            found: false,
            authenticated: false,
            non_interactive: false,
            score: 0,
            status: "not found".to_string(),
            model: String::new(),
        },
    }
}

pub fn recommended_tool(results: &[DetectionResult]) -> Option<DetectionResult> {
    results
        .iter()
        .filter(|result| result.found && result.non_interactive)
        .max_by_key(|result| result.score)
        .cloned()
}

pub fn run_agent(
    tool_id: &str,
    req: &TranslationRequest,
    timeout_ms: u64,
    options: &RunOptions,
) -> AppResult<String> {
    let effective_timeout = if timeout_ms == 0 {
        default_timeout_ms()
    } else {
        timeout_ms
    };
    let temp_dir = TempDir::new()?;
    let runtime = RuntimeContext {
        timeout_ms: effective_timeout,
        work_dir: temp_work_dir(),
        schema_path: temp_dir.path.join("schema.json"),
        last_message_path: temp_dir.path.join("last-message.json"),
    };
    let spec = build_command(tool_id, req, &runtime, options)?;
    let raw = if spec.stream_json {
        run_streaming_json(&spec, tool_id, effective_timeout)?
    } else {
        run_command(&spec, tool_id, effective_timeout)?
    };
    extract_result(tool_id, &raw)
}

fn detect_codex(runtime: &DetectionRuntime) -> DetectionResult {
    let found_path = look_path("codex");
    let mut result = DetectionResult {
        id: "codex".to_string(),
        found: found_path.is_some(),
        authenticated: false,
        non_interactive: false,
        score: 0,
        status: "not found".to_string(),
        model: String::new(),
    };
    if !result.found {
        result.score = score(
            &result.id,
            result.found,
            result.authenticated,
            result.non_interactive,
            runtime,
        );
        return result;
    }

    result.non_interactive = true;
    if runtime.skip_auth {
        result.authenticated = true;
        result.status = "found".to_string();
        result.score = score(
            &result.id,
            result.found,
            result.authenticated,
            result.non_interactive,
            runtime,
        );
        return result;
    }

    if run_status_with_timeout("codex", &["login", "status"], Duration::from_secs(3))
        .unwrap_or(false)
    {
        result.authenticated = true;
        result.status = "found, authenticated".to_string();
    } else {
        result.status = "found, authentication unknown".to_string();
    }
    result.score = score(
        &result.id,
        result.found,
        result.authenticated,
        result.non_interactive,
        runtime,
    );
    result
}

fn detect_claude(runtime: &DetectionRuntime) -> DetectionResult {
    let found_path = look_path("claude");
    let mut result = DetectionResult {
        id: "claude".to_string(),
        found: found_path.is_some(),
        authenticated: false,
        non_interactive: found_path.is_some(),
        score: 0,
        status: if found_path.is_some() {
            "found"
        } else {
            "not found"
        }
        .to_string(),
        model: String::new(),
    };
    result.score = score(
        &result.id,
        result.found,
        result.authenticated,
        result.non_interactive,
        runtime,
    );
    result
}

fn detect_ollama(runtime: &DetectionRuntime) -> DetectionResult {
    let found_path = look_path("ollama");
    let mut result = DetectionResult {
        id: "ollama".to_string(),
        found: found_path.is_some(),
        authenticated: false,
        non_interactive: false,
        score: 0,
        status: "not found".to_string(),
        model: String::new(),
    };
    if !result.found {
        result.score = score(
            &result.id,
            result.found,
            result.authenticated,
            result.non_interactive,
            runtime,
        );
        return result;
    }

    if let Some(model) = resolve_detected_model("ollama", runtime, detect_ollama_model) {
        result.model = model;
        result.non_interactive = true;
        result.status = format!("found, model {}", result.model);
    } else {
        result.status = "found, no local models (run: ollama pull gemma3)".to_string();
    }
    result.score = score(
        &result.id,
        result.found,
        result.authenticated,
        result.non_interactive,
        runtime,
    );
    result
}

fn detect_lmstudio(runtime: &DetectionRuntime) -> DetectionResult {
    let found_path = look_path("lms");
    let mut result = DetectionResult {
        id: "lmstudio".to_string(),
        found: found_path.is_some(),
        authenticated: false,
        non_interactive: false,
        score: 0,
        status: "not found".to_string(),
        model: String::new(),
    };
    if !result.found {
        result.score = score(
            &result.id,
            result.found,
            result.authenticated,
            result.non_interactive,
            runtime,
        );
        return result;
    }

    if let Some(model) = resolve_detected_model("lmstudio", runtime, detect_lmstudio_model) {
        result.model = model;
        result.non_interactive = true;
        result.status = format!("found, model {}", result.model);
    } else {
        result.status = "found, no local LLM models (open LM Studio or run: lms get)".to_string();
    }
    result.score = score(
        &result.id,
        result.found,
        result.authenticated,
        result.non_interactive,
        runtime,
    );
    result
}

fn build_command(
    tool_id: &str,
    req: &TranslationRequest,
    runtime: &RuntimeContext,
    options: &RunOptions,
) -> AppResult<ExecSpec> {
    match by_id(tool_id).unwrap_or(tool_id) {
        "codex" => Ok(build_codex_command(req, runtime)),
        "claude" => Ok(build_claude_command(req)),
        "ollama" => build_ollama_command(req, options),
        "lmstudio" => build_lmstudio_command(req, options),
        _ => Err(AppError::new(
            crate::error::EXIT_ARGS,
            format!("unsupported tool: {tool_id}"),
        )
        .with_hint(available_tools_hint())),
    }
}

fn build_codex_command(req: &TranslationRequest, runtime: &RuntimeContext) -> ExecSpec {
    let _ = (
        &runtime.timeout_ms,
        &runtime.schema_path,
        &runtime.last_message_path,
    );
    let prompt = build_plain_text_prompt(req);
    let mut args = vec![
        "--ask-for-approval",
        "never",
        "--model",
        "gpt-5.3-codex-spark",
        "-c",
        "model_reasoning_effort=\"low\"",
        "-c",
        "include_permissions_instructions=false",
        "-c",
        "include_apps_instructions=false",
        "-c",
        "include_environment_context=false",
        "-c",
        "include_apply_patch_tool=false",
        "exec",
        "--skip-git-repo-check",
        "--ignore-user-config",
        "--ignore-rules",
        "--ephemeral",
        "--sandbox",
        "read-only",
        "--color",
        "never",
        "--json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    let mut stdin = String::new();
    if prompt.len() <= MAX_CODEX_PROMPT_ARG_BYTES {
        args.push(prompt);
    } else {
        args.push("-".to_string());
        stdin = prompt;
    }

    ExecSpec {
        command: "codex".to_string(),
        args,
        stdin,
        work_dir: runtime.work_dir.clone(),
        stream_json: true,
    }
}

fn build_claude_command(req: &TranslationRequest) -> ExecSpec {
    ExecSpec {
        command: "claude".to_string(),
        args: vec![
            "--bare".to_string(),
            "-p".to_string(),
            build_prompt(req),
            "--output-format".to_string(),
            "json".to_string(),
            "--json-schema".to_string(),
            JSON_SCHEMA.to_string(),
            "--no-session-persistence".to_string(),
            "--max-turns".to_string(),
            "1".to_string(),
            "--tools".to_string(),
            String::new(),
        ],
        stdin: String::new(),
        work_dir: PathBuf::new(),
        stream_json: false,
    }
}

fn build_ollama_command(req: &TranslationRequest, options: &RunOptions) -> AppResult<ExecSpec> {
    let model = runtime_model("ollama", options, detect_ollama_model)?;
    let prompt = build_plain_text_prompt(req);
    let mut args = vec!["run".to_string(), model];
    let mut stdin = String::new();
    if prompt.len() <= MAX_LOCAL_PROMPT_ARG_BYTES {
        args.push(prompt);
    } else {
        stdin = prompt;
    }
    Ok(ExecSpec {
        command: "ollama".to_string(),
        args,
        stdin,
        work_dir: PathBuf::new(),
        stream_json: false,
    })
}

fn build_lmstudio_command(req: &TranslationRequest, options: &RunOptions) -> AppResult<ExecSpec> {
    let model = runtime_model("lmstudio", options, detect_lmstudio_model)?;
    let prompt = build_plain_text_prompt(req);
    let mut args = vec!["chat".to_string(), model, "-p".to_string()];
    let mut stdin = String::new();
    if prompt.len() <= MAX_LOCAL_PROMPT_ARG_BYTES {
        args.push(prompt);
    } else {
        args.push(
            "Follow the translation instructions from stdin. Return only the translated text."
                .to_string(),
        );
        stdin = prompt;
    }
    Ok(ExecSpec {
        command: "lms".to_string(),
        args,
        stdin,
        work_dir: PathBuf::new(),
        stream_json: false,
    })
}

fn extract_result(tool_id: &str, raw: &ExecResult) -> AppResult<String> {
    match tool_id {
        "codex" => {
            let source = if raw.last_message_text.trim().is_empty() {
                &raw.stdout
            } else {
                &raw.last_message_text
            };
            normalize_allow_raw(source)
        }
        "claude" => normalize_allow_raw(&raw.stdout),
        "ollama" | "lmstudio" => normalize_allow_raw(&raw.stdout),
        _ => normalize_allow_raw(&raw.stdout),
    }
}

fn run_streaming_json(spec: &ExecSpec, id: &str, timeout_ms: u64) -> AppResult<ExecResult> {
    let mut child = spawn_spec(spec)?;
    write_child_stdin(&mut child, &spec.stdin)?;

    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let (final_tx, final_rx) = mpsc::channel::<String>();
    let stdout_thread = thread::spawn(move || {
        let mut stdout_text = String::new();
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            stdout_text.push_str(&line);
            stdout_text.push('\n');
            if let Some(text) = json_agent_message(&line) {
                let _ = final_tx.send(text);
                break;
            }
        }
        stdout_text
    });
    let stderr_thread = read_to_string_thread(stderr);
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);

    loop {
        if let Ok(final_text) = final_rx.try_recv() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stderr_thread.join();
            let _ = stdout_thread.join();
            return Ok(ExecResult {
                stdout: final_text,
                last_message_text: String::new(),
            });
        }

        if let Some(status) = child.try_wait()? {
            let stdout = stdout_thread.join().unwrap_or_default();
            let stderr = stderr_thread.join().unwrap_or_default();
            if !status.success() {
                return Err(agent_run_error(
                    id,
                    status.code(),
                    None,
                    if stderr.is_empty() { &stdout } else { &stderr },
                ));
            }
            if let Some(final_text) = json_agent_message_from_stream(&stdout) {
                return Ok(ExecResult {
                    stdout: final_text,
                    last_message_text: String::new(),
                });
            }
            return Ok(ExecResult {
                stdout,
                last_message_text: String::new(),
            });
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_thread.join();
            let _ = stderr_thread.join();
            return Err(AppError::new(
                EXIT_TIMEOUT,
                format!("translation timed out after {}s", timeout_ms / 1000),
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn run_command(spec: &ExecSpec, id: &str, timeout_ms: u64) -> AppResult<ExecResult> {
    let mut child = spawn_spec(spec)?;
    write_child_stdin(&mut child, &spec.stdin)?;

    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let stdout_thread = read_to_string_thread(stdout);
    let stderr_thread = read_to_string_thread(stderr);
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);

    loop {
        if let Some(status) = child.try_wait()? {
            let stdout = stdout_thread.join().unwrap_or_default();
            let stderr = stderr_thread.join().unwrap_or_default();
            if !status.success() {
                return Err(agent_run_error(id, status.code(), None, &stderr));
            }
            return Ok(ExecResult {
                stdout,
                last_message_text: String::new(),
            });
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_thread.join();
            let _ = stderr_thread.join();
            return Err(AppError::new(
                EXIT_TIMEOUT,
                format!("translation timed out after {}s", timeout_ms / 1000),
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn spawn_spec(spec: &ExecSpec) -> AppResult<std::process::Child> {
    let mut command = Command::new(&spec.command);
    command.args(&spec.args);
    if !spec.work_dir.as_os_str().is_empty() {
        command.current_dir(&spec.work_dir);
    }
    command.stdin(if spec.stdin.is_empty() {
        Stdio::null()
    } else {
        Stdio::piped()
    });
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command
        .spawn()
        .map_err(|error| AppError::new(crate::error::EXIT_AGENT, error.to_string()))
}

fn write_child_stdin(child: &mut std::process::Child, stdin: &str) -> AppResult<()> {
    if !stdin.is_empty() {
        if let Some(mut child_stdin) = child.stdin.take() {
            child_stdin.write_all(stdin.as_bytes())?;
        }
    }
    Ok(())
}

fn read_to_string_thread<R>(mut reader: R) -> thread::JoinHandle<String>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut text = String::new();
        let _ = reader.read_to_string(&mut text);
        text
    })
}

pub fn json_agent_message(line: &str) -> Option<String> {
    let event = serde_json::from_str::<Value>(line).ok()?;
    if event.get("type").and_then(Value::as_str) != Some("item.completed") {
        return None;
    }
    let item = event.get("item")?;
    if item.get("type").and_then(Value::as_str) != Some("agent_message") {
        return None;
    }
    let text = item.get("text").and_then(Value::as_str)?.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

fn json_agent_message_from_stream(stdout: &str) -> Option<String> {
    stdout.lines().filter_map(json_agent_message).last()
}

pub fn look_path(command: &str) -> Option<String> {
    let path_env = env::var_os("PATH")?;
    let extensions = if cfg!(target_os = "windows") {
        env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".to_string())
            .split(';')
            .map(str::to_string)
            .collect::<Vec<_>>()
    } else {
        vec![String::new()]
    };

    for dir in env::split_paths(&path_env) {
        for ext in &extensions {
            let candidate = dir.join(format!("{command}{ext}"));
            if is_executable(&candidate) {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn run_status_with_timeout(
    command: &str,
    args: &[&str],
    timeout: Duration,
) -> std::io::Result<bool> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status.success());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn run_output_with_timeout(
    command: &str,
    args: &[&str],
    timeout: Duration,
) -> std::io::Result<Option<String>> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().expect("stdout is piped");
    let stdout_thread = read_to_string_thread(stdout);
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            let stdout = stdout_thread.join().unwrap_or_default();
            return Ok(status.success().then_some(stdout));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_thread.join();
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn resolve_detected_model<F>(
    id: &str,
    runtime: &DetectionRuntime,
    detect_model: F,
) -> Option<String>
where
    F: FnOnce() -> Option<String>,
{
    env_model(id)
        .or_else(|| configured_model(id, runtime))
        .or_else(detect_model)
}

fn runtime_model<F>(id: &str, options: &RunOptions, detect_model: F) -> AppResult<String>
where
    F: FnOnce() -> Option<String>,
{
    env_model(id)
        .or_else(|| non_empty(options.model.clone()))
        .or_else(detect_model)
        .ok_or_else(|| {
            AppError::new(
                crate::error::EXIT_CONFIG,
                format!("{id} model is not configured"),
            )
            .with_hint(model_setup_hint(id))
        })
}

fn env_model(id: &str) -> Option<String> {
    let key = format!("TRANSLATE_CLI_{}_MODEL", id.to_uppercase());
    env::var(key).ok().and_then(non_empty).or_else(|| {
        if id == "lmstudio" {
            env::var("TRANSLATE_CLI_LMS_MODEL").ok().and_then(non_empty)
        } else {
            None
        }
    })
}

fn configured_model(id: &str, runtime: &DetectionRuntime) -> Option<String> {
    runtime
        .tool_models
        .get(id)
        .cloned()
        .or_else(|| {
            if id == "lmstudio" {
                runtime.tool_models.get("lms").cloned()
            } else {
                None
            }
        })
        .and_then(non_empty)
}

fn tool_enabled(id: &str, runtime: &DetectionRuntime) -> bool {
    runtime
        .tool_enabled
        .get(id)
        .copied()
        .or_else(|| {
            if id == "lmstudio" {
                runtime.tool_enabled.get("lms").copied()
            } else {
                None
            }
        })
        .unwrap_or(true)
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn model_setup_hint(id: &str) -> &'static str {
    match id {
        "ollama" => "run: ollama pull gemma3; then run: t --setup",
        "lmstudio" => "open LM Studio and download a model, or run: lms get; then run: t --setup",
        _ => "run: t --setup",
    }
}

fn detect_ollama_model() -> Option<String> {
    let stdout = run_output_with_timeout("ollama", &["ls"], Duration::from_secs(3)).ok()??;
    first_model(parse_ollama_models(&stdout))
}

fn parse_ollama_models(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("NAME") {
                return None;
            }
            let name = trimmed.split_whitespace().next()?;
            is_model_candidate(name).then(|| name.to_string())
        })
        .collect()
}

fn detect_lmstudio_model() -> Option<String> {
    for args in [
        &["ps", "--json"][..],
        &["ls", "--llm", "--json"][..],
        &["ps"][..],
        &["ls", "--llm"][..],
    ] {
        let Some(stdout) = run_output_with_timeout("lms", args, Duration::from_secs(3))
            .ok()
            .flatten()
        else {
            continue;
        };
        let models = if args.contains(&"--json") {
            parse_lmstudio_json_models(&stdout)
        } else {
            parse_lmstudio_text_models(&stdout)
        };
        if let Some(model) = first_model(models) {
            return Some(model);
        }
    }
    None
}

fn parse_lmstudio_json_models(stdout: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(stdout) else {
        return Vec::new();
    };
    let mut models = Vec::new();
    collect_model_candidates(&value, &mut models);
    models
}

fn collect_model_candidates(value: &Value, models: &mut Vec<String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_model_candidates(value, models);
            }
        }
        Value::Object(map) => {
            for key in ["identifier", "id", "model", "modelKey", "name", "path"] {
                if let Some(model) = map
                    .get(key)
                    .and_then(Value::as_str)
                    .filter(|value| is_model_candidate(value))
                {
                    push_unique_model(models, model);
                }
            }
            for value in map.values() {
                collect_model_candidates(value, models);
            }
        }
        _ => {}
    }
}

fn parse_lmstudio_text_models(stdout: &str) -> Vec<String> {
    let mut models = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(model) = trimmed.strip_prefix("Identifier:") {
            push_unique_model(&mut models, model.trim());
            continue;
        }
        let Some(candidate) = trimmed.split_whitespace().next() else {
            continue;
        };
        if is_model_candidate(candidate) {
            push_unique_model(&mut models, candidate);
        }
    }
    models
}

fn first_model(models: Vec<String>) -> Option<String> {
    models.into_iter().find(|model| is_model_candidate(model))
}

fn push_unique_model(models: &mut Vec<String>, value: &str) {
    let Some(model) = non_empty(value.to_string()) else {
        return;
    };
    if is_model_candidate(&model) && !models.iter().any(|existing| existing == &model) {
        models.push(model);
    }
}

fn is_model_candidate(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) {
        return false;
    }
    !matches!(
        trimmed,
        "NAME"
            | "ID"
            | "SIZE"
            | "MODIFIED"
            | "LLMs"
            | "Embedding"
            | "Embeddings"
            | "Models"
            | "PARAMS"
            | "ARCHITECTURE"
    ) && !trimmed.starts_with("...")
        && !trimmed.ends_with(':')
}

fn score(
    id: &str,
    found: bool,
    authenticated: bool,
    non_interactive: bool,
    runtime: &DetectionRuntime,
) -> i32 {
    let mut total = 0;
    if found {
        total += 50;
        total += popularity_score(id);
    }
    if authenticated {
        total += 30;
    }
    if non_interactive {
        total += 20;
    }
    let existing_default = by_id(&runtime.existing_default).unwrap_or(&runtime.existing_default);
    let env_tool = by_id(&runtime.env_tool).unwrap_or(&runtime.env_tool);
    if existing_default == id {
        total += 100;
    }
    if env_tool == id {
        total += 100;
    }
    total
}

fn popularity_score(id: &str) -> i32 {
    match id {
        "ollama" => 80,
        "lmstudio" => 60,
        "codex" => 20,
        "claude" => 10,
        _ => 0,
    }
}

fn temp_work_dir() -> PathBuf {
    if !cfg!(target_os = "windows") && Path::new("/tmp").is_dir() {
        PathBuf::from("/tmp")
    } else {
        env::temp_dir()
    }
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> AppResult<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = env::temp_dir().join(format!("translate-cli-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::must_language;

    #[test]
    fn extracts_codex_agent_message() {
        let line =
            r#"{"type":"item.completed","item":{"type":"agent_message","text":"こんにちは"}}"#;
        assert_eq!(json_agent_message(line).unwrap(), "こんにちは");
    }

    #[test]
    fn extracts_codex_agent_message_from_completed_stream() {
        let stdout = concat!(
            "{\"type\":\"thread.started\",\"thread_id\":\"fake\"}\n",
            "{\"type\":\"turn.started\"}\n",
            "{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",\"text\":\"こんにちは\"}}\n"
        );
        assert_eq!(
            json_agent_message_from_stream(stdout).unwrap(),
            "こんにちは"
        );
    }

    #[test]
    fn builds_codex_fast_path_flags() {
        let req = TranslationRequest {
            text: "hello".to_string(),
            target_lang: Some(must_language("ja")),
            local_lang: must_language("en"),
            mode: TranslationMode::Target,
        };
        let runtime = RuntimeContext {
            timeout_ms: 60_000,
            work_dir: PathBuf::from("/tmp"),
            schema_path: PathBuf::from("/tmp/schema.json"),
            last_message_path: PathBuf::from("/tmp/out.json"),
        };
        let spec = build_codex_command(&req, &runtime);
        assert_eq!(spec.command, "codex");
        assert!(spec.args.contains(&"--json".to_string()));
        assert!(spec.args.contains(&"--ignore-user-config".to_string()));
        assert!(spec.args.contains(&"gpt-5.3-codex-spark".to_string()));
        assert_eq!(spec.stdin, "");
    }

    #[test]
    fn parses_ollama_ls_models() {
        let models = parse_ollama_models(
            "NAME            ID              SIZE      MODIFIED\ngemma3:latest   abc123          3.3 GB    1 hour ago\n",
        );
        assert_eq!(models, vec!["gemma3:latest"]);
    }

    #[test]
    fn parses_lmstudio_json_models() {
        let models =
            parse_lmstudio_json_models(r#"[{"identifier":"lmstudio-community/gemma-3-4b-it"}]"#);
        assert_eq!(models, vec!["lmstudio-community/gemma-3-4b-it"]);
    }

    #[test]
    fn builds_ollama_command_with_configured_model() {
        let req = TranslationRequest {
            text: "hello".to_string(),
            target_lang: Some(must_language("ja")),
            local_lang: must_language("en"),
            mode: TranslationMode::Target,
        };
        let spec = build_ollama_command(
            &req,
            &RunOptions {
                model: "gemma3:latest".to_string(),
            },
        )
        .unwrap();
        assert_eq!(spec.command, "ollama");
        assert_eq!(spec.args[0], "run");
        assert_eq!(spec.args[1], "gemma3:latest");
        assert!(spec.args[2].contains("Return only the translated text."));
    }

    #[test]
    fn builds_lmstudio_command_with_configured_model() {
        let req = TranslationRequest {
            text: "hello".to_string(),
            target_lang: Some(must_language("ja")),
            local_lang: must_language("en"),
            mode: TranslationMode::Target,
        };
        let spec = build_lmstudio_command(
            &req,
            &RunOptions {
                model: "lmstudio-community/gemma-3-4b-it".to_string(),
            },
        )
        .unwrap();
        assert_eq!(spec.command, "lms");
        assert_eq!(spec.args[0], "chat");
        assert_eq!(spec.args[1], "lmstudio-community/gemma-3-4b-it");
        assert_eq!(spec.args[2], "-p");
        assert!(spec.args[3].contains("Return only the translated text."));
    }
}
