// Minimal ACP client probe: spawn the agent over stdio, initialize, open a
// session, send one prompt, and print every JSON-RPC line both ways.
//
// Usage: node tools/acp-probe.mjs [binary] [cwd] [prompt]
//   binary  path to the agent binary (default: target/debug/your-own-ai-build)
//   cwd     session working directory (default: a temp dir)
//   prompt  prompt text (default: 'Reply with exactly: ACP path works')
//
// Node >= 18, no dependencies. Auto-approves permission requests (probe only -
// never point it at a directory you care about with a model you don't trust).
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const BIN = process.argv[2] || "target/debug/your-own-ai-build";
const CWD = process.argv[3] || mkdtempSync(join(tmpdir(), "acp-probe-"));
const PROMPT = process.argv[4] || "Reply with exactly: ACP path works";

const child = spawn(BIN, ["agent", "stdio"], { stdio: ["pipe", "pipe", "pipe"] });

let id = 0;
const send = (method, params) => {
  const msg = { jsonrpc: "2.0", id: ++id, method, params };
  console.log(">>", JSON.stringify(msg));
  child.stdin.write(JSON.stringify(msg) + "\n");
  return id;
};

let buf = "";
child.stdout.on("data", (d) => {
  buf += d.toString();
  let nl;
  while ((nl = buf.indexOf("\n")) >= 0) {
    const line = buf.slice(0, nl);
    buf = buf.slice(nl + 1);
    if (!line.trim()) continue;
    console.log("<<", line);
    try {
      handle(JSON.parse(line));
    } catch {
      /* non-JSON line, already printed */
    }
  }
});
child.stderr.on("data", (d) => console.error("[stderr]", d.toString().trimEnd()));
child.on("exit", (c, s) => {
  console.log(`[agent exited code=${c} signal=${s}]`);
  process.exit(0);
});

let sessionId = null;
function handle(msg) {
  if (msg.method === "session/request_permission" && msg.id !== undefined) {
    const opt = msg.params?.options?.[0];
    const resp = {
      jsonrpc: "2.0",
      id: msg.id,
      result: { outcome: { outcome: "selected", optionId: opt?.optionId } },
    };
    console.log(">>", JSON.stringify(resp));
    child.stdin.write(JSON.stringify(resp) + "\n");
    return;
  }
  if (msg.id === 1 && msg.result) {
    send("session/new", { cwd: CWD, mcpServers: [] });
  } else if (msg.id === 2 && msg.result) {
    sessionId = msg.result.sessionId;
    console.log("[session opened:", sessionId, "]");
    send("session/prompt", { sessionId, prompt: [{ type: "text", text: PROMPT }] });
  } else if (msg.id === 3) {
    console.log("[prompt turn finished:", JSON.stringify(msg.result ?? msg.error), "]");
    child.kill("SIGTERM");
  }
}

send("initialize", {
  protocolVersion: 1,
  clientCapabilities: { fs: { readTextFile: false, writeTextFile: false }, terminal: false },
});

setTimeout(() => {
  console.log("[timeout, killing agent]");
  child.kill("SIGKILL");
  process.exit(1);
}, 120000);
