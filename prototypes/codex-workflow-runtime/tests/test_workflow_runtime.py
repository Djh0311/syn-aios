from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "workflow_runtime.py"


def load_runtime_module():
    spec = importlib.util.spec_from_file_location("codex_workflow_runtime", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


runtime = load_runtime_module()


class WorkflowRuntimeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory(prefix="codex-workflow-runtime-")
        self.addCleanup(self.tmp.cleanup)
        self.output_dir = Path(self.tmp.name)

    def test_dry_run_generates_accepted_state_flow_and_artifacts(self) -> None:
        run = runtime.run_dry_run(runtime.DEFAULT_GOAL, self.output_dir)

        self.assertEqual(run["mode"], "dry-run")
        self.assertEqual(run["status"], "accepted")
        self.assertEqual(
            [item["status"] for item in run["state_flow"]],
            ["planned", "dispatched", "running", "reported", "recovered", "accepted"],
        )
        self.assertEqual({node["node_id"] for node in run["nodes"]}, {
            "director_plan",
            "worker_run",
            "worker_readback",
            "director_review",
            "runtime_summary",
        })
        self.assertTrue((self.output_dir / "run.json").is_file())
        self.assertTrue((self.output_dir / "director-task.json").is_file())
        self.assertTrue((self.output_dir / "worker-transcript.json").is_file())
        self.assertIn("dry_run_only", run["warnings"])

    def test_run_json_schema_contains_required_runtime_sections(self) -> None:
        run = runtime.run_dry_run(runtime.DEFAULT_GOAL, self.output_dir)
        loaded = json.loads((self.output_dir / "run.json").read_text(encoding="utf-8"))

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
            self.assertIn(key, loaded)
        self.assertEqual(runtime.validate_run(run), [])

    def test_dry_run_does_not_create_real_session_reference(self) -> None:
        run = runtime.run_dry_run(runtime.DEFAULT_GOAL, self.output_dir)
        session = run["sessions"]["worker-session"]

        self.assertIsNone(session["thread_id"])
        self.assertFalse(session["created_new"])
        self.assertFalse(session["existing_business_session_touched"])
        self.assertIn("dry_run_no_real_session", session["warnings"])

    def test_real_probe_requires_exact_approval_text(self) -> None:
        with self.assertRaises(runtime.RuntimeErrorWithCode) as ctx:
            runtime.run_real_codex_probe(runtime.DEFAULT_GOAL, self.output_dir, "批准")

        self.assertEqual(ctx.exception.code, "real_probe_not_approved")
        self.assertFalse((self.output_dir / "worker-events.jsonl").exists())

    def test_director_review_accepts_required_marker(self) -> None:
        review = runtime.director_review("WORKER_DONE_2026_05_29", {"total_events": 3})

        self.assertEqual(review["decision"], "accepted")
        self.assertIn("worker last message contains required marker", review["basis"])

    def test_worker_prompt_is_no_business_and_contains_required_marker(self) -> None:
        task = runtime.director_task(runtime.DEFAULT_GOAL)
        prompt = runtime.worker_prompt(task)

        self.assertIn("无业务控制探针", prompt)
        self.assertIn("不要修改文件", prompt)
        self.assertIn("WORKER_DONE_2026_05_29", prompt)


if __name__ == "__main__":
    unittest.main()
