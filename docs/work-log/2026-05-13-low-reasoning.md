# Low reasoning for Codex translation

Date: 2026-05-13

## Issue

Real `t` usage through Codex felt slow. The local Codex config had `model_reasoning_effort = "xhigh"`, so translation runs inherited a setting intended for heavier coding work.

## Change

- Added `--model gpt-5.3-codex-spark` to the Codex adapter command for faster translation responses.
- Added `-c 'model_reasoning_effort="low"'` to the Codex adapter command before `exec`.
- Added `--ignore-user-config`, `--ignore-rules`, and `--ephemeral` to reduce Codex session/config overhead and keep translation runs stateless.
- Removed `--output-schema` from the Codex fast path and switched Codex extraction to raw final-message text.
- Run Codex subprocesses from `/tmp` instead of the caller's repository worktree to avoid unnecessary repo-context initialization.
- Pass short Codex prompts as positional arguments and disable model-visible permissions/apps/environment/apply_patch instructions for the translation-only fast path.
- Skip Codex login preflight during normal translation runs and run Codex from `/tmp` when available.
- Stream Codex JSON events and return when the final agent message is available instead of waiting for full CLI teardown.
- Tested lighter Codex model candidates; catalog models were slower than Spark, and non-catalog lightweight models were rejected by Codex with a ChatGPT account.
- Updated Codex adapter tests to assert the fast model and low reasoning override.
- Updated `testdata/fake-codex/codex` to fail unless the fast model, low reasoning config, stateless flags, and raw-text prompt are passed.
- Added an architecture note for this adapter runtime decision.

## Verification

- `codex --ask-for-approval never -c 'model_reasoning_effort="low"' exec --help` succeeds.
- After the latest startup-path changes, five Codex-backed `t ja "hello"` runs measured `3.32s`, `2.92s`, `5.46s`, `3.83s`, and `2.42s`. The CLI can return below 3 seconds, but repeated fresh Codex starts still show server-side latency variance.
