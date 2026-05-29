# Local LLM adapter decision

## Context

Translate CLI originally delegated translation to Agent CLIs, with Codex and Claude as the MVP tools. The local LLM expansion needs to detect common local LLM solutions, guide users toward a runnable setup, and keep the existing setup recommendation biased toward the tool most users are likely to have and use.

## Decision

Ollama and LM Studio are first-class supported tools:

- `ollama` maps to `ollama run <model> <prompt>`.
- `lmstudio` maps to `lms chat <model> -p <prompt>`.
- `lms` is accepted as a CLI alias for `lmstudio`.

Both adapters stay inside the existing local CLI adapter boundary. Translate CLI does not call OpenAI-compatible HTTP endpoints directly, even when a local tool exposes one.

## Detection

Detection separates installed tools from runnable tools:

- Ollama is installed when `ollama` is on `PATH`.
- Ollama is runnable when a model is configured through config/env or detected from `ollama ls`.
- LM Studio is installed when `lms` is on `PATH`.
- LM Studio is runnable when a model is configured through config/env or detected from `lms ps --json`, `lms ls --llm --json`, or text fallbacks.

Tools with `enabled = false` are excluded from detection and setup recommendation. Installed local LLM runners without local models are shown as not ready and are not selectable as the setup default.

## Recommendation

Setup scoring keeps explicit user intent strongest:

- Existing `default_tool` gets a large bonus.
- `TRANSLATE_CLI_TOOL` gets a large bonus.
- Otherwise, popularity weighting is `ollama > lmstudio > codex > claude`.

This makes Ollama the first-run recommendation when it is runnable, including when Codex is also installed. The rationale is that Ollama is the most common CLI-first local LLM runner and is the most direct fit for a local LLM setup flow.

## Model Selection

Config supports per-tool model names:

```toml
[tools.ollama]
enabled = true
model = "gemma3:latest"

[tools.lmstudio]
enabled = true
model = "lmstudio-community/gemma-3-4b-it"
```

Environment overrides:

- `TRANSLATE_CLI_OLLAMA_MODEL`
- `TRANSLATE_CLI_LMSTUDIO_MODEL`
- `TRANSLATE_CLI_LMS_MODEL`

Translate CLI does not auto-pull or auto-download models. If no local model is available, setup guides the user to `ollama pull gemma3` or LM Studio / `lms get`.

## Deferred

`llama.cpp` server, LocalAI, and generic OpenAI-compatible local HTTP endpoints are intentionally deferred. Supporting them would require a separate architecture decision because direct HTTP adapter calls are outside the current "local CLI adapter" boundary.
