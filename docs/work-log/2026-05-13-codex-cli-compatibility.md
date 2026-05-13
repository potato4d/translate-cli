# Codex CLI compatibility fix

Date: 2026-05-13

## Issue

Running:

```sh
t ja "Good morning"
```

failed with:

```text
error: codex failed to run: exit status 2
hint: Run: codex
```

The installed Codex CLI is `codex-cli 0.130.0-alpha.5`. In this version, `--ask-for-approval` is accepted as a top-level Codex option before the `exec` subcommand, but not as an option after `codex exec`.

Invalid for this version:

```sh
codex exec --ask-for-approval never ...
```

Valid for this version:

```sh
codex --ask-for-approval never exec ...
```

## Change

- Moved `--ask-for-approval never` before `exec` in the Codex adapter command.
- Updated `testdata/fake-codex/codex` to validate that global option placement.
- Included stderr details in Agent execution errors so future CLI argument failures expose the underlying tool message.

## Verification

- `codex --ask-for-approval never exec --help` succeeds.
- `codex exec --ask-for-approval never --help` fails with the same class of argument error.
- `go test ./...` passes.
- `go vet ./...` passes.
- fake Codex execution through `go run ./cmd/t ja "hello"` returns `こんにちは`.
