# Codex runtime options

Date: 2026-05-13

## Decision

The Codex adapter always overrides Codex model and reasoning effort for translation runs:

```sh
codex --ask-for-approval never \
  --model gpt-5.3-codex-spark \
  -c 'model_reasoning_effort="low"' \
  -c include_permissions_instructions=false \
  -c include_apps_instructions=false \
  -c include_environment_context=false \
  -c include_apply_patch_tool=false \
  exec \
  --ignore-user-config \
  --ignore-rules \
  --ephemeral \
  --sandbox read-only \
  --json \
  ...
```

## Reasoning

Translation requests should be fast and cheap relative to normal coding-agent work. The user's global Codex config may be tuned for development work, for example `model = "gpt-5.5"` and `model_reasoning_effort = "xhigh"`, but this CLI should not inherit those heavier settings for simple translation.

`gpt-5.3-codex-spark` is selected as the default Codex translation model because the Codex model catalog describes it as an ultra-fast coding model. This keeps the Agent CLI integration intact while reducing latency compared with inheriting a frontier coding model such as `gpt-5.5`.

Within the Codex CLI model catalog available to this account, `gpt-5.3-codex-spark` measured faster for translation than `gpt-5.4-mini`, `gpt-5.3-codex`, `gpt-5.2`, and `codex-auto-review`. Lighter model names outside the catalog, such as `gpt-5-mini` and `gpt-4.1-mini`, were rejected by Codex when using a ChatGPT account, so they are not viable for this adapter path.

The adapter also passes `--ignore-user-config`, `--ignore-rules`, and `--ephemeral` to keep translation runs stateless and avoid loading user/project agent configuration that is useful for coding sessions but unnecessary for translation.

Codex subprocesses run from `/tmp` when available instead of the caller's repository worktree. Translation does not need repository context, and measuring direct Codex runs showed that avoiding the repository workdir reduces unnecessary context initialization and latency variance. Transient output files still live in a private temporary directory.

## Fast output mode

Codex no longer uses `--output-schema` in the fast path. The prompt asks Codex to return only translated text and the adapter accepts that raw final message.

This trades a small amount of structured-output strictness for lower latency. Claude still uses JSON schema because its non-interactive JSON mode is cheap and direct.

For short translations, Codex receives the prompt as a positional argument instead of stdin. The adapter falls back to stdin for larger prompts to avoid command-line argument limits. The Codex fast path also disables model-visible permissions, apps, environment, and apply_patch instructions because translation does not use tools; the subprocess still runs with `--sandbox read-only` and `--ask-for-approval never`.

Regular translation runs skip `codex login status` preflight and let `codex exec` surface authentication failures. This avoids starting extra Codex processes on every invocation.

Codex runs with `--json`, and the Rust runner returns as soon as the final `agent_message` item completes. The translated text is already available at that point, so the CLI does not wait for later Codex teardown.
