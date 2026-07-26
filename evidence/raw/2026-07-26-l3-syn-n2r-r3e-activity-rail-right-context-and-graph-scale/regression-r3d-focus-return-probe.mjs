import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";
const { chromium } = playwrightCore;
const browser = await chromium.launch({ headless: true, executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" });
const context = await browser.newContext({ viewport: { width: 900, height: 760 }, reducedMotion: "reduce" });
const page = await context.newPage();
page.on("pageerror", (e) => console.log("PAGEERROR", String(e)));
await page.goto("http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html", { waitUntil: "networkidle" });
await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
const root = page.locator('[data-active-group="true"] .native-knowledge-graph');
await root.waitFor();
await root.locator(".native-graph-node-button").first().waitFor();
await page.waitForTimeout(600);
await page.locator('[data-knowledge-region="left"] button[aria-label="折叠左侧栏"]').click();
const rightToggle = page.locator('[data-knowledge-region="activity"] button[aria-label="切换右侧上下文"]');
if (await rightToggle.getAttribute("aria-pressed") === "true") await rightToggle.click();
const opener = root.locator("[data-graph-filter-opener]");
await opener.click();
await root.locator("[data-graph-filter-panel]").waitFor();
await page.waitForFunction(() => document.activeElement?.getAttribute("aria-label") === "关系图文字筛选");
await page.evaluate(() => { window.__probeOpener = document.querySelector("[data-graph-filter-opener]"); });
await page.keyboard.press("Escape");
await root.locator("[data-graph-filter-panel]").waitFor({ state: "detached" });
for (const wait of [0, 30, 100, 300, 800]) {
  if (wait) await page.waitForTimeout(wait);
  const state = await page.evaluate(() => {
    const live = document.querySelector("[data-graph-filter-opener]");
    const active = document.activeElement;
    return {
      activeTag: active?.tagName,
      activeLabel: active?.getAttribute("aria-label") ?? active?.textContent?.trim().slice(0, 12) ?? null,
      activeIsLiveOpener: active === live,
      storedStillConnected: window.__probeOpener?.isConnected ?? null,
      storedIsLive: window.__probeOpener === live,
      liveExists: Boolean(live),
    };
  });
  console.log("after", wait, JSON.stringify(state));
}
await browser.close();
