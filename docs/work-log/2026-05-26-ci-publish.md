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

## Remote Verification

- Commit `f7fb37d` was pushed to `master`.
- CI run `26425695327` completed successfully for commit `f7fb37d`.
- Tag `v0.1.3` was pushed to `origin`.
- Release workflow run `26425723814` completed successfully for tag `v0.1.3`.
- Main repo release `v0.1.3` was published with all required assets:
  - `t-darwin-amd64.tar.gz`
  - `t-darwin-arm64.tar.gz`
  - `t-linux-amd64.tar.gz`
  - `t-linux-arm64.tar.gz`
  - `t-windows-amd64.zip`
  - `checksums.txt`
- The release workflow skipped Homebrew tap steps because `HOMEBREW_TAP_TOKEN` was not present in Actions.
- Published tap release `translate-cli-v0.1.3` manually in `potato4d/homebrew-tap` with the same six assets.
- Generated `dist/homebrew/Formula/translate-cli.rb` from the release assets and pushed tap commit `0b98f60`.
- Verified the remote Formula:
  - `version "0.1.3"`
  - URLs point to `translate-cli-v0.1.3`
  - no `node`, `npm`, or `depends_on` entries.
- `brew update` reported `potato4d/tap` updated.
- `brew info potato4d/tap/translate-cli` reported `0.1.2 -> stable 0.1.3`.
- `brew upgrade potato4d/tap/translate-cli` upgraded the local install from `0.1.2` to `0.1.3`.
- `/opt/homebrew/opt/translate-cli/bin/t --version`: `translate-cli 0.1.3`.

## Completion Audit

- CI is passing on the published source commit: run `26425695327` succeeded for `f7fb37d`.
- CLI release is public: GitHub release `v0.1.3` exists and is the latest main repo release.
- Release artifacts cover all configured platforms from `.github/workflows/release.yml`.
- Homebrew distribution is public: tap release `translate-cli-v0.1.3` exists, and the tap Formula points at its assets.
- Formula remains native-binary only: no Node.js dependency is present.
- End-user install path works: Homebrew upgraded and the installed binary prints `translate-cli 0.1.3`.
