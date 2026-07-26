import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const browser = await playwrightCore.chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});
const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await context.newPage();
await page.goto("http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html", { waitUntil: "networkidle" });
await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
await page.locator('[data-active-group="true"] .native-knowledge-graph').waitFor();
await page.waitForTimeout(200);
const layout = await page.locator('[data-active-group="true"] .native-knowledge-graph').evaluate((root) => {
  const entries = [];
  let element = root;
  while (element instanceof HTMLElement && entries.length < 9) {
    const rect = element.getBoundingClientRect();
    const style = getComputedStyle(element);
    entries.push({
      tag: element.tagName.toLowerCase(),
      className: element.className,
      dataPanel: element.dataset.knowledgeGroupPanel ?? null,
      rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
      client: { width: element.clientWidth, height: element.clientHeight },
      scroll: { width: element.scrollWidth, height: element.scrollHeight },
      display: style.display,
      position: style.position,
      height: style.height,
      minHeight: style.minHeight,
      gridTemplateRows: style.gridTemplateRows,
      flex: style.flex,
      overflow: style.overflow,
    });
    element = element.parentElement;
  }
  return entries;
});
console.log(JSON.stringify(layout, null, 2));
await context.close();
await browser.close();
