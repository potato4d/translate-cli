# README customization work log

## Objective

README を読んだ人が、`translate-cli` を「すぐ試せる小さな翻訳コマンド」として理解し、インストールから初回翻訳まで進める状態にする。

## Acceptance checklist

- [x] 最初の画面で `t` の価値と使い方が伝わる。
- [x] Homebrew / source install / setup / first translation の流れが迷わず読める。
- [x] `docs/APPLICATION_DESIGN.md` の MVP コマンド仕様から外れた機能を約束しない。
- [x] Codex / Claude adapter の安全寄りの実行方針を簡潔に説明する。
- [x] stdin、明示的な target language、`--tool` の使用例を含める。
- [x] 差分と基本コマンドで README 内容と実装のずれがないことを確認する。

## Implementation notes

- `README.md` を、短い価値提案、Quick Start、利用例、Supported Agent CLIs、Configuration、Command Reference の順に再構成した。
- 既存実装で確認できる範囲に説明を限定し、外部 API 直接利用や API key 管理をしない点を強調した。
- `docs/APPLICATION_DESIGN.md` は未追跡ファイルとして存在していたため、仕様確認のために読むだけにし、編集・stage 対象から除外する。

## Verification

- `cargo run -- --help`: README の Command Reference と実装の `usage()` が一致することを確認。
- `cargo fmt --check`: pass.
- `cargo test --workspace`: pass. Unit tests 11 件、integration tests 4 件が成功。
- `cargo build --release`: pass.
- `./target/release/t --version`: `translate-cli 0.1.3`.
- `cargo run -p xtask -- build-release`: pass. `dist/checksums.txt` と `dist/t-darwin-arm64.tar.gz` を生成。
- `git diff --check`: pass.

## Commit scope note

作業中に今回編集していない tracked files の差分が見えているが、他作業または検証過程の生成差分として扱い、commit 対象には含めない。

- `docs/work-log/2026-05-13-completion-audit.md`
- `src/wizard.rs`
- `tests/cli.rs`

また、未追跡の `docs/APPLICATION_DESIGN.md` と `docs/work-log/2026-05-28-brand-casing.md` も commit 対象外とする。
