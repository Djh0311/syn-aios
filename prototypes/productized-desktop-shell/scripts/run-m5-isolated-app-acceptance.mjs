import { createHash, randomBytes } from "node:crypto";
import { spawn } from "node:child_process";
import { chmod, lstat, mkdir, mkdtemp, readFile, realpath, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const PROFILE_ENV = "SYN_R4_ACCEPTANCE_PROFILE";
const REENTRY_CAPABILITY_ENV = "SYN_R4_REENTRY_CAPABILITY";
const PROFILE_PURPOSE = "syn-r4-isolated-runtime-profile";
const ROOT_PREFIX = "syn-r4-acceptance-";
const FIXTURE_PREFIX = "SYN R4 ISOLATED ACCEPTANCE ";
const MODE_0700 = 0o700;
const MODE_0600 = 0o600;

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const desktopRoot = resolve(scriptDirectory, "..");
const screenshotHelper = resolve(scriptDirectory, "m5-x11-screenshot.py");
const binaryPath = resolve(
  desktopRoot,
  "src-tauri/target/debug/codex-governance-workbench",
);

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function stableId(value) {
  let output = "";
  for (const character of value) {
    const code = character.charCodeAt(0);
    const isAsciiAlphanumeric =
      (code >= 48 && code <= 57) ||
      (code >= 65 && code <= 90) ||
      (code >= 97 && code <= 122);
    if (isAsciiAlphanumeric) output += character.toLowerCase();
    else if (!output.endsWith("-")) output += "-";
  }
  return output.replace(/^-+|-+$/g, "").slice(0, 96);
}

async function ensurePrivateDirectory(path) {
  await mkdir(path, { recursive: true, mode: MODE_0700 });
  await chmod(path, MODE_0700);
}

async function writeJson(path, value) {
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: MODE_0600,
  });
  await chmod(path, MODE_0600);
}

function runChild(command, args, options) {
  return new Promise((resolveChild) => {
    const child = spawn(command, args, options);
    const chunks = [];
    child.stdout?.on("data", (chunk) => chunks.push(chunk));
    child.stderr?.on("data", (chunk) => chunks.push(chunk));
    child.once("error", (error) => {
      resolveChild({
        launched: false,
        exit_code: null,
        signal: null,
        output: String(error),
        pid: null,
      });
    });
    child.once("exit", (code, signal) => {
      resolveChild({
        launched: true,
        exit_code: code,
        signal: signal ?? null,
        output: Buffer.concat(chunks).toString("utf8").slice(-8000),
        pid: child.pid ?? null,
      });
    });
  });
}

function spawnDetached(command, args, options) {
  const child = spawn(command, args, {
    ...options,
    detached: true,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const output = [];
  child.stdout?.on("data", (chunk) => output.push(chunk));
  child.stderr?.on("data", (chunk) => output.push(chunk));
  return { child, output };
}

async function waitForFile(path, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (existsSync(path)) {
      try {
        return JSON.parse(await readFile(path, "utf8"));
      } catch {
        // Writer may still be flushing.
      }
    }
    await new Promise((resolveWait) => setTimeout(resolveWait, 400));
  }
  throw new Error(`timeout_waiting_for:${path}`);
}

async function captureScreenshot(outPath) {
  const result = await runChild("python3", [screenshotHelper, outPath], {
    cwd: desktopRoot,
    env: {
      ...process.env,
      DISPLAY: process.env.DISPLAY || ":0",
      GDK_BACKEND: "x11",
    },
    stdio: ["ignore", "pipe", "pipe"],
  });
  return {
    path: outPath,
    exit_code: result.exit_code,
    sha256: existsSync(outPath) ? sha256(await readFile(outPath)) : null,
    output: result.output.slice(-500),
  };
}

async function createIsolatedProfile() {
  const canonicalTemp = await realpath(tmpdir());
  const created = await mkdtemp(join(canonicalTemp, ROOT_PREFIX));
  await chmod(created, MODE_0700);
  const root = await realpath(created);
  const runId = `syn-r4-${randomBytes(8).toString("hex")}`;
  const projectRelativePath = `fixture/${FIXTURE_PREFIX}${runId}`;
  const projectRoot = resolve(root, projectRelativePath);
  const projectId = `project:${stableId(projectRoot)}`;
  const workflowId = `workflow:${stableId(projectRoot)}:default`;
  const nowMs = Date.now();
  const timestamp = new Date(nowMs).toISOString();
  const profile = {
    schema_version: 1,
    purpose: PROFILE_PURPOSE,
    run_id: runId,
    expires_at_ms: nowMs + 60 * 60 * 1000,
    project: { id: projectId, relative_path: projectRelativePath },
    workflow: { id: workflowId },
    paths: {
      index_relative_path: "fixture/codex-index.json",
      tasks_relative_path: "fixture/tasks.md",
      workflow_state_relative_path: "workflow-state/workflow-state.v0.json",
      app_data_relative_path: "app-data",
      canvas_relative_path: "app-data/canvas-v1",
      codex_db_relative_path: "codex-db/state.sqlite",
    },
  };
  await writeJson(join(root, "profile.json"), profile);
  await ensurePrivateDirectory(join(root, "fixture"));
  await ensurePrivateDirectory(projectRoot);
  await ensurePrivateDirectory(join(root, "workflow-state"));
  await ensurePrivateDirectory(join(root, "app-data"));
  await ensurePrivateDirectory(join(root, "codex-db"));
  await ensurePrivateDirectory(join(root, "logs"));
  await writeJson(join(root, "fixture/codex-index.json"), {
    generated_at: timestamp,
    projects: [
      {
        project_root: projectRoot,
        active_hint: true,
        thread_count: 0,
        active_thread_count: 0,
        archived_thread_count: 0,
        authority_files: [],
        handoff_files: [],
        evidence_files: [],
        harness_candidates: [],
        harness_resources: [],
        context_warnings: [],
        warnings: [],
      },
    ],
    threads: [],
    skills: [],
    plugins: [],
    warnings: [],
  });
  await writeFile(join(root, "fixture/tasks.md"), "", { encoding: "utf8", flag: "wx", mode: MODE_0600 });
  await chmod(join(root, "fixture/tasks.md"), MODE_0600);
  await writeJson(join(root, "workflow-state/workflow-state.v0.json"), {
    schema_version: "workflow_state_v0",
    workflow_version: 1,
    revision: 0,
    workspace_id: `workspace:${runId}`,
    created_at: timestamp,
    updated_at: timestamp,
    source_kind: "isolated_acceptance_fixture",
    permission_level: "user_confirmed_write",
    projects: [
      {
        project_id: projectId,
        display_name: `${FIXTURE_PREFIX}${runId}`,
        root_path: projectRoot,
        source_kind: "codex_index",
        permission_level: "read_only",
        created_at: timestamp,
        updated_at: timestamp,
        warnings: [],
      },
    ],
    agent_adapters: [],
    workflows: [
      {
        workflow_id: workflowId,
        workflow_version: 1,
        project_id: projectId,
        title: `${FIXTURE_PREFIX}${runId} workflow`,
        state: "draft",
        source_kind: "isolated_acceptance_fixture",
        permission_level: "user_confirmed_write",
        model_policy: "none",
        created_at: timestamp,
        updated_at: timestamp,
      },
    ],
    nodes: [],
    edges: [],
    work_items: [],
    artifacts: [],
    reviews: [],
    workflow_node_session_bindings: [],
    workflow_node_dispatches: [],
    audit_events: [],
    capabilities: [],
    harness_resources: [],
  });
  const capability = randomBytes(32).toString("hex");
  return {
    root,
    profilePath: join(root, "profile.json"),
    projectRoot,
    logs: join(root, "logs"),
    capability,
    runId,
    profileFingerprint: sha256(await readFile(join(root, "profile.json"))),
  };
}

function isolatedEnv(profile, launchOrdinal, extra = {}) {
  const environment = { ...process.env, ...extra };
  for (const name of [
    "SYN_M3C07_ISOLATED_ACCEPTANCE",
    "SYN_M4C09_ISOLATED_ACCEPTANCE",
    "SYN_M4R02_ORDINARY_COMPOSITION_DRIVER",
    "SYN_M4R03_ORDINARY_CLOCK_DRIVER",
    "SYN_M4R04_ORDINARY_ROUTE_DRIVER",
    "SYN_M4R05_ORDINARY_CONVERSATION_DRIVER",
    "SYN_M4R06_ORDINARY_LEGACY_READ_DRIVER",
  ]) {
    delete environment[name];
  }
  environment[PROFILE_ENV] = profile.profilePath;
  environment[REENTRY_CAPABILITY_ENV] = profile.capability;
  environment.SYN_M5R07_ISOLATED_ACCEPTANCE = "1";
  environment.SYN_M5R07_LAUNCH_ORDINAL = String(launchOrdinal);
  environment.SYN_M5R07_SCENE = launchOrdinal === 0 ? "both" : "resume";
  environment.DISPLAY = process.env.DISPLAY || ":0";
  environment.GDK_BACKEND = "x11";
  delete environment.WAYLAND_DISPLAY;
  environment.WEBKIT_DISABLE_DMABUF_RENDERER = "1";
  environment.WEBKIT_DISABLE_COMPOSITING_MODE = "1";
  return environment;
}

async function startVite() {
  const child = spawnDetached("npm", ["run", "dev"], {
    cwd: desktopRoot,
    env: { ...process.env, BROWSER: "none" },
  });
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    const text = Buffer.concat(child.output).toString("utf8");
    if (text.includes("127.0.0.1:5173") || text.includes("Local:")) {
      return child;
    }
    await new Promise((resolveWait) => setTimeout(resolveWait, 300));
  }
  return child;
}

async function main() {
  if (process.argv.length !== 2) {
    process.stderr.write("m5r07_acceptance_wrapper_rejects_arguments\n");
    process.exitCode = 64;
    return;
  }
  const openingHash = sha256(
    await runChild("git", ["rev-parse", "HEAD"], { cwd: desktopRoot }).then(
      (result) => result.output.trim() || "unknown",
    ),
  );
  const profile = await createIsolatedProfile();
  const cargoBuild = await runChild(
    "cargo",
    ["build", "--offline", "--bins"],
    {
      cwd: resolve(desktopRoot, "src-tauri"),
      env: { ...process.env, CARGO_TERM_COLOR: "never" },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const vite = await startVite();
  const first = spawnDetached(binaryPath, [], {
    cwd: desktopRoot,
    env: isolatedEnv(profile, 0),
  });
  let sceneA = null;
  let sceneB = null;
  let resume = null;
  let firstError = null;
  try {
    sceneA = await waitForFile(join(profile.logs, "m5r07-ui-scene-a.json"), 180_000);
    sceneB = await waitForFile(join(profile.logs, "m5r07-ui-scene-b.json"), 180_000);
  } catch (error) {
    firstError = String(error);
  }
  const shotA = await captureScreenshot(join(profile.logs, "m5r07-window-scene-b.ppm"));
  if (typeof first.child.pid === "number") {
    try {
      process.kill(-first.child.pid, "SIGKILL");
    } catch {
      try {
        process.kill(first.child.pid, "SIGKILL");
      } catch {
        // already gone
      }
    }
  }
  await new Promise((resolveWait) => setTimeout(resolveWait, 1500));
  const second = spawnDetached(binaryPath, [], {
    cwd: desktopRoot,
    env: isolatedEnv(profile, 1),
  });
  let resumeError = null;
  try {
    resume = await waitForFile(join(profile.logs, "m5r07-ui-resume.json"), 180_000);
  } catch (error) {
    resumeError = String(error);
  }
  const shotB = await captureScreenshot(join(profile.logs, "m5r07-window-resume.ppm"));
  if (typeof second.child.pid === "number") {
    try {
      process.kill(-second.child.pid, "SIGTERM");
    } catch {
      try {
        process.kill(second.child.pid, "SIGTERM");
      } catch {
        // already gone
      }
    }
  }
  if (typeof vite.child.pid === "number") {
    try {
      process.kill(-vite.child.pid, "SIGTERM");
    } catch {
      try {
        process.kill(vite.child.pid, "SIGTERM");
      } catch {
        // already gone
      }
    }
  }
  const grants = existsSync(
    join(profile.root, "app-data/local.codex.governance.workbench/m5/orchestration.sqlite"),
  );
  const receipt = {
    schema: "syn.m5r07.isolated-app-launcher.v1",
    profile_root: profile.root,
    profile_fingerprint: profile.profileFingerprint,
    run_id: profile.runId,
    cargo_build_exit: cargoBuild.exit_code,
    cargo_build_signal: cargoBuild.signal,
    binary_present: existsSync(binaryPath),
    scene_a: sceneA,
    scene_b: sceneB,
    resume,
    first_launch_error: firstError,
    resume_error: resumeError,
    same_binding:
      Boolean(sceneA?.binding_id) && sceneA?.binding_id === resume?.binding_id,
    same_role_session:
      Boolean(sceneA?.role_session_id) &&
      sceneA?.role_session_id === resume?.role_session_id,
    scene_a_zero_grant: sceneA?.grants === 0,
    scene_b_exact_join: Boolean(sceneB?.grant_join?.claim_id && sceneB?.grant_join?.review_id),
    scene_b_deep_link_resolves: sceneB?.deep_link_resolves === true,
    receipts_backend_derived:
      sceneA?.derived_from === "backend_store" &&
      sceneB?.derived_from === "backend_store" &&
      resume?.derived_from === "backend_store",
    window_scene_b: shotA,
    window_resume: shotB,
    m5_store_present: grants,
    opening_head_sha256: openingHash,
    display: process.env.DISPLAY || ":0",
    isolated_modes: {
      m3c07: false,
      m4c09: false,
      m5r07: true,
    },
  };
  const receiptPath = join(profile.logs, "m5r07-launcher-receipt.json");
  await writeFile(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`, {
    encoding: "utf8",
    mode: MODE_0600,
  });
  process.stdout.write(`${JSON.stringify({ receipt_path: receiptPath, ...receipt }, null, 2)}\n`);
  const passed =
    cargoBuild.exit_code === 0 &&
    sceneA &&
    sceneB &&
    resume &&
    receipt.same_binding &&
    receipt.same_role_session &&
    sceneA.spawned === false &&
    sceneA.grants === 0 &&
    sceneB.dispatched === true &&
    sceneB.deep_link_resolves === true &&
    receipt.receipts_backend_derived &&
    shotA.sha256 &&
    shotB.sha256;
  process.exitCode = passed ? 0 : 1;
}

await main();
