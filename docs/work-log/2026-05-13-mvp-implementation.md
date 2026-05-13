# MVP implementation work log

Date: 2026-05-13

## Implemented

- Created the Go module `github.com/potato4d/translate-cli`.
- Added the `cmd/t` entrypoint and `translate-cli 0.1.0` version output.
- Implemented CLI parsing for:
  - `t <text>`
  - `t <lang> <text>`
  - `t --tool <tool> <text>`
  - `t --tool <tool> <lang> <text>`
  - `--setup`
  - `--no-wizard`
  - `--version`
- Implemented positional text joining and stdin fallback, with positional text taking priority.
- Implemented common language alias resolution for Japanese, English, French, Traditional Chinese, Simplified Chinese, and other common languages.
- Implemented prompt construction and JSON schema for `translated_text`.
- Implemented config path selection and TOML persistence.
- Implemented first-run wizard with tool detection, recommended tool selection, local language confirmation, config save, and privacy notice.
- Implemented Codex and Claude adapters with safe non-interactive command flags.
- Implemented command execution with timeout handling and output normalization.
- Added fake Codex/Claude CLIs for tests.
- Added unit, adapter, E2E-style, and prompt snapshot tests.
- Added README, GitHub Actions CI/release workflows, GoReleaser config, Homebrew formula generation config, and npm wrapper scaffold.

## Verification

- Installed Go locally with Homebrew after user approval.
- Ran `gofmt` on all Go sources.
- Ran `go test ./...` successfully.
- Ran `go vet ./...` successfully.
- Ran `go build -o /tmp/translate-cli-t ./cmd/t` successfully.
- Ran `go run ./cmd/t --version` successfully and confirmed `translate-cli 0.1.0`.
- Ran `node --check npm/bin/t` successfully.
- Ran `node --check npm/scripts/postinstall.js` successfully.
- Ran `git diff --check` successfully.
- Installed GoReleaser locally and ran `goreleaser release --snapshot --clean --skip=publish` successfully.
- Confirmed the snapshot release generated `t-darwin-arm64.tar.gz`, `t-darwin-amd64.tar.gz`, `t-linux-amd64.tar.gz`, `t-linux-arm64.tar.gz`, and `t-windows-amd64.zip`.
- CI workflow has been added to run `go test ./...` with `actions/setup-go`.

## Notes

- `docs/APPLICATION_DESIGN.md` has internal scope tension around npm wrapper timing. The implementation includes npm wrapper scaffolding to satisfy the MVP table in section 16.
- `--setup` and `--no-wizard` are implemented because setup messages and wizard conditions reference them, even though `--no-wizard` is listed as a future candidate elsewhere.
- The npm wrapper fails clearly if the bundled binary is unavailable instead of recursively invoking itself through `t`.
- `goreleaser check` reports the Homebrew Formula publisher as deprecated in GoReleaser v2.15.4. The Formula publisher is intentionally retained because the application design explicitly requires Homebrew Formula support.
