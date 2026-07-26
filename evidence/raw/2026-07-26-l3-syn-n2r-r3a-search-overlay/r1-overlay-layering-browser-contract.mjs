import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const phase = process.argv.includes("--phase=green") ? "green" : "red";
const reportPath = fileURLToPath(
  new URL(`./r1-overlay-layering-${phase}.json`, import.meta.url),
);
const screenshots =
  phase === "green"
    ? {
        quickOpen: "02-r1-1180-quick-open-results.png",
        command: "03-r1-900-command-filter-results.png",
      }
    : {
        quickOpen: "r1-overlay-layering-red-quick-open.png",
        command: "r1-overlay-layering-red-command.png",
      };
const expectedReadAllowlist = [
  "knowledge_workspace_graph",
  "knowledge_workspace_read_canvas",
  "knowledge_workspace_read_markdown",
  "knowledge_workspace_search",
  "knowledge_workspace_snapshot",
];
const report = {
  contract: "N2R-R3A-R1 overlay layering",
  phase,
  fixture: "real React + production CSS + pure-synthetic data",
  assertions: 0,
  failed: 0,
  failures: [],
  contexts: [],
  screenshots: Object.values(screenshots),
};

function expect(name, condition, detail = {}) {
  report.assertions += 1;
  if (!condition) {
    report.failed += 1;
    report.failures.push({ name, detail });
  }
}

async function collectMetrics(page, scenario) {
  await page.evaluate((nextScenario) => {
    document.documentElement.dataset.fixtureScenario = nextScenario;
    window.dispatchEvent(new Event("n2r-r2-capture"));
  }, scenario);
  await page.waitForTimeout(180);
  return JSON.parse(
    await page.locator("#knowledge-workbench-visual-metrics").textContent(),
  );
}

function noHorizontalOverflow(metrics) {
  const values = {
    document: metrics.overflow.documentElement,
    body: metrics.overflow.body,
    shell: metrics.overflow.shell,
  };
  return {
    values,
    ok: Object.values(values).every(
      (metric) => metric && metric.scrollWidth <= metric.clientWidth,
    ),
  };
}

async function inspectLayering(page, basePng, overlayPng) {
  const overlayBox = await page.locator(".syn-knowledge-overlay").boundingBox();
  return page.evaluate(
    async ({ baseDataUrl, overlayDataUrl, box }) => {
      const loadImage = (src) =>
        new Promise((resolve, reject) => {
          const image = new Image();
          image.onload = () => resolve(image);
          image.onerror = reject;
          image.src = src;
        });
      const readPixels = async (src) => {
        const image = await loadImage(src);
        const canvas = document.createElement("canvas");
        canvas.width = image.naturalWidth;
        canvas.height = image.naturalHeight;
        const context = canvas.getContext("2d", { willReadFrequently: true });
        context.drawImage(image, 0, 0);
        return {
          width: canvas.width,
          height: canvas.height,
          pixels: context.getImageData(0, 0, canvas.width, canvas.height).data,
        };
      };
      const parseAlpha = (color) => {
        const canvas = document.createElement("canvas");
        canvas.width = 1;
        canvas.height = 1;
        const context = canvas.getContext("2d", { willReadFrequently: true });
        context.clearRect(0, 0, 1, 1);
        context.fillStyle = color;
        context.fillRect(0, 0, 1, 1);
        return context.getImageData(0, 0, 1, 1).data[3] / 255;
      };
      const luminance = (pixels, index) =>
        pixels[index] * 0.2126 +
        pixels[index + 1] * 0.7152 +
        pixels[index + 2] * 0.0722;
      const outsidePanel = (x, y) =>
        !box ||
        x < box.x - 16 ||
        x > box.x + box.width + 16 ||
        y < box.y - 16 ||
        y > box.y + box.height + 16;
      const edgeEnergy = ({ width, height, pixels }) => {
        let total = 0;
        let samples = 0;
        const buckets = new Set();
        for (let y = 4; y < height - 4; y += 4) {
          for (let x = 4; x < width - 4; x += 4) {
            if (
              !outsidePanel(x, y) ||
              !outsidePanel(x + 4, y) ||
              !outsidePanel(x, y + 4)
            ) {
              continue;
            }
            const index = (y * width + x) * 4;
            const rightIndex = (y * width + x + 4) * 4;
            const downIndex = ((y + 4) * width + x) * 4;
            const here = luminance(pixels, index);
            total += Math.abs(here - luminance(pixels, rightIndex));
            total += Math.abs(here - luminance(pixels, downIndex));
            samples += 2;
            buckets.add(
              `${pixels[index] >> 4}:${pixels[index + 1] >> 4}:${pixels[index + 2] >> 4}`,
            );
          }
        }
        return {
          meanEdgeEnergy: samples > 0 ? total / samples : 0,
          sampledEdges: samples,
          quantizedColorBuckets: buckets.size,
        };
      };

      const backdrop = document.querySelector(
        ".syn-knowledge-overlay-backdrop",
      );
      const panel = document.querySelector(".syn-knowledge-overlay");
      const backdropStyle = getComputedStyle(backdrop);
      const panelStyle = getComputedStyle(panel);
      const base = edgeEnergy(await readPixels(baseDataUrl));
      const overlay = edgeEnergy(await readPixels(overlayDataUrl));
      const retainedEdgeRatio =
        base.meanEdgeEnergy > 0
          ? overlay.meanEdgeEnergy / base.meanEdgeEnergy
          : 0;

      return {
        backdrop: {
          backgroundColor: backdropStyle.backgroundColor,
          backgroundAlpha: parseAlpha(backdropStyle.backgroundColor),
          backgroundImage: backdropStyle.backgroundImage,
          backdropFilter:
            backdropStyle.backdropFilter ||
            backdropStyle.webkitBackdropFilter ||
            "none",
          filter: backdropStyle.filter,
        },
        panel: {
          backgroundColor: panelStyle.backgroundColor,
          backgroundAlpha: parseAlpha(panelStyle.backgroundColor),
          borderTopColor: panelStyle.borderTopColor,
          borderTopWidth: panelStyle.borderTopWidth,
          boxShadow: panelStyle.boxShadow,
        },
        screenshotAnalysis: {
          base,
          overlay,
          retainedEdgeRatio,
          workspaceContextVisible:
            retainedEdgeRatio >= 0.12 &&
            retainedEdgeRatio <= 0.7 &&
            overlay.quantizedColorBuckets >= 8,
        },
        overlayBox: box,
        viewport: {
          width: window.innerWidth,
          height: window.innerHeight,
        },
      };
    },
    {
      baseDataUrl: `data:image/png;base64,${basePng.toString("base64")}`,
      overlayDataUrl: `data:image/png;base64,${overlayPng.toString("base64")}`,
      box: overlayBox,
    },
  );
}

function assertLayering(name, layering) {
  expect(
    `${name}: computed backdrop alpha is strictly between zero and one`,
    layering.backdrop.backgroundAlpha > 0 &&
      layering.backdrop.backgroundAlpha < 1,
    layering.backdrop,
  );
  expect(
    `${name}: screenshot outside the overlay retains a subdued workspace`,
    layering.screenshotAnalysis.workspaceContextVisible,
    layering.screenshotAnalysis,
  );
  expect(
    `${name}: foreground overlay remains an opaque raised surface`,
    layering.panel.backgroundAlpha === 1 &&
      Number.parseFloat(layering.panel.borderTopWidth) > 0,
    layering.panel,
  );
  expect(
    `${name}: backdrop adds no gradient, blur, or filter`,
    layering.backdrop.backgroundImage === "none" &&
      !layering.backdrop.backdropFilter.includes("blur") &&
      layering.backdrop.filter === "none",
    layering.backdrop,
  );
}

async function withFreshFixture(browser, name, viewport, action) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = { name, viewport, audit: {}, evidence: {} };

  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  page.on("pageerror", (error) => pageErrors.push(String(error)));
  page.on("request", (request) => {
    const requestUrl = request.url();
    if (
      !requestUrl.startsWith("http://127.0.0.1:5173/") &&
      !requestUrl.startsWith("data:") &&
      !requestUrl.startsWith("blob:")
    ) {
      externalRequests.push(requestUrl);
    }
  });

  try {
    await page.goto(fixtureUrl, { waitUntil: "networkidle" });
    await page.waitForFunction(
      () => document.documentElement.dataset.fixtureReady === "true",
    );
    const before = JSON.parse(
      await page.locator("#knowledge-workbench-visual-metrics").textContent(),
    );
    contextReport.audit.localStorageEmptyBeforeMount =
      before.fixture.localStorageEmptyBeforeMount;
    expect(
      `${name}: localStorage was empty before mount`,
      before.fixture.localStorageEmptyBeforeMount,
      { observed: before.fixture.localStorageEmptyBeforeMount },
    );

    const basePng = await page.screenshot();
    contextReport.evidence = await action(page, basePng);
    const metrics = await collectMetrics(page, name);
    const overflow = noHorizontalOverflow(metrics);
    contextReport.metrics = metrics;
    contextReport.overflow = overflow;
    contextReport.audit.readAllowlist = metrics.mock.allowedReadCommands;
    contextReport.audit.callsByCommand = metrics.mock.callsByCommand;
    contextReport.audit.writeCallCount = metrics.mock.writeCallCount;
    contextReport.audit.unrecognizedCallCount =
      metrics.mock.unrecognizedCallCount;
    contextReport.audit.externalRequestCount = externalRequests.length;
    contextReport.audit.consoleErrorCount = consoleErrors.length;
    contextReport.audit.pageErrorCount = pageErrors.length;
    contextReport.audit.externalRequests = externalRequests;
    contextReport.audit.consoleErrors = consoleErrors;
    contextReport.audit.pageErrors = pageErrors;

    expect(
      `${name}: fixture exposes the exact read allowlist`,
      JSON.stringify(metrics.mock.allowedReadCommands) ===
        JSON.stringify(expectedReadAllowlist),
      { observed: metrics.mock.allowedReadCommands },
    );
    expect(
      `${name}: no unexpected mock command is called`,
      Object.keys(metrics.mock.callsByCommand).every((command) =>
        expectedReadAllowlist.includes(command),
      ),
      { callsByCommand: metrics.mock.callsByCommand },
    );
    expect(
      `${name}: mock write calls stay zero`,
      metrics.mock.writeCallCount === 0,
      { observed: metrics.mock.writeCallCount },
    );
    expect(
      `${name}: mock unrecognized calls stay zero`,
      metrics.mock.unrecognizedCallCount === 0,
      { observed: metrics.mock.unrecognizedCallCount },
    );
    expect(
      `${name}: external request count stays zero`,
      externalRequests.length === 0,
      { externalRequests },
    );
    expect(
      `${name}: console error count stays zero`,
      consoleErrors.length === 0,
      { consoleErrors },
    );
    expect(
      `${name}: page error count stays zero`,
      pageErrors.length === 0,
      { pageErrors },
    );
    expect(
      `${name}: document/body/shell have no horizontal overflow`,
      overflow.ok,
      overflow.values,
    );
  } catch (error) {
    expect(`${name}: browser evidence completed without inspection errors`, false, {
      error: String(error?.stack ?? error),
    });
    contextReport.inspectionError = String(error?.stack ?? error);
  } finally {
    report.contexts.push(contextReport);
    await context.close();
  }
}

const browser = await chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});

try {
  await withFreshFixture(
    browser,
    "quick-open",
    { width: 1180, height: 760 },
    async (page, basePng) => {
      const trigger = page.getByRole("button", { name: "搜索" });
      await trigger.focus();
      await page.keyboard.press("Meta+o");
      const dialog = page.getByRole("dialog", { name: "快速打开" });
      await dialog.waitFor();
      const input = dialog.getByRole("combobox", { name: "快速打开" });
      const initialFocus = await page.evaluate(
        () => document.activeElement?.id ?? null,
      );
      expect(
        "quick-open: ⌘O places focus in the primary input",
        initialFocus === "native-knowledge-quick-open",
        { initialFocus },
      );
      await input.fill("合成");
      await dialog.getByRole("button", { name: "查找" }).click();
      await page.waitForFunction(
        () =>
          document.querySelectorAll(
            ".syn-knowledge-overlay [role=listbox] [role=option]",
          ).length > 0,
      );
      await input.press("ArrowDown");
      const state = await page.evaluate(() => ({
        optionCount: document.querySelectorAll(
          ".syn-knowledge-overlay [role=listbox] [role=option]",
        ).length,
        currentCount: document.querySelectorAll(
          '.syn-knowledge-overlay [role=listbox] [role=option][aria-selected="true"]',
        ).length,
        activeDescendant:
          document.activeElement?.getAttribute("aria-activedescendant") ?? null,
        hints: [
          ...document.querySelectorAll(
            ".syn-knowledge-overlay .native-workspace-overlay-hints span",
          ),
        ].map((element) => element.textContent),
      }));
      expect(
        "quick-open: results, current item, Arrow navigation, and hints remain green",
        state.optionCount > 0 &&
          state.currentCount === 1 &&
          state.activeDescendant === "native-knowledge-quick-open-option-1" &&
          state.hints.join(" ").includes("↑↓") &&
          state.hints.join(" ").includes("Enter") &&
          state.hints.join(" ").includes("Esc"),
        state,
      );

      const screenshotPath = fileURLToPath(
        new URL(`./${screenshots.quickOpen}`, rawDirectory),
      );
      const overlayPng = await page.screenshot({ path: screenshotPath });
      const layering = await inspectLayering(page, basePng, overlayPng);
      assertLayering("quick-open", layering);

      await input.press("Escape");
      await page.waitForTimeout(100);
      const afterEscape = await page.evaluate(() => ({
        dialogCount: document.querySelectorAll(".syn-knowledge-overlay").length,
        focusAriaLabel:
          document.activeElement?.getAttribute("aria-label") ?? null,
      }));
      expect(
        "quick-open: Escape closes and restores trigger focus",
        afterEscape.dialogCount === 0 &&
          afterEscape.focusAriaLabel === "搜索",
        afterEscape,
      );
      return {
        query: "合成",
        initialFocus,
        state,
        layering,
        afterEscape,
        screenshot: screenshots.quickOpen,
      };
    },
  );

  await withFreshFixture(
    browser,
    "command",
    { width: 900, height: 760 },
    async (page, basePng) => {
      const trigger = page.getByRole("button", { name: "Syn 命令" });
      await trigger.focus();
      await page.keyboard.press("Meta+p");
      const dialog = page.getByRole("dialog", { name: "Syn 命令" });
      await dialog.waitFor();
      const input = dialog.getByRole("combobox", {
        name: "筛选 Syn 命令",
      });
      const initialCount = await dialog
        .locator("[role=listbox] [role=option]")
        .count();
      expect(
        "command: both existing safe commands remain available",
        initialCount === 2,
        { initialCount },
      );
      await input.fill("目录");
      await page.waitForFunction(
        () =>
          document.querySelectorAll(
            ".syn-knowledge-overlay [role=listbox] [role=option]",
          ).length === 1,
      );
      const state = await page.evaluate(() => ({
        optionCount: document.querySelectorAll(
          ".syn-knowledge-overlay [role=listbox] [role=option]",
        ).length,
        currentCount: document.querySelectorAll(
          '.syn-knowledge-overlay [role=listbox] [role=option][aria-selected="true"]',
        ).length,
        currentText:
          document
            .querySelector(
              '.syn-knowledge-overlay [role=listbox] [role=option][aria-selected="true"]',
            )
            ?.textContent?.trim() ?? null,
        focusedId: document.activeElement?.id ?? null,
        hints: [
          ...document.querySelectorAll(
            ".syn-knowledge-overlay .native-workspace-overlay-hints span",
          ),
        ].map((element) => element.textContent),
      }));
      expect(
        "command: filtering, current item, input focus, and hints remain green",
        state.optionCount === 1 &&
          state.currentCount === 1 &&
          state.currentText?.includes("新建目录") &&
          state.focusedId === "native-knowledge-command-filter" &&
          state.hints.join(" ").includes("↑↓") &&
          state.hints.join(" ").includes("Enter") &&
          state.hints.join(" ").includes("Esc"),
        state,
      );

      const screenshotPath = fileURLToPath(
        new URL(`./${screenshots.command}`, rawDirectory),
      );
      const overlayPng = await page.screenshot({ path: screenshotPath });
      const layering = await inspectLayering(page, basePng, overlayPng);
      assertLayering("command", layering);

      await input.press("Enter");
      await page.waitForTimeout(80);
      const safeForm = await page.evaluate(() => ({
        pathInputCount: document.querySelectorAll(
          '.syn-knowledge-overlay input[aria-label="新建条目的相对路径"]',
        ).length,
        createButtonDisabled:
          document.querySelector(
            ".syn-knowledge-overlay button.secondary-button",
          ) instanceof HTMLButtonElement
            ? document.querySelector(
                ".syn-knowledge-overlay button.secondary-button",
              ).disabled
            : null,
      }));
      expect(
        "command: Enter only reaches the existing disabled safe-path form",
        safeForm.pathInputCount === 1 &&
          safeForm.createButtonDisabled === true,
        safeForm,
      );
      await page.keyboard.press("Escape");
      await page.waitForTimeout(100);
      const afterEscape = await page.evaluate(() => ({
        dialogCount: document.querySelectorAll(".syn-knowledge-overlay").length,
        focusAriaLabel:
          document.activeElement?.getAttribute("aria-label") ?? null,
      }));
      expect(
        "command: Escape closes and restores trigger focus",
        afterEscape.dialogCount === 0 &&
          afterEscape.focusAriaLabel === "Syn 命令",
        afterEscape,
      );
      return {
        query: "目录",
        initialCount,
        state,
        layering,
        safeForm,
        afterEscape,
        screenshot: screenshots.command,
      };
    },
  );
} finally {
  await browser.close();
  report.outcome = report.failed === 0 ? "PASS" : "FAIL";
  await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

console.log(
  JSON.stringify(
    {
      phase,
      outcome: report.outcome,
      assertions: report.assertions,
      failed: report.failed,
      failures: report.failures.map((failure) => failure.name),
    },
    null,
    2,
  ),
);
process.exitCode = report.failed === 0 ? 0 : 1;
