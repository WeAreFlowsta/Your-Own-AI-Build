# Your Own AI Build

An offline-first AI agent for your terminal. It runs as a full-screen TUI that understands your codebase, edits files, executes shell commands, and manages long-running tasks - interactively, headlessly for scripting/CI, or embedded in editors and apps via the Agent Client Protocol (ACP).

Your projects, your models, your machine - nothing leaves unless you say so.

- **Local-first by default.** Out of the box it talks to a local OpenAI-compatible inference server (the [Your Own AI](https://github.com/WeAreFlowsta/Your-Own-AI) desktop app, llama.cpp, Ollama, or any self-hosted endpoint). No account, no sign-in, no cloud round trip.
- **No phone-home, verifiably.** The cloud upload, telemetry, and auto-update code paths from the upstream project have been removed from the source, not just switched off. Read the diff.
- **Bring any model.** Local servers, or any OpenAI- or Anthropic-compatible API, configured per model in `config.toml`. Coding is the first surface, not the ceiling.
- **Your data stays put.** Config, sessions, and skills live in `~/.your-own-ai-build/`. Scanning other tools' config trees (Claude, Cursor, Codex) is off by default and opt-in per surface via the `[compat]` config section.

## Quickstart

Build from source, then point the agent at your local inference server.

Requirements:

- **Rust** - the toolchain is pinned by [`rust-toolchain.toml`](rust-toolchain.toml); `rustup` installs it automatically on first build.
- **protoc** - proto codegen uses `protoc` from `PATH` (or `$PROTOC`). Alternatively install [DotSlash](https://dotslash-cli.com) so the hermetic [`bin/protoc`](bin/protoc) wrapper can download and run it.
- macOS and Linux are supported build hosts; Windows builds are best-effort.

```sh
cargo build -p xai-grok-pager-bin --release   # binary: target/release/your-own-ai-build
```

Configure your model in `~/.your-own-ai-build/config.toml`:

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

Then run `your-own-ai-build` in a project directory. No sign-in step: with a local or custom endpoint there is nothing to authenticate against unless your provider requires an API key.

For editor or app integration, `your-own-ai-build agent stdio` speaks ACP over stdio.

## Working with the Your Own AI desktop app

The [Your Own AI](https://github.com/WeAreFlowsta/Your-Own-AI) desktop app serves an OpenAI-compatible endpoint on `localhost:11435` for both its offline and online models. The agent's model list auto-populates from that server, and each of your AIs appears as a selectable model (use the `<name>:agent` variant for tool-calling work).

## Documentation

The user guide ships with the pager crate: [`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/) - getting started, keyboard shortcuts, slash commands, configuration, custom models, theming, MCP servers, skills, plugins, hooks, headless mode, sandboxing, and more.

## Repository layout

| Path | Contents |
|------|----------|
| `crates/codegen/xai-grok-pager-bin` | Composition-root package; builds the `your-own-ai-build` binary |
| `crates/codegen/xai-grok-pager` | The TUI: scrollback, prompt, modals, rendering |
| `crates/codegen/xai-grok-shell` | Agent runtime + leader/stdio/headless entry points |
| `crates/codegen/xai-grok-tools` | Tool implementations (terminal, file edit, search, ...) |
| `crates/codegen/xai-grok-workspace` | Host filesystem, VCS, execution, checkpoints |
| `crates/codegen/...` | The rest of the CLI crate closure (config, MCP, markdown, sandbox, ...) |
| `crates/common/`, `crates/build/`, `prod/mc/` | Small shared leaf crates pulled in by the closure |
| `third_party/` | Vendored upstream source (Mermaid diagram stack) |

Internal crate names keep the upstream `xai-grok-*` prefix so that periodic merges from upstream stay tractable.

> [!IMPORTANT]
> The root `Cargo.toml` (workspace members, dependency versions, lints,
> profiles) is **generated** upstream - treat it as read-only. Prefer editing
> per-crate `Cargo.toml` files.

## Development

```sh
cargo check -p <crate>        # always target specific crates; full-workspace builds are slow
cargo test -p xai-grok-config # per-crate tests
cargo clippy -p <crate>       # lint config: clippy.toml at the repo root
cargo fmt --all               # rustfmt.toml at the repo root
```

## License and attribution

First-party code in this repository is licensed under the **Apache License, Version 2.0** - see [`LICENSE`](LICENSE).

Your Own AI Build is a fork of [Grok Build](https://github.com/xai-org/grok-build) by xAI, used under the Apache License 2.0. This fork's principal modifications: rebranding, local-first default endpoints, removal of cloud upload/telemetry/auto-update code, and opt-in (rather than automatic) discovery of other tools' configuration. This repository periodically merges upstream's published code.

Third-party and vendored code remains under its original licenses. See:

- [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES) - crates.io / git dependencies, bundled UI themes, and **in-tree source ports** (including openai/codex and sst/opencode tool implementations)
- [`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md) - crate-local notice for the codex and opencode ports (license texts + Apache section 4(b) change notice)
- [`third_party/NOTICE`](third_party/NOTICE) - vendored Mermaid-stack index
