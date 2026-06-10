#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import path from "node:path";

const CONFIRMATION = "promote codex native app saved project representatives";

function parseArgs(argv) {
  const args = {
    codexHome: "/Users/yoyi/.codex",
    apply: false,
    dryRun: false,
    confirm: "",
    topWindow: 25,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--apply") args.apply = true;
    else if (arg === "--dry-run") args.dryRun = true;
    else if (arg === "--codex-home") args.codexHome = argv[++i] ?? args.codexHome;
    else if (arg === "--confirm") args.confirm = argv[++i] ?? "";
    else if (arg === "--top-window") args.topWindow = Number(argv[++i]);
    else throw new Error(`unknown argument: ${arg}`);
  }
  if (!args.apply && !args.dryRun) args.dryRun = true;
  if (args.apply && args.dryRun) throw new Error("--apply and --dry-run cannot be combined");
  if (!Number.isSafeInteger(args.topWindow) || args.topWindow <= 0) {
    throw new Error("--top-window must be a positive integer");
  }
  return args;
}

function timestamp() {
  return new Date().toISOString().replace(/[-:.TZ]/g, "").slice(0, 14);
}

function readVisibleThreads(codexHome) {
  const dbPath = path.join(codexHome, "state_5.sqlite");
  if (!existsSync(dbPath)) throw new Error(`missing sqlite: ${dbPath}`);
  const stdout = execFileSync(
    "sqlite3",
    ["-json", `file:${dbPath}?mode=ro`, "select id,cwd,updated_at_ms from threads where archived = 0;"],
    { encoding: "utf8" },
  );
  return JSON.parse(stdout);
}

function readGlobalState(codexHome) {
  const statePath = path.join(codexHome, ".codex-global-state.json");
  if (!existsSync(statePath)) throw new Error(`missing global state: ${statePath}`);
  return JSON.parse(readFileSync(statePath, "utf8"));
}

function readSessionIndex(indexPath) {
  if (!existsSync(indexPath)) throw new Error(`missing session index: ${indexPath}`);
  const lines = readFileSync(indexPath, "utf8").split(/\n/).filter(Boolean);
  const records = [];
  let parseErrors = 0;
  for (const line of lines) {
    try {
      records.push(JSON.parse(line));
    } catch {
      parseErrors += 1;
    }
  }
  return { lines, records, parseErrors };
}

function parseTime(value) {
  const parsed = Date.parse(String(value ?? ""));
  return Number.isFinite(parsed) ? parsed : -Infinity;
}

function sortRecords(records) {
  return [...records].sort((a, b) => {
    const timeDelta = parseTime(a.updated_at) - parseTime(b.updated_at);
    if (timeDelta !== 0) return timeDelta;
    return String(a.id).localeCompare(String(b.id));
  });
}

function buildPromotion({ globalState, records, threads, topWindow }) {
  const savedRoots = new Set(globalState["electron-saved-workspace-roots"] ?? []);
  const byId = new Map(records.filter((record) => record.id).map((record) => [record.id, record]));
  const cwdById = new Map(threads.map((thread) => [thread.id, thread.cwd]));
  const latestThreadByRoot = new Map();

  for (const thread of threads) {
    if (!savedRoots.has(thread.cwd)) continue;
    if (!byId.has(thread.id)) continue;
    const current = latestThreadByRoot.get(thread.cwd);
    if (!current || Number(thread.updated_at_ms || 0) > Number(current.updated_at_ms || 0)) {
      latestThreadByRoot.set(thread.cwd, thread);
    }
  }

  const descending = [...records].sort((a, b) => {
    const timeDelta = parseTime(b.updated_at) - parseTime(a.updated_at);
    if (timeDelta !== 0) return timeDelta;
    return String(a.id).localeCompare(String(b.id));
  });
  const rankById = new Map(descending.map((record, index) => [record.id, index + 1]));
  const topRoots = [...new Set(descending.slice(0, topWindow).map((record) => cwdById.get(record.id) ?? null))];

  const candidates = [...latestThreadByRoot.entries()]
    .map(([root, thread]) => {
      const record = byId.get(thread.id);
      return {
        root,
        threadId: thread.id,
        oldRank: rankById.get(thread.id) ?? null,
        oldUpdatedAt: record?.updated_at ?? null,
      };
    })
    .filter((candidate) => candidate.oldRank == null || candidate.oldRank > topWindow)
    .sort((a, b) => a.root.localeCompare(b.root));

  const baseMs = Date.now();
  const promoted = new Map();
  candidates.forEach((candidate, index) => {
    promoted.set(candidate.threadId, {
      ...candidate,
      newUpdatedAt: new Date(baseMs - index * 1000).toISOString(),
    });
  });

  const nextRecords = sortRecords(records.map((record) => {
    const promotion = promoted.get(record.id);
    return promotion ? { ...record, updated_at: promotion.newUpdatedAt } : record;
  }));

  return {
    nextRecords,
    summary: {
      savedWorkspaceRoots: savedRoots.size,
      topWindow,
      topRoots,
      visibleSavedRootsWithSessionIndexEntry: latestThreadByRoot.size,
      promotedCount: promoted.size,
      promoted: [...promoted.values()],
      afterRecords: nextRecords.length,
      afterUniqueIds: new Set(nextRecords.map((record) => record.id)).size,
    },
  };
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const indexPath = path.join(args.codexHome, "session_index.jsonl");
  const globalState = readGlobalState(args.codexHome);
  const threads = readVisibleThreads(args.codexHome);
  const { lines, records, parseErrors } = readSessionIndex(indexPath);
  const { nextRecords, summary } = buildPromotion({
    globalState,
    records,
    threads,
    topWindow: args.topWindow,
  });
  const output = {
    mode: args.dryRun ? "dry-run" : "apply",
    indexPath,
    beforeLines: lines.length,
    parseErrors,
    ...summary,
  };

  if (args.dryRun) {
    console.log(JSON.stringify(output, null, 2));
    return;
  }

  if (parseErrors > 0) {
    throw new Error("session_index.jsonl has parse errors; refusing to apply automatically");
  }
  if (args.confirm !== CONFIRMATION) {
    throw new Error(`missing exact --confirm ${JSON.stringify(CONFIRMATION)}`);
  }

  const backupDir = path.join(args.codexHome, "backups_state", "native-conversation-list-repair", timestamp());
  mkdirSync(backupDir, { recursive: true });
  const backupPath = path.join(backupDir, "session_index.jsonl.before-promote");
  copyFileSync(indexPath, backupPath);

  const tmpPath = `${indexPath}.tmp-native-promote-${timestamp()}`;
  const body = nextRecords.map((record) => JSON.stringify(record)).join("\n");
  writeFileSync(tmpPath, `${body}\n`, { mode: 0o644 });
  renameSync(tmpPath, indexPath);

  console.log(JSON.stringify({ ...output, backupPath }, null, 2));
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
