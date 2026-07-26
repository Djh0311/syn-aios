import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const rawDirectory = new URL("./", import.meta.url);
const browser = await chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});

try {
  const context = await browser.newContext({ viewport: { width: 1180, height: 760 }, deviceScaleFactor: 1 });
  const page = await context.newPage();
  await page.goto("http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html", { waitUntil: "networkidle" });
  await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
  await page.locator('[data-knowledge-region="activity"] button[aria-label="Canvas"]').click();
  await page.locator('[data-active-group="true"] [data-canvas-file-trigger]').click();
  await page.waitForSelector('[data-active-group="true"] [data-canvas-file-panel]');
  await page.waitForTimeout(100);
  const observed = await page.evaluate(() => {
    const selectors = {
      central: ".knowledge-workbench-central",
      group: '[data-active-group="true"]',
      panelHost: '[data-active-group="true"] [data-knowledge-group-panel="active"]',
      root: ".native-knowledge-canvas",
      chrome: ".native-canvas-chrome",
      workspace: ".native-canvas-workspace",
      filePanel: "[data-canvas-file-panel]",
      entry: '[data-canvas-file-panel] button[title="canvas/visual-baseline.canvas"]',
    };
    const result = {};
    for (const [name, selector] of Object.entries(selectors)) {
      const element = document.querySelector(selector);
      if (!(element instanceof HTMLElement)) {
        result[name] = null;
        continue;
      }
      const rect = element.getBoundingClientRect();
      const style = window.getComputedStyle(element);
      result[name] = {
        bounds: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
        position: style.position,
        zIndex: style.zIndex,
        overflow: style.overflow,
        pointerEvents: style.pointerEvents,
      };
    }
    const entry = document.querySelector(selectors.entry);
    const rect = entry?.getBoundingClientRect();
    const hit = rect ? document.elementFromPoint(rect.x + rect.width / 2, rect.y + rect.height / 2) : null;
    return {
      elements: result,
      hit: hit instanceof Element ? {
        tag: hit.tagName.toLowerCase(),
        className: hit.getAttribute("class"),
        text: hit.textContent?.trim().slice(0, 120) ?? "",
      } : null,
    };
  });
  await page.screenshot({
    path: fileURLToPath(new URL("./diagnostic-1180-file-panel-layering.png", rawDirectory)),
    fullPage: false,
  });
  await fs.writeFile(
    fileURLToPath(new URL("./panel-layering-diagnostic.json", rawDirectory)),
    `${JSON.stringify(observed, null, 2)}\n`,
  );
  console.log(JSON.stringify(observed, null, 2));
  await context.close();
} finally {
  await browser.close();
}
