#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import dataclass
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
from typing import Any, Callable


COMMANDS_TO_PROBE = [
    ("version", ["--version"]),
    ("help", ["--help"]),
    ("exec_help", ["exec", "--help"]),
    ("exec_resume_help", ["exec", "resume", "--help"]),
    ("resume_help", ["resume", "--help"]),
    ("fork_help", ["fork", "--help"]),
    ("mcp_server_help", ["mcp-server", "--help"]),
    ("app_server_help", ["app-server", "--help"]),
    ("remote_control_help", ["remote-control", "--help"]),
]

CAPABILITY_NAMES = [
    "discover_cli",
    "inspect_help",
    "create_session",
    "resume_session",
    "send_prompt",
    "wait_for_result",
    "read_back_with_transcript",
]

STATUS_VALUES = {"supported", "unsupported", "unknown", "blocked"}

KEYWORD_RE = re.compile(
    r"\b(exec|resume|fork|prompt|session|non-interactively|ephemeral|json|output-last-message|"
    r"mcp-server|app-server|remote-control|daemon|stdio|websocket|experimental)\b",
    re.IGNORECASE,
)

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

MAX_MATCHED_LINES = 14
MAX_LINE_CHARS = 220
DEFAULT_TIMEOUT_SECONDS = 8


@dataclass(frozen=True)
class CommandResult:
    args: list[str]
    exit_code: int | None
    stdout: str
    stderr: str
    timed_out: bool = False


CommandRunner = Callable[[list[str], int], CommandResult]


def utc_now_iso() -> str:
    return dt.datetime.now(dt.UTC).isoformat(timespec="seconds").replace("+00:00", "Z")


def safe_sha256_12(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", errors="replace")).hexdigest()[:12]


def sanitize_line(line: str) -> str:
    clean = SENSITIVE_RE.sub("[REDACTED_SENSITIVE]", line.strip())
    if len(clean) > MAX_LINE_CHARS:
        clean = clean[:MAX_LINE_CHARS].rstrip() + "..."
    return clean


def output_warning_codes(text: str) -> list[str]:
    warnings: list[str] = []
    lowered = text.lower()
    if "could not update path" in lowered:
        warnings.append("codex_path_update_warning")
    if SENSITIVE_RE.search(text):
        warnings.append("sensitive_like_help_output_redacted")
    return warnings


def matched_lines(text: str) -> list[str]:
    lines: list[str] = []
    seen: set[str] = set()
    for raw_line in text.splitlines():
        if not KEYWORD_RE.search(raw_line):
            continue
        line = sanitize_line(raw_line)
        if not line or line in seen:
            continue
        seen.add(line)
        lines.append(line)
        if len(lines) >= MAX_MATCHED_LINES:
            break
    return lines


def run_command(args: list[str], timeout_seconds: int) -> CommandResult:
    try:
        completed = subprocess.run(
            args,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as exc:
        return CommandResult(
            args=args,
            exit_code=None,
            stdout=exc.stdout or "",
            stderr=exc.stderr or "",
            timed_out=True,
        )
    except OSError as exc:
        return CommandResult(args=args, exit_code=None, stdout="", stderr=exc.__class__.__name__)
    return CommandResult(
        args=args,
        exit_code=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
    )


def summarize_command(label: str, result: CommandResult) -> dict[str, Any]:
    combined = "\n".join(part for part in [result.stdout, result.stderr] if part)
    warnings = output_warning_codes(combined)
    return {
        "label": label,
        "command": " ".join(result.args),
        "exit_code": result.exit_code,
        "timed_out": result.timed_out,
        "stdout_line_count": len(result.stdout.splitlines()),
        "stderr_line_count": len(result.stderr.splitlines()),
        "stdout_sha256_12": safe_sha256_12(result.stdout),
        "stderr_sha256_12": safe_sha256_12(result.stderr),
        "matched_lines": matched_lines(combined),
        "warnings": warnings,
    }


def first_non_warning_line(text: str) -> str | None:
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line:
            continue
        if line.upper().startswith("WARNING:"):
            continue
        return sanitize_line(line)
    return None


def parse_commands_from_help(text: str) -> list[str]:
    commands: list[str] = []
    in_commands = False
    for raw_line in text.splitlines():
        line = raw_line.rstrip()
        if line.strip() == "Commands:":
            in_commands = True
            continue
        if in_commands and line and not line.startswith(" "):
            break
        if not in_commands:
            continue
        match = re.match(r"\s{2}([a-z][a-z0-9-]+)\s{2,}", line)
        if match:
            commands.append(match.group(1))
    return commands


def text_for(outputs: dict[str, CommandResult], key: str) -> str:
    result = outputs.get(key)
    if result is None:
        return ""
    return "\n".join(part for part in [result.stdout, result.stderr] if part)


def contains_all(text: str, *needles: str) -> bool:
    lowered = text.lower()
    return all(needle.lower() in lowered for needle in needles)


def capability(
    status: str,
    *,
    basis: list[str] | None = None,
    candidate_entrypoints: list[str] | None = None,
    blocked_reason: str | None = None,
    warnings: list[str] | None = None,
) -> dict[str, Any]:
    if status not in STATUS_VALUES:
        raise ValueError(f"invalid capability status: {status}")
    item: dict[str, Any] = {
        "status": status,
        "basis": basis or [],
        "candidate_entrypoints": candidate_entrypoints or [],
        "blocked_reason": blocked_reason,
        "warnings": warnings or [],
    }
    return item


def blocked_or_unknown(candidate: bool, *, basis: list[str], entrypoints: list[str], authorized: bool) -> dict[str, Any]:
    if not candidate:
        return capability(
            "unknown",
            basis=basis or ["no matching no-side-effect help signal found"],
            candidate_entrypoints=[],
        )
    if authorized:
        return capability(
            "unknown",
            basis=basis + ["real execution flag was set, but this v1 script does not perform write probes"],
            candidate_entrypoints=entrypoints,
            blocked_reason="real_write_probe_not_implemented_in_v1",
        )
    return capability(
        "blocked",
        basis=basis,
        candidate_entrypoints=entrypoints,
        blocked_reason="real_session_probe_not_authorized",
    )


def build_capabilities(
    *,
    cli_available: bool,
    help_checked: bool,
    outputs: dict[str, CommandResult],
    transcript_reader_exists: bool,
    real_execution_authorized: bool,
) -> dict[str, Any]:
    if not cli_available:
        return {
            "discover_cli": capability("unsupported", basis=["codex executable not found on PATH"]),
            "inspect_help": capability("blocked", blocked_reason="codex_cli_missing"),
            "create_session": capability("unknown", blocked_reason="codex_cli_missing"),
            "resume_session": capability("unknown", blocked_reason="codex_cli_missing"),
            "send_prompt": capability("unknown", blocked_reason="codex_cli_missing"),
            "wait_for_result": capability("unknown", blocked_reason="codex_cli_missing"),
            "read_back_with_transcript": capability("unknown", blocked_reason="codex_cli_missing"),
        }

    top_help = text_for(outputs, "help")
    exec_help = text_for(outputs, "exec_help")
    exec_resume_help = text_for(outputs, "exec_resume_help")
    resume_help = text_for(outputs, "resume_help")
    all_help = "\n".join([top_help, exec_help, exec_resume_help, resume_help])

    create_candidate = contains_all(exec_help, "Usage: codex exec", "[PROMPT]") or contains_all(
        top_help, "Usage: codex [OPTIONS] [PROMPT]"
    )
    create_basis: list[str] = []
    create_entrypoints: list[str] = []
    if create_candidate:
        create_basis.append("help shows codex exec accepts an initial prompt")
        create_entrypoints.append("codex exec [PROMPT]")
    if "--ephemeral" in exec_help:
        create_basis.append("codex exec help lists --ephemeral")

    resume_candidate = contains_all(resume_help, "Usage: codex resume", "[SESSION_ID]") or contains_all(
        exec_resume_help, "Usage: codex exec resume", "[SESSION_ID]"
    )
    resume_basis: list[str] = []
    resume_entrypoints: list[str] = []
    if contains_all(resume_help, "Usage: codex resume", "[SESSION_ID]"):
        resume_basis.append("help shows codex resume accepts SESSION_ID")
        resume_entrypoints.append("codex resume [SESSION_ID] [PROMPT]")
    if contains_all(exec_resume_help, "Usage: codex exec resume", "[SESSION_ID]"):
        resume_basis.append("help shows codex exec resume accepts SESSION_ID")
        resume_entrypoints.append("codex exec resume [SESSION_ID] [PROMPT]")

    send_candidate = "[PROMPT]" in all_help or "Prompt to send after resuming" in exec_resume_help
    send_basis: list[str] = []
    send_entrypoints: list[str] = []
    if "[PROMPT]" in exec_help:
        send_basis.append("codex exec help includes [PROMPT]")
        send_entrypoints.append("codex exec [PROMPT]")
    if "[PROMPT]" in resume_help:
        send_basis.append("codex resume help includes [PROMPT]")
        send_entrypoints.append("codex resume [SESSION_ID] [PROMPT]")
    if "[PROMPT]" in exec_resume_help:
        send_basis.append("codex exec resume help includes [PROMPT]")
        send_entrypoints.append("codex exec resume [SESSION_ID] [PROMPT]")

    wait_candidate = "--output-last-message" in all_help or "--json" in all_help or "non-interactively" in all_help
    wait_basis: list[str] = []
    wait_entrypoints: list[str] = []
    if "--json" in exec_help or "--json" in exec_resume_help:
        wait_basis.append("exec help lists --json machine-readable event output")
    if "--output-last-message" in exec_help or "--output-last-message" in exec_resume_help:
        wait_basis.append("exec help lists --output-last-message")
    if "non-interactively" in exec_help.lower():
        wait_basis.append("exec help describes non-interactive execution")
    if wait_candidate:
        wait_entrypoints.extend(["codex exec --json", "codex exec resume --json"])

    readback_candidate = transcript_reader_exists and (create_candidate or resume_candidate)
    readback_basis = ["transcript_reader.py exists"] if transcript_reader_exists else ["transcript_reader.py missing"]
    if create_candidate or resume_candidate:
        readback_basis.append("CLI help has candidate session creation or resume entrypoint")

    return {
        "discover_cli": capability("supported", basis=["codex executable found on PATH"]),
        "inspect_help": capability(
            "supported" if help_checked else "unknown",
            basis=["codex --help completed"] if help_checked else ["codex --help did not complete"],
        ),
        "create_session": blocked_or_unknown(
            create_candidate,
            basis=create_basis,
            entrypoints=create_entrypoints,
            authorized=real_execution_authorized,
        ),
        "resume_session": blocked_or_unknown(
            resume_candidate,
            basis=resume_basis,
            entrypoints=resume_entrypoints,
            authorized=real_execution_authorized,
        ),
        "send_prompt": blocked_or_unknown(
            send_candidate,
            basis=send_basis,
            entrypoints=send_entrypoints,
            authorized=real_execution_authorized,
        ),
        "wait_for_result": blocked_or_unknown(
            wait_candidate,
            basis=wait_basis,
            entrypoints=wait_entrypoints,
            authorized=real_execution_authorized,
        ),
        "read_back_with_transcript": blocked_or_unknown(
            readback_candidate,
            basis=readback_basis,
            entrypoints=["transcript_reader.py after a verified persisted session write"]
            if readback_candidate
            else [],
            authorized=real_execution_authorized,
        ),
    }


def candidate_entrypoints(capabilities: dict[str, Any]) -> list[dict[str, Any]]:
    entries: dict[str, dict[str, Any]] = {}
    for name, item in capabilities.items():
        if not isinstance(item, dict):
            continue
        for entrypoint in item.get("candidate_entrypoints", []):
            entry = entries.setdefault(
                entrypoint,
                {
                    "entrypoint": entrypoint,
                    "candidate_for": [],
                    "verified_supported": False,
                },
            )
            entry["candidate_for"].append(name)
    return sorted(entries.values(), key=lambda item: item["entrypoint"])


def collect_blocked_reasons(capabilities: dict[str, Any]) -> list[str]:
    reasons: list[str] = []
    for name, item in capabilities.items():
        if not isinstance(item, dict):
            continue
        if item.get("status") != "blocked":
            continue
        reason = item.get("blocked_reason") or "unknown_blocked_reason"
        value = f"{name}:{reason}"
        if value not in reasons:
            reasons.append(value)
    return reasons


def validate_probe_result(result: dict[str, Any]) -> list[str]:
    problems: list[str] = []
    for key in ["generated_at", "probe_mode", "codex_cli", "capabilities", "evidence", "warnings", "blocked_reasons"]:
        if key not in result:
            problems.append(f"missing_top_level_key:{key}")
    capabilities = result.get("capabilities")
    if not isinstance(capabilities, dict):
        problems.append("capabilities_not_object")
        return problems
    for name in CAPABILITY_NAMES:
        item = capabilities.get(name)
        if not isinstance(item, dict):
            problems.append(f"missing_capability:{name}")
            continue
        status = item.get("status")
        if status not in STATUS_VALUES:
            problems.append(f"invalid_capability_status:{name}:{status}")
        if status == "supported" and not item.get("basis"):
            problems.append(f"supported_without_basis:{name}")
        if status == "blocked" and not item.get("blocked_reason"):
            problems.append(f"blocked_without_reason:{name}")
    return problems


def build_probe_result(
    *,
    codex_path: str | None = None,
    runner: CommandRunner = run_command,
    real_execution_authorized: bool = False,
    timeout_seconds: int = DEFAULT_TIMEOUT_SECONDS,
    transcript_reader_path: Path | None = None,
) -> dict[str, Any]:
    path = codex_path if codex_path is not None else shutil.which("codex")
    if path == "":
        path = None
    cli_available = bool(path)
    warnings: list[str] = []
    outputs: dict[str, CommandResult] = {}
    evidence: list[dict[str, Any]] = []

    if not real_execution_authorized:
        warnings.append("real_session_probe_not_authorized")

    if cli_available:
        for label, args in COMMANDS_TO_PROBE:
            command = [str(path), *args]
            result = runner(command, timeout_seconds)
            outputs[label] = result
            summary = summarize_command(label, result)
            evidence.append(summary)
            for warning in summary["warnings"]:
                if warning not in warnings:
                    warnings.append(warning)
            if result.timed_out:
                warnings.append(f"command_timed_out:{label}")
            if result.exit_code not in (0, None):
                warnings.append(f"command_nonzero_exit:{label}:{result.exit_code}")

    version_text = first_non_warning_line(text_for(outputs, "version")) if cli_available else None
    help_checked = bool(outputs.get("help") and outputs["help"].exit_code == 0)
    top_commands = parse_commands_from_help(text_for(outputs, "help"))
    transcript_path = transcript_reader_path or (Path(__file__).resolve().parent / "transcript_reader.py")
    transcript_reader_exists = transcript_path.exists()

    capabilities = build_capabilities(
        cli_available=cli_available,
        help_checked=help_checked,
        outputs=outputs,
        transcript_reader_exists=transcript_reader_exists,
        real_execution_authorized=real_execution_authorized,
    )

    result = {
        "generated_at": utc_now_iso(),
        "probe_mode": "no_side_effect" if not real_execution_authorized else "authorized_no_write_probe_only",
        "real_execution": {
            "authorized": real_execution_authorized,
            "executed": False,
            "reason": "v1 records no-side-effect CLI evidence only",
        },
        "codex_cli": {
            "available": cli_available,
            "path": path,
            "version": version_text,
            "version_checked": "version" in outputs,
            "help_checked": help_checked,
            "commands_detected": top_commands,
        },
        "capabilities": capabilities,
        "candidate_entrypoints": candidate_entrypoints(capabilities),
        "evidence": evidence,
        "warnings": warnings,
        "blocked_reasons": collect_blocked_reasons(capabilities),
    }
    problems = validate_probe_result(result)
    if problems:
        result["warnings"].extend(f"schema_problem:{problem}" for problem in problems)
    return result


def write_json(path: Path, data: dict[str, Any], pretty: bool) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(data, handle, ensure_ascii=False, indent=2 if pretty else None, sort_keys=True)
        handle.write("\n")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Probe Codex session control capabilities without write actions.")
    parser.add_argument("--output", type=Path, help="Optional JSON output path.")
    parser.add_argument("--pretty", action="store_true")
    parser.add_argument(
        "--allow-real-execution",
        action="store_true",
        help="Record explicit authorization. This v1 script still does not perform real write probes.",
    )
    parser.add_argument("--timeout-seconds", type=int, default=DEFAULT_TIMEOUT_SECONDS)
    args = parser.parse_args(argv)

    result = build_probe_result(
        real_execution_authorized=args.allow_real_execution,
        timeout_seconds=args.timeout_seconds,
    )
    problems = validate_probe_result(result)
    if problems:
        for problem in problems:
            print(f"probe_validation_failed:{problem}", file=sys.stderr)
        return 1

    if args.output:
        write_json(args.output, result, args.pretty)

    print(
        json.dumps(
            {
                "output": str(args.output) if args.output else None,
                "codex_cli_available": result["codex_cli"]["available"],
                "codex_cli_version": result["codex_cli"]["version"],
                "supported_capabilities": sorted(
                    name
                    for name, item in result["capabilities"].items()
                    if isinstance(item, dict) and item.get("status") == "supported"
                ),
                "blocked_capabilities": sorted(
                    name
                    for name, item in result["capabilities"].items()
                    if isinstance(item, dict) and item.get("status") == "blocked"
                ),
                "warning_count": len(result["warnings"]),
            },
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
