import { createHash, randomBytes } from "node:crypto";
import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  realpath,
  writeFile,
} from "node:fs/promises";
import { spawn } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const PROFILE_ENV = "SYN_R4_ACCEPTANCE_PROFILE";
const PROFILE_PURPOSE = "syn-r4-isolated-runtime-profile";
const PROFILE_SCHEMA_VERSION = 1;
const ROOT_PREFIX = "syn-r4-acceptance-";
const FIXTURE_PREFIX = "SYN R4 ISOLATED ACCEPTANCE ";
const PROFILE_FILE_NAME = "profile.json";
const RECEIPT_FILE_NAME = "preflight-receipt.json";
const UI_INSPECTION_FILE_NAME = "ui-inspection.json";
const PRELAUNCH_ROOT_ENTRY_NAMES = [
  PROFILE_FILE_NAME,
  "fixture",
  "workflow-state",
  "app-data",
  "codex-db",
  "logs",
];
const UI_INSPECTION_RELATIVE_PATH = join("logs", UI_INSPECTION_FILE_NAME);
const PROFILE_TTL_MS = 60 * 60 * 1000;
const MODE_0700 = 0o700;
const MODE_0600 = 0o600;
const MAX_UI_INSPECTION_BYTES = 4 * 1024;
const ACCEPTANCE_RUNTIME_PROFILE_INITIALIZATION_EXIT_CODE = 78;
const ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE = 79;
const UI_INSPECTION_SCHEMA_VERSION = 1;
const PRE_LIST_SIGKILL_DIAGNOSTIC_SCHEMA_VERSION = 1;
const PARENT_CAPTURE_SIGNALS = ["SIGTERM", "SIGINT", "SIGHUP"];
const PROCESS_RELATION_QUERY_MAX_BYTES = 512;
const EXTERNAL_UI_INSPECTION_PROVENANCE =
  "external_computer_use_ui_observation";
const PENDING_UI_INSPECTION_PROVENANCE = "launcher_pending_ui_observation";
const UI_INSPECTION_FAILURE_FAMILIES = new Set([
  "not_observed_by_launcher",
  "ui_observation_missing",
  "sky_target_discovery",
  "sky_attach",
  "home_ui_read",
  "non_synthetic_content",
  "screenshot_persist",
]);
const desktopRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tauriCliPath = resolve(
  desktopRoot,
  "../tauri-capability-probe/.tauri-cli/bin/cargo-tauri",
);
const CODESIGN_PATH = "/usr/bin/codesign";
const DEBUG_APP_BUNDLE_NAME = "CodexGovernanceWorkbench";
const DEBUG_APP_BUNDLE_IDENTIFIER = "local.codex.governance.workbench";
const DEBUG_APP_BUNDLE_RELATIVE_PATH =
  "src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app";
const DEBUG_APP_EXECUTABLE_RELATIVE_PATH =
  "src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app/Contents/MacOS/codex-governance-workbench";
const BUNDLE_BUILD_CONFIG = "{\"bundle\":{\"active\":true}}";
const debugAppBundlePath = resolve(
  desktopRoot,
  DEBUG_APP_BUNDLE_RELATIVE_PATH,
);
const debugAppExecutablePath = resolve(
  desktopRoot,
  DEBUG_APP_EXECUTABLE_RELATIVE_PATH,
);

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function makeRunId() {
  return `syn-r4-${randomBytes(8).toString("hex")}`;
}

function isContainedBy(root, candidate) {
  const pathRelativeToRoot = relative(root, candidate);
  return (
    pathRelativeToRoot !== "" &&
    pathRelativeToRoot !== ".." &&
    !pathRelativeToRoot.startsWith(`..${sep}`) &&
    !isAbsolute(pathRelativeToRoot)
  );
}

async function ensurePrivateDirectory(path) {
  await mkdir(path, { recursive: true, mode: MODE_0700 });
  await chmod(path, MODE_0700);
  const metadata = await lstat(path);
  if (!metadata.isDirectory() || metadata.isSymbolicLink()) {
    throw new Error("isolated runtime path must be a regular directory");
  }
  if ((metadata.mode & 0o777) !== MODE_0700) {
    throw new Error("isolated runtime directory permissions must be 0700");
  }
}

async function createIsolatedRoot() {
  const canonicalTempDirectory = await realpath(tmpdir());
  const createdRoot = await mkdtemp(join(canonicalTempDirectory, ROOT_PREFIX));
  await chmod(createdRoot, MODE_0700);
  const root = await realpath(createdRoot);
  const metadata = await lstat(root);
  if (
    dirname(root) !== canonicalTempDirectory ||
    !root.split(sep).at(-1)?.startsWith(ROOT_PREFIX) ||
    !metadata.isDirectory() ||
    metadata.isSymbolicLink() ||
    (metadata.mode & 0o777) !== MODE_0700
  ) {
    throw new Error("isolated runtime root did not satisfy the preflight contract");
  }
  return root;
}

function stableId(value) {
  let output = "";
  for (const character of value) {
    const code = character.charCodeAt(0);
    const isAsciiAlphanumeric =
      (code >= 48 && code <= 57) ||
      (code >= 65 && code <= 90) ||
      (code >= 97 && code <= 122);
    if (isAsciiAlphanumeric) {
      output += character.toLowerCase();
    } else if (!output.endsWith("-")) {
      output += "-";
    }
  }
  return output.replace(/^-+|-+$/g, "").slice(0, 96);
}

function buildFixtureIdentity(root, runId) {
  const projectRelativePath = `fixture/${FIXTURE_PREFIX}${runId}`;
  const projectRoot = resolve(root, projectRelativePath);
  if (!isContainedBy(root, projectRoot)) {
    throw new Error("synthetic project root escaped the isolated runtime root");
  }
  const canonicalProjectId = stableId(projectRoot);
  return {
    projectId: `project:${canonicalProjectId}`,
    projectRelativePath,
    projectRoot,
    runId,
    workflowId: `workflow:${canonicalProjectId}:default`,
  };
}

function buildProfile(identity, nowMs) {
  return {
    schema_version: PROFILE_SCHEMA_VERSION,
    purpose: PROFILE_PURPOSE,
    run_id: identity.runId,
    expires_at_ms: nowMs + PROFILE_TTL_MS,
    project: {
      id: identity.projectId,
      relative_path: identity.projectRelativePath,
    },
    workflow: {
      id: identity.workflowId,
    },
    paths: {
      index_relative_path: "fixture/codex-index.json",
      tasks_relative_path: "fixture/tasks.md",
      workflow_state_relative_path: "workflow-state/workflow-state.v0.json",
      app_data_relative_path: "app-data",
      canvas_relative_path: "app-data/canvas-v1",
      codex_db_relative_path: "codex-db/state.sqlite",
    },
  };
}

function pendingUiInspection(runHash) {
  return {
    schema_version: UI_INSPECTION_SCHEMA_VERSION,
    run_hash: runHash,
    ui_inspection_attempted: false,
    ui_inspection_completed: false,
    synthetic_home_verified: false,
    screenshot_saved: false,
    ui_inspection_failure_family: "not_observed_by_launcher",
    ui_inspection_provenance: PENDING_UI_INSPECTION_PROVENANCE,
  };
}

function invalidUiInspection() {
  return {
    ui_inspection_attempted: false,
    ui_inspection_completed: false,
    synthetic_home_verified: false,
    screenshot_saved: false,
    ui_inspection_failure_family: "ui_observation_invalid",
    ui_inspection_provenance: "launcher_observation_file_validation",
  };
}

function missingUiInspection() {
  return {
    ui_inspection_attempted: false,
    ui_inspection_completed: false,
    synthetic_home_verified: false,
    screenshot_saved: false,
    ui_inspection_failure_family: "ui_observation_missing",
    ui_inspection_provenance: "launcher_observation_file_validation",
  };
}

async function readUiInspection(uiInspectionPath, runHash) {
  try {
    const metadata = await lstat(uiInspectionPath);
    if (
      !metadata.isFile() ||
      metadata.isSymbolicLink() ||
      metadata.nlink !== 1 ||
      (metadata.mode & 0o777) !== MODE_0600 ||
      metadata.size > MAX_UI_INSPECTION_BYTES
    ) {
      return invalidUiInspection();
    }
    const observation = JSON.parse(await readFile(uiInspectionPath, "utf8"));
    const expectedKeys = [
      "schema_version",
      "run_hash",
      "ui_inspection_attempted",
      "ui_inspection_completed",
      "synthetic_home_verified",
      "screenshot_saved",
      "ui_inspection_failure_family",
      "ui_inspection_provenance",
    ];
    if (
      !observation ||
      typeof observation !== "object" ||
      Array.isArray(observation) ||
      Object.keys(observation).length !== expectedKeys.length ||
      !expectedKeys.every((key) => Object.hasOwn(observation, key)) ||
      observation.schema_version !== UI_INSPECTION_SCHEMA_VERSION ||
      observation.run_hash !== runHash ||
      typeof observation.ui_inspection_attempted !== "boolean" ||
      typeof observation.ui_inspection_completed !== "boolean" ||
      typeof observation.synthetic_home_verified !== "boolean" ||
      typeof observation.screenshot_saved !== "boolean" ||
      !(
        observation.ui_inspection_failure_family === null ||
        UI_INSPECTION_FAILURE_FAMILIES.has(
          observation.ui_inspection_failure_family,
        )
      ) ||
      observation.ui_inspection_provenance !==
        EXTERNAL_UI_INSPECTION_PROVENANCE ||
      !observation.ui_inspection_attempted ||
      (observation.ui_inspection_completed &&
        !observation.ui_inspection_attempted) ||
      (observation.synthetic_home_verified &&
        (!observation.ui_inspection_completed ||
          !observation.ui_inspection_attempted)) ||
      (observation.screenshot_saved && !observation.synthetic_home_verified) ||
      (observation.synthetic_home_verified &&
        observation.ui_inspection_failure_family !== null &&
        observation.ui_inspection_failure_family !== "screenshot_persist") ||
      (!observation.synthetic_home_verified &&
        observation.ui_inspection_failure_family === null)
    ) {
      return invalidUiInspection();
    }
    return observation;
  } catch (error) {
    if (error && typeof error === "object" && error.code === "ENOENT") {
      return missingUiInspection();
    }
    return invalidUiInspection();
  }
}

function startupFailureFamily(launchResult) {
  if (
    launchResult.exit_code ===
    ACCEPTANCE_RUNTIME_PROFILE_INITIALIZATION_EXIT_CODE
  ) {
    return "profile_initialization_failure";
  }
  if (launchResult.exit_code === ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE) {
    return "app_state_initialization_failure";
  }
  return null;
}

function completedUiInspection(uiInspection) {
  return (
    uiInspection.ui_inspection_attempted &&
    uiInspection.ui_inspection_completed &&
    uiInspection.synthetic_home_verified &&
    uiInspection.screenshot_saved
  );
}

function synExitDisposition(launchResult, uiInspection) {
  if (!launchResult.launched) {
    return "not_launched";
  }
  const startupFailure = startupFailureFamily(launchResult);
  if (startupFailure) {
    return startupFailure;
  }
  if (
    launchResult.signal === "SIGTERM" &&
    completedUiInspection(uiInspection)
  ) {
    return "terminated_after_completed_ui_inspection";
  }
  if (launchResult.exit_code === 0) {
    if (completedUiInspection(uiInspection)) {
      return "normal_exit_after_completed_ui_observation";
    }
    return "exit_zero_without_completed_ui_observation";
  }
  return "unexpected_exit";
}

function buildWorkflowState(identity, projectRoot, timestamp) {
  return {
    schema_version: "workflow_state_v0",
    workflow_version: 1,
    revision: 0,
    workspace_id: `workspace:${identity.runId}`,
    created_at: timestamp,
    updated_at: timestamp,
    source_kind: "isolated_acceptance_fixture",
    permission_level: "user_confirmed_write",
    projects: [
      {
        project_id: identity.projectId,
        display_name: `${FIXTURE_PREFIX}${identity.runId}`,
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
        workflow_id: identity.workflowId,
        workflow_version: 1,
        project_id: identity.projectId,
        title: `${FIXTURE_PREFIX}${identity.runId} workflow`,
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
  };
}

async function writeJson(path, value, mode = MODE_0600) {
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode,
  });
  await chmod(path, mode);
  const metadata = await lstat(path);
  if (!metadata.isFile() || metadata.isSymbolicLink()) {
    throw new Error("isolated runtime file must be a regular file");
  }
}

async function assertPrelaunchRootLayout(root) {
  const rootEntryNames = (await readdir(root)).sort();
  const expectedNames = [...PRELAUNCH_ROOT_ENTRY_NAMES].sort();
  if (
    rootEntryNames.length !== expectedNames.length ||
    rootEntryNames.some((name, index) => name !== expectedNames[index])
  ) {
    throw new Error("isolated prelaunch root layout did not match the fixed contract");
  }
  if ((await readdir(join(root, "logs"))).length !== 0) {
    throw new Error("isolated prelaunch logs must be empty");
  }
}

async function assertFreshDebugAppExecutable(buildStartedAtMs) {
  const metadata = await lstat(debugAppExecutablePath);
  if (
    !metadata.isFile() ||
    metadata.isSymbolicLink() ||
    metadata.mtimeMs < buildStartedAtMs
  ) {
    throw new Error("isolated debug app bundle executable was not rebuilt for this launch");
  }
}

async function sealAndVerifyDebugAppBundle(environment) {
  const childOptions = {
    cwd: desktopRoot,
    env: environment,
    shell: false,
    stdio: "ignore",
  };
  const sealResult = await runChild(
    CODESIGN_PATH,
    ["--force", "--deep", "--sign", "-", debugAppBundlePath],
    childOptions,
  );
  if (
    !sealResult.launched ||
    sealResult.exit_code !== 0 ||
    sealResult.signal !== null
  ) {
    throw new Error("fresh debug app bundle ad-hoc seal failed");
  }
  const verificationResult = await runChild(
    CODESIGN_PATH,
    ["--verify", "--deep", "--strict", debugAppBundlePath],
    childOptions,
  );
  if (
    !verificationResult.launched ||
    verificationResult.exit_code !== 0 ||
    verificationResult.signal !== null
  ) {
    throw new Error("fresh debug app bundle strict verification failed");
  }
}

async function createFixture(root, identity, profile) {
  const profilePath = join(root, PROFILE_FILE_NAME);
  const fixtureRoot = join(root, "fixture");
  const projectRoot = identity.projectRoot;
  const workflowStateDirectory = join(root, "workflow-state");
  const appDataRoot = join(root, "app-data");
  const codexDbDirectory = join(root, "codex-db");
  const logsRoot = join(root, "logs");
  const uiInspectionPath = join(root, UI_INSPECTION_RELATIVE_PATH);
  const expectedRoots = [
    profilePath,
    fixtureRoot,
    projectRoot,
    workflowStateDirectory,
    appDataRoot,
    codexDbDirectory,
    logsRoot,
    join(root, profile.paths.index_relative_path),
    join(root, profile.paths.tasks_relative_path),
    join(root, profile.paths.workflow_state_relative_path),
    join(root, profile.paths.canvas_relative_path),
    join(root, profile.paths.codex_db_relative_path),
  ];
  if (!expectedRoots.every((path) => isContainedBy(root, path))) {
    throw new Error("isolated fixture path escaped the isolated runtime root");
  }

  await writeJson(profilePath, profile);
  await ensurePrivateDirectory(fixtureRoot);
  await ensurePrivateDirectory(projectRoot);
  await ensurePrivateDirectory(workflowStateDirectory);
  await ensurePrivateDirectory(appDataRoot);
  await ensurePrivateDirectory(codexDbDirectory);
  await ensurePrivateDirectory(logsRoot);

  const timestamp = new Date().toISOString();
  await writeJson(join(root, profile.paths.index_relative_path), {
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
  await writeFile(join(root, profile.paths.tasks_relative_path), "", {
    encoding: "utf8",
    flag: "wx",
    mode: MODE_0600,
  });
  await chmod(join(root, profile.paths.tasks_relative_path), MODE_0600);
  await writeJson(
    join(root, profile.paths.workflow_state_relative_path),
    buildWorkflowState(identity, projectRoot, timestamp),
  );
  await assertPrelaunchRootLayout(root);

  return {
    indexPath: join(root, profile.paths.index_relative_path),
    projectRoot,
    tasksPath: join(root, profile.paths.tasks_relative_path),
    uiInspectionPath,
    workflowStatePath: join(root, profile.paths.workflow_state_relative_path),
  };
}

function runChild(command, args, options, onSpawn) {
  return new Promise((resolveChild) => {
    const child = spawn(command, args, options);
    onSpawn?.(child);
    let settled = false;
    const settle = (result) => {
      if (settled) {
        return;
      }
      settled = true;
      resolveChild(result);
    };
    child.once("error", () => {
      settle({ exit_code: null, launched: false, signal: null });
    });
    child.once("exit", (code, signal) => {
      settle({ exit_code: code, launched: true, signal: signal ?? null });
    });
  });
}

function pendingChildLifecycle() {
  return { observed: false, exit_code: null, signal: null };
}

function unavailableProcessRelation() {
  return {
    observed: false,
    child_parent_is_launcher: null,
    same_process_group: null,
    same_session: null,
    observation_failure_family: "unavailable",
  };
}

function createPreListSigkillDiagnostic() {
  return {
    schema_version: PRE_LIST_SIGKILL_DIAGNOSTIC_SCHEMA_VERSION,
    launcher_child_kill_attempted: false,
    launcher_self_signal_reraise_after_receipt: false,
    parent_signal_reraise_after_receipt: null,
    parent_received_signals: {
      SIGTERM: false,
      SIGINT: false,
      SIGHUP: false,
    },
    child_exit: pendingChildLifecycle(),
    child_close: pendingChildLifecycle(),
    process_relation: unavailableProcessRelation(),
  };
}

function parseParentChildProcessRelation(output, parentPid, childPid) {
  const records = new Map();
  for (const line of output.trim().split("\n")) {
    if (!line.trim()) {
      continue;
    }
    const fields = line.trim().split(/\s+/);
    if (fields.length !== 4 || fields.some((field) => !/^\d+$/.test(field))) {
      return unavailableProcessRelation();
    }
    const [pid, parentProcessId, processGroupId, sessionId] = fields.map(Number);
    if (
      ![pid, parentProcessId, processGroupId, sessionId].every(
        Number.isSafeInteger,
      )
    ) {
      return unavailableProcessRelation();
    }
    records.set(pid, {
      parentProcessId,
      processGroupId,
      sessionId,
    });
  }
  const parent = records.get(parentPid);
  const child = records.get(childPid);
  if (!parent || !child || records.size !== 2) {
    return unavailableProcessRelation();
  }
  return {
    observed: true,
    child_parent_is_launcher: child.parentProcessId === parentPid,
    same_process_group: child.processGroupId === parent.processGroupId,
    same_session: child.sessionId === parent.sessionId,
    observation_failure_family: null,
  };
}

function observeParentChildProcessRelation(parentPid, childPid) {
  if (!Number.isSafeInteger(parentPid) || !Number.isSafeInteger(childPid)) {
    return Promise.resolve(unavailableProcessRelation());
  }
  return new Promise((resolveRelation) => {
    const ps = spawn(
      "/bin/ps",
      ["-o", "pid=,ppid=,pgid=,sess=", "-p", `${parentPid},${childPid}`],
      {
        shell: false,
        stdio: ["ignore", "pipe", "ignore"],
      },
    );
    let settled = false;
    let output = "";
    let outputTooLarge = false;
    const settle = (relation) => {
      if (settled) {
        return;
      }
      settled = true;
      resolveRelation(relation);
    };
    ps.stdout?.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      if (output.length + text.length > PROCESS_RELATION_QUERY_MAX_BYTES) {
        outputTooLarge = true;
        return;
      }
      output += text;
    });
    ps.once("error", () => {
      settle(unavailableProcessRelation());
    });
    ps.once("close", (code, signal) => {
      if (code !== 0 || signal !== null || outputTooLarge) {
        settle(unavailableProcessRelation());
        return;
      }
      settle(parseParentChildProcessRelation(output, parentPid, childPid));
    });
  });
}

function installParentSignalLedger() {
  const receivedSignals = {
    SIGTERM: false,
    SIGINT: false,
    SIGHUP: false,
  };
  let firstReceivedSignal = null;
  const handlers = new Map(
    PARENT_CAPTURE_SIGNALS.map((signal) => [
      signal,
      () => {
        receivedSignals[signal] = true;
        firstReceivedSignal ??= signal;
      },
    ]),
  );
  for (const [signal, handler] of handlers) {
    process.on(signal, handler);
  }
  return {
    snapshot() {
      return { ...receivedSignals };
    },
    firstReceivedSignal() {
      return firstReceivedSignal;
    },
    dispose() {
      for (const [signal, handler] of handlers) {
        process.removeListener(signal, handler);
      }
    },
  };
}

function runDiagnosedChild(command, args, options, onSpawn) {
  const diagnostic = createPreListSigkillDiagnostic();
  const parentSignalLedger = installParentSignalLedger();
  return new Promise((resolveChild) => {
    const child = spawn(command, args, options);
    const processRelation = observeParentChildProcessRelation(
      process.pid,
      child.pid,
    );
    let settled = false;
    const settle = async (result) => {
      if (settled) {
        return;
      }
      settled = true;
      diagnostic.parent_received_signals = parentSignalLedger.snapshot();
      diagnostic.process_relation = await processRelation;
      const parentSignalToReraise = parentSignalLedger.firstReceivedSignal();
      if (parentSignalToReraise) {
        diagnostic.launcher_self_signal_reraise_after_receipt = true;
        diagnostic.parent_signal_reraise_after_receipt = parentSignalToReraise;
      }
      parentSignalLedger.dispose();
      resolveChild({
        launch_result: result,
        diagnostic,
        parent_signal_to_reraise: parentSignalToReraise,
      });
    };
    child.once("error", () => {
      void settle({ exit_code: null, launched: false, signal: null });
    });
    child.once("exit", (code, signal) => {
      diagnostic.child_exit = {
        observed: true,
        exit_code: code,
        signal: signal ?? null,
      };
    });
    child.once("close", (code, signal) => {
      diagnostic.child_close = {
        observed: true,
        exit_code: code,
        signal: signal ?? null,
      };
      void settle({ exit_code: code, launched: true, signal: signal ?? null });
    });
    onSpawn?.(child);
  });
}

function redactedReceipt(
  identity,
  fixture,
  profile,
  runHash,
  buildResult,
  launchResult,
  uiInspection,
  preListSigkillDiagnostic,
) {
  const rootContainment = {
    app_data: isContainedBy(fixture.root, join(fixture.root, profile.paths.app_data_relative_path)),
    canvas: isContainedBy(fixture.root, join(fixture.root, profile.paths.canvas_relative_path)),
    codex_db: isContainedBy(fixture.root, join(fixture.root, profile.paths.codex_db_relative_path)),
    index: isContainedBy(fixture.root, fixture.indexPath),
    logs: isContainedBy(fixture.root, join(fixture.root, "logs")),
    project: isContainedBy(fixture.root, fixture.projectRoot),
    recovery_backups: isContainedBy(
      fixture.root,
      join(fixture.root, "app-data/knowledge-workspace-recovery"),
    ),
    tasks: isContainedBy(fixture.root, fixture.tasksPath),
    vault: isContainedBy(fixture.root, join(fixture.root, "app-data/knowledge-vault")),
    workflow_state: isContainedBy(fixture.root, fixture.workflowStatePath),
  };
  return {
    schema_version: "syn_r4_isolated_preflight_receipt.v3",
    run_hash: runHash,
    declared_fixture_path_containment: rootContainment,
    fixture_path_containment_provenance:
      "launcher_declared_fixture_path_projection",
    fixture_synthetic_identity_hash: sha256(
      `${identity.projectId}\u0000${identity.workflowId}`,
    ),
    profile_declared_session_source: "IndexOnly",
    build: buildResult,
    syn: launchResult,
    syn_exit_disposition: synExitDisposition(launchResult, uiInspection),
    ui_inspection_attempted: uiInspection.ui_inspection_attempted,
    ui_inspection_completed: uiInspection.ui_inspection_completed,
    synthetic_home_verified: uiInspection.synthetic_home_verified,
    screenshot_saved: uiInspection.screenshot_saved,
    ui_inspection_failure_family: uiInspection.ui_inspection_failure_family,
    ui_inspection_provenance: uiInspection.ui_inspection_provenance,
    pre_list_sigkill_diagnostic: preListSigkillDiagnostic,
  };
}

const initialHome = process.env.HOME;
const initialCodexHome = process.env.CODEX_HOME;
const homeInitialViewConfigPinned =
  !Object.hasOwn(process.env, "VITE_STAGE_K_INITIAL_VIEW") ||
  process.env.VITE_STAGE_K_INITIAL_VIEW === "home";
let root;
let identity;
let profile;
let fixture;
let runHash;
let buildResult = { exit_code: null, launched: false, signal: null };
let launchResult = { exit_code: null, launched: false, signal: null };
let preListSigkillDiagnostic = createPreListSigkillDiagnostic();
let parentSignalToReraise = null;
let failureStage = null;
let uiInspection = pendingUiInspection("");

try {
  if (!homeInitialViewConfigPinned) {
    failureStage = "initial_view";
    process.exitCode = 1;
  } else {
    root = await createIsolatedRoot();
    identity = buildFixtureIdentity(root, makeRunId());
    profile = buildProfile(identity, Date.now());
    runHash = sha256(identity.runId);
    const fixturePaths = await createFixture(root, identity, profile);
    fixture = { root, ...fixturePaths };

    const normalBuildEnvironment = { ...process.env };
    normalBuildEnvironment.VITE_STAGE_K_INITIAL_VIEW = "home";
    delete normalBuildEnvironment[PROFILE_ENV];
    const bundleBuildStartedAtMs = Date.now();
    buildResult = await runChild(
      tauriCliPath,
      [
        "build",
        "--debug",
        "--bundles",
        "app",
        "--config",
        BUNDLE_BUILD_CONFIG,
      ],
      {
        cwd: desktopRoot,
        env: normalBuildEnvironment,
        shell: false,
        stdio: "ignore",
      },
    );
    if (!buildResult.launched || buildResult.exit_code !== 0) {
      failureStage = "normal_build";
      process.exitCode = 1;
    } else {
      try {
        await assertFreshDebugAppExecutable(bundleBuildStartedAtMs);
      } catch {
        failureStage = "bundled_target";
        process.exitCode = 1;
      }
      if (!failureStage) {
        try {
          await sealAndVerifyDebugAppBundle(normalBuildEnvironment);
        } catch {
          failureStage = "bundle_integrity";
          process.exitCode = 1;
        }
      }
      if (!failureStage) {
        const finalSynEnvironment = {
          ...normalBuildEnvironment,
          [PROFILE_ENV]: join(root, PROFILE_FILE_NAME),
        };
        const diagnosedLaunch = await runDiagnosedChild(
          debugAppExecutablePath,
          [],
          {
            cwd: desktopRoot,
            env: finalSynEnvironment,
            shell: false,
            stdio: "ignore",
          },
          (child) => {
            process.stdout.write(
              `${JSON.stringify({
                schema_version: "syn_r4_ui_inspection_ready.v1",
                run_hash: runHash,
                syn_pid: child.pid ?? null,
                target_bundle_name: DEBUG_APP_BUNDLE_NAME,
                target_bundle_identifier: DEBUG_APP_BUNDLE_IDENTIFIER,
                ui_inspection_path: fixture.uiInspectionPath,
              })}\n`,
            );
          },
        );
        launchResult = diagnosedLaunch.launch_result;
        preListSigkillDiagnostic = diagnosedLaunch.diagnostic;
        parentSignalToReraise = diagnosedLaunch.parent_signal_to_reraise;
        uiInspection = await readUiInspection(fixture.uiInspectionPath, runHash);
        const controlledTerminationAfterCompletedInspection =
          launchResult.signal === "SIGTERM" &&
          completedUiInspection(uiInspection);
        const startupFailure = startupFailureFamily(launchResult);
        if (!launchResult.launched) {
          failureStage = "isolated_syn_launch";
          process.exitCode = 1;
        } else if (startupFailure) {
          failureStage = `isolated_syn_${startupFailure}`;
          process.exitCode = 1;
        } else if (
          !uiInspection.ui_inspection_completed ||
          !uiInspection.synthetic_home_verified ||
          !uiInspection.screenshot_saved
        ) {
          failureStage = "ui_inspection";
          process.exitCode = 1;
        } else if (
          launchResult.exit_code !== 0 &&
          !controlledTerminationAfterCompletedInspection
        ) {
          failureStage = "isolated_syn_exit";
          process.exitCode = 1;
        }
      }
    }
  }
} catch {
  failureStage ??= "fixture_or_launcher";
  process.exitCode = 1;
} finally {
  if (root && identity && profile && fixture) {
    const receipt = {
      ...redactedReceipt(
        identity,
        fixture,
        profile,
        runHash,
        buildResult,
        launchResult,
        uiInspection,
        preListSigkillDiagnostic,
      ),
      ...(failureStage ? { failure_stage: failureStage } : {}),
      environment_unchanged:
        process.env.HOME === initialHome && process.env.CODEX_HOME === initialCodexHome,
      home_initial_view_config_pinned: homeInitialViewConfigPinned,
    };
    await writeJson(join(root, RECEIPT_FILE_NAME), receipt);
    process.stdout.write(`${JSON.stringify(receipt)}\n`);
    if (parentSignalToReraise) {
      process.kill(process.pid, parentSignalToReraise);
    }
  }
}
