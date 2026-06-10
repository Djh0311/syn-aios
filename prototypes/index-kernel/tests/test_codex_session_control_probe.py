from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "codex_session_control_probe.py"


def load_probe_module():
    spec = importlib.util.spec_from_file_location("index_kernel_codex_session_control_probe", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


probe = load_probe_module()


def fake_result(args: list[str], stdout: str = "", stderr: str = "", exit_code: int = 0):
    return probe.CommandResult(args=args, exit_code=exit_code, stdout=stdout, stderr=stderr)


class CodexSessionControlProbeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory(prefix="codex-session-control-probe-")
        self.addCleanup(self.tmp.cleanup)
        self.fixture_root = Path(self.tmp.name)
        self.transcript_reader = self.fixture_root / "transcript_reader.py"
        self.transcript_reader.write_text("# fixture\n", encoding="utf-8")

    def build_with_outputs(
        self,
        outputs: dict[str, tuple[str, str, int]],
        *,
        codex_path: str | None = "/usr/local/bin/codex",
        authorized: bool = False,
    ) -> dict[str, object]:
        label_by_suffix = {tuple(args): label for label, args in probe.COMMANDS_TO_PROBE}

        def runner(args: list[str], timeout_seconds: int):
            label = label_by_suffix[tuple(args[1:])]
            stdout, stderr, exit_code = outputs.get(label, ("", "", 0))
            return fake_result(args, stdout=stdout, stderr=stderr, exit_code=exit_code)

        return probe.build_probe_result(
            codex_path=codex_path,
            runner=runner,
            real_execution_authorized=authorized,
            transcript_reader_path=self.transcript_reader,
        )

    def test_missing_codex_command_reports_unavailable(self) -> None:
        result = probe.build_probe_result(
            codex_path="",
            runner=lambda args, timeout_seconds: fake_result(args),
            transcript_reader_path=self.transcript_reader,
        )

        self.assertFalse(result["codex_cli"]["available"])
        self.assertEqual(result["capabilities"]["discover_cli"]["status"], "unsupported")
        self.assertEqual(result["capabilities"]["inspect_help"]["status"], "blocked")

    def test_help_without_session_control_keeps_capabilities_unknown(self) -> None:
        result = self.build_with_outputs(
            {
                "version": ("codex-cli 0.1.0\n", "", 0),
                "help": ("Codex CLI\n\nUsage: codex [OPTIONS]\n\nCommands:\n  help  Print help\n", "", 0),
                "exec_help": ("Usage: codex exec [OPTIONS]\n", "", 0),
                "exec_resume_help": ("Usage: codex exec resume [OPTIONS]\n", "", 0),
                "resume_help": ("Usage: codex resume [OPTIONS]\n", "", 0),
            }
        )

        self.assertTrue(result["codex_cli"]["available"])
        self.assertEqual(result["capabilities"]["discover_cli"]["status"], "supported")
        self.assertEqual(result["capabilities"]["inspect_help"]["status"], "supported")
        self.assertEqual(result["capabilities"]["resume_session"]["status"], "unknown")
        self.assertEqual(result["capabilities"]["send_prompt"]["status"], "unknown")

    def test_help_with_resume_exec_prompt_creates_blocked_candidates_without_authorization(self) -> None:
        result = self.build_with_outputs(
            {
                "version": ("codex-cli 0.134.0\n", "", 0),
                "help": (
                    "Codex CLI\n\nUsage: codex [OPTIONS] [PROMPT]\n\nCommands:\n"
                    "  exec    Run Codex non-interactively\n"
                    "  resume  Resume a previous interactive session\n",
                    "",
                    0,
                ),
                "exec_help": (
                    "Run Codex non-interactively\n\nUsage: codex exec [OPTIONS] [PROMPT]\n"
                    "      --ephemeral\n      --json\n  -o, --output-last-message <FILE>\n",
                    "",
                    0,
                ),
                "exec_resume_help": (
                    "Usage: codex exec resume [OPTIONS] [SESSION_ID] [PROMPT]\n"
                    "Prompt to send after resuming the session.\n      --json\n",
                    "",
                    0,
                ),
                "resume_help": (
                    "Usage: codex resume [OPTIONS] [SESSION_ID] [PROMPT]\n"
                    "Conversation/session id (UUID) or thread name.\n",
                    "",
                    0,
                ),
            }
        )

        self.assertEqual(result["capabilities"]["create_session"]["status"], "blocked")
        self.assertEqual(result["capabilities"]["resume_session"]["status"], "blocked")
        self.assertEqual(result["capabilities"]["send_prompt"]["status"], "blocked")
        self.assertEqual(result["capabilities"]["wait_for_result"]["status"], "blocked")
        self.assertEqual(result["capabilities"]["read_back_with_transcript"]["status"], "blocked")
        self.assertIn("real_session_probe_not_authorized", result["warnings"])
        self.assertIn("codex exec [PROMPT]", result["capabilities"]["create_session"]["candidate_entrypoints"])
        self.assertEqual(result["real_execution"]["executed"], False)

    def test_authorized_flag_still_does_not_mark_real_control_supported(self) -> None:
        result = self.build_with_outputs(
            {
                "version": ("codex-cli 0.134.0\n", "", 0),
                "help": ("Usage: codex [OPTIONS] [PROMPT]\nCommands:\n  exec  Run Codex non-interactively\n", "", 0),
                "exec_help": ("Usage: codex exec [OPTIONS] [PROMPT]\n      --json\n", "", 0),
                "exec_resume_help": ("Usage: codex exec resume [OPTIONS] [SESSION_ID] [PROMPT]\n", "", 0),
                "resume_help": ("Usage: codex resume [OPTIONS] [SESSION_ID] [PROMPT]\n", "", 0),
            },
            authorized=True,
        )

        self.assertEqual(result["capabilities"]["create_session"]["status"], "unknown")
        self.assertEqual(
            result["capabilities"]["create_session"]["blocked_reason"],
            "real_write_probe_not_implemented_in_v1",
        )
        self.assertEqual(result["real_execution"]["executed"], False)

    def test_json_schema_is_stable(self) -> None:
        result = self.build_with_outputs({"version": ("codex-cli 0.134.0\n", "", 0), "help": ("Usage: codex\n", "", 0)})

        problems = probe.validate_probe_result(result)

        self.assertEqual(problems, [])
        for key in ["generated_at", "probe_mode", "codex_cli", "capabilities", "evidence", "warnings"]:
            self.assertIn(key, result)
        for name in probe.CAPABILITY_NAMES:
            self.assertIn(name, result["capabilities"])
            self.assertIn(result["capabilities"][name]["status"], probe.STATUS_VALUES)

    def test_sensitive_like_help_output_is_redacted_from_evidence_lines(self) -> None:
        secret = "Authorization: Bearer abcdefghijklmnopqrstuvwxyz123456"
        result = self.build_with_outputs(
            {
                "version": ("codex-cli 0.134.0\n", "", 0),
                "help": (
                    "Usage: codex [OPTIONS] [PROMPT]\n"
                    f"  exec    Run Codex non-interactively with {secret}\n",
                    "",
                    0,
                ),
                "exec_help": ("Usage: codex exec [OPTIONS] [PROMPT]\n", "", 0),
            }
        )

        serialized = json.dumps(result, ensure_ascii=False)

        self.assertNotIn(secret, serialized)
        self.assertIn("[REDACTED_SENSITIVE]", serialized)
        self.assertIn("sensitive_like_help_output_redacted", result["warnings"])


if __name__ == "__main__":
    unittest.main()
