import { renderToStaticMarkup } from "react-dom/server.browser";
import { M3AcceptancePanel } from "../src/views/agent/M3AcceptancePanel";
import {
  JiaobanNoWorkflowFallback,
  JiaobanNonTestProjectFallback,
} from "../src/views/projects/ProjectJiaobanPanel";
import type { M3C07AcceptanceStatus } from "../src/lib/tauri";
import { assert } from "./helpers/offlineInteractionTestUtils";

const nodeProcess = (globalThis as typeof globalThis & { process?: { cwd?: () => string } }).process;
if (!nodeProcess?.cwd) throw new Error("M3C07 离线 IPC 审计需要 Node cwd");
const nodeFsSpecifier: string = "node:fs";
const nodeFs = await import(nodeFsSpecifier) as { readFileSync: (path: string, encoding: "utf8") => string };
const commandRegistrySource = nodeFs.readFileSync(
  `${nodeProcess.cwd()}/src-tauri/src/command_registry.rs`,
  "utf8",
);
const launcherSource = nodeFs.readFileSync(
  `${nodeProcess.cwd()}/scripts/run-r4-isolated-app-preflight.mjs`,
  "utf8",
);

type LauncherModePolicy = {
  normalizeInheritedMarkerNames: (
    environment: Record<string, string | undefined>,
    markerNames: readonly string[],
  ) => string[];
  resolveLauncherModeConflict: (input: {
    m2ReferenceSliceMode: boolean;
    m3c07IsolatedMode: boolean;
    inheritedM2ReferenceSliceMarkers: readonly string[];
    inheritedM3C07ModeMarker: boolean;
  }) => string | null;
};

const launcherModePolicyStart = launcherSource.indexOf("function normalizeInheritedMarkerNames");
const launcherModePolicyEnd = launcherSource.indexOf("const initialHome", launcherModePolicyStart);
assert(
  launcherModePolicyStart >= 0 && launcherModePolicyEnd > launcherModePolicyStart,
  "launcher 必须保留可脱离 GUI 执行的 mode conflict 纯函数",
);
const launcherModePolicy = new Function(
  "M3C07_M2_REFERENCE_SLICE_MODE_CONFLICT",
  "M2_REFERENCE_SLICE_M3C07_MODE_CONFLICT",
  `${launcherSource.slice(launcherModePolicyStart, launcherModePolicyEnd)}\nreturn { normalizeInheritedMarkerNames, resolveLauncherModeConflict };`,
)(
  "m3c07_m2_reference_slice_mode_conflict",
  "m2_reference_slice_m3c07_mode_conflict",
) as LauncherModePolicy;

function fixture(host: "agent" | "jiaoban"): M3C07AcceptanceStatus {
  return {
    runtimeVersion: "syn.m3c07.isolated-runtime.v1",
    host,
    lifecycleState: "TURN_PENDING",
    sessionState: "ACTIVE",
    turnState: "PENDING",
    labels: {
      role: `m3c07:role:${host}`,
      project: "m3c07:scope:isolated-profile",
      object: "m3c07:object:acceptance-fixture",
      channel: `m3c07:channel:${host}`,
      permission: "m3c07:permission:fake-read-only",
    },
    ledger: { fakeDispatches: 2, fakeReadbacks: 1, realProviderAttempts: 0, persistentLedger: true },
    receipt: {
      schemaVersion: "syn.m3c07.isolated-acceptance-receipt.v1",
      receiptId: `receipt:${host}`,
      host,
      action: "continue",
      outcome: "AUTHORITATIVE_FAKE_READBACK_PENDING",
      replayed: false,
      rollbackApplied: false,
      realProviderAttempts: 0,
      redaction: "opaque_refs_only",
    },
    recovery: { state: "READBACK_ONLY_ON_RESTART", restartReadbacks: 0, dispatchesAfterRestart: 0 },
    objectNavigation: { available: false, state: "OBJECT_NAVIGATION_ABSENT" },
  };
}

for (const host of ["agent", "jiaoban"] as const) {
  const markup = renderToStaticMarkup(<M3AcceptancePanel host={host} initialStatus={fixture(host)} />);
  assert(markup.includes(`data-m3c07-host=\"${host}\"`), `M3C07 ${host} panel 必须固定 host`);
  assert(markup.includes("新建") && markup.includes("继续") && markup.includes("停止"), "M3C07 panel 必须展示 new/continue/stop");
  assert(
    markup.includes("落 CREATE pending")
      && markup.includes("落 START pending")
      && markup.includes("落 STOP pending"),
    "M3C07 panel 必须展示有限的强退前 durable stage 入口",
  );
  assert(markup.includes("重启恢复 readback"), "M3C07 panel 必须展示重启只读恢复入口");
  assert(
    markup.includes("审计回滚注入") && markup.includes("Handoff exact replay"),
    "M3C07 panel 必须展示 rollback 与 handoff replay 证据入口",
  );
  assert(markup.includes("对象导航"), "M3C07 panel 必须如实展示对象导航状态入口");
  assert(
    markup.includes('data-m3c07-replay="false"')
      && markup.includes('data-m3c07-rollback="false"'),
    "M3C07 panel 必须呈现 receipt 的 replay/rollback 状态",
  );
  assert(!markup.includes("conversation_transport"), "M3C07 panel 不得走 legacy transport");
}

for (const host of ["agent", "jiaoban"] as const) {
  const normalModeMarkup = renderToStaticMarkup(
    <M3AcceptancePanel host={host} initialError="M3_BINDING_UNAVAILABLE" />,
  );
  assert(normalModeMarkup === "", `普通 ${host} M3 unavailable 模式不得渲染验收 panel 或动作按钮`);
}

const undiscoveredModeMarkup = renderToStaticMarkup(
  <M3AcceptancePanel host="agent" initialStatus={null} initialError={null} />,
);
assert(
  undiscoveredModeMarkup === "",
  "普通模式在初始 readback 前也不得闪现验收 panel 或动作按钮",
);

const nonTestProjectMarkup = renderToStaticMarkup(
  <JiaobanNonTestProjectFallback
    latestSession={null}
    onOpenAgentSession={() => undefined}
    initialM3AcceptanceStatus={fixture("jiaoban")}
  />,
);
assert(
  nonTestProjectMarkup.includes("这个项目现在用智能体直连")
    && nonTestProjectMarkup.includes('data-m3c07-host="jiaoban"')
    && nonTestProjectMarkup.includes("新建"),
  "非测试项目 browser fallback 在 M3 gate 就绪后必须同时呈现老实说明与 Jiaoban acceptance panel",
);

const noWorkflowMarkup = renderToStaticMarkup(
  <JiaobanNoWorkflowFallback initialM3AcceptanceStatus={fixture("jiaoban")} />,
);
assert(
  noWorkflowMarkup.includes("这个项目还没准备好交办")
    && noWorkflowMarkup.includes('data-m3c07-host="jiaoban"')
    && noWorkflowMarkup.includes("停止"),
  "缺 workflow browser fallback 在 M3 gate 就绪后必须呈现 Jiaoban acceptance panel",
);

const nonTestProjectInitialMarkup = renderToStaticMarkup(
  <JiaobanNonTestProjectFallback latestSession={null} onOpenAgentSession={() => undefined} />,
);
const noWorkflowInitialMarkup = renderToStaticMarkup(<JiaobanNoWorkflowFallback />);
assert(
  !nonTestProjectInitialMarkup.includes("data-m3c07-host")
    && !noWorkflowInitialMarkup.includes("data-m3c07-host")
    && !nonTestProjectInitialMarkup.includes("新建")
    && !noWorkflowInitialMarkup.includes("新建"),
  "普通或未知首帧的 Jiaoban fallback 不得闪现验收 panel 或动作按钮",
);

assert(
  commandRegistrySource.includes("let workbench_handler")
    && commandRegistrySource.includes("tauri::generate_handler![")
    && commandRegistrySource.includes("reject_unapproved_tauri_command(&command)")
    && commandRegistrySource.includes("invoke.resolver.reject(error);"),
  "M3C07 隔离 child 必须在服务端 invoke dispatch、而非 renderer UI 层封堵 legacy IPC",
);
for (const command of [
  "load_agent_m3c07_acceptance_status",
  "operate_agent_m3c07_acceptance",
  "load_jiaoban_m3c07_acceptance_status",
  "operate_jiaoban_m3c07_acceptance",
  "start_agent_conversation_transport",
  "start_supervisor_conversation_transport",
  "poll_conversation_transport_attempt",
  "stop_conversation_transport_attempt",
  "run_manual_codex_relay_gui_direct",
  "execute_project_workflow_node",
]) {
  assert(
    commandRegistrySource.includes(command),
    `normal handler registration 与 M3C07 pre-dispatch audit 必须包含 ${command}`,
  );
}

const m2MarkerNames = [
  "SYN_M2_R4_REFERENCE_SLICE_DRIVER",
  "SYN_M2_R4_REFERENCE_SLICE_ATTEMPT",
  "SYN_M2_R4_REFERENCE_SLICE_PHASE",
  "SYN_M2_R4_REFERENCE_SLICE_NONCE",
  "SYN_M2_R4_REFERENCE_SLICE_EXTERNAL_EFFECT",
] as const;
for (const marker of m2MarkerNames) {
  const inheritedMarkers = launcherModePolicy.normalizeInheritedMarkerNames(
    { [marker]: "inherited" },
    m2MarkerNames,
  );
  assert(
    inheritedMarkers.length === 1 && inheritedMarkers[0] === marker,
    `launcher 必须识别继承的 M2 marker ${marker}`,
  );
  assert(
    launcherModePolicy.resolveLauncherModeConflict({
      m2ReferenceSliceMode: false,
      m3c07IsolatedMode: true,
      inheritedM2ReferenceSliceMarkers: inheritedMarkers,
      inheritedM3C07ModeMarker: false,
    }) === "m3c07_m2_reference_slice_mode_conflict",
    `M3 CLI + ${marker} 必须在启动前固定拒绝`,
  );
}
const normalizedEveryM2Marker = launcherModePolicy.normalizeInheritedMarkerNames(
  Object.fromEntries(m2MarkerNames.slice().reverse().map((marker) => [marker, ""])),
  m2MarkerNames.slice().reverse(),
);
assert(
  normalizedEveryM2Marker.join(",") === m2MarkerNames.slice().sort().join(","),
  "完整 M2 marker family 必须按名称归一化且不依赖 parent env 顺序",
);
assert(
  launcherModePolicy.resolveLauncherModeConflict({
    m2ReferenceSliceMode: true,
    m3c07IsolatedMode: false,
    inheritedM2ReferenceSliceMarkers: [],
    inheritedM3C07ModeMarker: true,
  }) === "m2_reference_slice_m3c07_mode_conflict",
  "M2 CLI + 继承 M3 marker 必须对称地在启动前拒绝",
);
for (const [label, input] of [
  ["普通 R4", {
    m2ReferenceSliceMode: false,
    m3c07IsolatedMode: false,
    inheritedM2ReferenceSliceMarkers: [...m2MarkerNames],
    inheritedM3C07ModeMarker: true,
  }],
  ["纯 M2", {
    m2ReferenceSliceMode: true,
    m3c07IsolatedMode: false,
    inheritedM2ReferenceSliceMarkers: [...m2MarkerNames],
    inheritedM3C07ModeMarker: false,
  }],
  ["纯 M3", {
    m2ReferenceSliceMode: false,
    m3c07IsolatedMode: true,
    inheritedM2ReferenceSliceMarkers: [],
    inheritedM3C07ModeMarker: true,
  }],
] as const) {
  assert(
    launcherModePolicy.resolveLauncherModeConflict(input) === null,
    `${label} 不能被 inherited marker 误拒绝`,
  );
}
assert(
  launcherModePolicy.resolveLauncherModeConflict({
    m2ReferenceSliceMode: true,
    m3c07IsolatedMode: true,
    inheritedM2ReferenceSliceMarkers: [],
    inheritedM3C07ModeMarker: false,
  }) === "mode_argument",
  "两个 CLI flag 的既有 mode_argument 语义必须保留",
);
const conflictGateOffset = launcherSource.indexOf("} else if (launcherModeConflict) {");
const rootCreationOffset = launcherSource.lastIndexOf("root = await createIsolatedRoot();");
const environmentScrubOffset = launcherSource.indexOf("delete normalBuildEnvironment[M3C07_MODE_ENV];");
const bundleBuildOffset = launcherSource.indexOf("buildResult = await runChild(");
const m3ChildSpawnOffset = launcherSource.lastIndexOf(
  "m3c07Restart = await runM3C07SameProfileRestart({",
);
assert(
  conflictGateOffset >= 0
    && conflictGateOffset < rootCreationOffset
    && rootCreationOffset < environmentScrubOffset
    && environmentScrubOffset < bundleBuildOffset
    && conflictGateOffset < m3ChildSpawnOffset,
  "继承环境冲突判定必须发生在 root、scrub、build 与 M3 child spawn 之前",
);
