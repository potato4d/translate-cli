use crate::agent::{by_id, detect, DetectionRuntime, TranslationRequest};
use crate::cli::{parse_args, usage};
use crate::config::{default_config_path, load_config, Config};
use crate::error::{AppError, AppResult, EXIT_ARGS, EXIT_CONFIG, EXIT_TOOL_NOT_FOUND};
use crate::input::read_stdin_if_available;
use crate::lang::must_language;
use crate::wizard::run_wizard;

pub fn run(args: &[String]) -> AppResult<()> {
    let mut inv = parse_args(args)?;
    if inv.help {
        println!("{}", usage());
        return Ok(());
    }
    if inv.version {
        println!("translate-cli {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if inv.text.is_empty() && !inv.setup {
        if let Some(stdin) = read_stdin_if_available()? {
            inv.text = stdin;
        }
    }

    let config_path = default_config_path()?;
    let (mut config, exists) = load_config(&config_path)?;

    if inv.setup || needs_wizard(&config, exists) {
        if inv.no_wizard {
            return Err(AppError::new(EXIT_CONFIG, "setup is required"));
        }
        config = run_wizard(&config_path, config)?;
        if inv.setup && inv.text.is_empty() && inv.target_lang.is_none() {
            return Ok(());
        }
    }

    if inv.text.is_empty() {
        if let Some(stdin) = read_stdin_if_available()? {
            inv.text = stdin;
        }
    }
    if inv.text.trim().is_empty() {
        return Err(AppError::new(EXIT_ARGS, "missing text to translate").with_hint(usage()));
    }

    let tool_id = selected_tool(&inv.tool, &config);
    if by_id(&tool_id).is_none() {
        return Err(
            AppError::new(EXIT_ARGS, format!("unsupported tool: {tool_id}"))
                .with_hint("available tools: codex, claude"),
        );
    }

    let detection = detect(
        &tool_id,
        &DetectionRuntime {
            existing_default: config.default_tool.clone(),
            env_tool: std::env::var("TRANSLATE_CLI_TOOL").unwrap_or_default(),
            skip_auth: true,
        },
    );
    if !detection.found {
        return Err(AppError::new(
            EXIT_TOOL_NOT_FOUND,
            format!("tool \"{tool_id}\" is not installed or not found in PATH"),
        ));
    }
    if tool_id == "codex" && !detection.authenticated {
        return Err(
            AppError::new(crate::error::EXIT_AGENT, "codex is not authenticated")
                .with_hint("Run: codex"),
        );
    }

    let req = TranslationRequest {
        text: inv.text,
        target_lang: inv.target_lang,
        local_lang: must_language(&config.local_lang),
        mode: inv.mode,
    };
    let text = crate::agent::run_agent(&tool_id, &req, config.timeout_ms)?;
    println!("{text}");
    Ok(())
}

fn needs_wizard(config: &Config, exists: bool) -> bool {
    if !exists || config.default_tool.is_empty() || config.local_lang.is_empty() {
        return true;
    }
    if by_id(&config.default_tool).is_none() {
        return true;
    }
    let detection = detect(
        &config.default_tool,
        &DetectionRuntime {
            existing_default: config.default_tool.clone(),
            env_tool: std::env::var("TRANSLATE_CLI_TOOL").unwrap_or_default(),
            skip_auth: true,
        },
    );
    !detection.found
}

fn selected_tool(inv_tool: &str, config: &Config) -> String {
    if !inv_tool.is_empty() {
        return inv_tool.to_string();
    }
    if let Ok(env_tool) = std::env::var("TRANSLATE_CLI_TOOL") {
        if !env_tool.is_empty() {
            return env_tool;
        }
    }
    config.default_tool.clone()
}
