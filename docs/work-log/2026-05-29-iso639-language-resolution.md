# ISO 639-1 language resolution

## Objective

言語指定の2文字コードを、未知の2文字全許可ではなく ISO 639-1 の現行コード whitelist として広げる。

## Acceptance checklist

- [x] 現行 ISO 639-1 の2文字コードを基本として `t <lang> <text>` の target language として受け付ける。
- [x] 未知の2文字文字列は target language 扱いしない。
- [x] 任意の3文字以上は positional 構文では target language 扱いしない。
- [x] `kr` は既存挙動に合わせて Korean alias として扱う。
- [x] README と architecture note に解釈境界を記録する。

## Implementation notes

- `src/lang.rs` に ISO 639-1 の現行2文字コード表を追加した。ただし `kr` は既存互換の Korean alias を優先し、Kanuri は `kanuri` で解決できるようにした。
- 2文字コード表は Library of Congress の ISO 639-1 RDF/XML list を参照した。
- Deprecated identifiers は current-code whitelist から除外した。
- `src/cli.rs` に ISO 639-1 コードが target language になることと、未知2文字が text に残ることの parser test を追加した。

## Verification

- `cargo fmt --check`: passed
- `cargo test --workspace`: passed
- `cargo build --release`: passed
- `./target/release/t --version`: `translate-cli 0.1.3`
- `cargo run -p xtask -- build-release`: passed
- `tar -xOf dist/t-darwin-arm64.tar.gz README.md | rg -n "ISO 639-1|kr|tl"`: release README contains the updated language-code documentation
- `rg --files dist | rg -i 'formula|\.rb$'`: no matches; local `xtask` output did not include a Formula artifact
- `git diff --check`: passed
