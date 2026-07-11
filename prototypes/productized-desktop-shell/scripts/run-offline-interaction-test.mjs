import { build } from "esbuild";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const tempDir = await mkdtemp(join(tmpdir(), "codex-workbench-offline-test-"));
const testEntries = [
  "tests/offline-permission-dialog.test.tsx",
  "tests/report-on-face-yellow-flag.test.tsx",
  "tests/report-fact-confirm-recall.test.tsx",
  "tests/report-boundary-opinion.test.tsx",
  "tests/raw-session-bridge.test.tsx",
  "tests/global-supervisor-review-section.test.tsx",
  "tests/advice-only-authorize-face.test.tsx",
  "tests/secretary-pending-board.test.ts",
  "tests/secretary-pending-board-face.test.tsx",
  "tests/jiaoban-history-and-secretary-board.test.tsx",
  "tests/jiaoban-needs-rework-disposal.test.tsx",
  "tests/jiaoban-task-session-binding.test.tsx",
  "tests/jiaoban-merged-layout.test.tsx",
  "tests/jiaoban-plan-preview-canvas.test.tsx",
  "tests/jiaoban-running-graph.test.tsx",
  "tests/jiaoban-director-planning-progress.test.tsx",
  "tests/jiaoban-chain-result-semantics.test.tsx",
  "tests/jiaoban-page-content-cleanup.test.tsx",
  "tests/r4-page-read-model-settings.test.tsx",
  "tests/r4-page-read-model-query-contract.test.ts",
  "tests/r4-page-read-model-runtime.test.ts",
  "tests/r4-page-selectors.test.ts",
];

try {
  for (const entryPoint of testEntries) {
    const outfile = join(tempDir, `${entryPoint.replace(/[^a-z0-9]+/gi, "-")}.mjs`);
    await build({
      entryPoints: [entryPoint],
      outfile,
      bundle: true,
      platform: "node",
      format: "esm",
      target: "node22",
      jsx: "automatic",
      logLevel: "silent",
    });

    await import(pathToFileURL(outfile).href);
  }
} finally {
  await rm(tempDir, { recursive: true, force: true });
}

process.exit(0);
