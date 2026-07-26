import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const browser = await playwrightCore.chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});
const context = await browser.newContext({ viewport: { width: 1180, height: 760 } });
const page = await context.newPage();
await page.goto("http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html", { waitUntil: "networkidle" });
await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
const root = page.locator('[data-active-group="true"] .native-knowledge-graph');
await root.waitFor();
await root.locator(".native-graph-node-button").first().waitFor();
await page.evaluate(() => {
  const internals = window.__TAURI_INTERNALS__;
  const originalInvoke = internals.invoke;
  window.__n2rR3dActivationInspect = [];
  internals.invoke = async (command, payload, options) => {
    window.__n2rR3dActivationInspect.push({ command, payload });
    return originalInvoke(command, payload, options);
  };
});
const target = root.locator(
  '[data-graph-relative-path="research/graph-projection.md"] .native-graph-node-button',
);
const before = await target.evaluate((button) => ({
  disabled: button.disabled,
  ariaLabel: button.getAttribute("aria-label"),
  active: document.activeElement === button,
  selected: button.closest(".react-flow__node")?.classList.contains("selected") ?? false,
}));
await target.click();
await page.waitForTimeout(1000);
const after = await page.evaluate(() => ({
  invokes: window.__n2rR3dActivationInspect ?? [],
  graphExists: Boolean(document.querySelector('[data-active-group="true"] .native-knowledge-graph')),
  activeTab: document.querySelector('[data-active-group="true"] [role="tab"][aria-selected="true"]')?.textContent?.trim() ?? null,
  selectedDocument: document.querySelector(".native-workspace-document-head strong")?.textContent?.trim() ?? null,
  notice: document.querySelector(".native-workspace-notice")?.textContent?.trim() ?? null,
  activeElement: {
    tag: document.activeElement?.tagName.toLowerCase() ?? null,
    ariaLabel: document.activeElement?.getAttribute("aria-label") ?? null,
  },
}));
if (after.invokes.length === 0) {
  await target.evaluate((button) => button.click());
  await page.waitForTimeout(1000);
}
const diagnosticProgrammaticClick = await page.evaluate(() => ({
  invokes: window.__n2rR3dActivationInspect ?? [],
  graphExists: Boolean(document.querySelector('[data-active-group="true"] .native-knowledge-graph')),
  activeTab: document.querySelector('[data-active-group="true"] [role="tab"][aria-selected="true"]')?.textContent?.trim() ?? null,
}));
console.log(JSON.stringify({ before, afterPhysicalClick: after, diagnosticProgrammaticClick }, null, 2));
await context.close();
await browser.close();
