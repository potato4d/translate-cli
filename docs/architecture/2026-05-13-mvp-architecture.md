# MVP architecture notes

Date: 2026-05-13

## Scope

The implementation follows `docs/APPLICATION_DESIGN.md` and keeps the CLI thin:

- `cmd/t` contains only process entrypoint wiring.
- `internal/cli` owns argument parsing, stdin priority, setup orchestration, tool selection, and exit rendering.
- `internal/config` owns config path selection and TOML read/write for the MVP schema.
- `internal/locale` owns local language detection and common language alias resolution.
- `internal/translate` owns request types, the JSON schema, and prompt construction.
- `internal/agent` owns Codex/Claude detection, command construction, execution, and output extraction.
- `internal/wizard` owns first-run setup prompts and config persistence.
- `internal/output` normalizes Agent JSON/text output into a final translation string.

## Adapter boundary

The CLI layer resolves the selected tool by ID and then uses the Adapter interface. Codex/Claude-specific command flags stay inside their adapters.

Codex uses:

- `codex exec`
- stdin prompt
- read-only sandbox
- approval disabled
- color disabled
- JSON schema file
- final-message output file

Claude uses:

- `claude -p`
- `--bare`
- JSON output
- JSON schema argument
- no session persistence
- one turn
- empty tools list

## Config format

The MVP keeps a small TOML reader/writer instead of adding a dependency. It supports the schema required by the design:

- `version`
- `default_tool`
- `local_lang`
- `timeout_ms`
- `[tools.codex] enabled`
- `[tools.claude] enabled`

The parser intentionally ignores unknown sections and keys to leave room for future config expansion.

## Testing strategy

Unit tests cover parsing, language aliases, prompt snapshots, config persistence, output normalization, errors, and adapter command construction.

Agent integration tests use `testdata/fake-codex/codex` and `testdata/fake-claude/claude` so CI never calls real Agent CLIs.

## Distribution

GitHub Releases are configured through GoReleaser with the design-specified asset names:

- `t-darwin-arm64.tar.gz`
- `t-darwin-amd64.tar.gz`
- `t-linux-amd64.tar.gz`
- `t-linux-arm64.tar.gz`
- `t-windows-amd64.zip`

The npm wrapper expects those release assets and exposes the command as `t`.
