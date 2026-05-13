# Low reasoning for Codex translation

Date: 2026-05-13

## Issue

Real `t` usage through Codex felt slow. The local Codex config had `model_reasoning_effort = "xhigh"`, so translation runs inherited a setting intended for heavier coding work.

## Change

- Added `-c 'model_reasoning_effort="low"'` to the Codex adapter command before `exec`.
- Updated Codex adapter tests to assert the low reasoning override.
- Updated `testdata/fake-codex/codex` to fail unless the low reasoning config is passed.
- Added an architecture note for this adapter runtime decision.

## Verification

- `codex --ask-for-approval never -c 'model_reasoning_effort="low"' exec --help` succeeds.
