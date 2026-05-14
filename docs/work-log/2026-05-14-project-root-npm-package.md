# Project-root npm package

Date: 2026-05-14

## Issue

The Node.js package lived under `npm/`, but the repository no longer has a separate Go implementation. Keeping the npm package at the project root makes local development, package publishing, and CI simpler.

Generated minified JavaScript artifacts also need to be ignored so build outputs do not appear as untracked source changes.

## Change

- Moved `package.json`, `package-lock.json`, `tsconfig.json`, `src/`, and `scripts/` to the project root.
- Kept generated JavaScript under root `dist/`.
- Added `prepack` so `npm pack` and publish flows build `dist/t.js` before packaging.
- Updated tests and release script paths for the root package layout.
- Added `*.min.js` and `*.min.js.map` to `.gitignore`.

## Verification

- `npm ci` passed.
- `npm test` passed.
- `node dist/t.js --version` printed `translate-cli 0.1.0`.
- `npm run test:binary` passed.
- `npm run build:release` passed.
- `npm pack --dry-run` included `dist/t.js` and `package.json`.
