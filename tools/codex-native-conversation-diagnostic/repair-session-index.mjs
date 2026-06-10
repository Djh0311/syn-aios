#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import path from "node:path";

const CONFIRMATION = "repair codex native app conversation list session index";

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

function readVisibleThreads(codexHome) {
  const dbPath = path.join(codexHome, "state_5.sqlite");
  if (!existsSync(dbPath)) throw new Error(`missing sqlite: ${dbPath}`);
  const stdout = execFileSync(
    "sqlite3",
    [
      "-json",
      `file:${dbPath}?mode=ro`,
      "select id,substr(title,1,200) as title,cwd,updated_at,updated_at_ms from threads where archived = 0;",
    ],
    { encoding: "utf8", maxBuffer: 1024 * 1024 * 8 },
  );
  return JSON.parse(stdout);
}

function readSavedWorkspaceRoots(codexHome) {
  const statePath = path.join(codexHome, ".codex-global-state.json");
  if (!existsSync(statePath)) return new Set();
  const state = JSON.parse(readFileSync(statePath, "utf8"));
  return new Set(state["electron-saved-workspace-roots"] ?? []);
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

function sqliteTimeToIso(thread) {
  const value = Number(thread.updated_at_ms || 0) || Number(thread.updated_at || 0) * 1000;
  if (!Number.isFinite(value) || value <= 0) return new Date(0).toISOString();
  return new Date(value).toISOString();
}

function shortThreadName(value) {
  const normalized = String(value ?? "").replace(/\s+/g, " ").trim();
  if (!normalized) return "Untitled";
  return Array.from(normalized).slice(0, 36).join("");
}

function buildRepair(existingRecords, visibleThreads, savedWorkspaceRoots) {
  const existingById = new Map();
  let missingIdLines = 0;
  let duplicateLines = 0;

  for (const record of existingRecords) {
    if (!record.id) {
      missingIdLines += 1;
      continue;
    }
    const current = existingById.get(record.id);
    if (!current) {
      existingById.set(record.id, record);
      continue;
    }
    duplicateLines += 1;
    if (parseTime(record.updated_at) >= parseTime(current.updated_at)) {
      existingById.set(record.id, record);
    }
  }

  const visibleById = new Map(visibleThreads.map((thread) => [thread.id, thread]));
  const missingVisibleThreads = visibleThreads.filter((thread) => !existingById.has(thread.id));
  const missingVisibleByRoot = {};
  const savedWorkspaceMissingByRoot = {};

  for (const thread of missingVisibleThreads) {
    missingVisibleByRoot[thread.cwd] = (missingVisibleByRoot[thread.cwd] ?? 0) + 1;
    if (savedWorkspaceRoots.has(thread.cwd)) {
      savedWorkspaceMissingByRoot[thread.cwd] = (savedWorkspaceMissingByRoot[thread.cwd] ?? 0) + 1;
    }
  }

  let existingIdsNotInSqliteVisible = 0;
  for (const id of existingById.keys()) {
    if (!visibleById.has(id)) existingIdsNotInSqliteVisible += 1;
  }

  const addedRecords = missingVisibleThreads.map((thread) => ({
    id: thread.id,
    thread_name: shortThreadName(thread.title),
    updated_at: sqliteTimeToIso(thread),
  }));

  const nextRecords = [...existingById.values(), ...addedRecords].sort((a, b) => {
    const timeDelta = parseTime(a.updated_at) - parseTime(b.updated_at);
    if (timeDelta !== 0) return timeDelta;
    return String(a.id).localeCompare(String(b.id));
  });

  return {
    nextRecords,
    summary: {
      beforeUniqueIds: existingById.size,
      missingIdLines,
      duplicateLines,
      visibleSqliteThreads: visibleThreads.length,
      missingVisibleThreads: missingVisibleThreads.length,
      generatedThreadNameMaxLength: 36,
      addedRecords: addedRecords.length,
      existingIdsNotInSqliteVisible,
      afterRecords: nextRecords.length,
      afterUniqueIds: new Set(nextRecords.map((record) => record.id)).size,
      missingVisibleByRoot,
      savedWorkspaceMissingByRoot,
    },
  };
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const indexPath = path.join(args.codexHome, "session_index.jsonl");
  const visibleThreads = readVisibleThreads(args.codexHome);
  const savedWorkspaceRoots = readSavedWorkspaceRoots(args.codexHome);
  const { lines, records, parseErrors } = readSessionIndex(indexPath);
  const { nextRecords, summary } = buildRepair(records, visibleThreads, savedWorkspaceRoots);
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
  const backupPath = path.join(backupDir, "session_index.jsonl.before");
  copyFileSync(indexPath, backupPath);

  const tmpPath = `${indexPath}.tmp-native-repair-${timestamp()}`;
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
