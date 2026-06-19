# UI Prototype Landing · Project Side Panel Disclosure Evidence v1

Date: 2026-06-19

Plan:
- `/Users/yoyi/workspace/product-line/docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`

Scope:
- Landed the project workflow "one screen, overflow folded" direction for the right-side workflow control stream.
- Kept the existing data/actions intact; this is layout disclosure, not a workflow semantics change.

Changed:
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowSidePanel.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/styles.css`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/helpers/offlineShellScenarioTextFixtures.ts`

Result:
- The side panel now keeps the main working path visible: node detail, unified execution, and current work-item orchestration.
- Secondary groups are folded into `details` sections:
  - `运行检查`
  - `方案与授权`
  - `事实与记忆`
- Offline text fixtures assert those disclosure entries exist in the workflow canvas surface.

Verification:
- `npm run typecheck` passed.
- `npm run test:offline-interaction` passed.
- `git diff --check` passed.

Notes:
- The hidden content is still rendered and reachable through native disclosure controls.
- This does not change Agent or KnowledgeBase pages.
