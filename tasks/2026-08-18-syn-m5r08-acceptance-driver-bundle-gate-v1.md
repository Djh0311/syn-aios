# M5R08 narrow implementation package: default-bundle gate for the M5R07 acceptance driver

## Authority and boundary

- Authority: current `M5R08-m1-consumption-runtime-idempotency-and-acceptance-debts` leaf only.
- Implementer: Grok. Do not commit.
- Preserve all existing working-copy changes. The allowed file is clean at package start.
- Do not change UI layout, ordinary product interactions, Rust commands, backend acceptance status, other M2/M4 bridges, build tooling, package manifests, harness lifecycle, or syn-shell.

## Only allowed product file

`prototypes/productized-desktop-shell/src/main.tsx`

Opening SHA-256: `768ac6139b2cc1a2e7c6f3853469535c7d9411bec03ded7d825d36eb73f0f1a5`

## Problem to close

The M5R07 ordinary and isolated acceptance drivers are declared in `main.tsx` and are installed unconditionally at normal frontend startup. Even though each later checks backend status, the acceptance driver is still part of and entered from the normal frontend bundle. The accepted debt requires the normal/default build not to carry or start it, while preserving an explicit acceptance-only build/run path.

## Required behavior

1. Add one explicit build-time opt-in flag named `VITE_SYN_M5R07_ACCEPTANCE_DRIVER`. Only the exact value `"1"` may enable the M5R07 acceptance-driver entry.
2. Keep the existing backend `status.active` / `status.isolated` checks as the runtime half of the gate. Build opt-in alone must not make an inactive backend driver perform the flow.
3. With no flag, both `installM5R07OrdinaryControlAcceptanceDriver` and `installM5R07IsolatedAcceptanceDriver` must be unreachable at startup and removed by the production bundler together with their M5R07-only helper code and M5R07-only API imports. A normal `pnpm build` output must contain none of these markers: `m5r07_`, `m5r07Ordinary`, `m5r07Isolated`, `syn-m5r07`.
4. With `VITE_SYN_M5R07_ACCEPTANCE_DRIVER=1`, a production build must retain the acceptance driver markers and call both install functions through the explicit gate; the backend runtime checks remain intact.
5. Do not use `DEV` as the enabling condition, do not read a generic environment variable, and do not default the gate open.
6. Keep this a wiring/bundle-boundary change. Do not alter the acceptance flow, selectors, receipt contents, visible layout, or normal product code.

## Verification before handoff

From `prototypes/productized-desktop-shell` with the existing local dependencies:

```bash
pnpm typecheck
pnpm build
rg -n 'm5r07_|m5r07Ordinary|m5r07Isolated|syn-m5r07' dist
VITE_SYN_M5R07_ACCEPTANCE_DRIVER=1 pnpm build
rg -n 'm5r07_|m5r07Ordinary|m5r07Isolated|syn-m5r07' dist
```

The first `rg` must exit 1 (no marker); the second must exit 0 (explicit acceptance build retained). Report all command exits separately. Do not commit.
