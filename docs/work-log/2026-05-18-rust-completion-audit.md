# Completion audit: Rust implementation

Date: 2026-05-18

## Objective

`translate-cli` のコードを全て Rust 化する。

## Audit

| Requirement | Evidence | Status |
| --- | --- | --- |
| CLI implementation is Rust | `Cargo.toml`, `src/*.rs`, `tests/cli.rs`; `cargo test --workspace` passed | Complete |
| Public CLI shape is preserved | `src/cli.rs` unit tests; `tests/cli.rs` integration tests for version, fake Codex, fake Claude stdin, and setup | Complete |
| Config schema remains compatible | `src/config.rs`; integration tests write existing TOML schema and translate successfully | Complete |
| Codex adapter keeps the optimized safe path | `src/agent.rs` command construction test checks JSON, ignored config/rules, Spark model; `testdata/fake-codex/codex` validates sandbox, approval, low reasoning, read-only, ignored rules/config, no permissions/apps/environment/apply_patch, and JSON streaming | Complete |
| Claude adapter keeps non-interactive safe flags | `testdata/fake-claude/claude` validates `--bare`, JSON output, schema, no session persistence, one turn, and empty tools; integration test passes | Complete |
| Output normalization is ported | `src/output.rs` tests cover structured JSON, embedded JSON, and raw text fallback | Complete |
| Real Agent CLIs are not called in tests | `tests/cli.rs` sets PATH to `testdata/fake-codex` / `testdata/fake-claude`; `cargo test` passes | Complete |
| TypeScript / Node / Bun source is removed | `src/t.ts`, `src/t.test.ts`, `scripts/build-release.mjs`, `package.json`, `package-lock.json`, and `tsconfig.json` are deleted; repository source search excluding ignored build dirs finds no TS/MJS/package files | Complete |
| Release packaging is Rust-based | `xtask/src/main.rs`; `cargo run -p xtask -- build-release` generated `dist/t-darwin-arm64.tar.gz` and `dist/checksums.txt` on the local arm64 macOS host | Complete |
| GitHub Release archive names are preserved | `.github/workflows/release.yml` matrix builds the five existing archive names | Complete |
| Homebrew Formula has no Node.js dependency | `xtask/src/main.rs` formula template installs `t` and does not declare Node.js; formula is generated when the full release archive set is present | Complete |
| Documentation reflects Rust state | `README.md`, `AGENTS.md`, `docs/architecture/2026-05-13-mvp-architecture.md`, `docs/architecture/2026-05-13-codex-runtime-options.md`, and this work log | Complete |

## Commands

- `cargo fmt --check`
- `cargo test --workspace`
- `cargo build --release`
- `./target/release/t --version`
- `cargo run -p xtask -- build-release`
- `find . -path './.git' -prune -o -path './target' -prune -o -path './dist' -prune -o -path './node_modules' -prune -o \( -name '*.ts' -o -name '*.mjs' -o -name 'package.json' -o -name 'package-lock.json' -o -name 'tsconfig.json' \) -print`
- `git diff --check`

## Remaining risk

- The full five-platform release matrix is represented in GitHub Actions but was not executed locally. Local verification covered the current host archive; the workflow is responsible for aggregating all platform archives and generating the final Homebrew Formula.
- The repository still contains historical docs that mention previous Go, Node.js, TypeScript, npm, and Bun implementations. Those are retained as history and are not active source code.
