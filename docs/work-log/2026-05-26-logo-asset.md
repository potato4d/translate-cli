# Logo Asset Setup

Date: 2026-05-26

## Acceptance Checklist

- Create a faceted circular icon for the CLI tool with `imagegen`.
- Generate the source on a green chroma-key background, then remove the background locally.
- Save the final transparent logo in the repository.
- Reference the logo from `README.md`.
- Verify the image transparency and record the result.

## Implementation

- Generated a `logo-brand` raster icon with the built-in `imagegen` tool.
- Used a flat green chroma-key background in the image generation prompt.
- Copied the selected source image to `target/tmp/imagegen/translate-cli-logo-chromakey.png` for local processing.
- Removed the chroma-key background with `${CODEX_HOME:-$HOME/.codex}/skills/.system/imagegen/scripts/remove_chroma_key.py`.
- Saved the final transparent asset as `assets/translate-cli-logo.png`.
- Added the logo to the top of `README.md`.

## Validation

- Chroma-key removal command wrote `assets/translate-cli-logo.png`.
- Helper output:
  - Key color: `#04f805`
  - Transparent pixels: `696041/1572516`
  - Partially transparent pixels: `2658/1572516`
- PNG validation with Pillow:
  - Size: `1254x1254`
  - Mode: `RGBA`
  - Corner alpha values: `[0, 0, 0, 0]`
  - Visible greenish pixels after despill: `0`
- `git diff --check`: passed.
- `cargo fmt --check`: passed.
- `cargo test --workspace`: passed, `14` tests total.
- `cargo build --release`: passed.
- `./target/release/t --version`: `translate-cli 0.1.2`.
- `cargo run -p xtask -- build-release`: passed and generated the current `darwin-arm64` release archive under ignored `dist/`.
- The local `xtask build-release` output did not include a generated Homebrew Formula, so there was no Formula Node.js dependency to inspect in this run.

## Completion Audit

- Logo exists in-repository: `assets/translate-cli-logo.png`.
- Logo is a transparent PNG: verified as `RGBA` with transparent corners.
- Source generation followed the requested green-background workflow: `imagegen` generated the green-background source, then the chroma-key helper produced the transparent output.
- README uses the logo: `README.md` references `assets/translate-cli-logo.png`.
