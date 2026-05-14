import { spawn } from "node:child_process";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import assert from "node:assert/strict";
import test from "node:test";

import { parseArgs, usage } from "./t.js";

test("prints version", async () => {
  const result = await runCLI(["--version"]);
  assert.equal(result.stdout, "translate-cli 0.1.0\n");
  assert.equal(result.stderr, "");
});

test("parses target language arguments", () => {
  const inv = parseArgs(["--tool", "codex", "ja", "hello"]);
  assert.equal(inv.tool, "codex");
  assert.equal(inv.mode, "target");
  assert.equal(inv.targetLang?.code, "ja");
  assert.equal(inv.text, "hello");
});

test("usage keeps existing CLI shape", () => {
  assert.match(usage(), /t <lang> <text>/);
  assert.match(usage(), /--tool <codex\|claude>/);
  assert.match(usage(), /--setup/);
});

test("translates through fake Codex", async () => {
  const tmp = await mkdtemp(path.join(tmpdir(), "translate-cli-test-"));
  const configPath = path.join(tmp, "config.toml");
  await writeConfig(configPath, "codex");

  const result = await runCLI(["ja", "hello"], {
    TRANSLATE_CLI_CONFIG: configPath,
    PATH: withFakePath("fake-codex")
  });
  assert.equal(result.stdout, "こんにちは\n");
  assert.equal(result.stderr, "");
});

test("translates stdin through fake Claude", async () => {
  const tmp = await mkdtemp(path.join(tmpdir(), "translate-cli-test-"));
  const configPath = path.join(tmp, "config.toml");
  await writeConfig(configPath, "claude");

  const result = await runCLI(["--tool", "claude", "ja"], {
    TRANSLATE_CLI_CONFIG: configPath,
    PATH: withFakePath("fake-claude")
  }, "hello");
  assert.equal(result.stdout, "こんにちは\n");
  assert.equal(result.stderr, "");
});

test("runs first-run setup", async () => {
  const tmp = await mkdtemp(path.join(tmpdir(), "translate-cli-test-"));
  const configPath = path.join(tmp, "config.toml");

  const result = await runCLI(["--setup"], {
    TRANSLATE_CLI_CONFIG: configPath,
    PATH: withFakePath("fake-codex")
  }, "\n\n");

  assert.equal(result.stdout, "");
  assert.match(result.stderr, /translate CLI setup/);
  assert.match(result.stderr, /Saved config:/);
  const config = await readFile(configPath, "utf8");
  assert.match(config, /default_tool = "codex"/);
  assert.match(config, /local_lang = /);
});

async function runCLI(
  args: string[],
  env: NodeJS.ProcessEnv = {},
  input = ""
): Promise<{ stdout: string; stderr: string }> {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [path.join("dist", "t.js"), ...args], {
      cwd: path.resolve("."),
      env: {
        ...process.env,
        ...env
      },
      stdio: ["pipe", "pipe", "pipe"]
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk: Buffer) => {
      stdout += chunk.toString("utf8");
    });
    child.stderr.on("data", (chunk: Buffer) => {
      stderr += chunk.toString("utf8");
    });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code !== 0) {
        reject(new Error(`CLI exited with ${code}: ${stderr}`));
        return;
      }
      resolve({ stdout, stderr });
    });
    child.stdin.end(input);
  });
}

async function writeConfig(configPath: string, tool: string): Promise<void> {
  await writeFile(
    configPath,
    `version = 1
default_tool = "${tool}"
local_lang = "ja"
timeout_ms = 60000

[tools.codex]
enabled = true

[tools.claude]
enabled = true
`
  );
}

function withFakePath(name: string): string {
  return [
    path.resolve("testdata", name),
    process.env.PATH ?? ""
  ].join(path.delimiter);
}
