#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys
from typing import Any


EVENT_FIELDS = [
    "event_id",
    "timestamp",
    "event_type",
    "actor",
    "role",
    "turn_id",
    "call_id",
    "tool_name",
    "text",
    "arguments",
    "output",
    "stdout",
    "stderr",
    "exit_code",
    "metadata",
    "warnings",
]

SENSITIVE_RE = re.compile(
    r"("
    r"authorization\s*[:=]\s*bearer\s+[A-Za-z0-9._~+/=-]+"
    r"|api[_ -]?key\s*[:=]\s*[A-Za-z0-9._~+/=-]{12,}"
    r"|secret\s*[:=]\s*[A-Za-z0-9._~+/=-]{12,}"
    r"|token\s*[:=]\s*[A-Za-z0-9._~+/=-]{12,}"
    r"|sk-[A-Za-z0-9]{20,}"
    r"|ghp_[A-Za-z0-9]{20,}"
    r"|xox[baprs]-[A-Za-z0-9-]{20,}"
    r"|AKIA[0-9A-Z]{16}"
    r")",
    re.IGNORECASE,
)


class TranscriptReadError(RuntimeError):
    pass


def add_warning(warnings: list[str], warning: str) -> None:
    if warning not in warnings:
        warnings.append(warning)


def is_relative_to(path: Path, parent: Path) -> bool:
    try:
        path.resolve(strict=False).relative_to(parent.resolve(strict=False))
        return True
    except ValueError:
        return False


def load_index(path: Path) -> dict[str, Any]:
    try:
        with path.open("r", encoding="utf-8") as handle:
            data = json.load(handle)
    except FileNotFoundError as exc:
        raise TranscriptReadError(f"missing_index:{path}") from exc
    except json.JSONDecodeError as exc:
        raise TranscriptReadError(f"invalid_index_json:{exc.lineno}") from exc
    except OSError as exc:
        raise TranscriptReadError(f"index_read_failed:{exc.__class__.__name__}") from exc
    if not isinstance(data, dict):
        raise TranscriptReadError("index_root_not_object")
    return data


def find_thread(index: dict[str, Any], thread_id: str) -> dict[str, Any]:
    threads = index.get("threads")
    if not isinstance(threads, list):
        raise TranscriptReadError("index_threads_not_list")
    matches = [thread for thread in threads if isinstance(thread, dict) and thread.get("thread_id") == thread_id]
    if not matches:
        raise TranscriptReadError(f"thread_not_in_index:{thread_id}")
    return matches[0]


def codex_home_from_index(index: dict[str, Any]) -> Path:
    source_stats = index.get("source_stats")
    if not isinstance(source_stats, dict):
        raise TranscriptReadError("missing_index_source_stats")
    codex_home = source_stats.get("codex_home")
    if not isinstance(codex_home, dict):
        raise TranscriptReadError("missing_index_codex_home")
    path = codex_home.get("path")
    if not isinstance(path, str) or not path:
        raise TranscriptReadError("missing_index_codex_home_path")
    return Path(path)


def allowed_rollout_dirs(index: dict[str, Any]) -> list[Path]:
    codex_home = codex_home_from_index(index)
    return [codex_home / "sessions", codex_home / "archived_sessions"]


def rollout_path_for_thread(index: dict[str, Any], thread: dict[str, Any]) -> Path:
    raw_path = thread.get("rollout_path")
    if not isinstance(raw_path, str) or not raw_path:
        raise TranscriptReadError("missing_rollout_path")

    path = Path(raw_path)
    allowed_dirs = allowed_rollout_dirs(index)
    if not any(is_relative_to(path, allowed_dir) for allowed_dir in allowed_dirs):
        raise TranscriptReadError("rollout_path_outside_allowed_session_dirs")
    if not path.exists():
        raise TranscriptReadError(f"missing_rollout_file:{path}")
    return path


def parse_jsonish(value: Any) -> Any:
    if not isinstance(value, str):
        return value
    text = value.strip()
    if not text:
        return value
    if not (text.startswith("{") or text.startswith("[")):
        return value
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return value


def has_sensitive_like_content(value: Any) -> bool:
    if value is None:
        return False
    if isinstance(value, str):
        return bool(SENSITIVE_RE.search(value))
    if isinstance(value, dict):
        return any(has_sensitive_like_content(item) for item in value.values())
    if isinstance(value, list):
        return any(has_sensitive_like_content(item) for item in value)
    return False


def strip_encrypted_content(value: Any, warnings: list[str]) -> Any:
    if isinstance(value, dict):
        clean: dict[str, Any] = {}
        for key, item in value.items():
            if key == "encrypted_content":
                add_warning(warnings, "encrypted_content_omitted")
                clean[key] = {"present": item is not None, "omitted": True}
                continue
            clean[key] = strip_encrypted_content(item, warnings)
        return clean
    if isinstance(value, list):
        return [strip_encrypted_content(item, warnings) for item in value]
    return value


def text_from_value(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, str):
        return value
    if isinstance(value, list):
        parts: list[str] = []
        for item in value:
            text = text_from_value(item)
            if text:
                parts.append(text)
        return "\n".join(parts) if parts else None
    if isinstance(value, dict):
        for key in ("text", "message", "content", "summary"):
            if key in value and key != "encrypted_content":
                text = text_from_value(value.get(key))
                if text:
                    return text
    return None


def payload_text(payload: dict[str, Any]) -> str | None:
    for key in ("message", "text", "content", "summary"):
        if key in payload:
            text = text_from_value(payload.get(key))
            if text:
                return text
    return None


def empty_event(line_number: int, item: dict[str, Any] | None = None) -> dict[str, Any]:
    event = {field: None for field in EVENT_FIELDS}
    event["event_id"] = f"line-{line_number:06d}"
    event["metadata"] = {"line_number": line_number}
    event["warnings"] = []
    if item is not None:
        event["timestamp"] = item.get("timestamp")
    return event


def base_metadata(event: dict[str, Any], raw_type: Any, payload: Any) -> None:
    metadata = event["metadata"]
    metadata["raw_type"] = raw_type
    if isinstance(payload, dict):
        metadata["payload_type"] = payload.get("type")
        metadata["payload_keys"] = sorted(str(key) for key in payload.keys())
    else:
        metadata["payload_type"] = None
        metadata["payload_value_type"] = type(payload).__name__


def set_unknown_event(event: dict[str, Any], item: Any, payload: Any) -> None:
    event["event_type"] = "unknown"
    event["actor"] = "system"
    add_warning(event["warnings"], "unknown_event_type")
    event["metadata"]["raw_event"] = strip_encrypted_content(item, event["warnings"])
    if payload is not None and not isinstance(payload, dict):
        add_warning(event["warnings"], "payload_not_object")


def command_fields_from_payload(payload: dict[str, Any], output: Any = None) -> dict[str, Any]:
    parsed_output = parse_jsonish(output if output is not None else payload.get("output"))
    fields = {
        "stdout": payload.get("stdout"),
        "stderr": payload.get("stderr"),
        "exit_code": payload.get("exit_code"),
        "output": parsed_output,
    }
    if isinstance(parsed_output, dict):
        fields["stdout"] = fields["stdout"] if fields["stdout"] is not None else parsed_output.get("stdout")
        fields["stderr"] = fields["stderr"] if fields["stderr"] is not None else parsed_output.get("stderr")
        fields["exit_code"] = (
            fields["exit_code"] if fields["exit_code"] is not None else parsed_output.get("exit_code")
        )
    return fields


def looks_like_command_result(payload: dict[str, Any], output: Any) -> bool:
    if any(key in payload for key in ("stdout", "stderr", "exit_code")):
        return True
    parsed_output = parse_jsonish(output)
    return isinstance(parsed_output, dict) and any(key in parsed_output for key in ("stdout", "stderr", "exit_code"))


def parse_event(item: Any, line_number: int) -> dict[str, Any]:
    if not isinstance(item, dict):
        event = empty_event(line_number)
        set_unknown_event(event, item, None)
        add_warning(event["warnings"], "event_not_object")
        return event

    event = empty_event(line_number, item)
    raw_type = item.get("type")
    payload = item.get("payload")
    base_metadata(event, raw_type, payload)

    if not isinstance(payload, dict):
        if raw_type in {"turn_context", "session_meta", "compacted"}:
            event["event_type"] = str(raw_type)
            event["actor"] = "system"
            add_warning(event["warnings"], "payload_not_object")
            return event
        set_unknown_event(event, item, payload)
        return event

    payload_type = payload.get("type")
    event["turn_id"] = payload.get("turn_id")
    event["call_id"] = payload.get("call_id")

    if raw_type == "turn_context":
        event["event_type"] = "turn_context"
        event["actor"] = "system"
        event["metadata"]["payload"] = strip_encrypted_content(payload, event["warnings"])
    elif raw_type == "session_meta":
        event["event_type"] = "session_meta"
        event["actor"] = "system"
        event["metadata"]["payload"] = strip_encrypted_content(payload, event["warnings"])
    elif raw_type == "compacted":
        event["event_type"] = "compacted"
        event["actor"] = "system"
        event["text"] = payload_text(payload)
        event["metadata"]["payload"] = strip_encrypted_content(payload, event["warnings"])
    elif raw_type == "event_msg":
        if payload_type == "user_message":
            event["event_type"] = "user_message"
            event["actor"] = "user"
            event["role"] = "user"
            event["text"] = payload_text(payload)
        elif payload_type == "agent_message":
            event["event_type"] = "assistant_message"
            event["actor"] = "assistant"
            event["role"] = "assistant"
            event["text"] = payload_text(payload)
        elif payload_type == "patch_apply_end":
            event["event_type"] = "command_output"
            event["actor"] = "tool"
            event["tool_name"] = "apply_patch"
            fields = command_fields_from_payload(payload)
            event.update(fields)
            event["metadata"]["status"] = payload.get("status")
            event["metadata"]["success"] = payload.get("success")
        elif payload_type in {"task_started", "task_complete", "token_count"}:
            event["event_type"] = "system_context"
            event["actor"] = "system"
            event["metadata"]["payload"] = strip_encrypted_content(payload, event["warnings"])
        else:
            set_unknown_event(event, item, payload)
    elif raw_type == "response_item":
        if payload_type == "message":
            role = payload.get("role")
            event["role"] = role if isinstance(role, str) else None
            event["actor"] = role if role in {"user", "assistant", "system"} else "assistant"
            event["event_type"] = "user_message" if role == "user" else "assistant_message"
            event["text"] = payload_text(payload)
            if "phase" in payload:
                event["metadata"]["phase"] = payload.get("phase")
        elif payload_type in {"function_call", "custom_tool_call"}:
            event["event_type"] = "tool_call"
            event["actor"] = "assistant"
            event["tool_name"] = payload.get("name")
            event["arguments"] = parse_jsonish(payload.get("arguments", payload.get("input")))
            if "status" in payload:
                event["metadata"]["status"] = payload.get("status")
        elif payload_type in {"function_call_output", "custom_tool_call_output"}:
            output = payload.get("output")
            if looks_like_command_result(payload, output):
                event["event_type"] = "command_output"
                event["actor"] = "tool"
                event.update(command_fields_from_payload(payload, output))
            else:
                event["event_type"] = "tool_result"
                event["actor"] = "tool"
                event["output"] = parse_jsonish(output)
        elif payload_type == "reasoning":
            event["event_type"] = "system_context"
            event["actor"] = "assistant"
            event["text"] = payload_text(payload)
            event["metadata"]["payload"] = strip_encrypted_content(payload, event["warnings"])
        else:
            set_unknown_event(event, item, payload)
    else:
        set_unknown_event(event, item, payload)

    for field in ("text", "arguments", "output", "stdout", "stderr", "metadata"):
        if has_sensitive_like_content(event.get(field)):
            add_warning(event["warnings"], "sensitive_like_content")
            break

    return event


def read_jsonl_events(path: Path) -> tuple[list[dict[str, Any]], list[str], dict[str, Any]]:
    events: list[dict[str, Any]] = []
    warnings: list[str] = []
    stats = {
        "line_count": 0,
        "parsed_line_count": 0,
        "bad_json_line_count": 0,
    }

    try:
        with path.open("r", encoding="utf-8") as handle:
            for line_number, line in enumerate(handle, start=1):
                stats["line_count"] += 1
                text = line.strip()
                if not text:
                    continue
                try:
                    item = json.loads(text)
                except json.JSONDecodeError:
                    stats["bad_json_line_count"] += 1
                    warnings.append(f"invalid_json_line:{line_number}")
                    continue
                stats["parsed_line_count"] += 1
                events.append(parse_event(item, line_number))
    except OSError as exc:
        raise TranscriptReadError(f"rollout_read_failed:{exc.__class__.__name__}") from exc

    return events, warnings, stats


def count_by_key(items: list[dict[str, Any]], key: str) -> dict[str, int]:
    counts: dict[str, int] = {}
    for item in items:
        value = item.get(key)
        label = str(value) if value is not None else "null"
        counts[label] = counts.get(label, 0) + 1
    return dict(sorted(counts.items()))


def build_transcript(index: dict[str, Any], thread_id: str) -> dict[str, Any]:
    thread = find_thread(index, thread_id)
    rollout_path = rollout_path_for_thread(index, thread)
    events, read_warnings, jsonl_stats = read_jsonl_events(rollout_path)
    event_type_counts = count_by_key(events, "event_type")
    raw_type_counts: dict[str, int] = {}
    payload_type_counts: dict[str, int] = {}
    for event in events:
        metadata = event.get("metadata") if isinstance(event.get("metadata"), dict) else {}
        raw_type = metadata.get("raw_type")
        payload_type = metadata.get("payload_type")
        raw_label = str(raw_type) if raw_type is not None else "null"
        payload_label = str(payload_type) if payload_type is not None else "null"
        raw_type_counts[raw_label] = raw_type_counts.get(raw_label, 0) + 1
        payload_type_counts[payload_label] = payload_type_counts.get(payload_label, 0) + 1

    event_warning_count = sum(len(event.get("warnings", [])) for event in events)
    encrypted_count = sum(
        1
        for event in events
        if "encrypted_content_omitted" in event.get("warnings", [])
    )
    sensitive_count = sum(
        1
        for event in events
        if "sensitive_like_content" in event.get("warnings", [])
    )
    unknown_count = event_type_counts.get("unknown", 0)
    warnings = list(read_warnings)
    if unknown_count:
        warnings.append(f"unknown_event_count:{unknown_count}")
    if encrypted_count:
        warnings.append(f"encrypted_content_event_count:{encrypted_count}")
    if sensitive_count:
        warnings.append(f"sensitive_like_event_count:{sensitive_count}")

    summary = {
        "total_events": len(events),
        "event_type_counts": event_type_counts,
        "unknown_event_count": unknown_count,
        "warning_count": len(warnings) + event_warning_count,
        "encrypted_content_event_count": encrypted_count,
        "sensitive_like_event_count": sensitive_count,
    }
    source_stats = {
        "index_thread_count": len(index.get("threads", [])) if isinstance(index.get("threads"), list) else None,
        "jsonl": jsonl_stats,
        "raw_type_counts": dict(sorted(raw_type_counts.items())),
        "payload_type_counts": dict(sorted(payload_type_counts.items())),
    }
    return {
        "thread_id": thread_id,
        "rollout_path": str(rollout_path),
        "project_path": thread.get("project_root"),
        "title": thread.get("title"),
        "created_at_ms": thread.get("created_at_ms"),
        "updated_at_ms": thread.get("updated_at_ms"),
        "events": events,
        "summary": summary,
        "warnings": warnings,
        "source_stats": source_stats,
    }


def write_transcript(transcript: dict[str, Any], output_path: Path, pretty: bool) -> None:
    try:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with output_path.open("w", encoding="utf-8") as handle:
            json.dump(transcript, handle, ensure_ascii=False, indent=2 if pretty else None, sort_keys=True)
            handle.write("\n")
    except OSError as exc:
        raise TranscriptReadError(f"output_write_failed:{exc.__class__.__name__}") from exc


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Read one Codex session transcript from an existing index.")
    parser.add_argument("--index", type=Path, required=True, help="Path to codex-index.json.")
    parser.add_argument("--thread-id", required=True, help="Thread id to read.")
    parser.add_argument("--output", type=Path, required=True, help="Output transcript JSON path.")
    parser.add_argument("--pretty", action="store_true")
    args = parser.parse_args(argv)

    try:
        index = load_index(args.index)
        transcript = build_transcript(index, args.thread_id)
        write_transcript(transcript, args.output, args.pretty)
    except TranscriptReadError as exc:
        print(f"transcript_read_failed:{exc}", file=sys.stderr)
        return 1

    print(
        json.dumps(
            {
                "output": str(args.output),
                "thread_id": transcript["thread_id"],
                "event_count": transcript["summary"]["total_events"],
                "warning_count": transcript["summary"]["warning_count"],
                "unknown_event_count": transcript["summary"]["unknown_event_count"],
                "bad_json_line_count": transcript["source_stats"]["jsonl"]["bad_json_line_count"],
            },
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
