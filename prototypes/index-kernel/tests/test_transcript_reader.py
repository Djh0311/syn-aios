from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
import stat
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "transcript_reader.py"


def load_transcript_reader_module():
    spec = importlib.util.spec_from_file_location("index_kernel_transcript_reader", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


transcript_reader = load_transcript_reader_module()


class TranscriptReaderFixtureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory(prefix="transcript-reader-")
        self.addCleanup(self.tmp.cleanup)
        self.fixture_root = Path(self.tmp.name)
        self.codex_home = self.fixture_root / "fake-codex-home"
        self.sessions_dir = self.codex_home / "sessions"
        self.archived_sessions_dir = self.codex_home / "archived_sessions"
        self.sessions_dir.mkdir(parents=True)
        self.archived_sessions_dir.mkdir(parents=True)
        self.rollout_path = self.sessions_dir / "thread-ok.jsonl"
        self.thread_id = "thread-ok"
        self.index_path = self.fixture_root / "codex-index.json"

    def write_jsonl(self, path: Path, rows: list[object], *, append_bad_line: bool = False) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        lines = [json.dumps(row, ensure_ascii=False) for row in rows]
        if append_bad_line:
            lines.append("{bad json")
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def write_index(self, *, rollout_path: Path | None = None, thread_id: str | None = None) -> dict[str, object]:
        index = {
            "generated_at": "2026-05-29T00:00:00Z",
            "threads": [
                {
                    "thread_id": thread_id or self.thread_id,
                    "title": "Fixture thread",
                    "project_root": str(self.fixture_root / "project"),
                    "rollout_path": str(rollout_path or self.rollout_path),
                    "rollout_exists": True,
                    "created_at_ms": 1,
                    "updated_at_ms": 2,
                    "warnings": [],
                }
            ],
            "source_stats": {
                "codex_home": {
                    "path": str(self.codex_home),
                    "role": "data_source_root",
                }
            },
            "warnings": [],
        }
        self.index_path.write_text(json.dumps(index), encoding="utf-8")
        return index

    def run_main(self, args: list[str]) -> tuple[int, str, str]:
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            exit_code = transcript_reader.main(args)
        return exit_code, stdout.getvalue(), stderr.getvalue()

    def build_basic_rows(self) -> list[object]:
        return [
            {
                "timestamp": "2026-05-29T00:00:00Z",
                "type": "session_meta",
                "payload": {"id": self.thread_id, "cwd": "/tmp/project", "model_provider": "ai"},
            },
            {
                "timestamp": "2026-05-29T00:00:01Z",
                "type": "turn_context",
                "payload": {"turn_id": "turn-1", "cwd": "/tmp/project", "model": "fixture-model"},
            },
            {
                "timestamp": "2026-05-29T00:00:02Z",
                "type": "event_msg",
                "payload": {"type": "user_message", "message": "User asks for a fixture."},
            },
            {
                "timestamp": "2026-05-29T00:00:03Z",
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "output_text", "text": "Assistant answers fixture."}],
                },
            },
            {
                "timestamp": "2026-05-29T00:00:04Z",
                "type": "response_item",
                "payload": {
                    "type": "function_call",
                    "call_id": "call-1",
                    "name": "functions.exec_command",
                    "arguments": "{\"cmd\":\"pwd\"}",
                },
            },
            {
                "timestamp": "2026-05-29T00:00:05Z",
                "type": "response_item",
                "payload": {
                    "type": "function_call_output",
                    "call_id": "call-1",
                    "output": "{\"stdout\":\"/tmp/project\\n\",\"stderr\":\"\",\"exit_code\":0}",
                },
            },
            {
                "timestamp": "2026-05-29T00:00:06Z",
                "type": "compacted",
                "payload": {"summary": "Compacted fixture summary."},
            },
        ]

    def test_basic_transcript_parses_messages_tool_call_tool_result_and_context(self) -> None:
        self.write_jsonl(self.rollout_path, self.build_basic_rows())
        index = self.write_index()

        transcript = transcript_reader.build_transcript(index, self.thread_id)
        event_types = [event["event_type"] for event in transcript["events"]]

        self.assertEqual(transcript["thread_id"], self.thread_id)
        self.assertEqual(transcript["rollout_path"], str(self.rollout_path))
        self.assertIn("session_meta", event_types)
        self.assertIn("turn_context", event_types)
        self.assertIn("user_message", event_types)
        self.assertIn("assistant_message", event_types)
        self.assertIn("tool_call", event_types)
        self.assertIn("command_output", event_types)
        self.assertIn("compacted", event_types)
        tool_call = next(event for event in transcript["events"] if event["event_type"] == "tool_call")
        self.assertEqual(tool_call["tool_name"], "functions.exec_command")
        self.assertEqual(tool_call["arguments"], {"cmd": "pwd"})
        output = next(event for event in transcript["events"] if event["event_type"] == "command_output")
        self.assertEqual(output["stdout"], "/tmp/project\n")
        self.assertEqual(output["stderr"], "")
        self.assertEqual(output["exit_code"], 0)

    def test_command_output_from_event_msg_distinguishes_stdout_stderr_exit_code(self) -> None:
        self.write_jsonl(
            self.rollout_path,
            [
                {
                    "timestamp": "2026-05-29T00:00:00Z",
                    "type": "event_msg",
                    "payload": {
                        "type": "patch_apply_end",
                        "call_id": "call-patch",
                        "turn_id": "turn-1",
                        "stdout": "patched",
                        "stderr": "warning",
                        "exit_code": 1,
                        "status": "failed",
                    },
                }
            ],
        )
        index = self.write_index()

        transcript = transcript_reader.build_transcript(index, self.thread_id)
        event = transcript["events"][0]

        self.assertEqual(event["event_type"], "command_output")
        self.assertEqual(event["tool_name"], "apply_patch")
        self.assertEqual(event["stdout"], "patched")
        self.assertEqual(event["stderr"], "warning")
        self.assertEqual(event["exit_code"], 1)

    def test_encrypted_content_is_marked_and_not_output(self) -> None:
        secret = "ENCRYPTED_PAYLOAD_SHOULD_NOT_APPEAR"
        self.write_jsonl(
            self.rollout_path,
            [
                {
                    "timestamp": "2026-05-29T00:00:00Z",
                    "type": "response_item",
                    "payload": {
                        "type": "reasoning",
                        "summary": [],
                        "encrypted_content": secret,
                    },
                }
            ],
        )
        index = self.write_index()

        transcript = transcript_reader.build_transcript(index, self.thread_id)
        serialized = json.dumps(transcript, ensure_ascii=False)

        self.assertNotIn(secret, serialized)
        self.assertIn("encrypted_content_omitted", transcript["events"][0]["warnings"])
        self.assertEqual(transcript["summary"]["encrypted_content_event_count"], 1)

    def test_bad_jsonl_line_records_warning_and_keeps_other_events(self) -> None:
        self.write_jsonl(self.rollout_path, self.build_basic_rows()[:2], append_bad_line=True)
        index = self.write_index()

        transcript = transcript_reader.build_transcript(index, self.thread_id)

        self.assertEqual(len(transcript["events"]), 2)
        self.assertIn("invalid_json_line:3", transcript["warnings"])
        self.assertEqual(transcript["source_stats"]["jsonl"]["bad_json_line_count"], 1)

    def test_unknown_event_preserves_diagnostic_metadata(self) -> None:
        self.write_jsonl(
            self.rollout_path,
            [
                {
                    "timestamp": "2026-05-29T00:00:00Z",
                    "type": "new_future_event",
                    "payload": {"type": "future_payload", "shape": {"value": 1}},
                }
            ],
        )
        index = self.write_index()

        transcript = transcript_reader.build_transcript(index, self.thread_id)
        event = transcript["events"][0]

        self.assertEqual(event["event_type"], "unknown")
        self.assertIn("unknown_event_type", event["warnings"])
        self.assertEqual(event["metadata"]["raw_type"], "new_future_event")
        self.assertEqual(event["metadata"]["payload_type"], "future_payload")
        self.assertIn("raw_event", event["metadata"])
        self.assertIn("unknown_event_count:1", transcript["warnings"])

    def test_sensitive_like_content_gets_warning(self) -> None:
        self.write_jsonl(
            self.rollout_path,
            [
                {
                    "timestamp": "2026-05-29T00:00:00Z",
                    "type": "event_msg",
                    "payload": {
                        "type": "user_message",
                        "message": "Authorization: Bearer abcdefghijklmnopqrstuvwxyz123456",
                    },
                }
            ],
        )
        index = self.write_index()

        transcript = transcript_reader.build_transcript(index, self.thread_id)

        self.assertIn("sensitive_like_content", transcript["events"][0]["warnings"])
        self.assertIn("sensitive_like_event_count:1", transcript["warnings"])

    def test_non_index_thread_is_rejected(self) -> None:
        self.write_jsonl(self.rollout_path, [])
        index = self.write_index()

        with self.assertRaisesRegex(transcript_reader.TranscriptReadError, "thread_not_in_index:missing"):
            transcript_reader.build_transcript(index, "missing")

    def test_rollout_path_outside_allowed_dirs_is_rejected(self) -> None:
        outside = self.fixture_root / "outside.jsonl"
        outside.write_text("", encoding="utf-8")
        index = self.write_index(rollout_path=outside)

        with self.assertRaisesRegex(
            transcript_reader.TranscriptReadError,
            "rollout_path_outside_allowed_session_dirs",
        ):
            transcript_reader.build_transcript(index, self.thread_id)

    def test_missing_rollout_file_gives_clear_error(self) -> None:
        missing = self.sessions_dir / "missing.jsonl"
        index = self.write_index(rollout_path=missing)

        with self.assertRaisesRegex(transcript_reader.TranscriptReadError, "missing_rollout_file:"):
            transcript_reader.build_transcript(index, self.thread_id)

    def test_unreadable_rollout_records_clear_error_or_skips_when_permissions_are_not_enforced(self) -> None:
        self.write_jsonl(self.rollout_path, self.build_basic_rows()[:1])
        index = self.write_index()
        self.rollout_path.chmod(0)
        try:
            with self.assertRaises(transcript_reader.TranscriptReadError) as ctx:
                transcript_reader.build_transcript(index, self.thread_id)
        except AssertionError:
            mode = stat.S_IMODE(os.stat(self.rollout_path).st_mode)
            if mode == 0:
                self.skipTest("chmod 000 did not produce PermissionError on this filesystem/user context")
            raise
        finally:
            self.rollout_path.chmod(0o600)

        self.assertIn("rollout_read_failed:PermissionError", str(ctx.exception))

    def test_cli_writes_transcript_and_stdout_only_reports_counts(self) -> None:
        sentinel = "CLI_TRANSCRIPT_BODY_SHOULD_ONLY_APPEAR_IN_OUTPUT_FILE"
        self.write_jsonl(
            self.rollout_path,
            [
                {
                    "timestamp": "2026-05-29T00:00:00Z",
                    "type": "event_msg",
                    "payload": {"type": "user_message", "message": sentinel},
                }
            ],
        )
        self.write_index()
        output_path = self.fixture_root / "transcript.json"

        exit_code, stdout, stderr = self.run_main(
            [
                "--index",
                str(self.index_path),
                "--thread-id",
                self.thread_id,
                "--output",
                str(output_path),
            ]
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(stderr, "")
        self.assertNotIn(sentinel, stdout)
        self.assertIn(sentinel, output_path.read_text(encoding="utf-8"))

    def test_default_index_fixture_does_not_contain_full_transcript_fields(self) -> None:
        index_path = MODULE_PATH.parent / "codex-index.json"
        if not index_path.exists():
            self.skipTest("repository index fixture is not present")
        data = json.loads(index_path.read_text(encoding="utf-8"))

        serialized_thread_keys = set()
        for thread in data.get("threads", []):
            if isinstance(thread, dict):
                serialized_thread_keys.update(thread.keys())

        self.assertNotIn("events", serialized_thread_keys)
        self.assertNotIn("transcript", serialized_thread_keys)
        self.assertNotIn("output", serialized_thread_keys)
        self.assertNotIn("stdout", serialized_thread_keys)
        self.assertNotIn("stderr", serialized_thread_keys)


if __name__ == "__main__":
    unittest.main()
