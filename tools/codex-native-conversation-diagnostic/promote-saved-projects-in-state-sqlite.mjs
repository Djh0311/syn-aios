#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import path from "node:path";

const CONFIRMATION = "promote codex native app sqlite thread window";

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

function sqlQuote(value) {
  return `'${String(value).replace(/'/g, "''")}'`;
}

function readGlobalState(codexHome) {
  const statePath = path.join(codexHome, ".codex-global-state.json");
  if (!existsSync(statePath)) throw new Error(`missing global state: ${statePath}`);
  return JSON.parse(readFileSync(statePath, "utf8"));
}

function readVisibleThreads(dbPath) {
  const stdout = execFileSync(
    "sqlite3",
    [
      "-json",
      `file:${dbPath}?mode=ro`,
      [
        "SELECT id, cwd, updated_at, updated_at_ms",
        "FROM threads",
        "WHERE archived = 0",
        "ORDER BY COALESCE(updated_at_ms, updated_at * 1000) DESC, id ASC;",
      ].join(" "),
    ],
    { encoding: "utf8" },
  );
  return stdout.trim() ? JSON.parse(stdout) : [];
}

function threadTimeMs(thread) {
  const ms = Number(thread.updated_at_ms);
  if (Number.isFinite(ms) && ms > 0) return ms;
  const seconds = Number(thread.updated_at);
  return Number.isFinite(seconds) ? seconds * 1000 : 0;
}

function sortThreadsDesc(threads) {
  return [...threads].sort((a, b) => {
    const delta = threadTimeMs(b) - threadTimeMs(a);
    if (delta !== 0) return delta;
    return String(a.id).localeCompare(String(b.id));
  });
}

function uniqueRoots(threads) {
  return [...new Set(threads.map((thread) => thread.cwd ?? null))];
}

function buildPromotion({ globalState, threads, topWindow }) {
  const savedRootOrder = globalState["electron-saved-workspace-roots"] ?? [];
  const savedRoots = new Set(savedRootOrder);
  const latestThreadByRoot = new Map();

  for (const thread of threads) {
    if (!savedRoots.has(thread.cwd)) continue;
    const current = latestThreadByRoot.get(thread.cwd);
    if (!current || threadTimeMs(thread) > threadTimeMs(current)) {
      latestThreadByRoot.set(thread.cwd, thread);
    }
  }

  const descending = sortThreadsDesc(threads);
  const rankById = new Map(descending.map((thread, index) => [thread.id, index + 1]));
  const topThreads = descending.slice(0, topWindow);
  const topRootSet = new Set(topThreads.map((thread) => thread.cwd));
  const topRootsBefore = uniqueRoots(topThreads);

  const candidates = savedRootOrder
    .map((root) => ({ root, thread: latestThreadByRoot.get(root) }))
    .filter(({ root, thread }) => thread && !topRootSet.has(root))
    .map(({ root, thread }) => ({
      root,
      threadId: thread.id,
      oldRank: rankById.get(thread.id) ?? null,
      oldUpdatedAt: Number(thread.updated_at),
      oldUpdatedAtMs: threadTimeMs(thread),
      oldUpdatedAtIso: new Date(threadTimeMs(thread)).toISOString(),
    }));

  const baseMs = Date.now();
  const promotions = candidates.map((candidate, index) => {
    const newUpdatedAtMs = baseMs - index * 1000;
    return {
      ...candidate,
      newUpdatedAt: Math.floor(newUpdatedAtMs / 1000),
      newUpdatedAtMs,
      newUpdatedAtIso: new Date(newUpdatedAtMs).toISOString(),
    };
  });

  const promotionById = new Map(promotions.map((promotion) => [promotion.threadId, promotion]));
  const afterThreads = threads.map((thread) => {
    const promotion = promotionById.get(thread.id);
    if (!promotion) return thread;
    return {
      ...thread,
      updated_at: promotion.newUpdatedAt,
      updated_at_ms: promotion.newUpdatedAtMs,
    };
  });

  return {
    promotions,
    summary: {
      savedWorkspaceRoots: savedRootOrder.length,
      visibleSavedRoots: latestThreadByRoot.size,
      sqliteVisibleThreads: threads.length,
      topWindow,
      topRootsBefore,
      topRootsAfter: uniqueRoots(sortThreadsDesc(afterThreads).slice(0, topWindow)),
      promotedCount: promotions.length,
      promoted: promotions,
    },
  };
}

function backupSqlite(dbPath, backupPath) {
  execFileSync("sqlite3", [dbPath, `.backup ${backupPath}`], { encoding: "utf8" });
}

function applyPromotion(dbPath, promotions) {
  const updates = promotions.map((promotion) => (
    [
      "UPDATE threads",
      `SET updated_at = ${promotion.newUpdatedAt},`,
      `updated_at_ms = ${promotion.newUpdatedAtMs}`,
      `WHERE id = ${sqlQuote(promotion.threadId)} AND archived = 0;`,
    ].join(" ")
  ));
  const sql = [
    "PRAGMA busy_timeout = 5000;",
    "BEGIN IMMEDIATE;",
    ...updates,
    "COMMIT;",
  ].join("\n");
  execFileSync("sqlite3", [dbPath, sql], { encoding: "utf8" });
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const dbPath = path.join(args.codexHome, "state_5.sqlite");
  if (!existsSync(dbPath)) throw new Error(`missing sqlite: ${dbPath}`);

  const globalState = readGlobalState(args.codexHome);
  const threads = readVisibleThreads(dbPath);
  const { promotions, summary } = buildPromotion({
    globalState,
    threads,
    topWindow: args.topWindow,
  });

  const output = {
    mode: args.dryRun ? "dry-run" : "apply",
    dbPath,
    ...summary,
    warnings: [
      "This changes only state_5.sqlite thread updated_at / updated_at_ms display metadata.",
      "It does not read rollout bodies and does not touch auth/token/config.",
    ],
  };

  if (args.dryRun) {
    console.log(JSON.stringify(output, null, 2));
    return;
  }

  if (args.confirm !== CONFIRMATION) {
    throw new Error(`missing exact --confirm ${JSON.stringify(CONFIRMATION)}`);
  }

  if (promotions.length === 0) {
    console.log(JSON.stringify(output, null, 2));
    return;
  }

  const backupDir = path.join(args.codexHome, "backups_state", "native-conversation-list-repair", timestamp());
  mkdirSync(backupDir, { recursive: true });
  const backupPath = path.join(backupDir, "state_5.sqlite.before-promote");
  backupSqlite(dbPath, backupPath);
  applyPromotion(dbPath, promotions);

  const afterThreads = readVisibleThreads(dbPath);
  const afterById = new Map(afterThreads.map((thread) => [thread.id, thread]));
  for (const promotion of promotions) {
    const after = afterById.get(promotion.threadId);
    if (!after || Number(after.updated_at_ms) !== promotion.newUpdatedAtMs) {
      throw new Error(`sqlite update verification failed for ${promotion.threadId}`);
    }
  }

  console.log(JSON.stringify({ ...output, backupPath }, null, 2));
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
