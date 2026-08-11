import { build } from "esbuild";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const tempDir = await mkdtemp(join(tmpdir(), "codex-workbench-offline-test-"));
const testEntries = [
  "tests/offline-permission-dialog.test.tsx",
  "tests/frontend-wiring-microbatch.test.tsx",
  "tests/agent-session-row.test.tsx",
  "tests/shared-conversation-transport.test.tsx",
  "tests/role-session-read-model.test.ts",
  "tests/memory-center-daily-inbox.test.tsx",
  "tests/memory-center-vision-restyle.test.tsx",
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
  "tests/jiaoban-merged-layout.test.tsx",
  "tests/jiaoban-conversation-center.test.tsx",
  "tests/jiaoban-plan-preview-canvas.test.tsx",
  "tests/jiaoban-running-graph.test.tsx",
  "tests/jiaoban-director-planning-progress.test.tsx",
  "tests/jiaoban-chain-result-semantics.test.tsx",
  "tests/jiaoban-approval-seal-and-flag-note.test.tsx",
  "tests/knowledge-vault-notes.test.tsx",
  "tests/knowledge-workbench-shell.test.tsx",
  "tests/obsidian-integration.test.tsx",
  "tests/knowledge-open-relay.test.tsx",
  "tests/native-knowledge-workspace.test.tsx",
  "tests/knowledge-graph.test.tsx",
  "tests/knowledge-canvas.test.tsx",
  "tests/knowledge-attachment-recovery.test.tsx",
  "tests/jiaoban-page-content-cleanup.test.tsx",
  "tests/jiaoban-supervisor-pilot-switch.test.tsx",
  "tests/r4-page-read-model-settings.test.tsx",
  "tests/r4-page-read-model-query-contract.test.ts",
  "tests/r4-page-read-model-runtime.test.ts",
  "tests/r4-page-selectors.test.ts",
  "tests/m3-isolated-desktop-acceptance.test.tsx",
  "tests/m4c06-secretary-home-ui.test.tsx",
  "tests/m4c06-secretary-read-model.test.ts",
  "tests/m4c07-secretary-daily-read-model.test.ts",
  "tests/m4c08-legacy-read-compatibility-migration.test.tsx",
  "tests/m4c09-isolated-product-app-acceptance.test.ts",
  "tests/m4r02-ordinary-composition-driver.test.ts",
  "tests/m4r03-server-due-clock-composition.test.ts",
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
