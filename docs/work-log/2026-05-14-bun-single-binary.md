# Bun single-binary switch

Date: 2026-05-14

## Goal

Check whether Bun single-file executables can preserve the current translation latency, then remove the Go implementation if the result is acceptable.

## Investigation

Bun's official executable docs describe `bun build --compile` for TypeScript/JavaScript entrypoints and `--target` for cross-compiling to macOS, Linux, and Windows targets.

The first local compile succeeded, but the generated executable failed because the direct-execution guard called `realpathSync` on Bun's virtual `/$bunfs/...` entry path. The guard now compares the invoked path before falling back to `realpathSync`, which works for Node.js, symlinked npm bins, and Bun executables.

## Performance

The Bun executable ran real Codex translations successfully:

| Run | Command | Result | Elapsed |
| --- | --- | --- | --- |
| 1 | `npm/dist/t-bun ja hello` | `こんにちは` | 2.36s |
| 2 | `npm/dist/t-bun ja hello` | `こんにちは` | 2.39s |
| 3 | `npm/dist/t-bun ja hello` | `こんにちは` | 3.94s |

This is within the expected Codex/server latency envelope and does not regress the previous Go or Node.js measurements.

## Change

- Added Bun as a dev dependency.
- Added `npm run build:binary --prefix npm` for a local compiled executable.
- Added `npm run build:release --prefix npm` to build release archives, checksums, and a Homebrew formula.
- Removed Go source files and `.goreleaser.yml`.
- Kept npm distribution on `dist/t.js` so npm users only need Node.js 20 or newer.
- Kept Homebrew distribution on compiled `t` executables so Homebrew users do not need Node.js.

## Verification

- `npm test --prefix npm` passed.
- `npm run test:binary --prefix npm` passed.
- `npm run build:release --prefix npm` generated all release artifacts.
- `dist/release/darwin-arm64/t --version` printed `translate-cli 0.1.0`.
