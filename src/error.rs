use std::fmt;
use std::io;

pub const EXIT_GENERAL: i32 = 1;
pub const EXIT_ARGS: i32 = 2;
pub const EXIT_CONFIG: i32 = 3;
pub const EXIT_TOOL_NOT_FOUND: i32 = 4;
pub const EXIT_AGENT: i32 = 5;
pub const EXIT_TIMEOUT: i32 = 6;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone)]
pub struct AppError {
    pub code: i32,
    pub message: String,
    pub hint: String,
}

impl AppError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: String::new(),
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::new(EXIT_GENERAL, error.to_string())
    }
}

pub fn render_error(error: &AppError) -> i32 {
    eprintln!("error: {}", error.message);
    if !error.hint.is_empty() {
        eprintln!("hint: {}", error.hint);
    }
    error.code
}

pub fn agent_run_error(
    id: &str,
    code: Option<i32>,
    signal: Option<String>,
    stderr: &str,
) -> AppError {
    let status = match code {
        Some(code) => format!("exit status {code}"),
        None => format!("signal {}", signal.unwrap_or_else(|| "unknown".to_string())),
    };
    let message = run_error_message(&status, stderr);
    match id {
        "codex" => AppError::new(EXIT_AGENT, format!("codex failed to run: {message}"))
            .with_hint("Run: codex"),
        "claude" => AppError::new(EXIT_AGENT, format!("claude failed to run: {message}"))
            .with_hint("Run: claude"),
        _ => AppError::new(EXIT_AGENT, format!("{id} failed to run: {message}")),
    }
}

fn run_error_message(status: &str, stderr: &str) -> String {
    let compact = stderr.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        status.to_string()
    } else if compact.len() > 500 {
        format!("{status}: {}...", &compact[..500])
    } else {
        format!("{status}: {compact}")
    }
}
