import { createHash, randomBytes } from "node:crypto";
import { existsSync } from "node:fs";
import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  realpath,
  unlink,
  writeFile,
} from "node:fs/promises";
import { spawn } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const PROFILE_ENV = "SYN_R4_ACCEPTANCE_PROFILE";
const REENTRY_CAPABILITY_ENV = "SYN_R4_REENTRY_CAPABILITY";
const M2_REFERENCE_SLICE_DRIVER_ENV = "SYN_M2_R4_REFERENCE_SLICE_DRIVER";
const M2_REFERENCE_SLICE_ATTEMPT_ENV = "SYN_M2_R4_REFERENCE_SLICE_ATTEMPT";
const M2_REFERENCE_SLICE_PHASE_ENV = "SYN_M2_R4_REFERENCE_SLICE_PHASE";
const M2_REFERENCE_SLICE_NONCE_ENV = "SYN_M2_R4_REFERENCE_SLICE_NONCE";
const M2_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV =
  "SYN_M2_R4_REFERENCE_SLICE_EXTERNAL_EFFECT";
const M2_REFERENCE_SLICE_DRIVER_VALUE = "workflow-state-reference-slice-v1";
const M2_REFERENCE_SLICE_EXTERNAL_EFFECT_VALUE =
  "workflow-state-external-effect-v1";
const M2_REFERENCE_SLICE_MODE_ARG = "--m2-reference-slice";
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
const REFERENCE_DRIVER_GATE_TIMEOUT_MS = 20_000;
const REFERENCE_DRIVER_OUTPUT_MAX_BYTES = 16 * 1024;
const REFERENCE_DRIVER_RESULT_PREFIX = "m2-reference-slice-";
const REFERENCE_DRIVER_RESULT_SUFFIX = ".json";
const REFERENCE_PROVENANCE_SCHEMA_VERSION = "syn_m2_r4_reference_slice_provenance.v1";
const REFERENCE_INVOCATION_SCHEMA_VERSION = "syn_m2_r4_reference_slice_invocation.v1";
const REFERENCE_STORE_FINGERPRINT_SCHEMA_VERSION =
  "syn_m2_r4_reference_slice_store_fingerprint.v1";
const REFERENCE_PROVENANCE_SOURCE_PATHS = [
  "scripts/run-r4-isolated-app-preflight.mjs",
  "src/main.tsx",
  "src-tauri/src/acceptance_runtime_profile.rs",
  "src-tauri/src/index_host_app_entrypoints.rs",
  "src-tauri/src/m2_r4_reference_slice_driver.rs",
  "src-tauri/src/workbench_sqlite_repository.rs",
  "src-tauri/src/workflow_run_dispatch_entrypoints.rs",
];
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
const localTauriCapabilityProbeRoot = resolve(
  desktopRoot,
  "../tauri-capability-probe",
);
const sharedTauriCapabilityProbeRoot = resolve(
  desktopRoot,
  "../../../product-line/prototypes/tauri-capability-probe",
);
const localTauriCliPath = resolve(
  localTauriCapabilityProbeRoot,
  ".tauri-cli/bin/cargo-tauri",
);
const tauriCapabilityProbeRoot = existsSync(localTauriCliPath)
  ? localTauriCapabilityProbeRoot
  : sharedTauriCapabilityProbeRoot;
const tauriCliPath = resolve(
  tauriCapabilityProbeRoot,
  ".tauri-cli/bin/cargo-tauri",
);
const tauriCargoHome = resolve(tauriCapabilityProbeRoot, ".cargo-home");
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

function referenceDriverResultPath(root, attempt, phase = "run") {
  const phaseSuffix =
    phase === "external-effect"
      ? "-external-effect"
      : phase === "external-readback"
        ? "-external-readback"
        : "";
  return join(
    root,
    "runtime-artifacts",
    `${REFERENCE_DRIVER_RESULT_PREFIX}${attempt}${phaseSuffix}${REFERENCE_DRIVER_RESULT_SUFFIX}`,
  );
}

async function createReferenceFixture() {
  const root = await createIsolatedRoot();
  const identity = buildFixtureIdentity(root, makeRunId());
  const profile = buildProfile(identity, Date.now());
  const runHash = sha256(identity.runId);
  const reentryCapability = randomBytes(32).toString("hex");
  const fixturePaths = await createFixture(root, identity, profile);
  return {
    root,
    identity,
    profile,
    runHash,
    reentryCapability,
    fixture: { root, ...fixturePaths },
  };
}

function boundedAppend(current, chunk) {
  const next = `${current}${chunk.toString("utf8")}`;
  return next.length > REFERENCE_DRIVER_OUTPUT_MAX_BYTES
    ? next.slice(-REFERENCE_DRIVER_OUTPUT_MAX_BYTES)
    : next;
}

function referenceDriverFailureCode(output) {
  const match = output.match(/\bm2_r4_reference_slice_driver_[a-z_]+\b/);
  return match?.[0] ?? "unclassified";
}

function referenceCommandBinding(attempt) {
  const nonce = randomBytes(16).toString("hex");
  return {
    operation: "update_work_item_state",
    attempt,
    nonce,
    command_id: `workflow-state-sidecar.m2.r4:${attempt}:${nonce}`,
  };
}

function launchReferenceDriver(
  fixture,
  normalBuildEnvironment,
  attempt,
  phase,
  commandBinding = null,
  externalEffect = false,
) {
  const binding = ["run", "external-effect", "external-readback"].includes(phase)
    ? (commandBinding ?? referenceCommandBinding(attempt))
    : null;
  const environment = {
    ...normalBuildEnvironment,
    [PROFILE_ENV]: join(fixture.root, PROFILE_FILE_NAME),
    [REENTRY_CAPABILITY_ENV]: fixture.reentryCapability,
    [M2_REFERENCE_SLICE_DRIVER_ENV]: M2_REFERENCE_SLICE_DRIVER_VALUE,
    [M2_REFERENCE_SLICE_ATTEMPT_ENV]: attempt,
    [M2_REFERENCE_SLICE_PHASE_ENV]: phase,
    ...(binding ? { [M2_REFERENCE_SLICE_NONCE_ENV]: binding.nonce } : {}),
    ...(externalEffect
      ? {
          [M2_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV]:
            M2_REFERENCE_SLICE_EXTERNAL_EFFECT_VALUE,
        }
      : {}),
  };
  const child = spawn(debugAppExecutablePath, [], {
    cwd: desktopRoot,
    env: environment,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const invocation = {
    schema_version: REFERENCE_INVOCATION_SCHEMA_VERSION,
    started_at_unix_ms: Date.now(),
    launcher_pid: process.pid,
    launcher_ppid: process.ppid,
    syn_pid: child.pid ?? null,
    argv: [debugAppExecutablePath],
    cwd: desktopRoot,
    attempt,
    phase,
    external_effect_requested: externalEffect,
    ...(binding
      ? {
          command_binding: {
            operation: binding.operation,
            attempt: binding.attempt,
            command_id_sha256: sha256(binding.command_id),
            nonce_sha256: sha256(binding.nonce),
          },
        }
      : {}),
  };
  const processRelation = observeParentChildProcessRelation(process.pid, child.pid);
  let stdout = "";
  let stderr = "";
  let resolveGate;
  const gateReady = new Promise((resolve) => {
    resolveGate = resolve;
  });
  child.stdout?.on("data", (chunk) => {
    stdout = boundedAppend(stdout, chunk);
  });
  child.stderr?.on("data", (chunk) => {
    stderr = boundedAppend(stderr, chunk);
    const text = chunk.toString("utf8");
    const match = text.match(/acceptance_(?:m2_reference_)?gate_armed:([a-z-]+):/);
    if (match) {
      resolveGate(match[1]);
    }
  });
  const completed = new Promise((resolve) => {
    const settle = async (result) => {
      resolve({
        ...result,
        invocation: {
          ...invocation,
          completed_at_unix_ms: Date.now(),
          process_relation: await processRelation,
        },
      });
    };
    child.once("error", () => {
      resolveGate(null);
      void settle({ exit_code: null, launched: false, signal: null });
    });
    child.once("close", (exitCode, exitSignal) => {
      resolveGate(null);
      void settle({
        exit_code: exitCode,
        launched: true,
        signal: exitSignal ?? null,
      });
    });
  });
  return {
    child,
    command_binding: binding,
    completed,
    gateReady,
    output() {
      return { stdout, stderr };
    },
  };
}

async function runReferenceDriver(
  fixture,
  normalBuildEnvironment,
  attempt,
  phase = "run",
  commandBinding = null,
  externalEffect = false,
) {
  const launched = launchReferenceDriver(
    fixture,
    normalBuildEnvironment,
    attempt,
    phase,
    commandBinding,
    externalEffect,
  );
  const result = await launched.completed;
  return {
    ...result,
    syn_pid: launched.child.pid ?? null,
    ...launched.output(),
  };
}

async function waitForReferenceGate(launched, expectedGate) {
  const timer = new Promise((_, reject) => {
    setTimeout(
      () => reject(new Error(`reference driver gate timeout:${expectedGate}`)),
      REFERENCE_DRIVER_GATE_TIMEOUT_MS,
    );
  });
  const gate = await Promise.race([launched.gateReady, timer]);
  if (gate !== expectedGate) {
    throw new Error(`reference driver observed wrong gate:${gate}`);
  }
}

async function armReferenceGate(root, gate, commandBinding) {
  if (
    !commandBinding ||
    commandBinding.operation !== "update_work_item_state" ||
    !/^[a-z0-9-]{1,48}$/.test(commandBinding.attempt) ||
    !/^[a-f0-9]{32}$/.test(commandBinding.nonce) ||
    commandBinding.command_id !==
      `workflow-state-sidecar.m2.r4:${commandBinding.attempt}:${commandBinding.nonce}`
  ) {
    throw new Error("reference gate requires exact command binding");
  }
  const directory = join(root, "runtime-artifacts", "acceptance-gates");
  await ensurePrivateDirectory(join(root, "runtime-artifacts"));
  await ensurePrivateDirectory(directory);
  const path = join(directory, `${gate}.pause`);
  await writeFile(path, `${JSON.stringify(commandBinding)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: MODE_0600,
  });
  await chmod(path, MODE_0600);
  return path;
}

async function removeReferenceGate(path) {
  await unlink(path);
}

async function readReferenceDriverReceipt(root, attempt, phase = "run") {
  const path = referenceDriverResultPath(root, attempt, phase);
  const metadata = await lstat(path);
  if (
    !metadata.isFile() ||
    metadata.isSymbolicLink() ||
    (metadata.mode & 0o777) !== MODE_0600 ||
    metadata.size > MAX_UI_INSPECTION_BYTES
  ) {
    throw new Error("reference driver receipt metadata invalid");
  }
  return { path, value: JSON.parse(await readFile(path, "utf8")) };
}

async function archiveReferenceDriverReceipt(receipt, label) {
  const archivePath = receipt.path.replace(/\.json$/, `-${label}.json`);
  await writeFile(archivePath, `${JSON.stringify(receipt.value, null, 2)}\n`, "utf8");
  return { path: archivePath, value: receipt.value };
}

async function writeReferenceDriverDiagnostic(root, attempt, output) {
  const path = join(root, "logs", `m2-reference-slice-${attempt}.stderr.log`);
  await writeFile(path, output.slice(-REFERENCE_DRIVER_OUTPUT_MAX_BYTES), {
    encoding: "utf8",
    flag: "wx",
    mode: MODE_0600,
  });
  await chmod(path, MODE_0600);
  return path;
}

async function referenceFileFingerprint(path) {
  const metadata = await lstat(path);
  if (!metadata.isFile() || metadata.isSymbolicLink()) {
    throw new Error("reference evidence file metadata invalid");
  }
  const bytes = await readFile(path);
  return {
    mtime_ms: Math.floor(metadata.mtimeMs),
    sha256: sha256(bytes),
    size: metadata.size,
  };
}

async function optionalReferenceFileFingerprint(path) {
  try {
    return { present: true, ...(await referenceFileFingerprint(path)) };
  } catch (error) {
    if (error && typeof error === "object" && error.code === "ENOENT") {
      return { present: false };
    }
    throw error;
  }
}

async function referenceStoreFingerprints(fixture) {
  const databasePath = join(fixture.root, "runtime-artifacts", "workbench.sqlite");
  return {
    schema_version: REFERENCE_STORE_FINGERPRINT_SCHEMA_VERSION,
    database: await referenceFileFingerprint(databasePath),
    database_wal: await optionalReferenceFileFingerprint(`${databasePath}-wal`),
    database_shm: await optionalReferenceFileFingerprint(`${databasePath}-shm`),
    workflow_state: await referenceFileFingerprint(fixture.fixture.workflowStatePath),
  };
}

async function referenceGitText(args) {
  const child = spawn("/usr/bin/git", args, {
    cwd: desktopRoot,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  const result = await new Promise((resolve) => {
    child.stdout?.on("data", (chunk) => {
      stdout = boundedAppend(stdout, chunk);
    });
    child.stderr?.on("data", (chunk) => {
      stderr = boundedAppend(stderr, chunk);
    });
    child.once("error", () => resolve({ exit_code: null, signal: null }));
    child.once("close", (exit_code, signal) =>
      resolve({ exit_code, signal: signal ?? null }),
    );
  });
  if (result.exit_code !== 0 || result.signal !== null) {
    throw new Error(`reference git text failed:${args[0]}:${stderr.slice(0, 128)}`);
  }
  return stdout.trim();
}

async function referenceGitDigest(args, countNul = false) {
  const child = spawn("/usr/bin/git", args, {
    cwd: desktopRoot,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const digest = createHash("sha256");
  let nulCount = 0;
  let stderr = "";
  const result = await new Promise((resolve) => {
    child.stdout?.on("data", (chunk) => {
      digest.update(chunk);
      if (countNul) {
        for (const byte of chunk) {
          if (byte === 0) {
            nulCount += 1;
          }
        }
      }
    });
    child.stderr?.on("data", (chunk) => {
      stderr = boundedAppend(stderr, chunk);
    });
    child.once("error", () => resolve({ exit_code: null, signal: null }));
    child.once("close", (exit_code, signal) =>
      resolve({ exit_code, signal: signal ?? null }),
    );
  });
  if (result.exit_code !== 0 || result.signal !== null) {
    throw new Error(`reference git digest failed:${args[0]}:${stderr.slice(0, 128)}`);
  }
  return { sha256: digest.digest("hex"), ...(countNul ? { count: nulCount } : {}) };
}

async function referenceSuiteProvenance() {
  const source_files = [];
  for (const sourcePath of REFERENCE_PROVENANCE_SOURCE_PATHS) {
    const absolutePath = resolve(desktopRoot, sourcePath);
    requireReference(
      isContainedBy(desktopRoot, absolutePath),
      "reference provenance source containment",
    );
    source_files.push({
      path: sourcePath,
      ...(await referenceFileFingerprint(absolutePath)),
    });
  }
  const [head, tree, worktreeDiff, indexDiff, untracked] = await Promise.all([
    referenceGitText(["rev-parse", "HEAD"]),
    referenceGitText(["rev-parse", "HEAD^{tree}"]),
    referenceGitDigest(["diff", "--no-ext-diff", "--binary", "HEAD"]),
    referenceGitDigest(["diff", "--cached", "--no-ext-diff", "--binary"]),
    referenceGitDigest(["ls-files", "--others", "--exclude-standard", "-z"], true),
  ]);
  requireReference(/^[0-9a-f]{40}$/.test(head), "reference provenance HEAD");
  requireReference(/^[0-9a-f]{40}$/.test(tree), "reference provenance tree");
  return {
    schema_version: REFERENCE_PROVENANCE_SCHEMA_VERSION,
    captured_at_unix_ms: Date.now(),
    git: {
      head,
      tree,
      worktree_diff_sha256: worktreeDiff.sha256,
      index_diff_sha256: indexDiff.sha256,
      untracked_paths_sha256: untracked.sha256,
      untracked_count: untracked.count,
    },
    app_executable: await referenceFileFingerprint(debugAppExecutablePath),
    source_files,
  };
}

function referenceSuiteProvenanceIsStable(before, after) {
  return (
    before.git.head === after.git.head &&
    before.git.tree === after.git.tree &&
    before.git.worktree_diff_sha256 === after.git.worktree_diff_sha256 &&
    before.git.index_diff_sha256 === after.git.index_diff_sha256 &&
    before.git.untracked_paths_sha256 === after.git.untracked_paths_sha256 &&
    before.git.untracked_count === after.git.untracked_count &&
    before.app_executable.sha256 === after.app_executable.sha256 &&
    before.source_files.length === after.source_files.length &&
    before.source_files.every(
      (source, index) =>
        source.path === after.source_files[index]?.path &&
        source.sha256 === after.source_files[index]?.sha256,
    )
  );
}

async function sqliteReferenceLedgerCounts(databasePath) {
  const query =
    "SELECT (SELECT COUNT(*) FROM command_receipts), (SELECT COUNT(*) FROM events), (SELECT COUNT(*) FROM audit_records), (SELECT COUNT(*) FROM outbox_items), (SELECT COUNT(*) FROM current_snapshots);";
  const child = spawn("/usr/bin/sqlite3", [databasePath, query], {
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  const result = await new Promise((resolve) => {
    child.stdout?.on("data", (chunk) => {
      stdout = boundedAppend(stdout, chunk);
    });
    child.stderr?.on("data", (chunk) => {
      stderr = boundedAppend(stderr, chunk);
    });
    child.once("error", () => resolve({ exit_code: null, signal: null }));
    child.once("close", (exit_code, signal) =>
      resolve({ exit_code, signal: signal ?? null }),
    );
  });
  if (result.exit_code !== 0 || result.signal !== null) {
    throw new Error(`reference ledger query failed:${stderr}`);
  }
  const fields = stdout.trim().split("|");
  if (fields.length !== 5 || fields.some((field) => !/^\d+$/.test(field))) {
    throw new Error("reference ledger query output invalid");
  }
  return fields.map(Number);
}

async function holdReferenceSqliteWriteLock(databasePath) {
  const child = spawn("/usr/bin/sqlite3", [databasePath], {
    shell: false,
    stdio: ["pipe", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  let lockReady;
  let rejectLock;
  const ready = new Promise((resolve, reject) => {
    lockReady = resolve;
    rejectLock = reject;
  });
  const completed = new Promise((resolve) => {
    child.once("error", () => resolve({ exit_code: null, signal: null }));
    child.once("close", (exit_code, signal) =>
      resolve({ exit_code, signal: signal ?? null }),
    );
  });
  child.stdout?.on("data", (chunk) => {
    stdout = boundedAppend(stdout, chunk);
    if (stdout.includes("syn-m2-r4-sqlite-lock-acquired")) {
      lockReady();
    }
  });
  child.stderr?.on("data", (chunk) => {
    stderr = boundedAppend(stderr, chunk);
  });
  child.once("error", () => {
    rejectLock(new Error(`reference SQLite lock launch failed:${stderr}`));
  });
  child.stdin?.write(
    "PRAGMA busy_timeout=0;\nBEGIN EXCLUSIVE;\nSELECT 'syn-m2-r4-sqlite-lock-acquired';\n",
  );
  const timer = new Promise((_, reject) => {
    setTimeout(() => reject(new Error("reference SQLite lock acquisition timeout")), REFERENCE_DRIVER_GATE_TIMEOUT_MS);
  });
  await Promise.race([ready, timer]);
  return {
    async release() {
      // The input stream deliberately remains open after the lock marker so
      // that this fixture-owned connection retains its write lock until the
      // actual App has made its one product-boundary attempt.
      // sqlite3 treats EOF as an implicit rollback, so ending the stream is
      // the narrowest reliable release without a second mutable control API.
      child.stdin?.end();
      const result = await completed;
      if (result.exit_code !== 0 || result.signal !== null) {
        throw new Error(`reference SQLite lock release failed:${stderr}`);
      }
    },
  };
}

async function corruptReferenceSqliteHeader(databasePath) {
  const bytes = await readFile(databasePath);
  if (bytes.length < 16 || bytes.subarray(0, 16).toString("utf8") !== "SQLite format 3\u0000") {
    throw new Error("reference SQLite header unavailable for corrupt-input scenario");
  }
  const corrupted = Buffer.from(bytes);
  corrupted[0] ^= 0xff;
  await writeFile(databasePath, corrupted, { encoding: null, flag: "w" });
}

async function referenceWorkItemState(workflowStatePath) {
  const state = JSON.parse(await readFile(workflowStatePath, "utf8"));
  const item = state.work_items?.find(
    (candidate) => candidate.title === "SYN M2 R4 workflow-state reference slice",
  );
  if (!item || typeof item.state !== "string") {
    throw new Error("reference work item state unavailable");
  }
  return item.state;
}

function requireReference(condition, reason) {
  if (!condition) {
    throw new Error(`reference scenario assertion failed:${reason}`);
  }
}

function referencePassReceipt(
  receipt,
  attempt,
  // The ordinary workflow-state mutation projects JSON internally and
  // rebuildably. It is not an external side-effect, so it declares no
  // outbox item unless the R4-only same-slice effect is explicitly armed.
  expectedLedgerCounts = [1, 1, 1, 0, 1],
) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "receipt schema",
  );
  requireReference(receipt.attempt === attempt, "receipt attempt");
  requireReference(receipt.outcome === "PASS", "receipt outcome");
  requireReference(
    typeof receipt.receipt_id_hash === "string" &&
      receipt.receipt_id_hash === receipt.replay_receipt_id_hash,
    "replay receipt identity",
  );
  requireReference(receipt.reconciliation_green === true, "reconciliation green");
  requireReference(
    JSON.stringify(receipt.ledger_counts) === JSON.stringify(expectedLedgerCounts),
    "reference slice ledger counts",
  );
}

function referenceReadbackReceipt(receipt, attempt, expectedRunReceipt) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "readback receipt schema",
  );
  requireReference(receipt.attempt === attempt, "readback receipt attempt");
  requireReference(receipt.outcome === "READBACK", "readback receipt outcome");
  requireReference(
    receipt.receipt_id_hash === expectedRunReceipt.receipt_id_hash &&
      receipt.replay_receipt_id_hash === expectedRunReceipt.replay_receipt_id_hash,
    "readback receipt identity",
  );
  requireReference(receipt.work_item_state === "running", "readback work item state");
  requireReference(
    receipt.workflow_state_sha256 === expectedRunReceipt.workflow_state_sha256 &&
      receipt.database_sha256 === expectedRunReceipt.database_sha256 &&
      JSON.stringify(receipt.ledger_counts) === JSON.stringify(expectedRunReceipt.ledger_counts),
    "readback DB and JSON unchanged",
  );
  requireReference(receipt.reconciliation_green === true, "readback reconciliation green");
}

function referenceRecoveredReadbackReceipt(
  receipt,
  attempt,
  expectedLedgerCounts = [1, 1, 3, 1, 1],
) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "recovery readback receipt schema",
  );
  requireReference(receipt.attempt === attempt, "recovery readback receipt attempt");
  requireReference(receipt.outcome === "READBACK", "recovery readback receipt outcome");
  requireReference(
    typeof receipt.receipt_id_hash === "string" &&
      receipt.receipt_id_hash === receipt.replay_receipt_id_hash,
    "recovery readback receipt identity",
  );
  requireReference(receipt.work_item_state === "running", "recovery readback work item state");
  requireReference(receipt.reconciliation_green === true, "recovery readback reconciliation green");
  requireReference(
    // The failed projection keeps the two audit records from the original
    // atomic mutation; startup reconciliation appends exactly one recovery
    // audit before accepting the result command.
    JSON.stringify(receipt.ledger_counts) === JSON.stringify(expectedLedgerCounts),
    "recovery result ledger counts",
  );
}

function referenceSeedReceipt(receipt, attempt) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "seed receipt schema",
  );
  requireReference(receipt.attempt === attempt, "seed receipt attempt");
  requireReference(receipt.outcome === "SEEDED", "seed receipt outcome");
  requireReference(receipt.reconciliation_green === true, "seed reconciliation green");
  requireReference(
    JSON.stringify(receipt.ledger_counts) === JSON.stringify([0, 0, 0, 0, 0]),
    "seed ledger counts",
  );
}

function referenceExternalEffectReceipt(receipt, attempt, ownerRunReceipt, binding) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "external effect receipt schema",
  );
  requireReference(receipt.attempt === attempt, "external effect receipt attempt");
  requireReference(
    receipt.outcome === "EXTERNAL_EFFECT_PASS",
    "external effect receipt outcome",
  );
  requireReference(
    receipt.receipt_id_hash === ownerRunReceipt.receipt_id_hash,
    "external effect owner receipt identity",
  );
  const effect = receipt.external_effect;
  requireReference(Boolean(effect), "external effect receipt presence");
  requireReference(
    effect.owning_command_id_hash === sha256(binding.command_id) &&
      effect.owning_receipt_id_hash === ownerRunReceipt.receipt_id_hash &&
      effect.correlation_id_hash === sha256(binding.command_id),
    "external effect same owning command correlation",
  );
  requireReference(
    effect.status === "RESULT_RECEIVED" &&
      effect.lease_extension_count === 2 &&
      effect.delivery_attempt_count === 1 &&
      effect.expiry_released_to_available === true &&
      effect.retry_recovered === true,
    "external effect lease expiry retry state",
  );
  requireReference(
    typeof effect.effect_id_hash === "string" &&
      typeof effect.result_receipt_id_hash === "string" &&
      effect.result_receipt_id_hash === effect.result_replay_receipt_id_hash &&
      receipt.replay_receipt_id_hash === effect.result_receipt_id_hash,
    "external effect result receipt replay",
  );
  requireReference(
    Array.isArray(receipt.ledger_counts) &&
      receipt.ledger_counts.length === 5 &&
      receipt.ledger_counts[0] === 2 &&
      receipt.ledger_counts[1] === 3 &&
      receipt.ledger_counts[2] >= 2 &&
      receipt.ledger_counts[3] === 1 &&
      receipt.ledger_counts[4] === 1,
    "external effect same-slice ledger topology",
  );
  requireReference(receipt.reconciliation_green === true, "external effect reconciliation");
}

function referenceExternalEffectReadbackReceipt(receipt, attempt, expectedEffectReceipt) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "external effect readback schema",
  );
  requireReference(
    receipt.attempt === attempt && receipt.outcome === "EXTERNAL_EFFECT_READBACK",
    "external effect readback identity",
  );
  requireReference(
    receipt.receipt_id_hash === expectedEffectReceipt.receipt_id_hash &&
      receipt.replay_receipt_id_hash === expectedEffectReceipt.replay_receipt_id_hash &&
      receipt.workflow_state_sha256 === expectedEffectReceipt.workflow_state_sha256 &&
      receipt.database_sha256 === expectedEffectReceipt.database_sha256 &&
      JSON.stringify(receipt.ledger_counts) === JSON.stringify(expectedEffectReceipt.ledger_counts) &&
      JSON.stringify(receipt.external_effect) === JSON.stringify(expectedEffectReceipt.external_effect),
    "external effect durable readback",
  );
  requireReference(receipt.reconciliation_green === true, "external effect readback reconciliation");
}

async function seedReferenceFixture(fixture, normalBuildEnvironment, scenario) {
  const attempt = `${scenario}-seed`;
  const seed = await runReferenceDriver(
    fixture,
    normalBuildEnvironment,
    attempt,
    "seed",
  );
  if (seed.exit_code !== 0 || seed.signal !== null) {
    const diagnosticPath = await writeReferenceDriverDiagnostic(
      fixture.root,
      attempt,
      seed.stderr,
    );
    throw new Error(
      `reference scenario assertion failed:${scenario} seed exit:${seed.exit_code ?? "null"}:${seed.signal ?? "none"}:${referenceDriverFailureCode(seed.stderr)}:${diagnosticPath}`,
    );
  }
  requireReference(
    seed.exit_code === 0 && seed.signal === null,
    `${scenario} seed exit`,
  );
  const receipt = await readReferenceDriverReceipt(fixture.root, attempt);
  referenceSeedReceipt(receipt.value, attempt);
  return { ...seed, receipt_path: receipt.path };
}

async function runM2ReferenceScenarioSuite(
  firstFixture,
  normalBuildEnvironment,
  provenanceBefore,
) {
  const newFixture = async () => createReferenceFixture();
  const scenarios = [];

  const s1 = firstFixture;
  const s1Seed = await seedReferenceFixture(s1, normalBuildEnvironment, "s1");
  const s1StoreAfterSeed = await referenceStoreFingerprints(s1);
  const s1Binding = referenceCommandBinding("s1-cold-start");
  const s1Gate = await armReferenceGate(s1.root, "after-command", s1Binding);
  const s1Launched = launchReferenceDriver(
    s1,
    normalBuildEnvironment,
    "s1-cold-start",
    "run",
    s1Binding,
  );
  await waitForReferenceGate(s1Launched, "after-command");
  requireReference(Number.isSafeInteger(s1Launched.child.pid), "s1 PID availability");
  const s1Receipt = await readReferenceDriverReceipt(s1.root, "s1-cold-start");
  referencePassReceipt(s1Receipt.value, "s1-cold-start");
  process.kill(s1Launched.child.pid, "SIGTERM");
  const s1Terminated = await s1Launched.completed;
  const s1Run = {
    ...s1Terminated,
    syn_pid: s1Launched.child.pid ?? null,
    ...s1Launched.output(),
  };
  const s1StoreAfterSigterm = await referenceStoreFingerprints(s1);
  await removeReferenceGate(s1Gate);
  requireReference(
    s1Run.exit_code === null && s1Run.signal === "SIGTERM",
    `s1 SIGTERM exit:${s1Run.exit_code ?? "null"}:${s1Run.signal ?? "none"}:${referenceDriverFailureCode(s1Run.stderr)}`,
  );
  const s1Readback = await runReferenceDriver(
    s1,
    normalBuildEnvironment,
    "s1-restart-readback",
    "readback",
  );
  requireReference(
    s1Readback.exit_code === 0 && s1Readback.signal === null,
    `s1 readback exit:${s1Readback.exit_code ?? "null"}:${s1Readback.signal ?? "none"}:${referenceDriverFailureCode(s1Readback.stderr)}`,
  );
  const s1ReadbackReceipt = await readReferenceDriverReceipt(
    s1.root,
    "s1-restart-readback",
  );
  referenceReadbackReceipt(
    s1ReadbackReceipt.value,
    "s1-restart-readback",
    s1Receipt.value,
  );
  const s1StoreAfterReadback = await referenceStoreFingerprints(s1);
  scenarios.push({
    name: "S1-cold-start-and-replay",
    root: s1.root,
    pid: s1Run.syn_pid,
    restart_pid: s1Readback.syn_pid,
    seed_receipt_path: s1Seed.receipt_path,
    receipt_path: s1Receipt.path,
    readback_receipt_path: s1ReadbackReceipt.path,
    sigterm_exit: s1Run,
    restart_exit: s1Readback,
    store_fingerprints: {
      after_seed: s1StoreAfterSeed,
      after_sigterm: s1StoreAfterSigterm,
      after_restart_readback: s1StoreAfterReadback,
    },
    result: "PASS",
  });

  const s2 = await newFixture();
  const s2Seed = await seedReferenceFixture(s2, normalBuildEnvironment, "s2");
  const s2StoreAfterSeed = await referenceStoreFingerprints(s2);
  const s2Binding = referenceCommandBinding("s2-precommit-kill");
  const s2Gate = await armReferenceGate(s2.root, "pre-commit", s2Binding);
  const s2Killed = launchReferenceDriver(
    s2,
    normalBuildEnvironment,
    "s2-precommit-kill",
    "run",
    s2Binding,
  );
  await waitForReferenceGate(s2Killed, "pre-commit");
  requireReference(Number.isSafeInteger(s2Killed.child.pid), "s2 PID availability");
  process.kill(s2Killed.child.pid, "SIGKILL");
  const s2KilledResult = await s2Killed.completed;
  const s2StoreAfterSigkill = await referenceStoreFingerprints(s2);
  const s2BeforeRecoveryLedger = await sqliteReferenceLedgerCounts(
    join(s2.root, "runtime-artifacts", "workbench.sqlite"),
  );
  requireReference(
    JSON.stringify(s2BeforeRecoveryLedger) === JSON.stringify([0, 0, 0, 0, 0]),
    "s2 no half-commit ledger",
  );
  requireReference(
    (await referenceWorkItemState(s2.fixture.workflowStatePath)) === "ready_to_dispatch",
    "s2 JSON state remains ready",
  );
  await removeReferenceGate(s2Gate);
  const s2Recovery = await runReferenceDriver(s2, normalBuildEnvironment, "s2-precommit-recovery");
  requireReference(s2Recovery.exit_code === 0 && s2Recovery.signal === null, "s2 recovery exit");
  const s2Receipt = await readReferenceDriverReceipt(s2.root, "s2-precommit-recovery");
  referencePassReceipt(s2Receipt.value, "s2-precommit-recovery");
  const s2StoreAfterRecovery = await referenceStoreFingerprints(s2);
  scenarios.push({
    name: "S2-pre-commit-SIGKILL",
    root: s2.root,
    pid: s2Killed.child.pid ?? null,
    killed_exit: s2KilledResult,
    ledger_before_recovery: s2BeforeRecoveryLedger,
    seed_receipt_path: s2Seed.receipt_path,
    receipt_path: s2Receipt.path,
    recovery_exit: s2Recovery,
    store_fingerprints: {
      after_seed: s2StoreAfterSeed,
      after_sigkill: s2StoreAfterSigkill,
      after_recovery: s2StoreAfterRecovery,
    },
    result: "PASS",
  });

  const s3 = await newFixture();
  const s3Seed = await seedReferenceFixture(s3, normalBuildEnvironment, "s3");
  const s3StoreAfterSeed = await referenceStoreFingerprints(s3);
  const s3Binding = referenceCommandBinding("s3-postcommit-kill");
  const s3Gate = await armReferenceGate(s3.root, "post-commit", s3Binding);
  const s3Killed = launchReferenceDriver(
    s3,
    normalBuildEnvironment,
    "s3-postcommit-kill",
    "run",
    s3Binding,
  );
  await waitForReferenceGate(s3Killed, "post-commit");
  requireReference(Number.isSafeInteger(s3Killed.child.pid), "s3 PID availability");
  process.kill(s3Killed.child.pid, "SIGKILL");
  const s3KilledResult = await s3Killed.completed;
  const s3StoreAfterSigkill = await referenceStoreFingerprints(s3);
  const s3BeforeRecoveryLedger = await sqliteReferenceLedgerCounts(
    join(s3.root, "runtime-artifacts", "workbench.sqlite"),
  );
  requireReference(
    JSON.stringify(s3BeforeRecoveryLedger) === JSON.stringify([1, 1, 1, 0, 1]),
    "s3 DB command committed without external outbox",
  );
  requireReference(
    (await referenceWorkItemState(s3.fixture.workflowStatePath)) === "ready_to_dispatch",
    "s3 JSON remains stale",
  );
  const s3DatabaseBeforeRecovery = await referenceFileFingerprint(
    join(s3.root, "runtime-artifacts", "workbench.sqlite"),
  );
  const s3JsonBeforeRecovery = await referenceFileFingerprint(
    s3.fixture.workflowStatePath,
  );
  await removeReferenceGate(s3Gate);
  // JSON is the internal, rebuildable projection of this slice.  A restart
  // replays the committed DB-primary state directly; no external lease or
  // result command participates in S3.
  const s3Recovery = await runReferenceDriver(
    s3,
    normalBuildEnvironment,
    "s3-postcommit-recovery",
    "readback",
  );
  requireReference(
    s3Recovery.exit_code === 0 && s3Recovery.signal === null,
    "s3 DB-primary projection recovery exit",
  );
  const s3RecoveryReceipt = await readReferenceDriverReceipt(
    s3.root,
    "s3-postcommit-recovery",
  );
  referenceRecoveredReadbackReceipt(
    s3RecoveryReceipt.value,
    "s3-postcommit-recovery",
    [1, 1, 1, 0, 1],
  );
  const s3StoreAfterRecovery = await referenceStoreFingerprints(s3);
  scenarios.push({
    name: "S3-post-commit-SIGKILL-DB-primary-projection-recovery",
    root: s3.root,
    pid: s3Killed.child.pid ?? null,
    killed_exit: s3KilledResult,
    ledger_before_recovery: s3BeforeRecoveryLedger,
    seed_receipt_path: s3Seed.receipt_path,
    recovery_exit: s3Recovery,
    recovery_receipt_path: s3RecoveryReceipt.path,
    database_before_recovery: s3DatabaseBeforeRecovery,
    json_before_recovery: s3JsonBeforeRecovery,
    store_fingerprints: {
      after_seed: s3StoreAfterSeed,
      after_sigkill: s3StoreAfterSigkill,
      after_recovery: s3StoreAfterRecovery,
    },
    result: "PASS",
  });

  const s4 = await newFixture();
  const s4Seed = await seedReferenceFixture(s4, normalBuildEnvironment, "s4");
  const s4StoreAfterSeed = await referenceStoreFingerprints(s4);
  const s4Binding = referenceCommandBinding("s4-projection-failure");
  const s4Gate = await armReferenceGate(s4.root, "projection-fail", s4Binding);
  const s4Failure = await runReferenceDriver(
    s4,
    normalBuildEnvironment,
    "s4-projection-failure",
    "run",
    s4Binding,
  );
  if (s4Failure.exit_code !== 81 || s4Failure.signal !== null) {
    const diagnosticPath = await writeReferenceDriverDiagnostic(
      s4.root,
      "s4-projection-failure",
      s4Failure.stderr,
    );
    throw new Error(
      `reference scenario assertion failed:s4 fail-closed exit:${s4Failure.exit_code ?? "null"}:${s4Failure.signal ?? "none"}:${referenceDriverFailureCode(s4Failure.stderr)}:${diagnosticPath}`,
    );
  }
  requireReference(s4Failure.exit_code === 81 && s4Failure.signal === null, "s4 fail-closed exit");
  const s4FailureReceipt = await readReferenceDriverReceipt(s4.root, "s4-projection-failure");
  requireReference(
    s4FailureReceipt.value.outcome === "EXPECTED_FAILURE" &&
      s4FailureReceipt.value.error_family === "projection_fail",
    "s4 injected failure receipt",
  );
  requireReference(
    (await referenceWorkItemState(s4.fixture.workflowStatePath)) === "ready_to_dispatch",
    "s4 JSON remains stale",
  );
  const s4LedgerAfterFailure = await sqliteReferenceLedgerCounts(
    join(s4.root, "runtime-artifacts", "workbench.sqlite"),
  );
  requireReference(
    JSON.stringify(s4LedgerAfterFailure) === JSON.stringify([1, 1, 1, 0, 1]),
    "s4 committed source has no external outbox",
  );
  const s4StoreAfterFailure = await referenceStoreFingerprints(s4);
  await removeReferenceGate(s4Gate);
  const s4Recovery = await runReferenceDriver(
    s4,
    normalBuildEnvironment,
    "s4-projection-recovery",
    "readback",
  );
  requireReference(s4Recovery.exit_code === 0 && s4Recovery.signal === null, "s4 recovery exit");
  const s4Receipt = await readReferenceDriverReceipt(s4.root, "s4-projection-recovery");
  referenceRecoveredReadbackReceipt(
    s4Receipt.value,
    "s4-projection-recovery",
    [1, 1, 1, 0, 1],
  );
  const s4StoreAfterRecovery = await referenceStoreFingerprints(s4);
  scenarios.push({
    name: "S4-projection-failure-and-replay",
    root: s4.root,
    pid: s4Failure.syn_pid,
    failure_exit: s4Failure,
    failure_receipt_path: s4FailureReceipt.path,
    seed_receipt_path: s4Seed.receipt_path,
    receipt_path: s4Receipt.path,
    ledger_after_failure: s4LedgerAfterFailure,
    recovery_exit: s4Recovery,
    store_fingerprints: {
      after_seed: s4StoreAfterSeed,
      after_projection_failure: s4StoreAfterFailure,
      after_recovery: s4StoreAfterRecovery,
    },
    result: "PASS",
  });

  const s5 = await newFixture();
  const s5Seed = await seedReferenceFixture(s5, normalBuildEnvironment, "s5");
  const s5StoreAfterSeed = await referenceStoreFingerprints(s5);
  const s5Run = await runReferenceDriver(s5, normalBuildEnvironment, "s5-duplicate");
  requireReference(s5Run.exit_code === 0 && s5Run.signal === null, "s5 app exit");
  const s5Receipt = await readReferenceDriverReceipt(s5.root, "s5-duplicate");
  referencePassReceipt(s5Receipt.value, "s5-duplicate");
  const s5StoreAfterDuplicate = await referenceStoreFingerprints(s5);
  scenarios.push({
    name: "S5-duplicate-command",
    root: s5.root,
    pid: s5Run.syn_pid,
    seed_receipt_path: s5Seed.receipt_path,
    receipt_path: s5Receipt.path,
    duplicate_exit: s5Run,
    store_fingerprints: {
      after_seed: s5StoreAfterSeed,
      after_duplicate: s5StoreAfterDuplicate,
    },
    result: "PASS",
  });

  const s6 = await newFixture();
  const s6Seed = await runReferenceDriver(s6, normalBuildEnvironment, "s6-seed", "seed");
  requireReference(s6Seed.exit_code === 0 && s6Seed.signal === null, "s6 seed exit");
  const s6SeedReceipt = await readReferenceDriverReceipt(s6.root, "s6-seed");
  requireReference(
    s6SeedReceipt.value.outcome === "SEEDED" &&
      s6SeedReceipt.value.reconciliation_green === true &&
      JSON.stringify(s6SeedReceipt.value.ledger_counts) === JSON.stringify([0, 0, 0, 0, 0]),
    "s6 DB-primary seed",
  );
  const s6StoreAfterSeed = await referenceStoreFingerprints(s6);
  const s6DatabasePath = join(s6.root, "runtime-artifacts", "workbench.sqlite");
  const s6DatabaseBefore = await referenceFileFingerprint(s6DatabasePath);
  const s6State = JSON.parse(await readFile(s6.fixture.workflowStatePath, "utf8"));
  const s6Item = s6State.work_items?.find(
    (candidate) => candidate.title === "SYN M2 R4 workflow-state reference slice",
  );
  requireReference(s6Item?.state === "ready_to_dispatch", "s6 seed state");
  const s6Node = s6State.nodes?.find(
    (candidate) => candidate.node_id === s6Item.current_node_id,
  );
  requireReference(Boolean(s6Node), "s6 seed node binding");
  const s6JsonLeadingRevision =
    Math.max(
      Number(s6State.revision ?? 0),
      Number(s6Item.workflow_revision_after ?? 0),
      Number(s6Node.workflow_revision_after ?? 0),
    ) + 1;
  const s6JsonLeadingTimestamp = String(Date.now());
  // Build a structurally valid, self-consistent JSON projection that is newer
  // than the unchanged SQLite work-item/node records.  It is not a divergent
  // same-revision edit: the product reconciler must classify it as
  // json_leading with no hash_mismatches and refuse DB-primary startup.
  s6Item.state = "running";
  s6Item.workflow_revision_after = s6JsonLeadingRevision;
  s6Item.updated_at = s6JsonLeadingTimestamp;
  s6Node.state = "running";
  s6Node.workflow_revision_after = s6JsonLeadingRevision;
  s6Node.updated_at = s6JsonLeadingTimestamp;
  s6State.revision = s6JsonLeadingRevision;
  s6State.updated_at = s6JsonLeadingTimestamp;
  await writeFile(
    s6.fixture.workflowStatePath,
    `${JSON.stringify(s6State, null, 2)}\n`,
    "utf8",
  );
  const s6JsonLeadingBefore = await referenceFileFingerprint(s6.fixture.workflowStatePath);
  const s6StoreJsonLeadingBefore = await referenceStoreFingerprints(s6);
  const s6Rejected = await runReferenceDriver(s6, normalBuildEnvironment, "s6-json-leading");
  requireReference(s6Rejected.exit_code === 80 && s6Rejected.signal === null, "s6 startup rejection exit");
  const s6ReconciliationDiagnosticPath = await writeReferenceDriverDiagnostic(
    s6.root,
    "s6-json-leading",
    s6Rejected.stderr,
  );
  requireReference(
    /work_items:db_leading=\[\]:json_leading=\[[^\]]+\]:hash_mismatches=\[\]/.test(
      s6Rejected.stderr,
    ),
    "s6 work item is JSON-leading without hash mismatch",
  );
  requireReference(
    /workflow_nodes:db_leading=\[\]:json_leading=\[[^\]]+\]:hash_mismatches=\[\]/.test(
      s6Rejected.stderr,
    ),
    "s6 node is JSON-leading without hash mismatch",
  );
  // The M2 command's product boundary is deliberately fail-closed: startup
  // rejects JSON-leading DB-primary state before the command can execute.
  // A second actual App invocation is the only honest write-entrypoint attempt
  // available here; it must reject identically rather than manufacture a
  // JSON-only fallback mutation for this M2 surface.
  const s6DeniedWrite = await runReferenceDriver(
    s6,
    normalBuildEnvironment,
    "s6-downgrade-write-attempt",
  );
  requireReference(
    s6DeniedWrite.exit_code === 80 && s6DeniedWrite.signal === null,
    "s6 product write entrypoint remains fail-closed",
  );
  const s6DatabaseAfter = await referenceFileFingerprint(s6DatabasePath);
  const s6JsonLeadingAfter = await referenceFileFingerprint(s6.fixture.workflowStatePath);
  requireReference(
    s6DatabaseAfter.sha256 === s6DatabaseBefore.sha256,
    "s6 database unchanged",
  );
  requireReference(
    s6JsonLeadingAfter.sha256 === s6JsonLeadingBefore.sha256,
    "s6 JSON not reverse-overwritten",
  );
  const s6StoreAfterRejectedWrite = await referenceStoreFingerprints(s6);
  scenarios.push({
    name: "S6-JSON-leading-startup-fail-closed",
    root: s6.root,
    pid: s6Rejected.syn_pid,
    startup_rejection: s6Rejected,
    downgrade_write_attempt: s6DeniedWrite,
    downgrade_write_disposition: "REJECTED_AT_STARTUP_NO_M2_JSON_FALLBACK",
    seed_receipt_path: s6SeedReceipt.path,
    reconciliation_diagnostic_path: s6ReconciliationDiagnosticPath,
    json_leading_revision: s6JsonLeadingRevision,
    database_before: s6DatabaseBefore,
    database_after: s6DatabaseAfter,
    json_leading_before: s6JsonLeadingBefore,
    json_leading_after: s6JsonLeadingAfter,
    store_fingerprints: {
      after_seed: s6StoreAfterSeed,
      json_leading_before_rejection: s6StoreJsonLeadingBefore,
      after_rejected_write: s6StoreAfterRejectedWrite,
    },
    result: "PASS",
  });

  // DAT-004/008 is deliberately exercised only after the exact same
  // `update_work_item_state` IPC owner command is armed in this R4 fixture.
  // The isolated adapter never owns a second workflow command: it leases the
  // stored owner outbox row and its independent result command is bound back
  // to that exact receipt, effect, scope, correlation and causation.
  const external = await newFixture();
  const externalSeed = await seedReferenceFixture(
    external,
    normalBuildEnvironment,
    "dat004-external-effect",
  );
  const externalBinding = referenceCommandBinding("dat004-external-effect");
  const externalOwnerRun = await runReferenceDriver(
    external,
    normalBuildEnvironment,
    "dat004-external-effect",
    "run",
    externalBinding,
    true,
  );
  if (externalOwnerRun.exit_code !== 0 || externalOwnerRun.signal !== null) {
    const diagnosticPath = await writeReferenceDriverDiagnostic(
      external.root,
      "dat004-external-effect-owner",
      externalOwnerRun.stderr,
    );
    throw new Error(
      `reference scenario assertion failed:dat004 same-slice owner IPC exit:${externalOwnerRun.exit_code ?? "null"}:${externalOwnerRun.signal ?? "none"}:${referenceDriverFailureCode(externalOwnerRun.stderr)}:${diagnosticPath}`,
    );
  }
  requireReference(
    externalOwnerRun.exit_code === 0 && externalOwnerRun.signal === null,
    "dat004 same-slice owner IPC exit",
  );
  const externalOwnerReceipt = await archiveReferenceDriverReceipt(
    await readReferenceDriverReceipt(
      external.root,
      "dat004-external-effect",
      "run",
    ),
    "owner",
  );
  referencePassReceipt(
    externalOwnerReceipt.value,
    "dat004-external-effect",
    // The armed same-slice owner UoW now contains its normal owning fact
    // plus the frozen declaration event/audit for the one declared effect.
    [1, 2, 2, 1, 1],
  );
  const externalStoreAfterOwner = await referenceStoreFingerprints(external);
  const externalEffectRun = await runReferenceDriver(
    external,
    normalBuildEnvironment,
    "dat004-external-effect",
    "external-effect",
    externalBinding,
    true,
  );
  requireReference(
    externalEffectRun.exit_code === 0 && externalEffectRun.signal === null,
    "dat004 same-slice effect lifecycle exit",
  );
  const externalEffectReceipt = await archiveReferenceDriverReceipt(
    await readReferenceDriverReceipt(
      external.root,
      "dat004-external-effect",
      "external-effect",
    ),
    "effect",
  );
  referenceExternalEffectReceipt(
    externalEffectReceipt.value,
    "dat004-external-effect",
    externalOwnerReceipt.value,
    externalBinding,
  );
  const externalStoreAfterEffect = await referenceStoreFingerprints(external);
  const externalReadbackRun = await runReferenceDriver(
    external,
    normalBuildEnvironment,
    "dat004-external-effect",
    "external-readback",
    externalBinding,
    true,
  );
  requireReference(
    externalReadbackRun.exit_code === 0 && externalReadbackRun.signal === null,
    "dat004 same-slice result readback exit",
  );
  const externalReadbackReceipt = await archiveReferenceDriverReceipt(
    await readReferenceDriverReceipt(
      external.root,
      "dat004-external-effect",
      "external-readback",
    ),
    "readback",
  );
  referenceExternalEffectReadbackReceipt(
    externalReadbackReceipt.value,
    "dat004-external-effect",
    externalEffectReceipt.value,
  );
  const externalStoreAfterReadback = await referenceStoreFingerprints(external);
  scenarios.push({
    name: "DAT-004-008-same-update-work-item-state-effect-result-recovery",
    root: external.root,
    owner_pid: externalOwnerRun.syn_pid,
    effect_pid: externalEffectRun.syn_pid,
    readback_pid: externalReadbackRun.syn_pid,
    seed_receipt_path: externalSeed.receipt_path,
    owner_receipt_path: externalOwnerReceipt.path,
    effect_receipt_path: externalEffectReceipt.path,
    readback_receipt_path: externalReadbackReceipt.path,
    store_fingerprints: {
      after_owner: externalStoreAfterOwner,
      after_effect: externalStoreAfterEffect,
      after_readback: externalStoreAfterReadback,
    },
    result: "PASS",
  });

  const busy = await newFixture();
  const busySeed = await seedReferenceFixture(busy, normalBuildEnvironment, "dat008-db-busy");
  const busyStoreAfterSeed = await referenceStoreFingerprints(busy);
  const busyDatabasePath = join(busy.root, "runtime-artifacts", "workbench.sqlite");
  const busyDatabaseBefore = await referenceFileFingerprint(busyDatabasePath);
  const busyJsonBefore = await referenceFileFingerprint(busy.fixture.workflowStatePath);
  const busyLedgerBefore = await sqliteReferenceLedgerCounts(busyDatabasePath);
  const busyLock = await holdReferenceSqliteWriteLock(busyDatabasePath);
  let busyFailure;
  try {
    busyFailure = await runReferenceDriver(
      busy,
      normalBuildEnvironment,
      "dat008-db-busy-rejected",
    );
  } finally {
    await busyLock.release();
  }
  const busyFailureDiagnosticPath = await writeReferenceDriverDiagnostic(
    busy.root,
    "dat008-db-busy-rejected",
    busyFailure.stderr,
  );
  requireReference(
    busyFailure.exit_code === 80 && busyFailure.signal === null,
    "dat008 DB busy fail-closed exit",
  );
  requireReference(
    busyFailure.stderr.includes("db_primary_projection_blocked") &&
      /database (?:is )?(?:locked|busy)|database table is locked/i.test(busyFailure.stderr),
    `dat008 DB busy failure family:${referenceDriverFailureCode(busyFailure.stderr)}`,
  );
  const busyDatabaseAfterFailure = await referenceFileFingerprint(busyDatabasePath);
  const busyJsonAfterFailure = await referenceFileFingerprint(busy.fixture.workflowStatePath);
  const busyLedgerAfterFailure = await sqliteReferenceLedgerCounts(busyDatabasePath);
  requireReference(
    busyDatabaseAfterFailure.sha256 === busyDatabaseBefore.sha256 &&
      busyJsonAfterFailure.sha256 === busyJsonBefore.sha256 &&
      JSON.stringify(busyLedgerAfterFailure) === JSON.stringify(busyLedgerBefore),
    "dat008 DB busy zero product mutation",
  );
  const busyStoreAfterFailure = await referenceStoreFingerprints(busy);
  const busyRecovery = await runReferenceDriver(
    busy,
    normalBuildEnvironment,
    "dat008-db-busy-recovery",
  );
  requireReference(
    busyRecovery.exit_code === 0 && busyRecovery.signal === null,
    "dat008 DB busy recovery exit",
  );
  const busyRecoveryReceipt = await readReferenceDriverReceipt(
    busy.root,
    "dat008-db-busy-recovery",
  );
  referencePassReceipt(busyRecoveryReceipt.value, "dat008-db-busy-recovery");
  const busyStoreAfterRecovery = await referenceStoreFingerprints(busy);

  const corrupt = await newFixture();
  const corruptSeed = await seedReferenceFixture(
    corrupt,
    normalBuildEnvironment,
    "dat008-db-corrupt",
  );
  const corruptStoreAfterSeed = await referenceStoreFingerprints(corrupt);
  const corruptDatabasePath = join(corrupt.root, "runtime-artifacts", "workbench.sqlite");
  const corruptJsonBefore = await referenceFileFingerprint(corrupt.fixture.workflowStatePath);
  await corruptReferenceSqliteHeader(corruptDatabasePath);
  const corruptDatabaseBeforeApp = await referenceFileFingerprint(corruptDatabasePath);
  const corruptStoreBeforeApp = await referenceStoreFingerprints(corrupt);
  const corruptFailure = await runReferenceDriver(
    corrupt,
    normalBuildEnvironment,
    "dat008-db-corrupt-rejected",
    "readback",
  );
  const corruptFailureDiagnosticPath = await writeReferenceDriverDiagnostic(
    corrupt.root,
    "dat008-db-corrupt-rejected",
    corruptFailure.stderr,
  );
  requireReference(
    corruptFailure.exit_code === 80 && corruptFailure.signal === null,
    "dat008 corrupt DB fail-closed exit",
  );
  requireReference(
    corruptFailure.stderr.includes("m2_r4_reference_slice_meta_query") &&
      /file is not a database|database disk image is malformed/i.test(corruptFailure.stderr),
    "dat008 corrupt DB rejection family",
  );
  const corruptDatabaseAfterFailure = await referenceFileFingerprint(corruptDatabasePath);
  const corruptJsonAfterFailure = await referenceFileFingerprint(corrupt.fixture.workflowStatePath);
  requireReference(
    corruptDatabaseAfterFailure.sha256 === corruptDatabaseBeforeApp.sha256 &&
      corruptJsonAfterFailure.sha256 === corruptJsonBefore.sha256,
    "dat008 corrupt DB and JSON preserved after rejection",
  );
  const corruptStoreAfterFailure = await referenceStoreFingerprints(corrupt);

  return {
    schema_version: "syn_m2_r4_reference_slice_suite.v1",
    provenance: {
      before: provenanceBefore,
      after: null,
      stable: null,
    },
    scenario_count: scenarios.length,
    scenarios,
    dat008: {
      same_slice_external_effect: {
        source_scenario:
          "DAT-004-008-same-update-work-item-state-effect-result-recovery",
        root: external.root,
        owner_exit: externalOwnerRun,
        effect_exit: externalEffectRun,
        readback_exit: externalReadbackRun,
        owner_receipt_path: externalOwnerReceipt.path,
        effect_receipt_path: externalEffectReceipt.path,
        readback_receipt_path: externalReadbackReceipt.path,
        result: "PASS",
      },
      internal_projection_recovery: {
        source_scenario: "S4-projection-failure-and-replay",
        ledger_after_failure: s4LedgerAfterFailure,
        recovery_receipt_path: s4Receipt.path,
      },
      db_busy: {
        root: busy.root,
        failure_exit: busyFailure,
        failure_diagnostic_path: busyFailureDiagnosticPath,
        seed_receipt_path: busySeed.receipt_path,
        database_before: busyDatabaseBefore,
        database_after_failure: busyDatabaseAfterFailure,
        json_before: busyJsonBefore,
        json_after_failure: busyJsonAfterFailure,
        ledger_before: busyLedgerBefore,
        ledger_after_failure: busyLedgerAfterFailure,
        recovery_exit: busyRecovery,
        recovery_receipt_path: busyRecoveryReceipt.path,
        store_fingerprints: {
          after_seed: busyStoreAfterSeed,
          after_failure: busyStoreAfterFailure,
          after_recovery: busyStoreAfterRecovery,
        },
        result: "PASS",
      },
      db_corrupt: {
        root: corrupt.root,
        failure_exit: corruptFailure,
        failure_diagnostic_path: corruptFailureDiagnosticPath,
        seed_receipt_path: corruptSeed.receipt_path,
        database_before_app: corruptDatabaseBeforeApp,
        database_after_failure: corruptDatabaseAfterFailure,
        json_before: corruptJsonBefore,
        json_after_failure: corruptJsonAfterFailure,
        store_fingerprints: {
          after_seed: corruptStoreAfterSeed,
          before_app: corruptStoreBeforeApp,
          after_failure: corruptStoreAfterFailure,
        },
        result: "PASS",
      },
    },
  };
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
const m2ReferenceSliceMode = process.argv.slice(2).includes(M2_REFERENCE_SLICE_MODE_ARG);
const homeInitialViewConfigPinned =
  !Object.hasOwn(process.env, "VITE_STAGE_K_INITIAL_VIEW") ||
  process.env.VITE_STAGE_K_INITIAL_VIEW === "home";
let root;
let identity;
let profile;
let fixture;
let runHash;
let reentryCapability;
let buildResult = { exit_code: null, launched: false, signal: null };
let launchResult = { exit_code: null, launched: false, signal: null };
let preListSigkillDiagnostic = createPreListSigkillDiagnostic();
let parentSignalToReraise = null;
let failureStage = null;
let uiInspection = pendingUiInspection("");
let m2ReferenceSliceSuite = null;

try {
  if (!homeInitialViewConfigPinned) {
    failureStage = "initial_view";
    process.exitCode = 1;
  } else {
    root = await createIsolatedRoot();
    identity = buildFixtureIdentity(root, makeRunId());
    profile = buildProfile(identity, Date.now());
    runHash = sha256(identity.runId);
    // This secret never enters the profile, receipt, stdout, or logs.  It is
    // passed only to the final isolated App process so a preseeded marker
    // cannot claim first-initialization eligibility.
    reentryCapability = randomBytes(32).toString("hex");
    const fixturePaths = await createFixture(root, identity, profile);
    fixture = { root, ...fixturePaths };

    const normalBuildEnvironment = { ...process.env };
    normalBuildEnvironment.VITE_STAGE_K_INITIAL_VIEW = "home";
    normalBuildEnvironment.CARGO_HOME ??= tauriCargoHome;
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
        if (m2ReferenceSliceMode) {
          try {
            const provenanceBefore = await referenceSuiteProvenance();
            m2ReferenceSliceSuite = await runM2ReferenceScenarioSuite(
              {
                root,
                identity,
                profile,
                runHash,
                reentryCapability,
                fixture,
              },
              normalBuildEnvironment,
              provenanceBefore,
            );
            const provenanceAfter = await referenceSuiteProvenance();
            requireReference(
              referenceSuiteProvenanceIsStable(provenanceBefore, provenanceAfter),
              "reference suite provenance drift",
            );
            m2ReferenceSliceSuite.provenance = {
              before: provenanceBefore,
              after: provenanceAfter,
              stable: true,
            };
          } catch (error) {
            m2ReferenceSliceSuite = {
              schema_version: "syn_m2_r4_reference_slice_suite_failure.v1",
              root,
              failure:
                error instanceof Error
                  ? error.message.slice(0, 256)
                  : "unclassified",
            };
            failureStage = "m2_reference_slice";
            process.exitCode = 1;
          }
        } else {
          const finalSynEnvironment = {
            ...normalBuildEnvironment,
            [PROFILE_ENV]: join(root, PROFILE_FILE_NAME),
            [REENTRY_CAPABILITY_ENV]: reentryCapability,
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
  }
} catch {
  failureStage ??= "fixture_or_launcher";
  process.exitCode = 1;
} finally {
  if (root && identity && profile && fixture) {
    const receipt = m2ReferenceSliceMode
      ? {
          schema_version: "syn_m2_r4_reference_slice_launcher_receipt.v1",
          build: buildResult,
          ...(m2ReferenceSliceSuite ? { suite: m2ReferenceSliceSuite } : {}),
          ...(failureStage ? { failure_stage: failureStage } : {}),
          environment_unchanged:
            process.env.HOME === initialHome && process.env.CODEX_HOME === initialCodexHome,
          home_initial_view_config_pinned: homeInitialViewConfigPinned,
        }
      : {
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
    await writeJson(
      join(
        root,
        m2ReferenceSliceMode
          ? "m2-reference-slice-suite-receipt.json"
          : RECEIPT_FILE_NAME,
      ),
      receipt,
    );
    process.stdout.write(`${JSON.stringify(receipt)}\n`);
    if (parentSignalToReraise) {
      process.kill(process.pid, parentSignalToReraise);
    }
  }
}
