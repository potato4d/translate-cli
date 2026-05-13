# Completion audit

Date: 2026-05-13

## Objective

Implement the major MVP features in `docs/APPLICATION_DESIGN.md`, continuously document work in `docs/work-log`, document design decisions in `docs/architecture`, and commit/push completed work units. The goal is complete when all major features are implemented and verified.

## Requirement-to-artifact checklist

| Requirement | Evidence | Status |
|---|---|---|
| Go implementation, binary name `t`, package `translate-cli` | `go.mod`, `cmd/t/main.go`, `internal/cli/command.go`, `.goreleaser.yml` binary `t` | Complete |
| `t <text>` auto-pair translation | `internal/cli/parser.go` auto-pair parsing, `internal/translate/prompt.go`, `internal/cli/parser_test.go` | Complete |
| `t <lang> <text>` target translation | `internal/cli/parser.go`, `internal/locale/language.go`, `internal/cli/parser_test.go`, `internal/locale/language_test.go` | Complete |
| Multiple positional words are joined as text | `internal/cli/parser.go`, `internal/cli/parser_test.go` | Complete |
| stdin input with positional text priority | `internal/cli/command.go`, `internal/cli/command_test.go` fake Claude stdin test | Complete |
| `--tool codex` / `--tool claude` override | `internal/cli/parser.go`, `internal/cli/command.go`, `internal/cli/command_test.go` | Complete |
| Unsupported tool error with available tool hint | `internal/cli/command.go`, exit rendering in `internal/errors/errors.go` | Complete |
| `t --version` | `internal/cli/command.go`, `internal/cli/command_test.go`; verified with `go run ./cmd/t --version` -> `translate-cli 0.1.0` | Complete |
| First-run wizard | `internal/wizard/wizard.go`, `internal/cli/command.go` setup trigger logic, `internal/wizard/wizard_test.go` | Complete |
| Tool detection with recommendation scoring | `internal/agent/detect.go`, adapter `Detect` methods, `internal/wizard/wizard.go` | Complete |
| Config file paths and TOML persistence | `internal/config/paths.go`, `internal/config/config.go`, `internal/config/config_test.go` | Complete |
| Local language detection | `internal/locale/locale.go`, config defaults in `internal/config/config.go` | Complete |
| Language aliases including `ja`, `日本語`, `english`, `fr`, `zh-TW` | `internal/locale/language.go`, `internal/locale/language_test.go` | Complete |
| Adapter responsibility boundary | `internal/agent/adapter.go`, `internal/agent/codex.go`, `internal/agent/claude.go`, `docs/architecture/2026-05-13-mvp-architecture.md` | Complete |
| Codex adapter uses safe `codex exec` flags, stdin prompt, schema, last-message file | `internal/agent/codex.go`, `internal/agent/adapter_test.go`, `testdata/fake-codex/codex` | Complete |
| Claude adapter uses `claude -p`, `--bare`, JSON schema, no persistence, max turns 1, no tools | `internal/agent/claude.go`, `internal/agent/adapter_test.go`, `testdata/fake-claude/claude` | Complete |
| Prompt injection boundary and translation-only prompt | `internal/translate/prompt.go`, prompt snapshots under `internal/translate/__snapshots__` | Complete |
| JSON schema and output normalization to translated text only | `internal/translate/prompt.go`, `internal/output/normalize.go`, `internal/output/normalize_test.go` | Complete |
| Error codes and stderr rendering | `internal/errors/errors.go`, `internal/errors/errors_test.go`, command error paths in `internal/cli/command.go` and `internal/agent/exec.go` | Complete |
| Fake CLI adapter tests without real Codex/Claude | `testdata/fake-codex/codex`, `testdata/fake-claude/claude`, `internal/cli/command_test.go` | Complete |
| Prompt snapshot tests | `internal/translate/prompt_test.go`, `internal/translate/__snapshots__/prompt_auto_pair.snap`, `internal/translate/__snapshots__/prompt_target.snap` | Complete |
| CI test workflow | `.github/workflows/ci.yml` | Complete |
| GitHub Releases artifacts | `.goreleaser.yml`; snapshot release generated all required archive names under `dist/` | Complete |
| Homebrew Formula installs `t` | `.goreleaser.yml`; snapshot generated `dist/homebrew/Formula/translate-cli.rb` with `bin.install "t"` | Complete |
| npm wrapper exposes `t` | `npm/package.json`, `npm/bin/t`, `npm/scripts/postinstall.js`; Node syntax checks passed | Complete |
| README usage and setup documentation | `README.md` | Complete |
| Work log and architecture notes | `docs/work-log/2026-05-13-mvp-implementation.md`, this audit, `docs/architecture/2026-05-13-mvp-architecture.md` | Complete |
| Commit and push completed work units | `9a53c4a Implement translate CLI MVP`, `0f60edc Verify release packaging`, both pushed to `origin/master` | Complete |

## Verification commands

All verification commands were run on 2026-05-13 after implementation:

- `go test ./...` passed.
- `go vet ./...` passed.
- `go build -o /tmp/translate-cli-t ./cmd/t` passed.
- `go run ./cmd/t --version` printed `translate-cli 0.1.0`.
- `node --check npm/bin/t` passed.
- `node --check npm/scripts/postinstall.js` passed.
- `goreleaser check` validated the config and reported only the expected Homebrew Formula deprecation warning.
- `goreleaser release --snapshot --clean --skip=publish` succeeded.
- `tar -tzf dist/t-darwin-arm64.tar.gz` showed `README.md` and `t`.
- `unzip -l dist/t-windows-amd64.zip` showed `README.md` and `t.exe`.
- `git ls-remote origin refs/heads/master` returned `0f60edc87a5c83dfb0a6fb7a3d14e0f12e3350a4`.

## Notes

- `docs/APPLICATION_DESIGN.md` remains untracked because it was provided as the input design document and was not edited by this agent. This follows the instruction to commit only files touched by this agent.
- GoReleaser v2.15.4 deprecates Homebrew Formula publishing in favor of casks, but the design explicitly requires Formula support. The Formula path is retained, and snapshot release verified formula generation.
- The design has a scope tension where npm wrapper is listed in the MVP table and also in v0.2.0. The implementation includes npm wrapper scaffolding so the stricter MVP reading is satisfied.
