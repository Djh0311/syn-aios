import json
import shutil
import sys
import time
from pathlib import Path

STATE = Path("/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json")
BACKUPS = STATE.parent / "backups"
TARGET = Path("/Users/yoyi/codex-workflow-mario-test")
TIMESTAMP_MS = str(int(time.time() * 1000))

control_id = sys.argv[1]
attempt_id = sys.argv[2]
exit_code = int(sys.argv[3])
last_message_path = sys.argv[4]

expected = ["index.html", "styles.css", "game.js", "README.md"]
existing = [name for name in expected if (TARGET / name).is_file()]
extra = []
if TARGET.exists():
    for path in TARGET.iterdir():
        if path.name not in expected:
            extra.append(path.name)

success = exit_code == 0 and len(existing) == len(expected) and not extra

data = json.loads(STATE.read_text())
BACKUPS.mkdir(parents=True, exist_ok=True)
backup = BACKUPS / f"workflow-state.v0.{TIMESTAMP_MS}.json"
shutil.copy2(STATE, backup)

for control in data.get("workflow_execution_controls", []):
    if control.get("control_id") == control_id:
        control["control_state"] = "completed" if success else "needs_changes"
        control["long_task_state"] = "completed" if success else "completed_with_boundary_warning"
        control["failure_reason"] = None if success else "Codex 会话未按允许文件清单完整创建测试项目，或出现允许范围外文件。"
        warnings = control.setdefault("warnings", [])
        warnings.extend([
            f"codex_exit_code:{exit_code}",
            f"created_files:{len(existing)}",
            f"extra_files:{len(extra)}"
        ])

for attempt in data.get("execution_attempts", []):
    if attempt.get("attempt_id") == attempt_id:
        attempt["state"] = "completed" if success else "needs_changes"
        attempt["ended_at"] = TIMESTAMP_MS
        attempt["failure_reason"] = None if success else "创建结果不满足四文件静态项目验收。"
        warnings = attempt.setdefault("warnings", [])
        warnings.extend([
            f"last_message_path:{last_message_path}",
            f"target_exists:{TARGET.exists()}",
            f"existing_files:{','.join(existing)}",
            f"extra_files:{','.join(extra)}"
        ])

data.setdefault("audit_events", []).append({
    "event_id": f"audit:workflow-mario-test-finished:{TIMESTAMP_MS}",
    "event_type": "workflow_execution_attempt_recorded",
    "target_ref": attempt_id,
    "actor_ref": "director_confirmed_desktop_shell",
    "source_kind": "workflow_control_and_codex_resume",
    "permission_level": "user_requested_write",
    "before_state": "running",
    "after_state": "completed" if success else "needs_changes",
    "created_at": TIMESTAMP_MS,
    "reason": "记录工作流测试项目创建结果和本地只读复核结论。"
})

data["updated_at"] = TIMESTAMP_MS
STATE.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n")

print(json.dumps({
    "timestamp_ms": TIMESTAMP_MS,
    "success": success,
    "existing_files": existing,
    "extra_files": extra,
    "backup": str(backup)
}, ensure_ascii=False, indent=2))
