---
name: ui-browser-verification
description: Use before claiming any frontend, UI, visual, layout, responsive, browser interaction, or web app behavior is complete or fixed; requires real-browser verification with Chrome DevTools MCP when available, or Codex Browser, Playwright MCP, Computer Use, or an equivalent browser harness as fallback
---

# UI Browser Verification

## Rule

Frontend completion requires real browser evidence. Unit tests, type checks, lint, and build output are not enough for user-facing UI changes.

Use Chrome DevTools MCP by default when available. If it is unavailable, use Codex in-app Browser, Playwright MCP, Computer Use, or the nearest equivalent browser harness and report the limitation.

## Required Workflow

1. Identify the target route, user path, and viewport(s) from the requirement or plan.
2. Start or find the local app server. Do not claim UI completion if the page was never opened.
3. Open the target route in a real browser harness.
4. Capture visible state with screenshot, DOM/accessibility snapshot, or trace.
5. Exercise the changed user path: click, type, submit, navigate, open/close, filter, resize, or scroll as relevant.
6. Check browser console for runtime, hydration, asset, and warning signals relevant to the changed path.
7. Check network activity for failed requests, wrong endpoints, wrong status codes, or unexpected payload shape when the path talks to an API.
8. Verify responsive states when layout, mobile, sidebar, modal, table, canvas, or viewport behavior changed.
9. Save or summarize evidence in `docs/evidence/` for Strict Path work, multi-agent work, bug fixes, important UI changes, and non-trivial UI changes.
10. Report exactly what was verified and what remains unverified.

## Minimum Evidence

- URL or route checked
- viewport(s) checked
- interaction path performed
- console status
- network status when relevant
- screenshot, DOM observation, trace, or clear browser-harness summary

## Stop Conditions

Stop and do not claim completion if:

- the page cannot be opened
- the primary user path was not exercised
- visible layout is broken
- text overlaps or controls cannot be used
- console contains relevant runtime errors
- network calls fail for the changed path
- the browser harness is unavailable and no equivalent verification was performed

## Reporting Format

```markdown
UI verification:
- Harness:
- Route:
- Viewport(s):
- Interactions:
- Console:
- Network:
- Evidence:
- Unverified:
```
