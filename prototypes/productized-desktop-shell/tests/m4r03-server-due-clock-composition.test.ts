import { assert } from "./helpers/offlineInteractionTestUtils";

const nodeProcess = (globalThis as typeof globalThis & {
  process?: { cwd?: () => string };
}).process;
if (!nodeProcess?.cwd) throw new Error("M4R03 scheduler 静态验收需要 Node cwd");
const nodeFsSpecifier: string = "node:fs";
const { readFileSync } = await import(nodeFsSpecifier) as {
  readFileSync: (path: string, encoding: "utf8") => string;
};
const root = nodeProcess.cwd();
const lib = readFileSync(`${root}/src-tauri/src/lib.rs`, "utf8");
const indexHost = readFileSync(`${root}/src-tauri/src/index_host_app_entrypoints.rs`, "utf8");
const ordinaryClockDriver = readFileSync(
  `${root}/src-tauri/src/m4r03_ordinary_clock_driver.rs`,
  "utf8",
);
const repository = readFileSync(`${root}/src-tauri/src/m4_secretary_repository.rs`, "utf8");
const main = readFileSync(`${root}/src/main.tsx`, "utf8");
const launcher = readFileSync(`${root}/scripts/run-r4-isolated-app-preflight.mjs`, "utf8");
const renderer = [
  main,
  readFileSync(`${root}/src/App.tsx`, "utf8"),
  readFileSync(`${root}/src/lib/tauri.ts`, "utf8"),
].join("\n");

const startSchedulerStart = lib.indexOf("fn start_m4_secretary_scheduler(");
const cycleHelperStart = lib.indexOf("fn run_m4_secretary_scheduler_cycle(");
const sourceDispatcherStart = lib.indexOf("fn start_m4_source_owner_dispatcher(");
assert(
  startSchedulerStart >= 0
    && cycleHelperStart > startSchedulerStart
    && sourceDispatcherStart > cycleHelperStart,
  "M4R03 必须保留普通 scheduler entry 与唯一 production cycle helper",
);
const startScheduler = lib.slice(startSchedulerStart, cycleHelperStart);
const cycleHelper = lib.slice(cycleHelperStart, sourceDispatcherStart);
const startupIndex = startScheduler.indexOf("M4SchedulerTrigger::StartupRecovery");
const threadIndex = startScheduler.indexOf("std::thread::Builder::new()");
const timerIndex = startScheduler.indexOf("M4SchedulerTrigger::TimerTick", threadIndex);
assert(
  startupIndex >= 0
    && startupIndex < threadIndex
    && timerIndex > threadIndex
    && startScheduler.match(/run_m4_secretary_scheduler_cycle\(/g)?.length === 2,
  "普通 AppState 必须在启线程前跑 StartupRecovery，并由同一 helper 驱动 TimerTick",
);
assert(
  cycleHelper.includes("repository")
    && cycleHelper.includes(".run_daily_scheduler_cycle(trigger)"),
  "production cycle helper 必须直接进入 M4 repository daily scheduler cycle",
);

const dailyStart = repository.indexOf("pub(crate) fn run_daily_scheduler_cycle(");
const refreshStart = repository.indexOf("pub(crate) fn refresh_and_read_daily_report(", dailyStart);
assert(dailyStart >= 0 && refreshStart > dailyStart, "M4R03 daily cycle source slice 缺失");
const dailyCycle = repository.slice(dailyStart, refreshStart);
assert(
  dailyCycle.match(/self\.clock\.capture_now\(\)/g)?.length === 1,
  "一个 scheduler cycle 必须只 capture 一次 server-now",
);
const activeIndex = dailyCycle.indexOf("M4PersistedSchedulerConfiguration::Active(active)");
const dueCallIndex = dailyCycle.indexOf("self.run_due_transition_batch(", activeIndex);
assert(
  activeIndex >= 0 && dueCallIndex > activeIndex,
  "due batch 必须由普通 daily cycle 的 Active scheduler 分支调用",
);

const batchStart = repository.indexOf("fn run_due_transition_batch(", dailyStart);
const refreshComment = repository.indexOf("/// Explicit open/refresh", batchStart);
assert(batchStart >= 0 && refreshComment > batchStart, "M4R03 due batch source slice 缺失");
const batch = repository.slice(batchStart, refreshComment);
for (const token of [
  "open_loop_transition_reason",
  "reminder_transition_reason",
  "load_due_transition_candidates",
  "due_transition_idempotency_key",
  "insert_coordination_command_receipt",
  "insert_coordination_event_and_audit",
  "reproject_current_daily_brief_for_server_due",
]) {
  assert(batch.includes(token), `M4R03 due batch 缺少 ${token}`);
}
assert(
  !batch.includes("advance_open_loop_clock(")
    && !batch.includes("fire_reminder("),
  "production due batch 不得委托单项验收 transition 或嵌套事务",
);
for (const token of [
  'then_some("SERVER_CLOCK")',
  '"syn.m4.server-clock-due-transition/v1"',
  "expected_revision",
  "due_marker_utc",
  "m4_parse_rfc3339_utc_key",
  "sort_by",
]) {
  assert(repository.includes(token), `M4R03 deterministic clock 证据缺少 ${token}`);
}

assert(
  !renderer.includes("OPEN_LOOP_CLOCK")
    && !renderer.includes("REMINDER_FIRE")
    && !renderer.includes("SERVER_CLOCK"),
  "renderer 不得拥有 clock/fire 命令或 SERVER_CLOCK authority",
);

assert(
  lib.includes("mod m4r03_ordinary_clock_driver;"),
  "M4R03 actual-App driver 必须进入普通应用 crate",
);
const ordinaryHandlerIndex = indexHost.indexOf(".invoke_handler(workbench_command_handler!())");
const ordinaryConstructorIndex = indexHost.indexOf(
  "AppState::try_new_with_isolated_product_profile(paths)",
);
const constructorProofIndex = indexHost.indexOf(
  "m4r03_ordinary_clock_driver::mark_ordinary_constructor_ready()",
);
const ordinaryManageIndex = indexHost.indexOf("app.manage(state)", constructorProofIndex);
const driverInstallIndex = indexHost.indexOf(
  "m4r03_ordinary_clock_driver::install_after_runtime_ready(app)",
);
assert(
  ordinaryHandlerIndex >= 0
    && ordinaryConstructorIndex > ordinaryHandlerIndex
    && constructorProofIndex > ordinaryConstructorIndex
    && ordinaryManageIndex > constructorProofIndex
    && driverInstallIndex > ordinaryManageIndex,
  "M4R03 必须复用普通 command registry、普通 AppState constructor 与已注册 AppState",
);
const legacyRuntimeSlice = indexHost.slice(
  indexHost.indexOf("let legacy_acceptance_runtime_requested"),
  indexHost.indexOf("let acceptance_state"),
);
assert(
  !legacyRuntimeSlice.includes("m4r03_ordinary_clock_requested"),
  "M4R03 不得装入历史 acceptance AppState/runtime",
);

for (const token of [
  'action: "OPEN_LOOP_SNOOZE"',
  'action: "REMINDER_CREATE"',
  'action: "REMINDER_SNOOZE"',
  "operateSecretaryCoordination({",
  "operateSecretaryPersonalObject({",
  "loadSecretaryHomeContext()",
]) {
  assert(main.includes(token), `M4R03 ordinary renderer bridge 缺少 ${token}`);
}
for (const operation of [
  "arm_startup_recovery",
  "observe_startup_recovery",
  "arm_timer_tick",
  "observe_timer_tick",
  "observe_repeat",
]) {
  assert(main.includes(`\"${operation}\"`), `M4R03 renderer operation 缺少 ${operation}`);
}
assert(
  main.includes("m4r03WriteCommandsInvoked += 1")
    && main.includes("write_commands_invoked: m4r03WriteCommandsInvoked"),
  "部分普通写失败时必须如实回报已尝试的 command 数，不能冒充零写",
);

for (const forbidden of [
  "m4_acceptance::",
  ".ingest_workflow_attention_source(",
  ".create_personal_action(",
  ".create_reminder(",
  ".snooze_open_loop(",
  ".snooze_reminder(",
  ".advance_open_loop_clock(",
  ".fire_reminder(",
  ".run_due_transition_batch(",
  "M4SecretaryRepository::",
]) {
  assert(
    !ordinaryClockDriver.includes(forbidden),
    `M4R03 actual-App driver 禁止直接调用 ${forbidden}`,
  );
}
for (const receiptToken of [
  "ordinary_composition: true",
  "renderer_due_transition_calls: 0",
  "renderer_fire_calls: 0",
  "acceptance_wrapper_calls: 0",
  "direct_repository_seed_calls: 0",
  "direct_transition_calls: 0",
  "external_capability_attempts: 0",
  "renderer_user_schedule_marker_calls",
  "previous_phase_receipt_sha256",
]) {
  assert(ordinaryClockDriver.includes(receiptToken), `M4R03 receipt 缺少 ${receiptToken}`);
}
assert(
  ordinaryClockDriver.includes("OpenFlags::SQLITE_OPEN_READ_ONLY")
    && ordinaryClockDriver.includes("const EARLY_PROCESS_DEADLINE: Duration = Duration::from_secs(240)")
    && ordinaryClockDriver.includes("const TIMER_OBSERVATION_DELAY: Duration = Duration::from_secs(98)")
    && ordinaryClockDriver.includes("std::thread::sleep(TIMER_OBSERVATION_DELAY)")
    && ordinaryClockDriver.includes("validate_prior_receipt(")
    && ordinaryClockDriver.includes("timer_tick_bound_due_receipt_rows")
    && ordinaryClockDriver.includes("distinct_due_batch_timestamps")
    && ordinaryClockDriver.includes("!= arm_evidence.timer_fired_event_rows")
    && ordinaryClockDriver.includes("<= timer_armed_evidence.timer_fired_event_rows")
    && ordinaryClockDriver.includes("PRAGMA integrity_check")
    && ordinaryClockDriver.includes("pragma_foreign_key_check"),
  "M4R03 driver 必须只读取证并覆盖真实 60s TimerTick 的合法运行上界",
);
const rendererArmPhase = main.slice(
  main.indexOf('case "arm_startup_recovery": {'),
  main.indexOf('case "observe_startup_recovery": {'),
);
const rendererTimerArmPhase = main.slice(
  main.indexOf('case "arm_timer_tick": {'),
  main.indexOf('case "observe_timer_tick": {'),
);
assert(
  main.includes("const M4R03_STARTUP_DUE_DELAY_MS = 45_000")
    && main.includes("const M4R03_TIMER_DUE_DELAY_MS = 30_000")
    && rendererArmPhase.includes('openLoopReceipt.outcome_code !== "APPLIED"')
    && rendererArmPhase.includes('reminderReceipt.outcome_code !== "CREATED"')
    && rendererArmPhase.includes("openLoopReceipt.replayed")
    && rendererTimerArmPhase.includes('openLoopReceipt.outcome_code !== "APPLIED"')
    && rendererTimerArmPhase.includes('reminderReceipt.outcome_code !== "APPLIED"'),
  "arm marker 必须留足 pre-due SIGKILL 窗口，普通 snooze/create receipt 必须分别绑定 fresh APPLIED/CREATED",
);
const armPhase = ordinaryClockDriver.slice(
  ordinaryClockDriver.indexOf("DriverPhase::Arm =>"),
  ordinaryClockDriver.indexOf("DriverPhase::RecoveryTimer =>"),
);
assert(
  armPhase.includes("Ok(false)") && !armPhase.includes("app_handle.exit("),
  "arm 阶段必须保持真实 App 进程存活，交给 launcher 在 due 前 SIGKILL",
);

for (const token of [
  'const M4R03_SERVER_CLOCK_MODE_ARG = "--m4r03-server-clock"',
  'const M4R03_ORDINARY_CLOCK_PHASES = ["arm", "recovery_timer", "repeat"]',
  'const M4R03_ORDINARY_CLOCK_NORMAL_PHASE_TIMEOUT_MS = 270 * 1000',
  'const M4R03_ORDINARY_CLOCK_REAL_TIMER_WAIT_SECONDS = 98',
  '"syn_m4r03_ordinary_clock_driver_receipt.v1"',
  '"m4r03-server-due-clock-composite-receipt.json"',
  '"../../../docs/harness/reports/M4R03-server-due-clock-behavior-receipt.json"',
  '"sqlite_integrity_check"',
  '"foreign_key_violation_rows"',
]) {
  assert(launcher.includes(token), `M4R03 direct-App launcher 合同缺少 ${token}`);
}
const spawnStart = launcher.indexOf("function spawnM4R03OrdinaryClockApp(");
const armRunnerStart = launcher.indexOf("async function runM4R03ArmPhase(", spawnStart);
const normalRunnerStart = launcher.indexOf(
  "async function runM4R03NormalPhase(",
  armRunnerStart,
);
const suiteStart = launcher.indexOf(
  "async function runM4R03ServerClockSuite(",
  normalRunnerStart,
);
const policyStart = launcher.indexOf("function normalizeInheritedMarkerNames(", suiteStart);
assert(
  spawnStart >= 0
    && armRunnerStart > spawnStart
    && normalRunnerStart > armRunnerStart
    && suiteStart > normalRunnerStart
    && policyStart > suiteStart,
  "M4R03 launcher source slices 缺失",
);
const spawnSlice = launcher.slice(spawnStart, armRunnerStart);
const armRunner = launcher.slice(armRunnerStart, normalRunnerStart);
const normalRunner = launcher.slice(normalRunnerStart, suiteStart);
const suite = launcher.slice(suiteStart, policyStart);
assert(
  spawnSlice.includes("spawn(debugAppExecutablePath, [], {")
    && !spawnSlice.includes("MACOS_OPEN_PATH"),
  "M4R03 必须直接 spawn bundle executable，不能把 LaunchServices waiter 当 App PID",
);
assert(
  armRunner.includes("expectedProcessIdSha256: sha256(String(pid))")
    && armRunner.includes("markerMs <= receiptObservedAtMs")
    && armRunner.includes('process.child.kill("SIGKILL")')
    && armRunner.includes('launch.signal !== "SIGKILL"')
    && armRunner.includes("killedAtMs >= markerMs")
    && armRunner.includes("sigkillConfirmedAtMs >= markerMs")
    && armRunner.includes("sigkill_confirmed_at_utc")
    && armRunner.includes("M4R03_ORDINARY_CLOCK_DUE_GRACE_MS"),
  "arm 必须绑定真实 App PID，在 due 前 SIGKILL 并确认 close signal",
);
assert(
  normalRunner.includes("M4R03_ORDINARY_CLOCK_NORMAL_PHASE_TIMEOUT_MS")
    && normalRunner.includes("launch.exit_code !== 0")
    && normalRunner.includes("launch.signal !== null"),
  "Recovery/Repeat 必须给 240s driver 留足正常退出上界",
);
for (const token of [
  "runM4R02OrdinaryCompositionSuite({",
  "new Set(Object.values(phaseNonces)).size",
  "nonce: phaseNonces.arm",
  "nonce: phaseNonces.recovery_timer",
  "nonce: phaseNonces.repeat",
  "expectedPreviousReceiptSha256: arm.receipt_sha256",
  "expectedPreviousReceiptSha256: recoveryTimer.receipt_sha256",
  '"arm_startup_object_binding"',
  '"arm_startup_open_loop_revision"',
  '"arm_startup_reminder_revision"',
  '"arm_startup_timer_fired_baseline"',
  '"repeat_zero_delta_evidence"',
]) {
  assert(suite.includes(token), `M4R03 cross-launch runner 合同缺少 ${token}`);
}
const rejectedStart = launcher.indexOf("...(m4r03ServerClockSuite ?? {");
const rejectedEnd = launcher.indexOf(
  "...(failureStage ? { failure_stage: failureStage } : {})",
  rejectedStart,
);
const rejected = launcher.slice(rejectedStart, rejectedEnd);
assert(
  rejected.includes('outcome: "REJECTED"')
    && rejected.includes("ordinary_composition: false")
    && !rejected.includes("acceptance_wrapper_calls: 0")
    && !rejected.includes("direct_repository_seed_calls: 0")
    && !rejected.includes("direct_transition_calls: 0"),
  "失败 composite 不得伪造未证明的成功 zero claims",
);
const portableStart = launcher.indexOf(
  'm4r03ServerClockSuite?.outcome === "PASS"',
);
const portableEnd = launcher.indexOf("process.stdout.write", portableStart);
const portable = launcher.slice(portableStart, portableEnd);
assert(
  portableStart >= 0
    && portable.includes("!failureStage")
    && portable.includes("receipt.ordinary_composition === true")
    && portable.includes("receipt.acceptance_wrapper_calls === 0")
    && portable.includes("receipt.direct_repository_seed_calls === 0")
    && portable.includes("receipt.direct_transition_calls === 0")
    && portable.includes("writeM4R03PortableReport(receipt)"),
  "portable report 只可在 actual-App exact PASS 后落盘",
);
assert(
  launcher.includes("async function writeM4R03PortableReport(value)")
    && launcher.includes("await rename(temporaryPath, M4R03_SERVER_CLOCK_PORTABLE_REPORT_PATH)"),
  "portable report 必须通过同目录临时文件原子更新，支持 fresh 再验收",
);

console.log("m4r03 server due clock composition: ok");
