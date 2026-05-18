use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult, EXIT_CONFIG};
use crate::lang::detect_local_language;

const DEFAULT_TIMEOUT_MS: u64 = 60_000;

#[derive(Clone, Debug)]
pub struct ToolConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub version: u32,
    pub default_tool: String,
    pub local_lang: String,
    pub timeout_ms: u64,
    pub tools: HashMap<String, ToolConfig>,
}

pub fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

pub fn default_config() -> Config {
    let mut tools = HashMap::new();
    tools.insert("codex".to_string(), ToolConfig { enabled: true });
    tools.insert("claude".to_string(), ToolConfig { enabled: true });
    Config {
        version: 1,
        default_tool: String::new(),
        local_lang: detect_local_language().code,
        timeout_ms: DEFAULT_TIMEOUT_MS,
        tools,
    }
}

pub fn load_config(config_path: &Path) -> AppResult<(Config, bool)> {
    let mut config = default_config();
    let data = match fs::read_to_string(config_path) {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((config, false)),
        Err(error) => return Err(error.into()),
    };

    parse_config(&mut config, &data).map_err(|message| {
        AppError::new(EXIT_CONFIG, format!("failed to parse config: {message}"))
            .with_hint(config_path.display().to_string())
    })?;

    if config.timeout_ms == 0 {
        config.timeout_ms = DEFAULT_TIMEOUT_MS;
    }
    Ok((config, true))
}

pub fn save_config(config_path: &Path, config: &Config) -> AppResult<()> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
        }
    }

    let codex_enabled = config
        .tools
        .get("codex")
        .map(|tool| tool.enabled)
        .unwrap_or(false);
    let claude_enabled = config
        .tools
        .get("claude")
        .map(|tool| tool.enabled)
        .unwrap_or(false);
    let body = format!(
        "version = {}\ndefault_tool = {}\nlocal_lang = {}\ntimeout_ms = {}\n\n[tools.codex]\nenabled = {}\n\n[tools.claude]\nenabled = {}\n",
        if config.version == 0 { 1 } else { config.version },
        json_string(&config.default_tool),
        json_string(&config.local_lang),
        if config.timeout_ms == 0 { DEFAULT_TIMEOUT_MS } else { config.timeout_ms },
        codex_enabled,
        claude_enabled
    );

    let mut options = fs::OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(config_path)?;
    file.write_all(body.as_bytes())?;
    Ok(())
}

pub fn default_config_path() -> AppResult<PathBuf> {
    if let Ok(value) = std::env::var("TRANSLATE_CLI_CONFIG") {
        if !value.is_empty() {
            return Ok(PathBuf::from(value));
        }
    }

    if cfg!(target_os = "macos") {
        return Ok(home_dir()?.join("Library/Application Support/translate-cli/config.toml"));
    }
    if cfg!(target_os = "windows") {
        let base = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                home_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("AppData/Roaming")
            });
        return Ok(base.join("translate-cli/config.toml"));
    }
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            home_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(".config")
        });
    Ok(base.join("translate-cli/config.toml"))
}

fn parse_config(config: &mut Config, data: &str) -> Result<(), String> {
    let mut section = String::new();
    for raw_line in data.lines() {
        let line = strip_comment(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        let Some(eq) = line.find('=') else {
            return Err(format!("invalid line {line:?}"));
        };
        let key = line[..eq].trim();
        let value = line[eq + 1..].trim();
        if section.is_empty() {
            parse_root_value(config, key, value)?;
        } else if section == "tools.codex" || section == "tools.claude" {
            let id = section.trim_start_matches("tools.").to_string();
            if key == "enabled" {
                config.tools.insert(
                    id,
                    ToolConfig {
                        enabled: parse_bool(value, key)?,
                    },
                );
            }
        }
    }
    Ok(())
}

fn parse_root_value(config: &mut Config, key: &str, value: &str) -> Result<(), String> {
    match key {
        "version" => config.version = parse_integer(value, key)? as u32,
        "default_tool" => config.default_tool = parse_string(value, key)?,
        "local_lang" => config.local_lang = parse_string(value, key)?,
        "timeout_ms" => config.timeout_ms = parse_integer(value, key)?,
        _ => {}
    }
    Ok(())
}

fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(index) => &line[..index],
        None => line,
    }
}

fn parse_integer(value: &str, key: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("{key}: invalid integer"))
}

fn parse_string(value: &str, key: &str) -> Result<String, String> {
    serde_json::from_str::<String>(value).map_err(|_| format!("{key}: invalid string"))
}

fn parse_bool(value: &str, key: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("{key}: invalid boolean")),
    }
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization cannot fail")
}

fn home_dir() -> AppResult<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| {
            AppError::new(
                crate::error::EXIT_GENERAL,
                "home directory is not available",
            )
        })
}
