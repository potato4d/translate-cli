# Rust Homebrew release

Date: 2026-05-19

## Objective

Publish the Rust implementation so `brew install potato4d/tap/translate-cli` installs the Rust-built `t` binary.

## Release plan

- Existing public release and tap release already use `v0.1.0`.
- Do not rewrite the existing `v0.1.0` tag.
- Bump the Rust package to `0.1.2`.
- Push `v0.1.2` and let `.github/workflows/release.yml` build the five platform archives.
- Verify the GitHub release, Homebrew tap release, tap Formula update, and local `brew install` behavior.

## Acceptance checklist

- `cargo fmt --check` passes.
- `cargo test --workspace` passes.
- `cargo build --release` passes.
- `./target/release/t --version` prints `translate-cli 0.1.2`.
- `cargo run -p xtask -- build-release` generates the current host release archive and checksum.
- `v0.1.2` tag is pushed to `origin`.
- GitHub Actions Release workflow completes successfully for `v0.1.2`.
- Main repo release `v0.1.2` has all five archives plus `checksums.txt`.
- Homebrew tap release `translate-cli-v0.1.2` has all five archives plus `checksums.txt`.
- `potato4d/homebrew-tap` Formula points at `translate-cli-v0.1.2`.
- `brew install potato4d/tap/translate-cli` installs a binary whose `t --version` prints `translate-cli 0.1.2`.

## Adjustment

The first `v0.1.1` release attempt was cancelled because the `macos-13` amd64 runner did not start after several minutes while all other platform builds had finished. The workflow was changed to build `darwin-amd64` on `macos-14` by installing the `x86_64-apple-darwin` Rust target before packaging.

## Verification

- `cargo fmt --check` passed.
- `cargo test --workspace` passed.
- `cargo build --release` passed.
- `./target/release/t --version` printed `translate-cli 0.1.2`.
- `cargo run -p xtask -- build-release` generated the local `t-darwin-arm64.tar.gz` archive and checksum.
- Commit `911ad09` was pushed to `origin/master`.
- Tag `v0.1.2` was pushed to `origin`.
- GitHub Actions Release run `26106417532` completed successfully.
- Main repo release `v0.1.2` was published with `checksums.txt` and all five archives.
- The tap release `translate-cli-v0.1.2` was created with `checksums.txt` and all five archives.
- Homebrew tap commit `4aad74f` updated `Formula/translate-cli.rb` to version `0.1.2`.
- `brew update` refreshed `potato4d/tap`.
- `brew info potato4d/tap/translate-cli` showed stable `0.1.2`.
- `brew upgrade potato4d/tap/translate-cli` upgraded the local install from `0.1.0` to `0.1.2`.
- `/opt/homebrew/opt/translate-cli/bin/t --version` printed `translate-cli 0.1.2`.
- `brew test potato4d/tap/translate-cli` passed.
- `brew install potato4d/tap/translate-cli` resolved successfully and reported `0.1.2` already installed and up-to-date.

## Notes

- Homebrew reported that `t` is shadowed by `/Users/potato4d/.volta/bin/t` in the local shell PATH. The installed Homebrew binary was therefore verified by direct path: `/opt/homebrew/opt/translate-cli/bin/t`.
- The release workflow skipped tap publication because `HOMEBREW_TAP_TOKEN` was not available to the run, so the tap release and Formula commit were completed manually with the refreshed local `gh` token.
