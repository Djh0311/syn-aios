import { spawn } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// C09 deliberately exposes no profile, provider, model, account, connector,
// path, or credential argument.  The R4 launcher mints the one isolated
// profile and seals the fixed debug-child environment itself.
if (process.argv.length !== 2) {
  process.stderr.write("m4c09_acceptance_wrapper_rejects_arguments\n");
  process.exitCode = 64;
} else {
  const scriptDirectory = dirname(fileURLToPath(import.meta.url));
  const preflight = resolve(scriptDirectory, "run-r4-isolated-app-preflight.mjs");
  const child = spawn(
    process.execPath,
    [preflight, "--m4c09-isolated-acceptance"],
    {
      cwd: resolve(scriptDirectory, ".."),
      env: { ...process.env },
      shell: false,
      stdio: "inherit",
    },
  );

  child.on("error", () => {
    process.stderr.write("m4c09_acceptance_launcher_spawn_failed\n");
    process.exitCode = 1;
  });
  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    process.exitCode = code ?? 1;
  });
}
