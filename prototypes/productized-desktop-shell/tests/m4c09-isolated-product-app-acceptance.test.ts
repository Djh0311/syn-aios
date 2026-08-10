import { assert } from "./helpers/offlineInteractionTestUtils";

const nodeProcess = (globalThis as typeof globalThis & {
  process?: { cwd?: () => string };
}).process;
if (!nodeProcess?.cwd) throw new Error("M4C09 静态验收需要 Node cwd");
const nodeFsSpecifier: string = "node:fs";
const { readFileSync } = await import(nodeFsSpecifier) as {
  readFileSync: (path: string, encoding: "utf8") => string;
};
const root = nodeProcess.cwd();
const rust = readFileSync(`${root}/src-tauri/src/m4_acceptance.rs`, "utf8");
const library = readFileSync(`${root}/src-tauri/src/lib.rs`, "utf8");
const commands = readFileSync(`${root}/src-tauri/src/commands.rs`, "utf8");
const registry = readFileSync(`${root}/src-tauri/src/command_registry.rs`, "utf8");
const launcher = readFileSync(`${root}/scripts/run-r4-isolated-app-preflight.mjs`, "utf8");
const wrapper = readFileSync(`${root}/scripts/run-m4-isolated-app-acceptance.mjs`, "utf8");

assert(
  library.includes("m4_acceptance::install_for_validated_profile(&paths)?")
    && library.indexOf("m4_acceptance::install_for_validated_profile(&paths)?")
      < library.indexOf("m3_acceptance::install_for_validated_profile(&paths)?"),
  "C09 必须先按显式 gate 安装普通 M3/M4 隔离组合，再回到既有 M3C07 分支",
);
assert(
  registry.includes("crate::m3_acceptance::reject_unapproved_tauri_command(&command)")
    && registry.includes(
      ".and_then(|_| crate::m4_acceptance::reject_unapproved_tauri_command(&command))",
    )
    && registry.includes("invoke.resolver.reject(error);"),
  "C09 必须在全局 invoke dispatch 前串接服务端 fail-closed 门禁",
);
for (const command of [
  "load_m4c09_acceptance_status",
  "m4c09_run_secretary_explain",
  "m4c09_load_secretary_home_context",
  "m4c09_operate_secretary_coordination",
]) {
  assert(registry.includes(command), `C09 registry 缺少 ${command}`);
}
assert(
  commands.includes("fn load_m4c09_acceptance_status(\n)")
    && !commands.includes("load_m4c09_acceptance_status(\n    request"),
  "C09 acceptance status IPC 必须是真正的无 renderer request 命令",
);

for (const token of [
  "SYN_M4C09_ISOLATED_ACCEPTANCE",
  "--m4c09-isolated-acceptance",
  "m4c09-runtime-receipt.json",
  "runM4C09SameProfileRestart",
  "m4c09RuntimeReceiptComplete",
  "same_profile_reused",
  "runtime_receipt_complete",
  "delete normalBuildEnvironment[M4C09_MODE_ENV]",
]) {
  assert(launcher.includes(token), `C09 launcher 缺少 ${token}`);
}
assert(
  wrapper.includes('[preflight, "--m4c09-isolated-acceptance"]')
    && wrapper.includes("process.argv.length !== 2")
    && !wrapper.includes("process.argv.slice")
    && !wrapper.includes("process.argv[2]"),
  "C09 wrapper 必须是固定入口，且拒绝 profile/provider/credential 参数",
);

for (const invariant of [
  "M4C09_FAKE_MODEL_FAILURE",
  "zero_item_read_model_calls",
  "deterministic_brief_unchanged_after_failure",
  "terminal_failure_recovered_after_restart",
  "carried_over_receipt_recovered",
  "external_network_writes: 0",
  "real_codex_message_attempts: 0",
  "MECHANICAL_AND_ISOLATED_PRODUCT_APP_ONLY_NOT_REAL_DAILY_USE",
]) {
  assert(rust.includes(invariant), `C09 Rust receipt 缺少 ${invariant}`);
}
assert(
  rust.includes("resolve_m4_primary_secretary_identity()")
    && rust.includes("status.role_ref != binding.role_ref.as_str()")
    && rust.includes("M4_PRIMARY_SECRETARY_SCOPE_ID"),
  "C09 必须先核对 M3 opaque binding，再由固定 identity 映射 canonical PersonalScope",
);

console.log("m4c09-isolated-product-app-acceptance: ok");
