# 决策：咨询只读路豁免 6 道执行授权 guard reason（高危#3 · 用户已认）

日期：2026-06-25　性质：**高危#3（改安全闸对该路的有效行为）· 用户明确授权**
相关：spec `docs/plans/2026-06-25-s3-agent-layer-consultant-first-slice-spec-v1.md`、impl `codex_local_runner.rs::readonly_codex_consult`、任务包 `tasks/2026-06-25-s3-consultant-first-slice-v0-build-v1.md`、回交 `handoffs/2026-06-25-s3-consultant-first-slice-evidence-v1.md`

## 拍了什么

S3 咨询第一刀的只读 codex 路 `readonly_codex_consult`：照常调 `inspect_codex_local_execution_guard` 取**读相关**安全检查，但**豁免 6 道「执行身份 / 执行授权」reason**：
`user_confirmation_required` / `authorization_scope_missing` / `audit_ref_missing` / `new_session_requires_work_item_id` / `node_id_missing` / `workflow_id_missing`。

（handoff 没把这 6 道明写、只说"结构性只读自带 confinement"；主导线核实物时挖出，本记录补全。）

## 为什么安全（豁免成立的前提）

- 咨询请求由 `build_readonly_consult_request` **写死** `sandbox="read-only"` + `allowed_write_roots=[]`、不收权限参数 → **结构性只读**：codex 写不了、跑不了命令。
- 那 6 道全是 gate 给**写/执行授权**用的；只读没有可授权的执行，对此路不适用。
- **非豁免 reason 仍拦**：adapter / 路径越界 / 密钥 deny / prompt 边界 / command_plan —— 读相关安全照旧（`readonly_codex_consult` 对非豁免 reason 一律 `Err`）。
- `command_plan_for` 沙箱本体**字节未改**（只调不改、diff 无删除行）。
- 思路同 S1 `canvas_node` 排除 3 道授权 reason 的先例、不碰 guard 本体。

## 风险与边界（明知并接受）

- 此路在**任意项目**起真 codex、**无 path-lock、无授权**——**唯一防线 = sandbox=read-only**（比 worker 路的 path-lock + 授权双防薄）。
- **接受理由**：只读读文档，read-only 是恰当 confinement；单防线对「读」够。
- **守住的前提**：read-only 沙箱必须真生效（`command_plan_for` 字节未改 + read-only 已被 j2_b_b1 探针验过生效）。**若将来让此路可写/可执行 = 另起高危决策，不可顺延。**
- 登记进「无漏网」表为 `read-only-confined` 类（见 handoff §3）。

## 复核 + 授权

- 主导线核实物：读了 +101 diff、确认结构性只读 + 非豁免 reason 仍拦 + 沙箱字节未改 + 写侧没破（猫猫点菜没被写、`~/.codex/auth.json` mtime 没变）。
- 用户 2026-06-25 明确「豁免可以认」。
