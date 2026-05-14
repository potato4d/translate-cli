# Homebrew tap setup

Date: 2026-05-14

## Issue

`brew install potato4d/tap/translate-cli` could not work because the tap repository did not exist. After creating the first source release, Homebrew still failed with a 404 because `potato4d/translate-cli` is private and release assets are not downloadable without GitHub authentication.

## Change

- Created `potato4d/homebrew-tap`.
- Added `Formula/translate-cli.rb`.
- Published Homebrew release assets to the public tap release `translate-cli-v0.1.0`.
- Updated the Formula URLs to use the public tap release assets.
- Changed `scripts/build-release.mjs` so generated Formula URLs default to `potato4d/homebrew-tap` and `translate-cli-v<version>`.

## Verification

- `brew tap potato4d/tap` succeeded.
- `brew install potato4d/tap/translate-cli` installed `translate-cli 0.1.0`.
- `/opt/homebrew/opt/translate-cli/bin/t --version` printed `translate-cli 0.1.0`.
- `brew test potato4d/tap/translate-cli` passed.
