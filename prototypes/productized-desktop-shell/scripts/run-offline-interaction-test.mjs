import { build } from "esbuild";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const tempDir = await mkdtemp(join(tmpdir(), "codex-workbench-offline-test-"));
const outfile = join(tempDir, "offline-permission-dialog.test.mjs");

try {
  await build({
    entryPoints: ["tests/offline-permission-dialog.test.tsx"],
    outfile,
    bundle: true,
    platform: "node",
    format: "esm",
    target: "node22",
    jsx: "automatic",
    logLevel: "silent",
  });

  await import(pathToFileURL(outfile).href);
} finally {
  await rm(tempDir, { recursive: true, force: true });
}

process.exit(0);
