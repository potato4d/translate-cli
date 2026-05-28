# Brand casing replacement

## Objective

小文字始まりの製品名表記を、すべて `Translate CLI` に統一する。

## Acceptance checklist

- [x] 実行時に表示される初回セットアップ文言を `Translate CLI` にする。
- [x] README と設計/作業ログ内の製品名表記を `Translate CLI` にする。
- [x] 表示文言を検証するテスト期待値を更新する。
- [x] 現在のワークツリーと生成済み release README に旧表記が残っていないことを確認する。

## Changes

- `src/wizard.rs` の初回セットアップ表示を `Translate CLI` に更新。
- `tests/cli.rs` の setup 出力アサーションを更新。
- `README.md` は最新 HEAD の README customization 後の内容で `Translate CLI` に統一済みであることを確認。
- 既存 completion audit 内の製品名表記を更新。
- 未追跡の `docs/APPLICATION_DESIGN.md` と ignore 対象の `dist/` 内 README も現在ワークツリー上では同じ表記に更新。

## Verification

- `rg -n -uuu --glob '!target/**' --glob '!.git/**' --glob '!dist/**/*.tar.gz' '[t]ranslate CLI'`: no matches
- `tar -xOf dist/t-darwin-arm64.tar.gz README.md | rg -n '[t]ranslate CLI|Translate CLI'`: archive README contains only `Translate CLI` matches
- `cargo fmt --check`: passed
- `cargo test --workspace`: passed
- `cargo build --release`: passed
- `./target/release/t --version`: `translate-cli 0.1.3`
- `cargo run -p xtask -- build-release`: passed
- `rg --files dist | rg -i 'formula|\.rb$'`: no matches; local `xtask` output did not include a Formula artifact
- `git diff --check`: passed
