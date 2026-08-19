# Getting Started

Your Own AI Build is a local-first AI agent for your terminal. It runs as a TUI (Terminal User Interface) that understands your codebase, executes shell commands, edits files, searches the web, and manages tasks - with your own models, on your own machine.

You can use it interactively as a full-screen TUI, run it headlessly for scripting and CI/CD, or integrate it into editors via the Agent Client Protocol (ACP).

---

## Installation

Build from source. Requirements:

- **Rust** - the toolchain is pinned by the repository's `rust-toolchain.toml`; `rustup` installs it automatically on first build.
- **protoc** - proto codegen uses `protoc` from `PATH` (or `$PROTOC`). Alternatively install [DotSlash](https://dotslash-cli.com) so the hermetic `bin/protoc` wrapper can download and run it.
- macOS and Linux are supported build hosts; Windows builds are best-effort.

```bash
cargo build -p xai-grok-pager-bin --release   # binary: target/release/your-own-ai-build
```

Put `target/release/your-own-ai-build` on your `PATH`, then verify:

```bash
your-own-ai-build --version
```

There is no auto-updater; to update, pull the latest source and rebuild.

---

## First Launch

Before the first run, tell the agent where your models live. Create `~/.your-own-ai-build/config.toml` pointing at a local OpenAI-compatible inference server (the [Your Own AI](https://github.com/WeAreFlowsta/Your-Own-AI) desktop app, llama.cpp, Ollama, or any self-hosted endpoint):

```toml
[model.my-local-model]
model = "<model id your server reports>"   # see GET /v1/models on your server
base_url = "http://localhost:11435/v1"     # Your Own AI desktop app default
api_key = "local"
api_backend = "chat_completions"
context_window = 32768

[models]
default = "my-local-model"
```

Models need a context window of at least 16k tokens to work well; 32k or more is recommended.

Then start the agent in a project directory:

```bash
your-own-ai-build
```

There is no account and no sign-in step. Hosted providers (OpenAI- or Anthropic-compatible APIs) work too, authenticated with your own API key per model - see [Authentication](02-authentication.md) and [Custom Models](11-custom-models.md).

---

## Basic Interaction

Once launched, the agent presents a full-screen TUI with two main areas:

- **Scrollback** -- the conversation history showing your prompts, the agent's responses, tool calls, file edits, and more.
- **Prompt** -- the input area at the bottom where you type messages.

Type a message and press `Enter` to send it. The agent reads files, runs commands, and edits code as needed. Each tool run streams into the scrollback in real time.

Press `Tab` to move focus between the prompt and the scrollback. While a turn is running, `Esc` cancels it (the exception is fullscreen vim scrollback mode, where mid-turn `Esc` is a no-op; minimal mode cancels even with vim on); `Ctrl+C` cancels once the composer is empty — with a draft, the first press only clears it. Idle, press `Esc` twice within 800ms to clear a non-empty prompt, or (with an empty prompt and conversation messages) to open rewind — see [Keyboard Shortcuts](03-keyboard-shortcuts.md#escape). With the scrollback focused, use the arrow keys to select entries and to collapse or expand them. To navigate with `j`/`k` and fold with `h`/`l` instead, enable Vim mode.

### File References

Use `@` in your prompt to attach files:

```
@src/main.rs              # Attach a file
@src/main.rs:10-50        # Attach lines 10-50
@src/                     # Browse a directory
```

The `@` operator opens a fuzzy file picker. By default it respects `.gitignore` and hides dotfiles. Prefix with `!` to search hidden files:

```
@!.github                 # Search hidden files
@!.env                    # Attach a .env file
```

### Permissions

By default, the agent asks for permission before executing shell commands or editing files. You can approve individually or toggle always-approve mode:

- Press `Ctrl+O` to toggle always-approve mode
- Use the `--yolo` flag at launch: `your-own-ai-build --yolo`
- Type `/always-approve` in the prompt to toggle the mode

---

## Key Concepts

### Sessions

Every conversation is a **session**. Sessions are automatically saved to `~/.your-own-ai-build/sessions/` and can be resumed later. Each session tracks the full conversation history, tool calls, file edits, and task state.

- Start a new session: `Ctrl+N` or `/new`
- Resume a previous session: `/resume` in the TUI, or `--resume <ID>` from the CLI
- Continue the most recent session: `your-own-ai-build -c`

### Scrollback

The scrollback is the main display area. It shows:

- **User prompts** -- your messages, rendered as sticky headers
- **Agent messages** -- the agent's responses with full markdown rendering and syntax highlighting
- **Thinking blocks** -- the model's reasoning process (collapsible)
- **Tool calls** -- file edits (with inline diffs), command executions, search results, and more
- **Task lists** -- TODO items tracking progress

Collapse or expand the selected entry with the `Left`/`Right` arrow keys (or `h`/`l` and `e` in Vim mode). In Vim mode, press `y` to copy its content and `Y` to copy its metadata (for example, the command that ran). Press `Enter` to open it in the fullscreen viewer (in any mode).

### Tools

The agent has built-in tools for:

| Tool | Description |
|------|-------------|
| `read_file` / `search_replace` | Read and edit files with line-precise changes |
| `grep` | Regex search across your codebase (powered by ripgrep) |
| `list_dir` | List directory contents |
| `run_terminal_command` | Execute shell commands |
| `web_search` / `web_fetch` | Search the web and fetch URLs |
| `todo_write` | Create and manage task lists |
| `spawn_subagent` | Spawn parallel subagent sessions |
| `memory_search` | Search cross-session memory |

Tools can be extended with [MCP servers](05-configuration.md#mcp-servers) for integrations like GitHub, databases, and more.

### Slash Commands

Type `/` in the prompt to access commands. These provide quick actions without writing a full prompt:

```
/model my-local-model             # Switch model
/compact                          # Compress conversation history
/always-approve                   # Toggle always-approve mode
/new                              # Start a new session
```

See [Slash Commands](04-slash-commands.md) for the complete reference.

---

## Common Launch Options

```bash
# Launch the interactive TUI and submit an initial prompt as the first turn
your-own-ai-build "fix the failing auth test and run it"

# Initial prompt in a new git worktree. Use --worktree=<name> (with `=`) so the
# prompt isn't swallowed as the worktree name — `your-own-ai-build -w "refactor module X"`
# would treat "refactor module X" as the worktree label, not the prompt.
your-own-ai-build --worktree=feat "refactor module X"

# Base the worktree on a specific branch (e.g. main) instead of the current HEAD:
your-own-ai-build -w --ref main "implement feature from main"


# Start in a specific project directory
your-own-ai-build --cwd ~/projects/my-app

# Add project-specific rules
your-own-ai-build --rules "Always use TypeScript. Prefer functional components."

# Auto-approve all tool executions
your-own-ai-build --yolo

# Use a specific model
your-own-ai-build -m my-local-model

# Resume a previous session
your-own-ai-build --resume <session-id>

# Continue the most recent session
your-own-ai-build -c

# Experimental scrollback-native render mode. Sticky: plain `your-own-ai-build` reopens in
# the mode last chosen via --minimal/--fullscreen (or /minimal//fullscreen).
your-own-ai-build --minimal

# Back to the standard fullscreen TUI (and make it sticky again)
your-own-ai-build --fullscreen

# Headless mode (for scripts)
your-own-ai-build -p "Explain this codebase"
```

---

## Headless Mode

Run the agent non-interactively for scripting, CI/CD, and automation:

```bash
your-own-ai-build -p "Your prompt here"
```

Output formats:

| Format | Flag | Description |
|--------|------|-------------|
| `plain` | (default) | Human-readable text |
| `json` | `--output-format json` | Single JSON object with `text`, `stopReason`, `sessionId`, and `requestId` |
| `streaming-json` | `--output-format streaming-json` | NDJSON event stream for real-time processing |

Example CI/CD usage:

```bash
your-own-ai-build -p "Review changes for bugs" --output-format json --yolo | jq -r '.text'
```

---

## Project Rules (AGENTS.md)

Add per-project instructions by creating an `AGENTS.md` file in your repository. The agent reads these files and injects their contents as a project-instructions message at the start of the conversation:

```
~/.your-own-ai-build/AGENTS.md           # Global rules (apply to all projects)
<repo-root>/AGENTS.md       # Repository-level rules
<cwd>/AGENTS.md             # Directory-level rules (highest priority)
```

Deeper files take precedence. Reading other tools' rule files (such as `CLAUDE.md`) is off by default and opt-in via the `[compat]` section in `config.toml`.

---

## Where to Go Next

| Document | What You Will Learn |
|----------|-------------------|
| [Authentication](02-authentication.md) | Running without sign-in, provider API keys, and credential resolution |
| [Keyboard Shortcuts](03-keyboard-shortcuts.md) | Complete reference for all key bindings |
| [Slash Commands](04-slash-commands.md) | All available `/` commands |
| [Configuration](05-configuration.md) | config.toml, pager.toml, environment variables |
| [Custom Models](11-custom-models.md) | Local servers, Ollama, and any OpenAI- or Anthropic-compatible API |
