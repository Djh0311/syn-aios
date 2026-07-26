import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const browser = await playwrightCore.chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});
for (const key of ["Enter", "Space"]) {
  const context = await browser.newContext({ viewport: { width: 1180, height: 760 } });
  const page = await context.newPage();
  await page.goto("http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html", { waitUntil: "networkidle" });
  await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
  await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
  const root = page.locator('[data-active-group="true"] .native-knowledge-graph');
  await root.waitFor();
  const selector = '[data-graph-relative-path="research/graph-projection.md"] .native-graph-node-button';
  for (let index = 0; index < 160; index += 1) {
    await page.keyboard.press("Tab");
    if (await page.evaluate((target) => document.activeElement?.matches(target) ?? false, selector)) break;
  }
  await page.waitForTimeout(100);
  const before = await page.evaluate((target) => {
    const active = document.activeElement;
    return {
      matches: active?.matches(target) ?? false,
      tag: active?.tagName.toLowerCase() ?? null,
      label: active?.getAttribute("aria-label") ?? null,
      disabled: active instanceof HTMLButtonElement ? active.disabled : null,
      selected: active?.closest(".react-flow__node")?.classList.contains("selected") ?? false,
    };
  }, selector);
  await page.keyboard.press(key);
  await page.waitForTimeout(1000);
  const after = await page.evaluate(() => {
    const metrics = JSON.parse(document.querySelector("#knowledge-workbench-visual-metrics")?.textContent ?? "{}");
    return {
      activeTab: document.querySelector('[data-active-group="true"] [role="tab"][aria-selected="true"]')?.textContent?.trim() ?? null,
      graphExists: Boolean(document.querySelector('[data-active-group="true"] .native-knowledge-graph')),
      calls: metrics.mock?.callsByCommand ?? null,
      activeTag: document.activeElement?.tagName.toLowerCase() ?? null,
      activeLabel: document.activeElement?.getAttribute("aria-label") ?? null,
    };
  });
  console.log(JSON.stringify({ key, before, after }, null, 2));
  await context.close();
}
await browser.close();
