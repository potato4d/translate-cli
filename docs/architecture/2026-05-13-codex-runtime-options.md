# Codex runtime options

Date: 2026-05-13

## Decision

The Codex adapter always overrides Codex reasoning effort to `low` for translation runs:

```sh
codex --ask-for-approval never -c 'model_reasoning_effort="low"' exec ...
```

## Reasoning

Translation requests should be fast and cheap relative to normal coding-agent work. The user's global Codex config may be tuned for development work, for example `model_reasoning_effort = "xhigh"`, but this CLI should not inherit that heavier setting for simple translation.

The adapter does not override the model itself. It only sets `model_reasoning_effort` so the user's selected Codex model/provider remains intact while translation uses low reasoning.
