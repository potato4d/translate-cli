# Local LLM support work log

## Objective

Add local LLM support by detecting major local LLM tools, guiding users toward a runnable setup, and recommending the most commonly used local LLM solution during setup.

## Acceptance checklist

- [x] Preserve existing Codex and Claude behavior.
- [x] Add `ollama` as a supported tool.
- [x] Add `lmstudio` / `lms` as a supported tool.
- [x] Detect local LLM models before treating local LLM tools as runnable.
- [x] Save and load per-tool local model settings.
- [x] Recommend Ollama during first-run setup when it is runnable and no existing/env override is present.
- [x] Guide users when Ollama or LM Studio is installed but has no local model.
- [x] Avoid auto-pulling or auto-downloading models.
- [x] Keep tests on fake CLIs, not real local tools.

## Implementation notes

- Extended `src/agent.rs` with `ollama` and `lmstudio` adapters.
- Added `RunOptions` so local model selection can come from config/env/detection.
- Added per-tool `model` to `src/config.rs` while preserving existing `enabled` flags.
- Updated setup wizard detection to separate installed tools from runnable tools.
- Updated CLI help, error hints, README, and fake CLI tests.
- Added architecture note: `docs/architecture/2026-05-29-local-llm-adapters.md`.

## Verification

- `cargo fmt --check`: pass.
- `cargo test --workspace`: pass. Unit tests 19, integration tests 10.
- `cargo build --release`: pass.
- `./target/release/t --version`: `translate-cli 0.1.4`.
- `./target/release/t --help`: lists `--tool <codex|claude|ollama|lmstudio>`.
- `cargo run -p xtask -- build-release`: pass. Generated the local `darwin-arm64` release archive.
- `git diff --check`: pass.
- `rg --files dist | sort`: local archive, staged README, staged binary, and checksums are present.
- `rg -n "node|Node|depends_on" dist`: no generated Formula was present; the staged README mentions that Node.js is not required.

## Completion audit

| Requirement | Evidence | Status |
|---|---|---|
| Detect major local LLM tools | `src/agent.rs` detects `ollama` and `lms`; fake CLI integration tests cover both | Complete |
| Guide users to a runnable local LLM setup | Wizard distinguishes installed vs runnable tools; no-model test checks `ollama pull gemma3` guidance | Complete |
| Recommend the most commonly used local LLM setup | Scoring uses `ollama > lmstudio > codex > claude` unless existing config or `TRANSLATE_CLI_TOOL` overrides it; setup tests cover Ollama over LM Studio and Codex | Complete |
| Use local LLM tools for translation | `build_ollama_command` and `build_lmstudio_command`; integration tests for `--tool ollama`, `--tool lmstudio`, and `--tool lms` | Complete |
| Preserve existing Codex / Claude behavior | Existing fake Codex / Claude tests still pass with isolated PATHs | Complete |
| Persist local model settings | `src/config.rs` reads/writes `[tools.ollama].model` and `[tools.lmstudio].model`; setup test verifies saved Ollama model | Complete |
| Avoid real tool calls in tests | Integration tests use `testdata/fake-*` directories and remove `TRANSLATE_CLI_*` overrides | Complete |
| Release build still works | `cargo build --release`, `./target/release/t --version`, and `cargo run -p xtask -- build-release` passed | Complete |
| Push state | Pending at audit authoring time; this work log is intended to be committed and pushed with the implementation | Pending |

## Current caveats

- `llama.cpp` server, LocalAI, and generic OpenAI-compatible local endpoints are not implemented in this pass because they would require direct HTTP adapter behavior outside the current CLI adapter boundary.
- `docs/APPLICATION_DESIGN.md` was present as an untracked file before this work and was read as project context only.
