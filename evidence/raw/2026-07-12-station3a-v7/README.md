# Station 3a v7 Raw Evidence Manifest

- Run: `supervisor:workflow-users-yoyi-codex-workflow-mario-test-default:1783852010526616000`
- Authorization: `plan-auth:project-users-yoyi-codex-workflow-mario-test-workflow-users-yoyi-codex-workflow-mario-test-default-proof-proof:1783852009221`
- Worker/dispatch: `dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:work-item-workflow-users-yoyi-codex-workflow-mario-test-default-project-director-planned-task-su:1783852212107`
- Native thread: `019f55e0-7886-7242-91ce-43928715595d`
- Target: `/Users/yoyi/codex-workflow-mario-test/station3a-control-core-proof-v7.txt`
- Exact content: `station3a control core proof v7 passed!`
- Target facts: 39 bytes, last byte `33`, no trailing newline, SHA-256 `7777cfb8a53af75923f665191c80e5acf83c81436658c0b4cc61a25a420c18f3`.
- Worker raw report SHA-256: `797a03764dab32ab3a46f8e83bab3369f8e7948b90a75f94738c4211e8a76add`.
- Runtime binary SHA-256: `6f1bb237b274f89fbac21709eea8f1f582752ee7b553dd5c78e24173682c8012`.

The five `*.before.json` files are the frozen pre-proposal state. The five `*.after-v7.json` files are the post-run authoritative state. `project-proposals.v1.preapproval.json` and `ui-preapproval.jpeg` preserve the user approval surface. `worker-last-message.txt` is the raw `/tmp` return copied immediately after worker completion. `supervisor-output/` contains every supervisor step's last-message and stderr file. `ui-final.jpeg` preserves the final user-visible closure. `target-file.snapshot.txt` is the byte-for-byte target snapshot; `target-verification.txt` records the independent byte checks; `SHA256SUMS` covers every frozen file except itself.

The completion claim is made in `../../2026-07-12-orchestrator-station3a-control-core-bridge-v1.md`; this directory preserves raw material and does not replace that evidence summary.
