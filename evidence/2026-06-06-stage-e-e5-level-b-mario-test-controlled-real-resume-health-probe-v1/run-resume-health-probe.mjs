import { spawn } from "node:child_process";
import { createWriteStream, writeFileSync } from "node:fs";

const codex = "/opt/homebrew/Cellar/node/23.11.0/bin/codex";
const cwd = "/Users/yoyi/Documents/mario test";
const threadId = "019e798a-6ce5-76c3-b8ee-33bd0fda841f";
const outDir =
  "/Users/yoyi/workspace/product-line/evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1";
const lastMessagePath = `${outDir}/last-message.txt`;
const stdoutPath = `${outDir}/codex-stdout.jsonl`;
const stderrPath = `${outDir}/codex-stderr.txt`;
const resultPath = `${outDir}/command-result.json`;
const prompt = `你正在参与 E5 Level B 真实 resume 健康探针，项目为 /Users/yoyi/Documents/mario test。
请只回复一行：
E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06
不要读取、列出或修改任何文件。不要运行命令。不要创建计划。不要调用工具。
`;

const argv = [
  "exec",
  "-C",
  cwd,
  "--sandbox",
  "read-only",
  "resume",
  "--skip-git-repo-check",
  "--json",
  "--output-last-message",
  lastMessagePath,
  threadId,
];

const startedAt = new Date().toISOString();
const child = spawn(codex, argv, {
  cwd,
  stdio: ["pipe", "pipe", "pipe"],
});

child.stdout.pipe(createWriteStream(stdoutPath));
child.stderr.pipe(createWriteStream(stderrPath));
child.stdin.end(prompt, "utf8");

const timeoutMs = 300000;
const timeout = setTimeout(() => {
  child.kill("SIGTERM");
}, timeoutMs);

child.on("error", (error) => {
  clearTimeout(timeout);
  writeFileSync(
    resultPath,
    `${JSON.stringify(
      {
        started_at: startedAt,
        finished_at: new Date().toISOString(),
        argv,
        cwd,
        thread_id: threadId,
        timeout_ms: timeoutMs,
        spawn_error: String(error),
      },
      null,
      2,
    )}\n`,
  );
  process.exitCode = 1;
});

child.on("close", (code, signal) => {
  clearTimeout(timeout);
  writeFileSync(
    resultPath,
    `${JSON.stringify(
      {
        started_at: startedAt,
        finished_at: new Date().toISOString(),
        argv,
        cwd,
        thread_id: threadId,
        timeout_ms: timeoutMs,
        exit_code: code,
        signal,
      },
      null,
      2,
    )}\n`,
  );
  process.exitCode = code ?? (signal ? 124 : 1);
});
