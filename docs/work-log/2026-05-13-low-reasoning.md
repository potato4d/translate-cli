# Low reasoning for Codex translation

Date: 2026-05-13

## Issue

Real `t` usage through Codex felt slow. The local Codex config had `model_reasoning_effort = "xhigh"`, so translation runs inherited a setting intended for heavier coding work.

## Change

- Added `--model gpt-5.3-codex-spark` to the Codex adapter command for faster translation responses.
- Added `-c 'model_reasoning_effort="low"'` to the Codex adapter command before `exec`.
- Updated Codex adapter tests to assert the fast model and low reasoning override.
- Updated `testdata/fake-codex/codex` to fail unless the fast model and low reasoning config are passed.
- Added an architecture note for this adapter runtime decision.

## Verification

- `codex --ask-for-approval never -c 'model_reasoning_effort="low"' exec --help` succeeds.
