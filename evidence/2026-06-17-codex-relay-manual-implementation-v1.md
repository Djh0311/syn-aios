# Codex relay manual implementation evidence v1

Date: 2026-06-18 01:17 CST
Task: `tasks/2026-06-17-codex-relay-manual-implementation-v1.md`
Design authority: `docs/plans/2026-06-17-codex-relay-stepping-stone-design-draft-v1.md`
Start HEAD: `ea4f2d8bc10d43dffe5ea53a03dba54cbdeacfbd`

## Scope Implemented

- Added `src-tauri/src/manual_relay.rs` as a new parallel contract, not a H2/K3/H5/PCR shortcut.
- Added four narrow Tauri commands in `src-tauri/src/commands.rs` and `src-tauri/src/command_registry.rs`:
  - `preview_manual_codex_relay`
  - `confirm_manual_codex_relay_once`
  - `run_manual_codex_relay_once`
  - `stop_manual_codex_relay_attempt`
- Added frontend `manual_relay` mode in the conversation composer while preserving the existing `decision-only` submit path.
- Added TypeScript relay contract types and Tauri wrappers.
- Added offline UI/metadata tests for exact payload, target binding, one-shot warning, visible Stop control, and `real_codex_executed=false`.
- Added separate `src/manualRelay.css`; did not grow the ratcheted `styles.css`.

## Boundary

- This package does not run real Codex and does not execute `codex exec` / `codex exec resume`.
- This package does not write `/Users/yoyi/.codex` and does not read Codex auth, token, rollout body, prompt body, or full transcript.
- This package does not unlock K3-B1 retry, K3-B2, H2/H3/H5/PCR product-command execution, background workers, automatic chaining, multi-agent execution, or general real-execution authorization.
- The `run_manual_codex_relay_once` command is fixture/mock only in this package. Its receipts keep `prompt_sent=false`, `real_codex_executed=false`, `syn_read_codex_home=false`, and `syn_wrote_codex_home=false`.
- Stop is implemented for the manual relay attempt registry and only stops the requested manual relay attempt; it does not kill unrelated Codex processes or sessions.
- Dirty target tree rollback is conservative: no destructive auto rollback is performed.

## Guard Coverage

- Prompt hash mismatch blocks before run.
- Target hash mismatch blocks before run.
- Confirmation id mismatch blocks before run.
- Running duplicate attempt for the same duplicate scope blocks.
- Secret/token/.env/keychain/OAuth/credential/full transcript/rollout/prompt body/`.codex` material requests block.
- Command plan is structured: program + argv + stdin prompt; no shell invocation and prompt is not placed in argv.
- `payload_layers` is empty in v1 and `future_hooks` are present as null placeholders.
- Policy is `manual_once=true` and `auto_chain=false`.
- Confirmation ids are consumed once; replay after a terminal fixture receipt is blocked.
- Receipts include target, prompt hash/length/exactness, command plan, timestamps, exit code, last-message size, changed files, and git before/after fields.
- Dirty tree does not auto rollback.

## Verification Output

### cargo test --lib manual_relay

```text
running 5 tests
test manual_relay::tests::manual_relay_consumes_confirmation_once_and_receipt_records_contract_fields ... ok
test manual_relay::tests::manual_relay_preview_keeps_payload_exact_and_structured_command_safe ... ok
test manual_relay::tests::manual_relay_blocks_sensitive_or_codex_home_requests ... ok
test manual_relay::tests::manual_relay_stop_only_kills_current_attempt_and_dirty_tree_never_auto_rolls_back ... ok
test manual_relay::tests::manual_relay_blocks_hash_mismatch_and_duplicate ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 550 filtered out; finished in 0.01s
```

### Old gate focused regression

```text
cargo test --lib codex_local_guard
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 548 filtered out; finished in 0.00s

cargo test --lib h2_real_resume
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 552 filtered out; finished in 0.04s

cargo test --lib h3_b
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 551 filtered out; finished in 0.06s

cargo test --lib k3_b
test result: ok. 11 passed; 0 failed; 2 ignored; 0 measured; 541 filtered out; finished in 0.10s

cargo test --lib h5
test result: ok. 6 passed; 0 failed; 2 ignored; 0 measured; 546 filtered out; finished in 0.00s

cargo test --lib product_command_store_summary_keeps_legacy_and_runner_blocked
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 553 filtered out; finished in 0.00s
```

### npm run typecheck

```text
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

### npm run test:offline-interaction

```text
> codex-governance-workbench@0.1.0 test:offline-interaction
> node scripts/run-offline-interaction-test.mjs

offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

### npm run build

```text
vite v7.3.3 building client environment for production...
✓ 259 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                     0.59 kB │ gzip:   0.42 kB
dist/assets/index-4h8GP3AC.css    146.46 kB │ gzip:  24.98 kB
dist/assets/index-4eqtDFGC.js   1,007.47 kB │ gzip: 276.12 kB

(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
✓ built in 1.03s
```

### cargo test --lib

```text
running 555 tests
...
test result: ok. 533 passed; 0 failed; 22 ignored; 0 measured; 0 filtered out; finished in 7.76s
```

### cargo fmt -- --check

```text
<no output; exit 0>
```

### shape gate

```text
Workbench shape gate: /Users/yoyi/workspace/product-line
Mode: check
Status: pass
Errors: 0
Warnings: 1
Info: 9
Tauri commands: 104 total; 0 in lib.rs
Findings:
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":104,"baseline":97}
```

Warning classification: expected for this package because it adds four narrow manual relay Tauri commands outside `lib.rs`.

### git diff --check

```text
<no output; exit 0>
```

### Dangerous-string scan on changed files

```text
prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx:194:  assert(relayPreviewMarkup.includes("manual_once / auto_chain=false"), "manual relay 必须显示一次一发且不自动连环");
prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx:289:        redacted_preview: "codex exec resume <session> <stdin prompt>",
prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx:89:            手动一次一发；本包只走 mock / fixture contract，不真跑 Codex、不写 .codex、不自动连环。
prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs:781:            "read /Users/yoyi/.codex/auth.json",
```

Classification:

- `自动连环`: only appears in denial/warning assertions and UI boundary text.
- `codex exec resume`: only appears in an offline redacted preview fixture, not in a new process-spawn path.
- `/Users/yoyi/.codex`: only appears in a negative manual relay sensitive-material test.
- No changed file introduces `Command::new`, real runner invocation, real Codex execution, or positive K3-B2/universal execution claims.

## Review Status

- Independent read-only review line: Hegel (`019ed696-09cb-78d3-897a-1a60f259a2c5`) initial result was `STATUS: FINDINGS`.
- Hegel P1 findings fixed in code:
  - confirmation replay after terminal receipt now blocks via consumed confirmation registry;
  - receipt contract now includes target, prompt hash/length/exactness, command plan, timestamps, exit code, last-message size, changed files, and git before/after fields;
  - tests now cover prompt hash mismatch, target hash mismatch, confirmation id mismatch, duplicate running attempts, confirmation replay, and H3 regression.
- Hegel P2 handling:
  - UI now displays allowed write roots.
  - Path handling now attempts filesystem `canonicalize` when possible and falls back to lexical clean for mock/fixture paths; true relay should still re-audit this before real execution.
- Final review file: `evidence/2026-06-17-codex-relay-manual-implementation-review-hegel-v1.md`.
- Final review status: `CLEAR_WITH_P2`.
- Consulting line/user review still required before commit.
