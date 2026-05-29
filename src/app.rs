use crate::agent::{
    available_tools_hint, by_id, detect, DetectionRuntime, RunOptions, TranslationRequest,
};
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

    let selected = selected_tool(&inv.tool, &config);
    let Some(tool_id) = by_id(&selected).map(str::to_string) else {
        return Err(
            AppError::new(EXIT_ARGS, format!("unsupported tool: {selected}"))
                .with_hint(available_tools_hint()),
        );
    };

    let detection = detect(&tool_id, &detection_runtime(&config, true));
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
    let run_options = config
        .tools
        .get(&tool_id)
        .or_else(|| {
            if tool_id == "lmstudio" {
                config.tools.get("lms")
            } else {
                None
            }
        })
        .map(|tool| RunOptions {
            model: tool.model.clone(),
        })
        .unwrap_or_default();
    let text = crate::agent::run_agent(&tool_id, &req, config.timeout_ms, &run_options)?;
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
    let detection = detect(&config.default_tool, &detection_runtime(config, true));
    !detection.found || !detection.non_interactive
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

fn detection_runtime(config: &Config, skip_auth: bool) -> DetectionRuntime {
    DetectionRuntime {
        existing_default: config.default_tool.clone(),
        env_tool: std::env::var("TRANSLATE_CLI_TOOL").unwrap_or_default(),
        skip_auth,
        tool_enabled: config
            .tools
            .iter()
            .map(|(id, tool)| (id.clone(), tool.enabled))
            .collect(),
        tool_models: config
            .tools
            .iter()
            .map(|(id, tool)| (id.clone(), tool.model.clone()))
            .collect(),
    }
}
