import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./red-browser-evidence.json", rawDirectory));
const screenshotPath = fileURLToPath(new URL("./red-1180-canvas-three-column.png", rawDirectory));
const report = {
  phase: "pre-implementation-red-first",
  fixture: "synthetic-only",
  viewport: { width: 1180, height: 760 },
  assertions: 0,
  failed: 0,
  failures: [],
  screenshot: "red-1180-canvas-three-column.png",
  observed: {},
};

function expect(name, condition, detail = {}) {
  report.assertions += 1;
  if (!condition) {
    report.failed += 1;
    report.failures.push({ name, detail });
  }
}

function bounds(element) {
  if (!element) return null;
  const value = element.getBoundingClientRect();
  return {
    x: Math.round(value.x),
    y: Math.round(value.y),
    width: Math.round(value.width),
    height: Math.round(value.height),
  };
}

const browser = await chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});

try {
  const context = await browser.newContext({ viewport: report.viewport, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  page.on("pageerror", (error) => pageErrors.push(String(error)));
  page.on("request", (request) => {
    const url = request.url();
    if (!url.startsWith("http://127.0.0.1:5173/") && !url.startsWith("data:") && !url.startsWith("blob:")) {
      externalRequests.push(url);
    }
  });
  await page.route("**/*", async (route) => {
    const url = route.request().url();
    if (url.startsWith("http://127.0.0.1:5173/") || url.startsWith("data:") || url.startsWith("blob:")) {
      await route.continue();
    } else {
      await route.abort("blockedbyclient");
    }
  });

  await page.goto(fixtureUrl, { waitUntil: "networkidle" });
  await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
  await page.locator('[data-knowledge-region="activity"] button[aria-label="Canvas"]').click();
  await page.waitForSelector(".native-knowledge-canvas");
  await page.getByRole("button", { name: "canvas/visual-baseline.canvas" }).click();
  await page.waitForSelector(".native-canvas-flow-stage .react-flow");
  await page.screenshot({ path: screenshotPath, fullPage: false });

  const observed = await page.evaluate(() => {
    const root = document.querySelector(".native-knowledge-canvas");
    const stage = document.querySelector(".native-canvas-flow-stage");
    const activePanel = document.querySelector('[data-active-group="true"] [data-knowledge-group-panel="active"]');
    const panelMetric = activePanel instanceof HTMLElement ? {
      clientWidth: activePanel.clientWidth,
      scrollWidth: activePanel.scrollWidth,
    } : null;
    return {
      rootCount: document.querySelectorAll(".native-knowledge-canvas").length,
      rootBounds: boundsFor(root),
      stageBounds: boundsFor(stage),
      stageRatio: root && stage ? stage.getBoundingClientRect().height / root.getBoundingClientRect().height : 0,
      legacyBrowserGrid: document.querySelectorAll(".native-canvas-browser-grid").length,
      pageHeader: document.querySelectorAll(".native-canvas-head").length,
      permanentCatalog: document.querySelectorAll(".native-canvas-catalog").length,
      permanentToolbar: document.querySelectorAll(".native-canvas-toolbar").length,
      emptyInspector: document.querySelectorAll(".native-canvas-inspector").length,
      compactChrome: document.querySelectorAll('[data-canvas-chrome="compact"]').length,
      fileTrigger: document.querySelector('[data-canvas-file-trigger]')?.getAttribute("aria-expanded") ?? null,
      continuousStage: document.querySelectorAll('[data-canvas-stage="continuous"]').length,
      floatingToolsInsideStage: Boolean(stage?.querySelector(".native-canvas-floating-tools")),
      panelMetric,
      localStorageEmptyBeforeMount: JSON.parse(document.querySelector("#knowledge-workbench-visual-metrics")?.textContent ?? "{}").fixture?.localStorageEmptyBeforeMount ?? null,
      metrics: JSON.parse(document.querySelector("#knowledge-workbench-visual-metrics")?.textContent ?? "{}"),
    };

    function boundsFor(element) {
      if (!element) return null;
      const value = element.getBoundingClientRect();
      return {
        x: Math.round(value.x),
        y: Math.round(value.y),
        width: Math.round(value.width),
        height: Math.round(value.height),
      };
    }
  });
  report.observed = observed;

  expect("one Canvas root remains", observed.rootCount === 1, { observed: observed.rootCount });
  expect("legacy three-column browser grid is retired", observed.legacyBrowserGrid === 0, { observed: observed.legacyBrowserGrid });
  expect("page-level Canvas header is retired", observed.pageHeader === 0, { observed: observed.pageHeader });
  expect("file catalog is closed and absent by default", observed.permanentCatalog === 0, { observed: observed.permanentCatalog });
  expect("legacy full-row toolbar is retired", observed.permanentToolbar === 0, { observed: observed.permanentToolbar });
  expect("empty inspector is absent by default", observed.emptyInspector === 0, { observed: observed.emptyInspector });
  expect("compact chrome exists", observed.compactChrome === 1, { observed: observed.compactChrome });
  expect("file trigger exposes collapsed ARIA state", observed.fileTrigger === "false", { observed: observed.fileTrigger });
  expect("continuous stage exists and owns at least 75 percent of Canvas height", (
    observed.continuousStage === 1 && observed.stageRatio >= 0.75
  ), { continuousStage: observed.continuousStage, stageRatio: observed.stageRatio });
  expect("node tools float inside the Canvas stage", observed.floatingToolsInsideStage === true, {
    observed: observed.floatingToolsInsideStage,
  });
  expect("active group panel has no horizontal overflow", (
    observed.panelMetric
    && observed.panelMetric.scrollWidth <= observed.panelMetric.clientWidth
  ), observed.panelMetric);
  expect("fresh context localStorage was empty before mount", observed.localStorageEmptyBeforeMount === true, {
    observed: observed.localStorageEmptyBeforeMount,
  });
  expect("synthetic fixture made no write or unknown call", (
    observed.metrics.mock?.writeCallCount === 0
    && observed.metrics.mock?.unrecognizedCallCount === 0
  ), observed.metrics.mock);
  expect("browser emitted no external request or runtime error", (
    externalRequests.length === 0 && consoleErrors.length === 0 && pageErrors.length === 0
  ), { externalRequests, consoleErrors, pageErrors });

  await context.close();
} finally {
  await browser.close();
  report.outcome = report.failed > 0 ? "EXPECTED_RED" : "UNEXPECTED_GREEN";
  await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

console.log(JSON.stringify({
  outcome: report.outcome,
  assertions: report.assertions,
  failed: report.failed,
  failures: report.failures.map((failure) => failure.name),
}, null, 2));

process.exitCode = report.failed > 0 ? 1 : 0;
