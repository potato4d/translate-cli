# Low reasoning for Codex translation

Date: 2026-05-13

## Issue

Real `t` usage through Codex felt slow. The local Codex config had `model_reasoning_effort = "xhigh"`, so translation runs inherited a setting intended for heavier coding work.

## Change

- Added `--model gpt-5.3-codex-spark` to the Codex adapter command for faster translation responses.
- Added `-c 'model_reasoning_effort="low"'` to the Codex adapter command before `exec`.
- Added `--ignore-user-config`, `--ignore-rules`, and `--ephemeral` to reduce Codex session/config overhead and keep translation runs stateless.
- Removed `--output-schema` from the Codex fast path and switched Codex extraction to raw final-message text.
- Run Codex subprocesses from a temporary directory instead of the caller's repository worktree to avoid unnecessary repo-context initialization.
- Updated Codex adapter tests to assert the fast model and low reasoning override.
- Updated `testdata/fake-codex/codex` to fail unless the fast model, low reasoning config, stateless flags, and raw-text prompt are passed.
- Added an architecture note for this adapter runtime decision.

## Verification

- `codex --ask-for-approval never -c 'model_reasoning_effort="low"' exec --help` succeeds.
- After moving Codex subprocesses to the adapter temp directory, three Codex-backed `t ja "hello"` runs measured `4.17s`, `3.93s`, and `4.03s`.
