# Authentication

Your Own AI Build has no account system and no sign-in flow. There is nothing to log into: the agent talks directly to the model endpoints you configure in `~/.your-own-ai-build/config.toml`, and the only credentials involved are API keys for providers that require them.

---

## No Sign-In Required

A local inference server - the [Your Own AI](https://github.com/WeAreFlowsta/Your-Own-AI) desktop app on `localhost:11435`, llama.cpp, Ollama, or any other self-hosted OpenAI-compatible endpoint - does not authenticate requests. At most it needs a placeholder `api_key` value:

```toml
# ~/.your-own-ai-build/config.toml
[model.my-local-model]
model = "<model id your server reports>"   # see GET /v1/models on your server
base_url = "http://localhost:11435/v1"     # Your Own AI desktop app default
api_key = "local"                          # placeholder; the server ignores it
api_backend = "chat_completions"
context_window = 32768

[models]
default = "my-local-model"
```

Run `your-own-ai-build` and start working. There is no browser step, no token cache, and no session that can expire.

---

## Provider API Keys

Hosted providers (OpenAI, Anthropic, Together AI, and any other OpenAI- or Anthropic-compatible API) authenticate with your own API key, set per model in `config.toml`. Two fields cover it:

| Field | Meaning |
|-------|---------|
| `api_key` | The key itself, inline in the config file |
| `env_key` | The name of an environment variable (or an array of names) to read the key from |

Prefer `env_key` when you don't want credentials on disk:

```toml
[model.gpt-4o]
model = "gpt-4o"
base_url = "https://api.openai.com/v1"
name = "GPT-4o"
env_key = "OPENAI_API_KEY"
```

Some providers use a custom auth header instead of `Authorization: Bearer`; Anthropic, for example, expects `x-api-key`, which you can pass through `extra_headers`.

See [Custom Models > Credential Resolution](11-custom-models.md#credential-resolution) for the full resolution order, and [Custom Models > Provider Examples](11-custom-models.md#provider-examples) for complete per-provider configs.

---

## Compatibility: `XAI_API_KEY`

The `XAI_API_KEY` environment variable is retained from the upstream project as a global fallback: a model with no `api_key` or `env_key` of its own falls back to it. It is just an environment variable name - the key in it is sent to whatever `base_url` the model defines, not to any hardcoded service. New configs should prefer per-model `api_key`/`env_key`.

---

## Troubleshooting

- **401 Unauthorized / authentication errors** - the provider rejected the key that was resolved for that model. Check the model's `api_key`/`env_key` fields and confirm the environment variable is set in the shell that launches `your-own-ai-build`. The resolution order is documented in [Custom Models > Credential Resolution](11-custom-models.md#credential-resolution).
- **Local server refuses requests** - confirm the server is running and reachable at the configured `base_url` (for the Your Own AI desktop app, `http://localhost:11435/v1`), and that the `model` id matches what `GET /v1/models` reports.
