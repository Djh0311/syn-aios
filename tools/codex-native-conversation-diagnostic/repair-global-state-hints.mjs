#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import path from "node:path";

const CONFIRMATION = "repair codex native app conversation list global state";

function parseArgs(argv) {
  const args = {
    codexHome: "/Users/yoyi/.codex",
    apply: false,
    dryRun: false,
    confirm: "",
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--apply") args.apply = true;
    else if (arg === "--dry-run") args.dryRun = true;
    else if (arg === "--codex-home") args.codexHome = argv[++i] ?? args.codexHome;
    else if (arg === "--confirm") args.confirm = argv[++i] ?? "";
    else throw new Error(`unknown argument: ${arg}`);
  }
  if (!args.apply && !args.dryRun) args.dryRun = true;
  if (args.apply && args.dryRun) throw new Error("--apply and --dry-run cannot be combined");
  return args;
}

function timestamp() {
  return new Date().toISOString().replace(/[-:.TZ]/g, "").slice(0, 14);
}

function readThreads(codexHome) {
  const dbPath = path.join(codexHome, "state_5.sqlite");
  if (!existsSync(dbPath)) throw new Error(`missing sqlite: ${dbPath}`);
  const stdout = execFileSync(
    "sqlite3",
    ["-json", `file:${dbPath}?mode=ro`, "select id,cwd,archived from threads;"],
    { encoding: "utf8" },
  );
  return JSON.parse(stdout);
}

function buildPatch(globalState, threads) {
  const savedRoots = new Set(globalState["electron-saved-workspace-roots"] ?? []);
  const beforeHints = globalState["thread-workspace-root-hints"] ?? {};
  const nextHints = { ...beforeHints };
  const perRoot = {};
  let added = 0;
  let changed = 0;
  let unchanged = 0;
  let skipped = 0;

  for (const thread of threads) {
    if (thread.archived) {
      skipped += 1;
      continue;
    }
    if (!savedRoots.has(thread.cwd)) {
      skipped += 1;
      continue;
    }
    perRoot[thread.cwd] = (perRoot[thread.cwd] ?? 0) + 1;
    if (nextHints[thread.id] === thread.cwd) {
      unchanged += 1;
      continue;
    }
    if (Object.prototype.hasOwnProperty.call(nextHints, thread.id)) changed += 1;
    else added += 1;
    nextHints[thread.id] = thread.cwd;
  }

  return {
    nextState: {
      ...globalState,
      "thread-workspace-root-hints": nextHints,
    },
    summary: {
      savedWorkspaceRoots: savedRoots.size,
      sqliteThreads: threads.length,
      added,
      changed,
      unchanged,
      skipped,
      totalHintsBefore: Object.keys(beforeHints).length,
      totalHintsAfter: Object.keys(nextHints).length,
      eligibleVisibleThreadsByRoot: perRoot,
    },
  };
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const statePath = path.join(args.codexHome, ".codex-global-state.json");
  if (!existsSync(statePath)) throw new Error(`missing global state: ${statePath}`);

  const globalState = JSON.parse(readFileSync(statePath, "utf8"));
  const threads = readThreads(args.codexHome);
  const { nextState, summary } = buildPatch(globalState, threads);

  if (args.dryRun) {
    console.log(JSON.stringify({ mode: "dry-run", statePath, ...summary }, null, 2));
    return;
  }

  if (args.confirm !== CONFIRMATION) {
    throw new Error(`missing exact --confirm ${JSON.stringify(CONFIRMATION)}`);
  }

  const backupDir = path.join(args.codexHome, "backups_state", "native-conversation-list-repair", timestamp());
  mkdirSync(backupDir, { recursive: true });
  const backupPath = path.join(backupDir, ".codex-global-state.json.before");
  copyFileSync(statePath, backupPath);

  const tmpPath = `${statePath}.tmp-native-repair-${timestamp()}`;
  writeFileSync(tmpPath, `${JSON.stringify(nextState, null, 2)}\n`, { mode: 0o644 });
  renameSync(tmpPath, statePath);

  console.log(JSON.stringify({ mode: "apply", statePath, backupPath, ...summary }, null, 2));
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}

