use crate::error::{AppError, AppResult, EXIT_ARGS};
use crate::lang::{resolve_language, Language};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TranslationMode {
    AutoPair,
    Target,
}

#[derive(Clone, Debug)]
pub struct Invocation {
    pub tool: String,
    pub text: String,
    pub target_lang: Option<Language>,
    pub mode: TranslationMode,
    pub version: bool,
    pub help: bool,
    pub setup: bool,
    pub no_wizard: bool,
}

impl Default for Invocation {
    fn default() -> Self {
        Self {
            tool: String::new(),
            text: String::new(),
            target_lang: None,
            mode: TranslationMode::AutoPair,
            version: false,
            help: false,
            setup: false,
            no_wizard: false,
        }
    }
}

pub fn parse_args(args: &[String]) -> AppResult<Invocation> {
    let mut inv = Invocation::default();
    let mut positional = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];
        if arg == "--" {
            positional.extend(args[i + 1..].iter().cloned());
            break;
        }
        match arg.as_str() {
            "--version" | "-v" => inv.version = true,
            "--help" | "-h" => inv.help = true,
            "--setup" => inv.setup = true,
            "--no-wizard" => inv.no_wizard = true,
            "--tool" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| AppError::new(EXIT_ARGS, "--tool requires a value"))?;
                inv.tool = value.clone();
                i += 1;
            }
            _ if arg.starts_with("--tool=") => {
                inv.tool = arg["--tool=".len()..].to_string();
                if inv.tool.is_empty() {
                    return Err(AppError::new(EXIT_ARGS, "--tool requires a value"));
                }
            }
            _ if arg.starts_with('-') => {
                return Err(AppError::new(EXIT_ARGS, format!("unknown option: {arg}")));
            }
            _ => positional.push(arg.clone()),
        }
        i += 1;
    }

    apply_positional(&mut inv, &positional);
    Ok(inv)
}

fn apply_positional(inv: &mut Invocation, positional: &[String]) {
    if positional.is_empty() {
        inv.mode = TranslationMode::AutoPair;
        return;
    }

    if let Some(lang) = resolve_language(&positional[0]) {
        inv.target_lang = Some(lang);
        inv.mode = TranslationMode::Target;
        if positional.len() > 1 {
            inv.text = positional[1..].join(" ");
        }
        return;
    }

    inv.mode = TranslationMode::AutoPair;
    inv.text = positional.join(" ");
}

pub fn usage() -> &'static str {
    "Usage:
  t <text>
  t <lang> <text>
  t --tool <tool> <text>
  t --tool <tool> <lang> <text>

Options:
  --tool <codex|claude>  Use a specific Agent CLI
  --setup               Run first-run setup
  --no-wizard           Fail instead of running setup automatically
  --version             Print version
  --help                Show help"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_target_language_arguments() {
        let inv = parse_args(&args(&["--tool", "codex", "ja", "hello"])).unwrap();
        assert_eq!(inv.tool, "codex");
        assert_eq!(inv.mode, TranslationMode::Target);
        assert_eq!(inv.target_lang.unwrap().code, "ja");
        assert_eq!(inv.text, "hello");
    }

    #[test]
    fn keeps_unresolved_first_arg_as_text() {
        let inv = parse_args(&args(&["hello", "world"])).unwrap();
        assert_eq!(inv.mode, TranslationMode::AutoPair);
        assert_eq!(inv.text, "hello world");
    }

    #[test]
    fn usage_keeps_cli_shape() {
        assert!(usage().contains("t <lang> <text>"));
        assert!(usage().contains("--tool <codex|claude>"));
        assert!(usage().contains("--setup"));
    }
}
