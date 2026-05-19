# Rust Homebrew release

Date: 2026-05-19

## Objective

Publish the Rust implementation so `brew install potato4d/tap/translate-cli` installs the Rust-built `t` binary.

## Release plan

- Existing public release and tap release already use `v0.1.0`.
- Do not rewrite the existing `v0.1.0` tag.
- Bump the Rust package to `0.1.1`.
- Push `v0.1.1` and let `.github/workflows/release.yml` build the five platform archives.
- Verify the GitHub release, Homebrew tap release, tap Formula update, and local `brew install` behavior.

## Acceptance checklist

- `cargo fmt --check` passes.
- `cargo test --workspace` passes.
- `cargo build --release` passes.
- `./target/release/t --version` prints `translate-cli 0.1.1`.
- `cargo run -p xtask -- build-release` generates the current host release archive and checksum.
- `v0.1.1` tag is pushed to `origin`.
- GitHub Actions Release workflow completes successfully for `v0.1.1`.
- Main repo release `v0.1.1` has all five archives plus `checksums.txt`.
- Homebrew tap release `translate-cli-v0.1.1` has all five archives plus `checksums.txt`.
- `potato4d/homebrew-tap` Formula points at `translate-cli-v0.1.1`.
- `brew install potato4d/tap/translate-cli` installs a binary whose `t --version` prints `translate-cli 0.1.1`.
