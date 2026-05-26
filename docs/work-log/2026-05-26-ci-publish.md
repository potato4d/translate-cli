# CI and Publish

Date: 2026-05-26

## Acceptance Checklist

- Fix the failing CI on `master`.
- Keep the logo work published in the repository.
- Publish a new release through the existing `v*` tag workflow.
- Verify the GitHub release and Homebrew tap artifacts after the workflow completes.

## CI Failure

The `Add logo asset to README` push triggered CI run `26407246363`, which failed in `cargo test --workspace`.

Failing test:

- `tests/cli.rs::translates_through_fake_codex`

Observed CI output:

- Left: the full fake Codex JSON event stream.
- Right: `こんにちは\n`.

Root cause:

- `run_streaming_json` had a race when the Codex process completed quickly.
- If the child process exited before the receiver observed the parsed final message, the function returned the collected stdout stream.
- On macOS local runs the receiver usually won the race; on Linux CI the child exit path won.

## Implementation

- Added a completed-stream parser that extracts the last Codex `agent_message` from collected stdout.
- Used that parser in the successful child-exit path of `run_streaming_json`.
- Added a unit test for extracting the final message from a completed Codex JSON stream.
- Bumped the release version from `0.1.2` to `0.1.3` so a new `v0.1.3` release tag can be published.

## Validation

- `cargo fmt --check`: passed.
- `cargo test --workspace`: passed, `15` tests total.
- `cargo build --release`: passed.
- `./target/release/t --version`: `translate-cli 0.1.3`.
- `cargo run -p xtask -- build-release`: passed and generated the current `darwin-arm64` archive locally.
- `git diff --check`: passed.

Remote CI, release, and Homebrew tap verification will be recorded after the push and release workflow complete.
