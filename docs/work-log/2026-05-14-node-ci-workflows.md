# Node CI workflows

Date: 2026-05-14

## Issue

After the Go implementation was removed, the CI workflow still used `actions/setup-go` with `go-version-file: go.mod`. The latest `master` run failed before tests because `go.mod` no longer exists.

## Change

- Replaced CI setup with `actions/setup-node@v4`.
- CI now runs `npm ci`, `npm test`, `npm run test:binary`, and `npm pack --dry-run`.
- Replaced the tag release workflow's GoReleaser path with the Node/Bun release path.
- Tag releases now run `npm run build:release` and publish the generated archives plus `dist/checksums.txt` through `gh release create`.

## Verification

- Confirmed the failing GitHub Actions log: `The specified go version file at: go.mod does not exist`.
- Ran the CI-equivalent commands locally after moving the npm package to the project root.
