# Completion audit: TypeScript and Bun distribution

Date: 2026-05-14

## Objective

Remove the Go implementation, keep the CLI behavior on Node.js + TypeScript, and use Bun single-file executables for GitHub Release and Homebrew distribution without regressing translation latency.

## Deliverables

| Requirement | Evidence | Status |
| --- | --- | --- |
| CLI implementation is Node.js + TypeScript | `npm/src/t.ts`, `npm/src/t.test.ts`, `npm/dist/t.js`, `npm/package.json` | Complete |
| Go implementation is removed | No `*.go`, `go.mod`, or `.goreleaser.yml` files remain | Complete |
| npm package exposes `t` directly | `npm/package.json` bin points to `dist/t.js`; `npm pack --dry-run` includes only `dist/t.js` and `package.json` | Complete |
| Bun single-file binary builds locally | `npm run test:binary --prefix npm` builds `npm/dist/t` and prints `translate-cli 0.1.0` | Complete |
| Release archives are Bun binaries | `npm/scripts/build-release.mjs` runs `bun build --compile` for macOS, Linux, and Windows targets | Complete |
| Homebrew does not require Node.js | Generated `dist/homebrew/Formula/translate-cli.rb` installs `bin.install "t"` and has no Node.js dependency | Complete |
| Codex fast path remains optimized | `npm/src/t.ts` uses Spark, low reasoning, read-only sandbox, ignored user config/rules, `/tmp`, prompt arg passing, and JSON event streaming | Complete |
| Bun executable speed is acceptable | Real Codex runs with the Bun executable measured 2.36s, 2.39s, and 3.94s for `ja hello` | Complete |
| Documentation reflects the new distribution model | `README.md`, `AGENTS.md`, `docs/architecture/`, and 2026-05-14 work logs updated | Complete |

## Verification

- `npm test --prefix npm` passed.
- `npm run test:binary --prefix npm` passed.
- `npm run build:release --prefix npm` passed.
- `dist/release/darwin-arm64/t --version` printed `translate-cli 0.1.0`.
- `npm pack --dry-run` from `npm/` showed `dist/t.js` and `package.json`.
- `git diff --check` passed.
- `rg --files -g '*.go' -g 'go.mod' -g '.goreleaser.yml'` returned no files.

## Notes

- `docs/APPLICATION_DESIGN.md` is an untracked local file and was not modified.
- This audit is committed with the implementation. Push status is verified after the commit because the audit file itself is part of the commit.
