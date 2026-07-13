# Station 3b attempt-4 run ledger

Captured: 2026-07-13 13:00 Asia/Shanghai

- supervisor run: `supervisor:workflow-users-yoyi-documents-mario-test-default:1783918485705864000`
- authorization: `plan-auth:project-users-yoyi-documents-mario-test-workflow-users-yoyi-documents-mario-test-default-node-node:1783918484464`
- work item: `work-item:workflow:users-yoyi-documents-mario-test:default:project-director:planned-task-supervisor-pilot-eb33d80132fa15315006376e`
- native worker thread: `019f59d4-1f7a-7a52-88f6-e46308dd9f09`
- dispatch / worker: `dispatch:workflow-users-yoyi-documents-mario-test-default:work-item-workflow-users-yoyi-documents-mario-test-default-project-director-planned-task-supervi:1783918513688`
- project root: `/Users/yoyi/Documents/mario test`
- allowed write roots: `[]`
- worker sandbox argv: `--sandbox read-only`
- supervisor sandbox argv: `--sandbox read-only`
- worker process group: wrapper PID/PGID `94133`, native child PID `94137`; both ended naturally and the durable registry entry was removed.
- worker count: `1`
- follow-up count: `0`
- session launch status: `exited`
- final verdict: `pass`
- final reason: worker really read all four files; README verdicts, top-five issues, exact source lines, node syntax check and no-write evidence were present.

Accepted controller actions, in order:

1. `dispatch_worker` at workflow revision 4; exactly one worker reserved and launched.
2. `inspect_worker` at revision 10; structured report parsed as `reported_completed`, `evidence_present=true`.
3. `finalize`; `verdict=pass`, `advisory_only=true`, `workflow_chain_state_written=false`.
4. `report_user`; user-visible completion report recorded, `user_decision_written=false`.

Build provenance recorded by the supervisor session:

- executable: `target/debug/bundle/macos/CodexGovernanceWorkbench.app/Contents/MacOS/codex-governance-workbench`
- bytes: `60210760`
- mtime: `1783918171`
- SHA-256: `08163d25c5e696f6dfca6d2ff9d5ca1db47d5622d21b3c2cecbf3853869e4fd3`
- supervisor contract: `supervisor_action_proposal.v1`
- supervisor contract SHA-256: `0803c153bf4c364ad11e9d9023387ba78079694687040999257401d880583e30`
- worker report contract SHA-256: `8f6bd0b60b53a9d80acb7988baf1d8a6da810678d1445bb5459e9e57b7560297`

Authoritative stores:

- `~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/supervisor-orchestrator.v1.json`
- `~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/supervisor-action-control.v1.json`
- `~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

Independent no-write comparison is in `attempt-4-post-run-baseline.txt`; full parsed worker report is in `attempt-4-worker-report.json`.
