use std::io::{self, Write};
use std::path::Path;

use crate::agent::{detect_all, recommended_tool, DetectionResult, DetectionRuntime};
use crate::config::{default_config, save_config, Config};
use crate::error::{AppError, AppResult, EXIT_TOOL_NOT_FOUND};
use crate::input::{read_all_stdin, stdin_is_terminal};
use crate::lang::{detect_local_language, must_language};

pub fn run_wizard(config_path: &Path, initial_config: Config) -> AppResult<Config> {
    let mut config = if initial_config.version == 0 {
        default_config()
    } else {
        initial_config
    };
    let runtime = DetectionRuntime {
        existing_default: config.default_tool.clone(),
        env_tool: std::env::var("TRANSLATE_CLI_TOOL").unwrap_or_default(),
        skip_auth: false,
    };
    let results = detect_all(&runtime);
    let tools = results
        .into_iter()
        .filter(|result| result.found)
        .collect::<Vec<_>>();
    if tools.is_empty() {
        return Err(
            AppError::new(EXIT_TOOL_NOT_FOUND, "no supported Agent CLI found")
                .with_hint("Install one of: codex, claude. Then run: t --setup"),
        );
    }

    let mut local = if config.local_lang.is_empty() {
        detect_local_language()
    } else {
        must_language(&config.local_lang)
    };
    let recommended = recommended_tool(&tools).unwrap_or_else(|| tools[0].clone());
    let mut scripted_answers = if stdin_is_terminal() {
        None
    } else {
        Some(
            read_all_stdin()?
                .lines()
                .map(str::to_string)
                .collect::<Vec<_>>(),
        )
    };

    eprintln!("Translate CLI setup\n");
    eprintln!("Translate CLI sends text to the selected Agent CLI.");
    eprintln!("The Agent CLI may send it to its configured model provider.");
    eprintln!("Translate CLI does not store API keys.\n");
    eprintln!(
        "Detected your local language: {} ({})\n",
        local.name, local.code
    );
    eprintln!("Available tools:");
    for (index, tool) in tools.iter().enumerate() {
        eprintln!("  {}. {:<6} {}", index + 1, tool.id, tool.status);
    }
    eprintln!();
    eprintln!("Recommended tool: {}", recommended.id);

    let mut selected = recommended.id.clone();
    let use_recommended = ask(
        &mut scripted_answers,
        &format!("Use {} as the default tool? [Y/n] ", recommended.id),
    )?;
    if is_no(&use_recommended) {
        selected = select_tool(&mut scripted_answers, &tools)?;
    }

    let use_pair = ask(
        &mut scripted_answers,
        &format!(
            "Use {} <-> English as default translation pair? [Y/n] ",
            local.name
        ),
    )?;
    if is_no(&use_pair) {
        let custom = ask(
            &mut scripted_answers,
            &format!("Local language code [{}]: ", local.code),
        )?;
        if !custom.is_empty() {
            local = must_language(&custom);
        }
    }

    config.default_tool = selected;
    config.local_lang = local.code;
    if config.timeout_ms == 0 {
        config.timeout_ms = crate::config::default_timeout_ms();
    }
    save_config(config_path, &config)?;
    eprintln!("\nSaved config: {}", config_path.display());
    Ok(config)
}

fn select_tool(
    scripted_answers: &mut Option<Vec<String>>,
    tools: &[DetectionResult],
) -> AppResult<String> {
    eprintln!("Select default tool:");
    for (index, tool) in tools.iter().enumerate() {
        eprintln!("  {}. {}", index + 1, tool.id);
    }
    let answer = ask(scripted_answers, "Tool [1]: ")?;
    let selected = answer
        .parse::<usize>()
        .ok()
        .and_then(|n| n.checked_sub(1))
        .and_then(|index| tools.get(index))
        .unwrap_or(&tools[0]);
    Ok(selected.id.clone())
}

fn ask(scripted_answers: &mut Option<Vec<String>>, prompt: &str) -> AppResult<String> {
    eprint!("{prompt}");
    io::stderr().flush()?;
    if let Some(answers) = scripted_answers {
        if answers.is_empty() {
            return Ok(String::new());
        }
        return Ok(answers.remove(0).trim().to_string());
    }
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(answer.trim().to_string())
}

fn is_no(answer: &str) -> bool {
    matches!(answer.to_lowercase().as_str(), "n" | "no")
}
