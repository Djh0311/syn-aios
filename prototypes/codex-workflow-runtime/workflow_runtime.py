#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys
import time
import uuid
from typing import Any


WORKFLOW_VERSION = "codex-workflow-runtime-v1"
DEFAULT_GOAL = "请让执行线完成一个无业务控制探针。执行线只需返回 WORKER_DONE_2026_05_29。"
REQUIRED_APPROVAL_TEXT = "批准执行 Codex 工作流运行模型 v1 的真实无业务探针"
DEFAULT_OUTPUT_DIR = Path("/tmp/codex-workflow-runtime-v1")
REPO_ROOT = Path(__file__).resolve().parents[3]
INDEX_KERNEL_DIR = REPO_ROOT / "product-line" / "prototypes" / "index-kernel"
BUILD_INDEX_PATH = INDEX_KERNEL_DIR / "build_index.py"
TRANSCRIPT_READER_PATH = INDEX_KERNEL_DIR / "transcript_reader.py"


class RuntimeErrorWithCode(RuntimeError):
    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


def now_ms() -> int:
    return int(time.time() * 1000)


def iso_from_ms(timestamp_ms: int) -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(timestamp_ms / 1000))


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(data, handle, ensure_ascii=False, indent=2, sort_keys=True)
        handle.write("\n")


def read_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise RuntimeErrorWithCode("json_root_not_object", f"{path} root is not an object")
    return data


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def artifact(path: Path, kind: str, *, description: str) -> dict[str, Any]:
    return {
        "artifact_id": path.stem.replace(".", "_"),
        "kind": kind,
        "path": str(path),
        "description": description,
    }


def event(
    *,
    event_id: str,
    event_type: str,
    node_id: str,
    summary: str,
    artifact_refs: list[str] | None = None,
    timestamp_ms: int | None = None,
) -> dict[str, Any]:
    return {
        "event_id": event_id,
        "event_type": event_type,
        "node_id": node_id,
        "timestamp_ms": timestamp_ms or now_ms(),
        "summary": summary,
        "artifact_refs": artifact_refs or [],
    }


def node(
    *,
    node_id: str,
    node_type: str,
    role: str,
    status: str,
    input_ref: str | None = None,
    output_ref: str | None = None,
    session_ref: str | None = None,
    warnings: list[str] | None = None,
    started_at_ms: int | None = None,
    ended_at_ms: int | None = None,
) -> dict[str, Any]:
    return {
        "node_id": node_id,
        "node_type": node_type,
        "role": role,
        "status": status,
        "started_at_ms": started_at_ms,
        "ended_at_ms": ended_at_ms,
        "input_ref": input_ref,
        "output_ref": output_ref,
        "session_ref": session_ref,
        "warnings": warnings or [],
    }


def base_run(goal: str, output_dir: Path, *, mode: str) -> dict[str, Any]:
    created_at_ms = now_ms()
    run_id = f"workflow-{uuid.uuid4()}"
    artifacts = [
        artifact(output_dir / "director-task.json", "task_instruction", description="结构化任务指令"),
        artifact(output_dir / "worker-events.jsonl", "codex_json_events", description="执行线 Codex JSONL 事件"),
        artifact(output_dir / "worker-last-message.txt", "codex_last_message", description="执行线最终回复"),
        artifact(output_dir / "index.json", "temporary_index", description="执行后临时索引"),
        artifact(output_dir / "worker-transcript.json", "transcript", description="执行线 transcript 读回"),
        artifact(output_dir / "director-review.json", "director_review", description="总指导回收意见"),
        artifact(output_dir / "run.json", "workflow_run", description="本次运行状态"),
    ]
    nodes = [
        node(
            node_id="director_plan",
            node_type="director_plan",
            role="director",
            status="pending",
            output_ref="director-task",
        ),
        node(
            node_id="worker_run",
            node_type="worker_run",
            role="worker",
            status="pending",
            input_ref="director-task",
            output_ref="worker-last-message",
            session_ref="worker-session",
        ),
        node(
            node_id="worker_readback",
            node_type="worker_readback",
            role="orchestrator",
            status="pending",
            input_ref="worker-session",
            output_ref="worker-transcript",
            session_ref="worker-session",
        ),
        node(
            node_id="director_review",
            node_type="director_review",
            role="director",
            status="pending",
            input_ref="worker-transcript",
            output_ref="director-review",
        ),
        node(
            node_id="runtime_summary",
            node_type="runtime_summary",
            role="orchestrator",
            status="pending",
            input_ref="director-review",
            output_ref="run",
        ),
    ]
    return {
        "run_id": run_id,
        "created_at_ms": created_at_ms,
        "created_at": iso_from_ms(created_at_ms),
        "workflow_version": WORKFLOW_VERSION,
        "mode": mode,
        "goal": goal,
        "status": "planned",
        "state_flow": [],
        "nodes": nodes,
        "edges": [
            {"from": "director_plan", "to": "worker_run"},
            {"from": "worker_run", "to": "worker_readback"},
            {"from": "worker_readback", "to": "director_review"},
            {"from": "director_review", "to": "runtime_summary"},
        ],
        "events": [],
        "sessions": {
            "worker-session": {
                "session_id": None,
                "thread_id": None,
                "rollout_path": None,
                "created_new": False,
                "existing_business_session_touched": False,
                "prompt_kind": "no_business_test",
                "warnings": [],
            }
        },
        "artifacts": artifacts,
        "warnings": [],
    }


def find_node(run: dict[str, Any], node_id: str) -> dict[str, Any]:
    for item in run["nodes"]:
        if item["node_id"] == node_id:
            return item
    raise RuntimeErrorWithCode("missing_node", f"missing node {node_id}")


def set_node_status(
    run: dict[str, Any],
    node_id: str,
    status: str,
    *,
    output_ref: str | None = None,
    warning: str | None = None,
) -> None:
    item = find_node(run, node_id)
    timestamp = now_ms()
    if item["started_at_ms"] is None:
        item["started_at_ms"] = timestamp
    item["status"] = status
    if output_ref is not None:
        item["output_ref"] = output_ref
    if status in {"completed", "accepted", "blocked", "failed"}:
        item["ended_at_ms"] = timestamp
    if warning:
        item["warnings"].append(warning)


def append_state(run: dict[str, Any], status: str) -> None:
    run["status"] = status
    run["state_flow"].append({"status": status, "timestamp_ms": now_ms()})


def director_task(goal: str) -> dict[str, Any]:
    return {
        "task_id": "worker-control-probe",
        "role": "worker",
        "goal": goal,
        "required_response": "WORKER_DONE_2026_05_29",
        "constraints": [
            "无业务内容",
            "不修改项目文件",
            "只返回指定完成标记",
        ],
    }


def worker_prompt(task: dict[str, Any]) -> str:
    return (
        "你是 Codex 工作流运行模型 v1 的执行线测试会话。\n"
        "这是无业务控制探针，不要修改文件，不要运行命令。\n"
        f"任务目标：{task['goal']}\n"
        f"请只回复：{task['required_response']}"
    )


def director_review(last_message: str | None, transcript_summary: dict[str, Any] | None) -> dict[str, Any]:
    accepted = bool(last_message and "WORKER_DONE_2026_05_29" in last_message)
    return {
        "review_id": "director-review",
        "decision": "accepted" if accepted else "needs_review",
        "basis": [
            "worker last message contains required marker" if accepted else "worker marker missing",
            "transcript summary present" if transcript_summary is not None else "transcript summary unavailable",
        ],
        "required_marker": "WORKER_DONE_2026_05_29",
        "transcript_summary": transcript_summary or {},
    }


def write_dry_run_artifacts(run: dict[str, Any], output_dir: Path) -> None:
    task = director_task(run["goal"])
    write_json(output_dir / "director-task.json", task)
    write_text(output_dir / "worker-events.jsonl", "")
    write_text(output_dir / "worker-last-message.txt", "DRY_RUN_PLACEHOLDER_WORKER_DONE_2026_05_29\n")
    write_json(output_dir / "index.json", {"dry_run": True, "threads": [], "warnings": []})
    write_json(
        output_dir / "worker-transcript.json",
        {
            "dry_run": True,
            "thread_id": None,
            "summary": {"total_events": 0},
            "warnings": ["dry_run_no_transcript"],
        },
    )
    review = director_review("DRY_RUN_PLACEHOLDER_WORKER_DONE_2026_05_29", {"total_events": 0, "dry_run": True})
    write_json(output_dir / "director-review.json", review)


def run_dry_run(goal: str, output_dir: Path) -> dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    run = base_run(goal, output_dir, mode="dry-run")

    append_state(run, "planned")
    set_node_status(run, "director_plan", "completed")
    run["events"].append(
        event(
            event_id="event-001",
            event_type="planned",
            node_id="director_plan",
            summary="总指导生成结构化任务指令",
            artifact_refs=["director-task"],
        )
    )

    append_state(run, "dispatched")
    set_node_status(run, "worker_run", "completed", warning="dry_run_no_codex_cli_execution")
    run["sessions"]["worker-session"]["warnings"].append("dry_run_no_real_session")
    run["events"].append(
        event(
            event_id="event-002",
            event_type="dispatched",
            node_id="worker_run",
            summary="dry-run 模式下只记录执行线会话占位",
            artifact_refs=["worker-events", "worker-last-message"],
        )
    )

    append_state(run, "running")
    set_node_status(run, "worker_readback", "completed", warning="dry_run_no_transcript")
    run["events"].append(
        event(
            event_id="event-003",
            event_type="running",
            node_id="worker_readback",
            summary="编排器进入执行结果读回阶段",
            artifact_refs=["worker-transcript"],
        )
    )
    run["events"].append(
        event(
            event_id="event-004",
            event_type="reported",
            node_id="worker_readback",
            summary="dry-run 模式下生成 transcript 占位摘要",
            artifact_refs=["worker-transcript"],
        )
    )

    append_state(run, "reported")
    set_node_status(run, "director_review", "completed")
    run["events"].append(
        event(
            event_id="event-005",
            event_type="recovered",
            node_id="director_review",
            summary="总指导基于 dry-run 占位结果生成回收意见",
            artifact_refs=["director-review"],
        )
    )

    append_state(run, "recovered")
    set_node_status(run, "runtime_summary", "accepted")
    run["events"].append(
        event(
            event_id="event-006",
            event_type="accepted",
            node_id="runtime_summary",
            summary="dry-run 状态流完整走到 accepted",
            artifact_refs=["run"],
        )
    )
    append_state(run, "accepted")

    run["warnings"].extend(["dry_run_only", "real_codex_probe_not_executed"])
    write_dry_run_artifacts(run, output_dir)
    write_json(output_dir / "run.json", run)
    return run


def require_real_probe_approval(approval_text: str | None) -> None:
    if approval_text != REQUIRED_APPROVAL_TEXT:
        raise RuntimeErrorWithCode(
            "real_probe_not_approved",
            f"real probe requires exact approval text: {REQUIRED_APPROVAL_TEXT}",
        )


def command_to_text(command: list[str]) -> str:
    return " ".join(command)


def run_command(command: list[str], *, cwd: Path, stdout_path: Path | None = None) -> subprocess.CompletedProcess[str]:
    if stdout_path is None:
        return subprocess.run(command, cwd=str(cwd), capture_output=True, text=True, check=False)
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("w", encoding="utf-8") as stdout_handle:
        return subprocess.run(
            command,
            cwd=str(cwd),
            stdout=stdout_handle,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )


def latest_thread(index_path: Path) -> dict[str, Any] | None:
    data = read_json(index_path)
    threads = data.get("threads")
    if not isinstance(threads, list) or not threads:
        return None
    candidates = [item for item in threads if isinstance(item, dict) and item.get("rollout_exists")]
    if not candidates:
        return None
    return sorted(candidates, key=lambda item: item.get("updated_at_ms") or 0, reverse=True)[0]


def run_real_codex_probe(goal: str, output_dir: Path, approval_text: str | None) -> dict[str, Any]:
    require_real_probe_approval(approval_text)
    output_dir.mkdir(parents=True, exist_ok=True)
    run = base_run(goal, output_dir, mode="real-codex-probe")
    task = director_task(goal)
    task_path = output_dir / "director-task.json"
    events_path = output_dir / "worker-events.jsonl"
    last_message_path = output_dir / "worker-last-message.txt"
    index_path = output_dir / "index.json"
    transcript_path = output_dir / "worker-transcript.json"
    review_path = output_dir / "director-review.json"
    write_json(task_path, task)

    append_state(run, "planned")
    set_node_status(run, "director_plan", "completed")
    run["events"].append(
        event(
            event_id="event-001",
            event_type="planned",
            node_id="director_plan",
            summary="总指导生成结构化任务指令",
            artifact_refs=["director-task"],
        )
    )

    prompt = worker_prompt(task)
    command = [
        "codex",
        "exec",
        "--skip-git-repo-check",
        "--json",
        "--output-last-message",
        str(last_message_path),
        prompt,
    ]
    append_state(run, "dispatched")
    append_state(run, "running")
    set_node_status(run, "worker_run", "running")
    run["events"].append(
        event(
            event_id="event-002",
            event_type="dispatched",
            node_id="worker_run",
            summary="执行线测试会话已通过 codex exec 启动",
            artifact_refs=["worker-events", "worker-last-message"],
        )
    )
    run["warnings"].append("real_codex_probe_executed_with_explicit_approval")
    run["events"].append(
        event(
            event_id="event-003",
            event_type="command_started",
            node_id="worker_run",
            summary=command_to_text(command[:6]) + " <no-business-prompt>",
            artifact_refs=["worker-events"],
        )
    )
    completed = run_command(command, cwd=REPO_ROOT, stdout_path=events_path)
    if completed.returncode != 0:
        set_node_status(run, "worker_run", "failed", warning=f"codex_exec_exit_code:{completed.returncode}")
        run["warnings"].append(f"codex_exec_failed:{completed.returncode}")
        write_text(output_dir / "worker-stderr.txt", completed.stderr or "")
        append_state(run, "failed")
        write_json(output_dir / "run.json", run)
        return run

    set_node_status(run, "worker_run", "completed")
    run["events"].append(
        event(
            event_id="event-004",
            event_type="running",
            node_id="worker_run",
            summary="执行线测试会话已完成，准备读回结果",
            artifact_refs=["worker-events", "worker-last-message"],
        )
    )
    run["events"].append(
        event(
            event_id="event-005",
            event_type="reported",
            node_id="worker_run",
            summary="执行线测试会话完成",
            artifact_refs=["worker-events", "worker-last-message"],
        )
    )
    append_state(run, "reported")

    build_index_cmd = ["python3", str(BUILD_INDEX_PATH), "--output", str(index_path)]
    index_completed = run_command(build_index_cmd, cwd=REPO_ROOT)
    if index_completed.returncode != 0:
        set_node_status(run, "worker_readback", "failed", warning="temporary_index_failed")
        run["warnings"].append("temporary_index_failed")
        append_state(run, "failed")
        write_json(output_dir / "run.json", run)
        return run

    thread = latest_thread(index_path)
    if thread is None or not isinstance(thread.get("thread_id"), str):
        set_node_status(run, "worker_readback", "failed", warning="new_thread_not_found")
        run["warnings"].append("new_thread_not_found")
        append_state(run, "failed")
        write_json(output_dir / "run.json", run)
        return run

    run["sessions"]["worker-session"].update(
        {
            "session_id": thread.get("thread_id"),
            "thread_id": thread.get("thread_id"),
            "rollout_path": thread.get("rollout_path"),
            "created_new": True,
            "existing_business_session_touched": False,
        }
    )

    transcript_cmd = [
        "python3",
        str(TRANSCRIPT_READER_PATH),
        "--index",
        str(index_path),
        "--thread-id",
        str(thread["thread_id"]),
        "--output",
        str(transcript_path),
    ]
    transcript_completed = run_command(transcript_cmd, cwd=REPO_ROOT)
    if transcript_completed.returncode != 0:
        set_node_status(run, "worker_readback", "failed", warning="transcript_readback_failed")
        run["warnings"].append("transcript_readback_failed")
        append_state(run, "failed")
        write_json(output_dir / "run.json", run)
        return run

    set_node_status(run, "worker_readback", "completed")
    transcript = read_json(transcript_path)
    last_message = last_message_path.read_text(encoding="utf-8") if last_message_path.exists() else ""
    run["events"].append(
        event(
            event_id="event-006",
            event_type="read_back",
            node_id="worker_readback",
            summary="transcript reader 已读回执行线测试会话",
            artifact_refs=["index", "worker-transcript"],
        )
    )

    review = director_review(last_message, transcript.get("summary") if isinstance(transcript, dict) else None)
    write_json(review_path, review)
    set_node_status(run, "director_review", "completed")
    run["events"].append(
        event(
            event_id="event-007",
            event_type="recovered",
            node_id="director_review",
            summary=f"总指导回收意见：{review['decision']}",
            artifact_refs=["director-review"],
        )
    )
    append_state(run, "recovered")

    final_status = "accepted" if review["decision"] == "accepted" else "needs_review"
    set_node_status(run, "runtime_summary", final_status)
    append_state(run, final_status)
    run["events"].append(
        event(
            event_id="event-008",
            event_type=final_status,
            node_id="runtime_summary",
            summary=f"运行模型状态流结束于 {final_status}",
            artifact_refs=["run"],
        )
    )
    write_json(output_dir / "run.json", run)
    return run


def validate_run(run: dict[str, Any]) -> list[str]:
    problems: list[str] = []
    for key in [
        "run_id",
        "created_at_ms",
        "workflow_version",
        "goal",
        "nodes",
        "edges",
        "events",
        "sessions",
        "artifacts",
        "warnings",
        "status",
    ]:
        if key not in run:
            problems.append(f"missing_top_level_key:{key}")
    if not isinstance(run.get("nodes"), list) or len(run.get("nodes", [])) != 5:
        problems.append("unexpected_node_count")
    if not isinstance(run.get("edges"), list) or len(run.get("edges", [])) != 4:
        problems.append("unexpected_edge_count")
    node_ids = {item.get("node_id") for item in run.get("nodes", []) if isinstance(item, dict)}
    expected_nodes = {"director_plan", "worker_run", "worker_readback", "director_review", "runtime_summary"}
    if node_ids != expected_nodes:
        problems.append("unexpected_node_ids")
    state_values = [item.get("status") for item in run.get("state_flow", []) if isinstance(item, dict)]
    if run.get("mode") == "dry-run" and state_values != [
        "planned",
        "dispatched",
        "running",
        "reported",
        "recovered",
        "accepted",
    ]:
        problems.append("dry_run_state_flow_not_accepted")
    return problems


def summarize_run(run: dict[str, Any]) -> dict[str, Any]:
    return {
        "run_id": run.get("run_id"),
        "mode": run.get("mode"),
        "status": run.get("status"),
        "state_flow": [item.get("status") for item in run.get("state_flow", []) if isinstance(item, dict)],
        "node_statuses": {
            item.get("node_id"): item.get("status") for item in run.get("nodes", []) if isinstance(item, dict)
        },
        "worker_thread_id": run.get("sessions", {}).get("worker-session", {}).get("thread_id"),
        "artifact_count": len(run.get("artifacts", [])) if isinstance(run.get("artifacts"), list) else None,
        "warning_count": len(run.get("warnings", [])) if isinstance(run.get("warnings"), list) else None,
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Run the Codex workflow runtime model v1 prototype.")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--dry-run", action="store_true", help="Generate a structured run without Codex CLI.")
    mode.add_argument(
        "--real-codex-probe",
        action="store_true",
        help="Run a real no-business Codex probe. Requires exact approval text.",
    )
    parser.add_argument("--approval-text", help="Exact approval text required for --real-codex-probe.")
    parser.add_argument("--goal", default=DEFAULT_GOAL)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    args = parser.parse_args(argv)

    try:
        if args.dry_run:
            run = run_dry_run(args.goal, args.output_dir)
        else:
            run = run_real_codex_probe(args.goal, args.output_dir, args.approval_text)
        problems = validate_run(run)
        if problems:
            for problem in problems:
                print(f"runtime_validation_failed:{problem}", file=sys.stderr)
            return 1
    except RuntimeErrorWithCode as exc:
        print(f"workflow_runtime_failed:{exc.code}:{exc}", file=sys.stderr)
        return 2
    except OSError as exc:
        print(f"workflow_runtime_failed:io:{exc.__class__.__name__}:{exc}", file=sys.stderr)
        return 2

    print(json.dumps(summarize_run(run), ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
