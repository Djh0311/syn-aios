# Conversation Module Native P0 Contract Inventory v1

Date: 2026-06-18
Plan: `docs/plans/2026-06-18-conversation-module-native-execution-plan-v1.md`
Status: P0 evidence; live event-stream probe executed from an empty temp directory. Private rollout-body probe is not yet approved/executed.

## Scope

Read scope used in this pass:

- `AGENTS.md`, `CURRENT.md`, `docs/sprint-contract.md`
- `docs/plans/2026-06-18-conversation-module-native-execution-plan-v1.md`
- Local skill docs `skills/using-superpowers/SKILL.md` and `skills/executing-plans/SKILL.md`
- Relevant source files under `prototypes/productized-desktop-shell/src-tauri/src/**` and `prototypes/productized-desktop-shell/src/**`
- Non-sensitive `codex` CLI help/version output

Write scope used in this pass:

- `docs/sprint-contract.md`
- `docs/evidence/2026-06-18-conversation-module-native-p0-contract-inventory-v1.md`

No product runtime code was changed in this P0 pass.

## CLI Surface Facts

Commands run:

```text
command -v codex
codex --version
codex exec --help
```

Observed:

- `codex` path: `/opt/homebrew/Cellar/node/23.11.0/bin/codex`
- version: `codex-cli 0.134.0`
- `codex exec --help` includes `resume`, `--json`, `--output-last-message`, `--ephemeral`, `--sandbox`, `--add-dir`, `--skip-git-repo-check`, `--dangerously-bypass-approvals-and-sandbox`.
- `codex exec --help` in this installed version did **not** list the plan's `--ask-for-approval` flag. Treat approval-flag assumptions as unverified for this installed CLI.
- The help command printed `WARNING: proceeding, even though we could not update PATH: Operation not permitted (os error 1)`; the command still returned usable help/version output.

Initial attempt from the product repo was blocked by the approval reviewer because it would contact Codex using local auth from a private repository context.

After user approval for option 1, the probe was rerun from an empty temp directory to minimize outbound context:

```text
codex exec --json --ephemeral --ignore-rules --skip-git-repo-check --sandbox read-only -C /private/tmp/codex-json-probe-empty "P0 JSON event probe. Reply with exactly: P0_JSON_EVENT_PROBE_OK. Do not run tools."
```

Result: exit code 0 in about 9.4s.

JSONL events observed:

```json
{"type":"thread.started","thread_id":"019edaea-6620-7832-902b-59ac34e64a8b"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"P0_JSON_EVENT_PROBE_OK"}}
{"type":"turn.completed","usage":{"input_tokens":14297,"cached_input_tokens":2432,"output_tokens":39,"reasoning_output_tokens":26}}
```

Contract facts confirmed for `codex-cli 0.134.0`:

- `thread.started` has top-level `thread_id`.
- `turn.started` may have no payload fields.
- Final assistant reply in this simple case is `item.completed.item.type="agent_message"` with `item.text`.
- Terminal success is `turn.completed`, with `usage.input_tokens`, `usage.cached_input_tokens`, `usage.output_tokens`, and `usage.reasoning_output_tokens`.

Operational warnings observed around the JSONL events:

- remote plugin catalog sync can warn/fail when ChatGPT auth for remote plugin catalog is unavailable.
- state DB discrepancy warnings can be emitted during thread lookup/list fallback.
- MCP/plugin startup/shutdown warnings can be emitted even for a no-tools prompt.
- A plugin manifest warning referenced a local plugin JSON path under `~/.codex/.tmp/plugins/...`.
- The probe used 14,297 input tokens despite the empty cwd and `--ignore-rules`; P1 should not assume a minimal prompt has minimal account-side context/cost.

P1 implication: product code must consume stdout JSONL and keep stderr/log warnings separate. It should ignore non-JSON stderr lines for event parsing, but record a bounded warning summary for diagnostics.

## Rollout / Transcript Parser Facts

Source: `prototypes/productized-desktop-shell/src-tauri/src/codex_transcript.rs`.

Observed implementation:

- `read_transcript_from_rollout()` and `read_transcript_page_from_rollout()` validate a rollout path, read JSONL, parse events, and build `CodexTranscript`.
- The parser already models the double-tag shape for local fixtures: outer `type:"event_msg"` and inner `payload.type:"user_message"` / `agent_message`.
- It maps:
  - `event_msg.user_message` -> `event_type="user_message"`
  - `event_msg.agent_message` -> `event_type="assistant_message"`
  - `event_msg.patch_apply_end` -> `event_type="command_output"` with `tool_name="apply_patch"`
  - `response_item.message` -> user/assistant message depending on role
  - `response_item.function_call` / `custom_tool_call` -> `tool_call`
  - `response_item.function_call_output` / `custom_tool_call_output` -> `command_output` or `tool_result`
  - `response_item.reasoning` -> `system_context`
  - `session_meta`, `turn_context`, `compacted` -> system/context events
- It strips `encrypted_content` and marks sensitive-like content with warnings.
- It has tail/older pagination via `read_jsonl_event_page()`.

Gaps against the native execution plan:

- Real `~/.codex/sessions/**.jsonl` body schema was not read in this pass. This needs exact approval or a freshly generated approved fixture.
- `event_msg.agent_reasoning`, `event_msg.turn_started`, `event_msg.turn_complete`, `response_item.local_shell_call`, `web_search_call`, and `parsed_cmd` rendering metadata are not confirmed as parsed into dedicated view-model fields.
- `patch_apply_end` is recognized, but Codex-style diff/change rendering is not yet derived.
- Current event names use `assistant_message`; the plan text sometimes says `agent_message`. P1/P3 should normalize view-model names deliberately rather than rely on raw naming.

## Existing Relay Facts

Sources:

- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

Observed implementation:

- Tauri commands currently include `preview_manual_codex_relay`, `confirm_manual_codex_relay_once`, `run_manual_codex_relay_once`, `run_manual_codex_relay_gui_direct`, and `stop_manual_codex_relay_attempt`.
- GUI direct send goes through `run_manual_relay_gui_direct_once()` and uses process mode `RealCodexProductGui`.
- The command plan builder in `codex_local_runner.rs` already adds `--json` and `--output-last-message`.
- Existing-session resume argv shape is currently:
  - `codex exec -C <cwd> --sandbox <sandbox> --add-dir <root> resume --skip-git-repo-check --json --output-last-message <path> <session_id>`
- New-session argv shape is currently:
  - `codex exec -C <cwd> --sandbox <sandbox> --add-dir <root> --skip-git-repo-check --json --output-last-message <path>`
- Prompt body is written to stdin, not placed in argv.
- The current spawn path sets stdout to `Stdio::null()` and stderr to `Stdio::null()` in `manual_relay.rs`; the older Phase B runner captures stderr but still does not consume stdout JSONL.
- Completion/readback today relies on a workbench-managed last-message file summary or returns a running receipt without completion monitoring.
- Stop already tracks running child processes in an in-memory registry and kills/waits only the requested attempt.
- Existing guardrails check project/cwd/write-root canonicalization, sandbox=`workspace-write`, `--add-dir <project_root>`, no shell invocation, no prompt in argv, and no `--full-auto` / dangerous bypass args.

P1 implication:

- P1 is not just adding `--json`; that flag already exists in the plan. The actual change is to pipe stdout, parse JSONL ThreadEvents, derive assistant reply and terminal status, and surface completion/unlock to the frontend while preserving the existing stop handle and guardrails.

## Existing Frontend Facts

Sources:

- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
- `prototypes/productized-desktop-shell/src/lib/conversationEngine.ts`
- `prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`

Observed implementation:

- `AgentConversationShell.tsx` binds relay target project root from the selected session's `project_root`.
- `handleSubmitConversationDraft()` calls `runManualCodexRelayGuiDirect()`, appends a pending user message, then clears the draft.
- There is no frontend path yet for receiving a completed assistant message from the relay event stream.
- `manualRelayBusy` and `manualRelayReceipt.status === "running"` drive input locking.
- `AgentChatComposer.tsx` shows a target strip and direct-send mode for bound Codex sessions.
- `TranscriptViews.tsx` still renders conversation messages through `ChatBubble`, with internal events behind a developer details area.

P1 implication:

- The frontend needs an explicit relay terminal result contract: running -> completed/failed/stopped, assistant reply text, and unlock timing.
- If the backend command remains a single Tauri invoke that returns immediately with `status="running"`, the frontend needs either polling/event subscription or a follow-up command to collect completion.
- If the backend command waits for completion, stop must still work through a registered attempt while the invoke is pending.

P3 implication:

- Native Codex rendering is a later P3 task. The current bubble renderer should not be redesigned during P1 except where necessary to show relay results.

## P0 Interface Proposal For P1

Backend minimum contract:

- Request: selected `thread_id`, canonical project root/cwd from selected session metadata, prompt body, sandbox=`workspace-write`, allowed write roots `[project_root]`.
- Spawn: `codex exec --json ... resume <thread_id>` for existing sessions, stdin prompt, stdout piped, stderr captured or summarized, no shell.
- Event handling:
  - collect stdout JSONL events into a bounded in-memory summary for the attempt;
  - extract assistant final/reply text from completed assistant/agent message events;
  - mark `turn.completed` as terminal success;
  - mark `turn.failed`, process failure, malformed JSONL, or timeout as terminal failure;
  - keep child handle registered for stop until terminal.
- Response to frontend:
- `relay_attempt_id`
- `status`: `running` / `completed_real_codex` / `failed_process` / `failed_event_stream` / `stopped_by_user`
- `assistant_message_text: string | null`
- `thread_event_summary` with at least `thread_id`, `assistant_item_id`, and `usage`
- `thread_id`
- `prompt_sha256`
- `real_codex_executed`
  - `readback_status`
  - `warnings`

Frontend minimum state machine:

- idle/draft -> sending -> running -> completed/failed/stopped.
- On send: optimistically append user message and lock input.
- On terminal completed: append assistant message or reload transcript; unlock input.
- On failure: show inline error; unlock input.
- On stop: call stop command, show stopped status; unlock input.
- On a second send after unlock: repeat the same loop.

## Verification Already Run

```text
node scripts/harness/guard-state-files.js --target .
```

Result: PASS. Output was extremely large because the guard enumerated many protected/task/build files, so this evidence records only the pass/fail result rather than embedding the full listing.

## Blockers Before P1 Acceptance

1. Need exact approval before reading private real rollout bodies under `~/.codex/sessions/**.jsonl`, or generate a new approved throwaway session and inspect only that rollout.
2. Need decide whether P1 backend should block until terminal completion or return running + stream/poll terminal updates. The current frontend needs a terminal result somehow; without it, the "reply回显 + 解锁" defect remains.
