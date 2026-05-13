# Codex runtime options

Date: 2026-05-13

## Decision

The Codex adapter always overrides Codex model and reasoning effort for translation runs:

```sh
codex --ask-for-approval never \
  --model gpt-5.3-codex-spark \
  -c 'model_reasoning_effort="low"' \
  exec \
  --ignore-user-config \
  --ignore-rules \
  --ephemeral \
  ...
```

## Reasoning

Translation requests should be fast and cheap relative to normal coding-agent work. The user's global Codex config may be tuned for development work, for example `model = "gpt-5.5"` and `model_reasoning_effort = "xhigh"`, but this CLI should not inherit those heavier settings for simple translation.

`gpt-5.3-codex-spark` is selected as the default Codex translation model because the Codex model catalog describes it as an ultra-fast coding model. This keeps the Agent CLI integration intact while reducing latency compared with inheriting a frontier coding model such as `gpt-5.5`.

The adapter also passes `--ignore-user-config`, `--ignore-rules`, and `--ephemeral` to keep translation runs stateless and avoid loading user/project agent configuration that is useful for coding sessions but unnecessary for translation.
