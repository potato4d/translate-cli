# Homebrew install without Node.js

Date: 2026-05-14

## Issue

After moving the CLI implementation to Node.js + TypeScript, Homebrew still needs to install `t` on machines that do not have Node.js installed.

## Decision

Homebrew and GitHub Releases now use Bun single-file executables generated from `npm/src/t.ts`.

- `npm run build:release --prefix npm` runs `bun build --compile` for macOS, Linux, and Windows release targets.
- Release archives keep the existing names: `t-darwin-amd64.tar.gz`, `t-darwin-arm64.tar.gz`, `t-linux-amd64.tar.gz`, `t-linux-arm64.tar.gz`, and `t-windows-amd64.zip`.
- x64 release targets use Bun baseline builds for wider CPU compatibility.
- The generated Homebrew formula installs `bin.install "t"` from the compiled archive and has no `depends_on "node"` entry.
- The npm package remains a Node.js package and exposes `dist/t.js`.

## Verification

- `npm run build:release --prefix npm` generated all release archives, `dist/checksums.txt`, and `dist/homebrew/Formula/translate-cli.rb`.
- `dist/release/darwin-arm64/t --version` printed `translate-cli 0.1.0`.
- `dist/t-darwin-arm64.tar.gz` contains `t` and `README.md`.
- The generated formula uses platform-specific release archive URLs, `sha256` values, `bin.install "t"`, and `system "#{bin}/t", "--version"` in the test block. It does not declare a Node.js dependency.
