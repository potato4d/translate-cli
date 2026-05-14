# Node.js TypeScript port

Date: 2026-05-14

## Issue

The npm package previously installed a thin wrapper that looked for a bundled Go binary. The goal was to check whether a Node.js + TypeScript implementation could keep the same Codex CLI startup performance while preserving the CLI UX and keeping dependencies minimal.

## Change

- Replaced the npm Go-binary wrapper with a TypeScript implementation compiled to `dist/t.js`.
- Kept runtime dependencies at zero. The package uses TypeScript, Node types, and Bun only as dev dependencies.
- Preserved CLI usage for `t <text>`, `t <lang> <text>`, `--tool`, `--setup`, `--no-wizard`, `--version`, and stdin input.
- Ported config loading/saving, language resolution, first-run setup, Codex and Claude detection, prompt building, output normalization, and Agent CLI execution.
- Matched the optimized Codex path: Spark model, low reasoning, read-only sandbox, approval disabled, ignored user config/rules, short prompt argument passing, `/tmp` working directory, and JSON event streaming with early return on the final agent message.
- Removed the npm `postinstall` release-asset downloader because the npm package now ships the Node CLI directly.

Follow-up: the repository later removed the Go implementation entirely, moved the npm package to the project root, and switched GitHub Release/Homebrew artifacts to Bun single-file executables generated from the same TypeScript source. The npm package still ships the Node.js `dist/t.js` entrypoint.

## Verification

- `npm test` passes against the existing fake Codex and Claude CLIs.
- `node dist/t.js ja hello` works against real Codex.
- In direct comparison during this investigation, Node.js fresh Codex starts measured `8.77s`, `5.55s`, `5.31s`, `15.42s`, and `14.98s`, while Go fresh Codex starts measured `12.74s`, `10.72s`, `13.67s`, `8.88s`, and `14.61s`. Both were dominated by Codex/server latency at measurement time rather than local CLI overhead.
- Fake CLI tests complete in hundreds of milliseconds, so the Node.js wrapper overhead is not material relative to real Codex startup time.
