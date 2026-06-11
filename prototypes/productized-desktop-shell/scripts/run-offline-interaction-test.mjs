import { build } from "esbuild";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const tempDir = await mkdtemp(join(tmpdir(), "codex-workbench-offline-test-"));
const testEntries = [
  "tests/offline-permission-dialog.test.tsx",
  "tests/r4-page-read-model-settings.test.tsx",
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
