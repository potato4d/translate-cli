# Translate CLI

<p align="center">
  <img src="assets/translate-cli-logo.png" alt="Translate CLI logo" width="160">
</p>

<p align="center">
  A one-letter translation command that turns your local Agent CLI into a focused translator.
</p>

```sh
t "こんにちは"
# Hello.

t ja "Ship the smallest useful version first."
# まずは最小限で役に立つバージョンを出荷します。
```

`Translate CLI` installs a native executable named `t`. It does not call hosted model APIs directly, does not store API keys, and does not try to become another translation service account. Instead, it delegates translation to Agent CLIs and local LLM runners you already use, such as Codex CLI, Claude Code, Ollama, or LM Studio, with a constrained prompt that asks for translated text only.

## Why Use It?

- **Fast from the terminal:** translate short messages, issue text, commit notes, README fragments, and stdin without opening a browser.
- **Works with your existing model setup:** use Codex, Claude, Ollama, or LM Studio from the same local login, model, and permission setup you already manage.
- **No translate-cli API secrets:** this tool stores only local preferences like default tool and local language.
- **Built for developer text:** the prompt asks the Agent to preserve markdown, code blocks, URLs, placeholders, product names, and line breaks where appropriate.
- **Small native binary:** the Homebrew install uses the release binary and does not require Node.js.

## Quick Start

Install with Homebrew:

```sh
brew install potato4d/tap/translate-cli
```

Make sure at least one supported tool is installed and available on `PATH`:

- `codex`
- `claude`
- `ollama` with at least one local model
- `lms` from LM Studio with at least one local LLM model

Then run setup once:

```sh
t --setup
```

After that, translate directly:

```sh
t "レビューお願いします"
t ja "Can you take a look at this PR?"
t fr "The release archive is ready."
echo "Translate stdin too" | t ja
```

## How It Feels

When no target language is provided, `t` translates between your configured local language and English.

For example, with `local_lang = "ja"`:

| Command | Result |
|---|---|
| `t "この仕様を確認してください"` | Translates to English |
| `t "Please check this spec"` | Translates to Japanese |
| `t ja "Good morning"` | Translates to Japanese |
| `t --tool claude fr "Good morning"` | Uses Claude and translates to French |
| `t --tool ollama ja "Good morning"` | Uses a local Ollama model |
| `t --tool lmstudio ja "Good morning"` | Uses a local LM Studio model |

The output is intentionally plain, so it composes with other shell tools:

```sh
git log -1 --pretty=%B | t ja
pbpaste | t en
cat docs/notes.md | t ja > /tmp/notes.ja.md
```

## Supported Tools

### Codex

`t` runs `codex exec` in non-interactive mode with a read-only sandbox, approval disabled, color disabled, low reasoning, the Spark model, and JSON event streaming. Short prompts are passed as an argument; larger prompts fall back to stdin.

```sh
t --tool codex "この文章を英語にして"
```

### Claude

`t` runs `claude -p` with `--bare`, JSON output, a JSON schema, no session persistence, one turn, and no tools.

```sh
t --tool claude ja "Summarize this in Japanese."
```

### Ollama

`t` detects installed Ollama models with `ollama ls` and runs `ollama run <model> <prompt>`. Setup recommends Ollama ahead of other first-run choices when it is installed and has a local model available, because it is the most common local LLM runner this CLI can use directly.

```sh
ollama pull gemma3
t --tool ollama ja "Summarize this in Japanese."
```

### LM Studio

`t` detects LM Studio models through `lms ps --json` and `lms ls --llm --json`, then runs `lms chat <model> -p <prompt>`. `--tool lms` is accepted as an alias for `--tool lmstudio`.

```sh
t --tool lmstudio ja "Summarize this in Japanese."
t --tool lms ja "Summarize this in Japanese."
```

## Configuration

On first run, or when you run `t --setup`, the wizard creates a TOML config file and asks for your default Agent CLI and local language.

Config locations:

- macOS: `~/Library/Application Support/translate-cli/config.toml`
- Linux: `~/.config/translate-cli/config.toml`
- Windows: `%AppData%\translate-cli\config.toml`

Example config:

```toml
version = 1
default_tool = "codex"
local_lang = "ja"
timeout_ms = 60000

[tools.codex]
enabled = true

[tools.claude]
enabled = true

[tools.ollama]
enabled = true
model = "gemma3:latest"

[tools.lmstudio]
enabled = true
model = "lmstudio-community/gemma-3-4b-it"
```

Environment overrides:

- `TRANSLATE_CLI_CONFIG`: use a custom config path
- `TRANSLATE_CLI_TOOL`: override the default tool when `--tool` is not supplied
- `TRANSLATE_CLI_OLLAMA_MODEL`: override the Ollama model
- `TRANSLATE_CLI_LMSTUDIO_MODEL`: override the LM Studio model
- `TRANSLATE_CLI_LMS_MODEL`: alias for `TRANSLATE_CLI_LMSTUDIO_MODEL`

## Install From Source

```sh
git clone https://github.com/potato4d/translate-cli.git
cd translate-cli
cargo install --path .
```

The installed binary is `t`.

## Command Reference

```text
t <text>
t <lang> <text>
t --tool <tool> <text>
t --tool <tool> <lang> <text>

Options:
  --tool <codex|claude|ollama|lmstudio>
                        Use a specific Agent CLI or local LLM
  --setup               Run first-run setup
  --no-wizard           Fail instead of running setup automatically
  --version             Print version
  --help                Show help
```

Language names, common aliases, and current ISO 639-1 two-letter codes are accepted. For compatibility, `kr` maps to Korean. Unknown two-letter strings and arbitrary three-or-more-character identifiers are treated as text in positional syntax.

```sh
t ja "Good morning"
t en "おはようございます"
t tl "Good morning"
t zh-TW "Good morning"
t japanese "Good morning"
t 日本語 "Good morning"
t kr "Good morning"
```

## Development

```sh
cargo fmt --check
cargo test --workspace
cargo build --release
./target/release/t --version
cargo run -p xtask -- build-release
```

The Rust test suite uses fake CLIs under `testdata/` and does not call real Codex or Claude processes.

`cargo run -p xtask -- build-release` writes an OS/architecture-specific archive and checksums under `dist/`. The release workflow builds the full macOS, Linux, and Windows archive set and writes the Homebrew formula from those artifacts.
