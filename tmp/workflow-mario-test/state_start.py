import json
import shutil
import time
from pathlib import Path

STATE = Path("/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json")
BACKUPS = STATE.parent / "backups"
TIMESTAMP_MS = str(int(time.time() * 1000))

CONTROL_ID = f"control:workflow-mario-test-project:{TIMESTAMP_MS}"
ATTEMPT_ID = f"attempt:workflow-mario-test-project:{TIMESTAMP_MS}"
WORKFLOW_ID = "workflow:users-yoyi-gameai-agent-world:default"
WORK_ITEM_ID = "work-item:workflow:users-yoyi-gameai-agent-world:default:1780032043420"
PROJECT_ID = "project:users-yoyi-gameai-agent-world"
TARGET_DIR = "/Users/yoyi/codex-workflow-mario-test"
THREAD_ID = "019e76d9-0f67-7433-81eb-72da585d28a4"

data = json.loads(STATE.read_text())
BACKUPS.mkdir(parents=True, exist_ok=True)
backup = BACKUPS / f"workflow-state.v0.{TIMESTAMP_MS}.json"
shutil.copy2(STATE, backup)

data.setdefault("workflow_execution_controls", []).append({
    "control_id": CONTROL_ID,
    "project_id": PROJECT_ID,
    "workflow_id": WORKFLOW_ID,
    "work_item_id": WORK_ITEM_ID,
    "control_state": "running",
    "long_task_state": "running",
    "retry_count": 0,
    "max_retries": 0,
    "timeout_seconds": 900,
    "cancel_requested_at": None,
    "failure_reason": None,
    "user_reviewed_instruction": {
        "instruction_id": "user-reviewed-instruction:workflow-mario-test-project-v1",
        "summary": "在 /Users/yoyi 下创建测试专用静态网页小游戏项目，用于验证工作流真实写入和回收。",
        "objective": "验证工作流能否向绑定 Codex 会话派发一个小型真实写入任务，并完成结果回收。",
        "allowed_reads": [
            f"{TARGET_DIR} 下本次创建的 index.html、styles.css、game.js、README.md"
        ],
        "allowed_writes": [
            TARGET_DIR,
            f"{TARGET_DIR}/index.html",
            f"{TARGET_DIR}/styles.css",
            f"{TARGET_DIR}/game.js",
            f"{TARGET_DIR}/README.md"
        ],
        "forbidden_actions": [
            "不读取 auth.json、.env、密钥、token、授权文件",
            "不读取完整 transcript",
            "不修改 /Users/yoyi/gameai/agent world",
            "不修改 /Users/yoyi/workspace/product-line",
            "不安装依赖、不联网、不运行 harness",
            "不删除、移动、归档任何 Codex 会话"
        ],
        "required_return": [
            "薄弱点",
            "创建了哪些文件",
            "是否写了允许范围外的文件",
            "是否读取了敏感文件",
            "如何运行",
            "自检结果"
        ],
        "approval_state": "user_requested_execution",
        "preview_markdown": f"工作流测试：在 {TARGET_DIR} 创建一个无依赖静态网页跳跃小游戏。"
    },
    "audit_event_types": [
        "workflow_user_reviewed_instruction_dispatched",
        "workflow_execution_attempt_started"
    ],
    "warnings": []
})

data.setdefault("execution_attempts", []).append({
    "attempt_id": ATTEMPT_ID,
    "project_id": PROJECT_ID,
    "workflow_id": WORKFLOW_ID,
    "work_item_id": WORK_ITEM_ID,
    "dispatch_id": None,
    "attempt_no": 1,
    "state": "running",
    "started_at": TIMESTAMP_MS,
    "ended_at": None,
    "failure_reason": None,
    "retry_scheduled_at": None,
    "timed_out_at": None,
    "cancel_requested_at": None,
    "warnings": [
        f"target_dir:{TARGET_DIR}",
        f"native_thread_id:{THREAD_ID}",
        "codex_exec_resume_will_write_codex_home",
        "second_attempt_should_use_cd_users_yoyi_if_target_parent_write_is_needed"
    ]
})

data.setdefault("audit_events", []).append({
    "event_id": f"audit:workflow-mario-test-started:{TIMESTAMP_MS}",
    "event_type": "workflow_user_reviewed_instruction_dispatched",
    "target_ref": WORK_ITEM_ID,
    "actor_ref": "director_confirmed_desktop_shell",
    "source_kind": "workflow_control_and_codex_resume",
    "permission_level": "user_requested_write",
    "before_state": "needs_changes",
    "after_state": "running",
    "created_at": TIMESTAMP_MS,
    "reason": "用户要求用新建测试项目验证工作流能否跑通；本记录只声明控制项开始，实际项目文件应由绑定 Codex 会话创建。"
})

data["updated_at"] = TIMESTAMP_MS
STATE.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n")

print(json.dumps({
    "timestamp_ms": TIMESTAMP_MS,
    "control_id": CONTROL_ID,
    "attempt_id": ATTEMPT_ID,
    "backup": str(backup)
}, ensure_ascii=False, indent=2))
