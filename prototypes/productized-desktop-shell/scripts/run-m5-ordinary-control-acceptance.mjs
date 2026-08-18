import { createHash, randomBytes } from "node:crypto";
import { spawn } from "node:child_process";
import { chmod, mkdir, mkdtemp, readFile, realpath, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const DRIVER_ENV = "SYN_M5R07_ORDINARY_CONTROL_ACCEPTANCE";
const PROFILE_ENV = "SYN_M5R07_ORDINARY_CONTROL_PROFILE";
const CAPABILITY_ENV = "SYN_M5R07_ORDINARY_CONTROL_CAPABILITY";
const PHASE_ENV = "SYN_M5R07_ORDINARY_CONTROL_PHASE";
const NONCE_ENV = "SYN_M5R07_ORDINARY_CONTROL_NONCE";
const DRIVER_VALUE = "ordinary-disposable-positive-tauri-v1";
const PURPOSE = "syn-m5r07-ordinary-disposable-positive-tauri-v1";
const COMPOSITION = "ORDINARY_TAURI_CONSTRUCTOR_SYNTHETIC_ISOLATED_INPUTS";
const IDENTITY_SOURCE_SCHEMA = "m1.ordinary-project-identity-source.v1";
const IDENTITY_SOURCE_FILE = "m1-ordinary-project-identity-source-v1.json";
const APP_DATA_RELATIVE_PATH = "app-data/local.codex.governance.workbench";
const ROOT_PREFIX = "syn-m5r07-ordinary-";
const MODE_0700 = 0o700;
const MODE_0600 = 0o600;

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const desktopRoot = resolve(scriptDirectory, "..");
const binaryPath = resolve(
  desktopRoot,
  "src-tauri/target/debug/codex-governance-workbench",
);

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
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

function killTree(pid) {
  if (typeof pid !== "number") return;
  try {
    process.kill(-pid, "SIGKILL");
  } catch {
    try {
      process.kill(pid, "SIGKILL");
    } catch {
      // already gone
    }
  }
}

async function createOrdinaryProfile() {
  const canonicalTemp = await realpath(tmpdir());
  const created = await mkdtemp(join(canonicalTemp, ROOT_PREFIX));
  await chmod(created, MODE_0700);
  const root = await realpath(created);
  const runId = `syn-m5r07-${randomBytes(8).toString("hex")}`;
  const projectRelativePath = "fixture/project";
  const projectRoot = resolve(root, projectRelativePath);
  const nowMs = Date.now();
  const timestamp = new Date(nowMs).toISOString();
  const capability = randomBytes(32).toString("hex");
  const profile = {
    schema_version: 1,
    purpose: PURPOSE,
    run_id: runId,
    expires_at_ms: nowMs + 60 * 60 * 1000,
    capability_sha256: sha256(capability),
    project_relative_path: projectRelativePath,
    paths: {
      app_data_relative_path: APP_DATA_RELATIVE_PATH,
      index_relative_path: "fixture/codex-index.json",
      tasks_relative_path: "fixture/tasks.md",
      logs_relative_path: "logs",
    },
  };
  await writeJson(join(root, "profile.json"), profile);
  await ensurePrivateDirectory(join(root, "fixture"));
  await ensurePrivateDirectory(projectRoot);
  await ensurePrivateDirectory(join(root, "app-data"));
  const appDataRoot = join(root, APP_DATA_RELATIVE_PATH);
  await ensurePrivateDirectory(appDataRoot);
  await ensurePrivateDirectory(join(root, "logs"));
  const canonicalProjectRoot = await realpath(projectRoot);
  const sourceRef = `synthetic://m5r07-ordinary-disposable/${runId}`;
  const identitySourceRelativePath = `${APP_DATA_RELATIVE_PATH}/${IDENTITY_SOURCE_FILE}`;
  const identitySourcePath = join(root, identitySourceRelativePath);
  const identitySource = {
    schema_version: IDENTITY_SOURCE_SCHEMA,
    source_id: `syn-m5r07-ordinary-synthetic-source-${runId}`,
    source_revision: 1,
    projects: [
      {
        entry_id: `syn-m5r07-ordinary-entry-${runId}`,
        mode: "migrate_legacy_project",
        source_ref: sourceRef,
        exact_alias: canonicalProjectRoot,
      },
    ],
  };
  await writeJson(identitySourcePath, identitySource);
  const identitySourceSha256 = sha256(await readFile(identitySourcePath));
  await writeJson(join(root, "fixture/codex-index.json"), {
    generated_at: timestamp,
    projects: [
      {
        project_root: canonicalProjectRoot,
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
  await writeFile(join(root, "fixture/tasks.md"), "# ordinary disposable fixture\n", {
    encoding: "utf8",
    flag: "wx",
    mode: MODE_0600,
  });
  return {
    root,
    profilePath: join(root, "profile.json"),
    projectRoot,
    logs: join(root, "logs"),
    capability,
    runId,
    profileFingerprint: sha256(await readFile(join(root, "profile.json"))),
    identitySource: {
      relative_path: identitySourceRelativePath,
      sha256: identitySourceSha256,
      schema_version: IDENTITY_SOURCE_SCHEMA,
      mode: "migrate_legacy_project",
      source_ref: sourceRef,
    },
  };
}

function ordinaryEnv(profile, phase, extra = {}) {
  const environment = { ...process.env, ...extra };
  for (const name of [
    "SYN_R4_ACCEPTANCE_PROFILE",
    "SYN_R4_REENTRY_CAPABILITY",
    "SYN_M5R07_ISOLATED_ACCEPTANCE",
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
  environment[DRIVER_ENV] = DRIVER_VALUE;
  environment[PROFILE_ENV] = profile.profilePath;
  environment[CAPABILITY_ENV] = profile.capability;
  environment[PHASE_ENV] = phase;
  environment[NONCE_ENV] = randomBytes(16).toString("hex");
  environment.DISPLAY = process.env.DISPLAY || ":99";
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

async function maybeStartXvfb() {
  if (process.env.DISPLAY) {
    return { started: false, display: process.env.DISPLAY, pid: null };
  }
  const display = ":99";
  const child = spawn("Xvfb", [display, "-screen", "0", "1280x800x24"], {
    detached: true,
    stdio: "ignore",
  });
  child.unref();
  process.env.DISPLAY = display;
  await new Promise((resolveWait) => setTimeout(resolveWait, 500));
  return { started: true, display, pid: child.pid ?? null };
}

function processReceipt(child, phase, launchOrdinal) {
  return {
    phase,
    launch_ordinal: launchOrdinal,
    pid: child.child.pid ?? null,
    launched: typeof child.child.pid === "number",
    window_capture: "NO_WINDOW_CAPTURE",
  };
}

async function readM1Registry(profileRoot) {
  const registryPath = join(profileRoot, APP_DATA_RELATIVE_PATH, "m1/project-index-v1.json");
  const bytes = await readFile(registryPath);
  const parsed = JSON.parse(bytes.toString("utf8"));
  return {
    path: registryPath,
    sha256: sha256(bytes),
    registry_revision: parsed.registry_revision ?? null,
  };
}

function sameExactObjectIds(left, right) {
  if (!left || !right) return false;
  const keys = [
    "proposal_id",
    "authorization_decision_id",
    "authorization_id",
    "workflow_run_id",
    "work_item_id",
    "attempt_id",
    "grant_id",
    "dispatch_id",
    "runtime_receipt_id",
    "execution_readback_id",
    "claim_id",
    "executed_claim_id",
    "review_id",
    "result_decision_id",
    "fact_id",
  ];
  return keys.every((key) => Boolean(left[key]) && left[key] === right[key]);
}

function sameClosedLoopCounts(left, right) {
  if (!left || !right) return false;
  const keys = [
    "grants",
    "attempts",
    "dispatches",
    "durable_operations",
    "execution_readbacks",
    "claims",
    "reviews",
    "result_decisions",
    "project_facts",
  ];
  return keys.every((key) => left[key] === right[key]);
}

async function main() {
  if (process.argv.length !== 2) {
    process.stderr.write("m5r07_ordinary_acceptance_wrapper_rejects_arguments\n");
    process.exitCode = 64;
    return;
  }
  const xvfb = await maybeStartXvfb();
  const profile = await createOrdinaryProfile();
  const cargoBuild = await runChild("cargo", ["build", "--offline", "--bins"], {
    cwd: resolve(desktopRoot, "src-tauri"),
    env: { ...process.env, CARGO_TERM_COLOR: "never" },
    stdio: ["ignore", "pipe", "pipe"],
  });
  const vite = await startVite();
  const first = spawnDetached(binaryPath, [], {
    cwd: desktopRoot,
    env: ordinaryEnv(profile, "first"),
  });
  const firstProcess = processReceipt(first, "first", 1);
  let firstError = null;
  let open = null;
  let rejected = null;
  let seeded = null;
  let retried = null;
  let runtime = null;
  let runtimeRepeat = null;
  let report = null;
  let review = null;
  let result = null;
  let firstRegistry = null;
  let failed = null;
  try {
    rejected = await waitForFile(join(profile.logs, "m5r07-ordinary-backend-rejected.json"), 180_000);
    open = existsSync(join(profile.logs, "m5r07-ordinary-backend-open.json"))
      ? JSON.parse(await readFile(join(profile.logs, "m5r07-ordinary-backend-open.json"), "utf8"))
      : null;
    firstRegistry = await readM1Registry(profile.root);
    seeded = await waitForFile(join(profile.logs, "m5r07-ordinary-backend-seeded.json"), 60_000);
    retried = await waitForFile(join(profile.logs, "m5r07-ordinary-backend-retried.json"), 60_000);
    runtime = await waitForFile(join(profile.logs, "m5r07-ordinary-backend-runtime.json"), 60_000);
    runtimeRepeat = await waitForFile(
      join(profile.logs, "m5r07-ordinary-backend-runtime_repeat.json"),
      60_000,
    );
    report = await waitForFile(join(profile.logs, "m5r07-ordinary-backend-report.json"), 60_000);
    review = await waitForFile(join(profile.logs, "m5r07-ordinary-backend-review.json"), 60_000);
    result = await waitForFile(join(profile.logs, "m5r07-ordinary-backend-result.json"), 60_000);
  } catch (error) {
    firstError = String(error);
    try {
      failed = JSON.parse(await readFile(join(profile.logs, "m5r07-ordinary-backend-failed.json"), "utf8"));
    } catch {
      // no failed receipt
    }
  }
  killTree(first.child.pid);
  let reopen = null;
  let reopenError = null;
  let secondRegistry = null;
  const second = spawnDetached(binaryPath, [], {
    cwd: desktopRoot,
    env: ordinaryEnv(profile, "reopen"),
  });
  const secondProcess = processReceipt(second, "reopen", 2);
  try {
    reopen = await waitForFile(join(profile.logs, "m5r07-ordinary-backend-reopen.json"), 180_000);
    secondRegistry = await readM1Registry(profile.root);
  } catch (error) {
    reopenError = String(error);
    try {
      secondRegistry = await readM1Registry(profile.root);
    } catch {
      // registry may be absent if the first process never constructed
    }
  }
  killTree(second.child.pid);
  killTree(vite.child.pid);
  if (xvfb.started && typeof xvfb.pid === "number") {
    killTree(xvfb.pid);
  }
  const sameBinding =
    Boolean(open?.binding_id) && open?.binding_id === reopen?.binding_id;
  const sameProject =
    Boolean(open?.project_id) &&
    open?.project_id === reopen?.project_id &&
    Boolean(result?.project_id) &&
    result?.project_id === reopen?.project_id;
  const sameSupervisorRoleSession =
    Boolean(open?.role_session_id) &&
    open?.role_session_id === reopen?.role_session_id &&
    Boolean(result?.supervisor_role_session_id) &&
    result?.supervisor_role_session_id === reopen?.supervisor_role_session_id;
  const sameWorkerRoleSession =
    Boolean(result?.worker_role_session_id) &&
    result?.worker_role_session_id === reopen?.worker_role_session_id;
  const sameReviewerRoleSession =
    Boolean(result?.reviewer_role_session_id) &&
    result?.reviewer_role_session_id === reopen?.reviewer_role_session_id;
  const sameExactIds = sameExactObjectIds(result, reopen);
  const sameCounts = sameClosedLoopCounts(result, reopen);
  const sameM1Revision =
    firstRegistry?.registry_revision != null &&
    firstRegistry?.registry_revision === secondRegistry?.registry_revision;
  const sameM1Bytes =
    Boolean(firstRegistry?.sha256) && firstRegistry?.sha256 === secondRegistry?.sha256;
  const noSecondEffect =
    runtime &&
    runtimeRepeat &&
    runtime.grants === runtimeRepeat.grants &&
    runtime.dispatches === runtimeRepeat.dispatches &&
    runtime.durable_operations === runtimeRepeat.durable_operations &&
    runtime.execution_readbacks === runtimeRepeat.execution_readbacks;
  const receipt = {
    schema: "syn.m5r07.ordinary-control-launcher.v2",
    composition: COMPOSITION,
    not_legacy_composition: true,
    not_stage_closeout: true,
    ordinary_disposable_fixture_only: true,
    real_ordinary_tauri_constructor: true,
    acceptance_only_m3_terminal_fixture: true,
    synthetic_inputs: "SYNTHETIC_INPUTS",
    no_real_user_data: "NO_REAL_USER_DATA",
    not_deployed: "NOT_DEPLOYED",
    not_real_user_legacy_project: true,
    not_daily_run: true,
    not_release: true,
    m1_identity_source: profile.identitySource,
    window_capture: "NO_WINDOW_CAPTURE",
    profile_root: profile.root,
    profile_fingerprint: profile.profileFingerprint,
    run_id: profile.runId,
    cargo_build_exit: cargoBuild.exit_code,
    cargo_build_signal: cargoBuild.signal,
    binary_present: existsSync(binaryPath),
    display: process.env.DISPLAY || xvfb.display,
    xvfb_started: xvfb.started,
    first_process: firstProcess,
    second_process: secondProcess,
    open,
    rejected,
    seeded,
    retried,
    runtime,
    runtime_repeat: runtimeRepeat,
    report,
    review,
    result,
    reopen,
    failed,
    first_launch_error: firstError,
    reopen_error: reopenError,
    first_m1_registry: firstRegistry,
    second_m1_registry: secondRegistry,
    same_binding: sameBinding,
    same_project: sameProject,
    same_supervisor_role_session: Boolean(sameSupervisorRoleSession),
    same_worker_role_session: Boolean(sameWorkerRoleSession),
    same_reviewer_role_session: Boolean(sameReviewerRoleSession),
    same_exact_object_ids: Boolean(sameExactIds),
    same_closed_loop_counts: Boolean(sameCounts),
    same_m1_registry_revision: Boolean(sameM1Revision),
    same_m1_registry_bytes: Boolean(sameM1Bytes),
    no_second_effect: Boolean(noSecondEffect),
    m1_m3_installed:
      open?.derived_from === "backend_store" &&
      Boolean(open?.binding_id) &&
      Boolean(open?.role_session_id),
  };
  const receiptPath = join(profile.logs, "m5r07-ordinary-launcher-receipt.json");
  await writeFile(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`, {
    encoding: "utf8",
    mode: MODE_0600,
  });
  process.stdout.write(`${JSON.stringify({ receipt_path: receiptPath, ...receipt }, null, 2)}\n`);
  const passed =
    cargoBuild.exit_code === 0 &&
    Boolean(open?.binding_id) &&
    Boolean(rejected) &&
    seeded?.formal_receipt_present === true &&
    rejected?.grants === 0 &&
    rejected?.durable_operations === 0 &&
    retried?.grants === (seeded?.grants ?? 0) + 1 &&
    Boolean(runtime) &&
    Boolean(report) &&
    Boolean(review) &&
    result?.exact_chain_complete === true &&
    result?.independent_reviewer === true &&
    reopen?.exact_chain_complete === true &&
    reopen?.independent_reviewer === true &&
    Boolean(noSecondEffect) &&
    Boolean(reopen?.binding_id) &&
    sameBinding &&
    sameProject &&
    Boolean(sameSupervisorRoleSession) &&
    Boolean(sameWorkerRoleSession) &&
    Boolean(sameReviewerRoleSession) &&
    Boolean(sameExactIds) &&
    Boolean(sameCounts) &&
    Boolean(sameM1Revision) &&
    Boolean(sameM1Bytes) &&
    !failed;
  process.exitCode = passed ? 0 : 1;
}

await main();
