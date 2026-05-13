# translate CLI

`translate CLI` is a small command named `t` that delegates translation to local Agent CLIs such as Codex CLI and Claude Code.

The CLI does not call model APIs directly and does not store API keys. It builds a constrained translation prompt, invokes the selected local Agent CLI in non-interactive mode, and prints only the translated text.

## Usage

```sh
t "こんにちは"
t ja "Good morning"
t --tool codex "こんにちは"
t --tool claude fr "Good morning"
echo "こんにちは" | t
echo "Good morning" | t ja
```

When the target language is omitted, `t` asks the Agent to translate between the configured local language and English. For example, with `local_lang = "ja"`, primarily English text is translated into Japanese, and other text is translated into English.

## Install

From source:

```sh
go install github.com/potato4d/translate-cli/cmd/t@latest
```

Homebrew and npm packaging are scaffolded for releases:

```sh
brew install potato4d/tap/translate-cli
npm i -g @potato4d/translate-cli
```

## Setup

On first run, `t` creates a TOML config file and asks which Agent CLI to use by default.

```sh
t --setup
```

Config locations:

- macOS: `~/Library/Application Support/translate-cli/config.toml`
- Linux: `~/.config/translate-cli/config.toml`
- Windows: `%AppData%\translate-cli\config.toml`

Example:

```toml
version = 1
default_tool = "codex"
local_lang = "ja"
timeout_ms = 60000

[tools.codex]
enabled = true

[tools.claude]
enabled = true
```

For tests or custom automation, set `TRANSLATE_CLI_CONFIG` to override the config path. `TRANSLATE_CLI_TOOL` overrides the default tool when `--tool` is not supplied.
The npm wrapper also supports `TRANSLATE_CLI_BIN` as an explicit path to a local `t` binary when a bundled release asset is unavailable.

## Supported Agent CLIs

### Codex

`t` uses `codex exec` with a read-only sandbox, approval disabled, color disabled, JSON schema output, and a final-message output file.

### Claude

`t` uses `claude -p` with `--bare`, JSON output, JSON schema, no session persistence, one turn, and no tools.

## Development

```sh
go test ./...
go run ./cmd/t --version
```

The test suite uses fake CLIs under `testdata/` and does not call real Codex or Claude processes.
