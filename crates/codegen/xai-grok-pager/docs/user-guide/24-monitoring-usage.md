# Monitoring Usage

Your Own AI Build has no usage dashboard, no credits, and no subscription to monitor. Model usage is a matter between you and the endpoints you configured - a local server, or a provider billed to your own API key. What the agent gives you is local visibility into what a session is consuming.

---

## In the TUI

### `/context`

Show context window usage and session stats: a categorical breakdown (system prompt, messages, reasoning/overhead, free), plus informational rows for tool definitions, the skills listing, and MCP server announcements with their estimated token cost.

```
/context
```

### `/session-info`

Show session details including model, turn count, and context usage.

```
/session-info
```

Token usage is also reported per turn as you work, and the agent auto-compacts the conversation when the context window fills up (see [Session Management](17-sessions.md)).

---

## In Headless Mode

For scripting and CI, `--output-format json` (and the `end` event of `streaming-json`) carries per-invocation spend fields: `usage` token totals, `num_turns`, and per-model `modelUsage`. See [Headless Mode > Output Formats](14-headless-mode.md#output-formats) for the exact field semantics.

```bash
your-own-ai-build -p "Explain this codebase" --output-format json | jq '.usage'
```

For provider-side spend, use your provider's own console - the agent only ever reports what the endpoint returned for each call.

---

## No Telemetry

The upstream project's telemetry and usage-export code paths (product analytics and the external OpenTelemetry stream) are removed from this build. Nothing about your usage is exported anywhere; sessions and their stats live on disk under `~/.your-own-ai-build/sessions/`.
