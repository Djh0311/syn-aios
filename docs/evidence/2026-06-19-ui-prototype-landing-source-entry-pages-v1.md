# UI Prototype Landing · Source Entry Pages Evidence v1

Date: 2026-06-19

Plan:
- `/Users/yoyi/workspace/product-line/docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`

Scope:
- Landed the plan decision that four source-style entries should exist first: ideas, proposal, tools, models.
- Kept them as read-only entry shells. No backend writes, no real runner, no model/provider calls, no credential reads.
- Did not change Agent page or KnowledgeBase page.

Changed:
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/SourceStylePlaceholder.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/sourceStylePlaceholder.css`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/ActiveWorkbenchView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/main.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/helpers/offlineShellScenarioTextFixtures.ts`

Result:
- Ideas page now has task clue / project reminder sections and a read-only idea boundary.
- Proposal page now has project boundary / workflow association sections and a proposal boundary.
- Tools page now has Harness resource / adapter action sections and a controlled tool boundary.
- Models page now has adapter / provider availability sections and a credential boundary.
- Offline tests render these pages through `renderActiveWorkbenchView`, so the route-to-entry wiring is covered.

Verification:
- `npm run typecheck` passed.
- `npm run test:offline-interaction` passed.
- `git diff --check` passed.

Notes:
- This is an entry-shell landing, not real feature completion for capture, proposal confirmation, runner execution, or credential management.
