import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./red-browser-evidence.json", rawDirectory));
const redScreenshot = "red-empty-opener-escape-wrong-focus.png";
const canvasPath = "canvas/visual-baseline.canvas";
const preferenceKey = "syn-native-knowledge-workspace-ui-v1";
const expectedReadAllowlist = [
  "knowledge_workspace_graph",
  "knowledge_workspace_read_canvas",
  "knowledge_workspace_read_markdown",
  "knowledge_workspace_search",
  "knowledge_workspace_snapshot",
];
const expectedFailures = [
  "empty opener -> Escape returns to the same empty opener",
  "empty opener -> explicit close returns to the same empty opener",
  "chrome opener -> successful selection focuses the continuous stage",
  "empty opener -> successful selection focuses the continuous stage",
];

const report = {
  phase: "pre-implementation-red-first",
  fixture: "synthetic-only",
  viewport: { width: 1180, height: 760 },
  assertions: 0,
  failed: 0,
  failures: [],
  expectedFailures,
  matrix: [],
  screenshots: [redScreenshot],
  outcome: "PENDING",
};

function storageContainsChromeOnly(storage) {
  const serialized = JSON.stringify(storage.latestNormalizedContent);
  return storage.keys.every((key) => key === preferenceKey)
    && !serialized.includes('"body"')
    && !serialized.includes("合成视觉基线")
    && !serialized.includes("这是一段只用于隔离浏览器视觉量尺的合成知识内容");
}

function noHorizontalOverflow(metrics) {
  const values = {
    documentElement: metrics.overflow.documentElement,
    body: metrics.overflow.body,
    shell: metrics.overflow.shell,
    activeGroupPanel: metrics.overflow.activeGroupPanel,
    canvasRoot: metrics.canvas.root.scroll,
  };
  return {
    values,
    ok: Object.values(values).every((metric) => metric && metric.scrollWidth <= metric.clientWidth),
  };
}

async function collectMetrics(page, scenario) {
  await page.evaluate((nextScenario) => {
    document.documentElement.dataset.fixtureScenario = nextScenario;
    window.dispatchEvent(new Event("n2r-r3c-capture"));
  }, scenario);
  await page.waitForTimeout(220);
  return JSON.parse(await page.locator("#knowledge-workbench-visual-metrics").textContent());
}

async function openEmptyCanvasSurface(page) {
  await page.locator('[data-knowledge-region="activity"] button[aria-label="Canvas"]').click();
  const root = page.locator('[data-active-group="true"] .native-knowledge-canvas');
  await root.waitFor();
  await root.locator(".native-canvas-empty").waitFor();
  return root;
}

async function openerIdentity(handle) {
  return handle.evaluate((opener) => {
    const active = document.activeElement;
    const activeElement = active instanceof HTMLElement ? active : null;
    const activeKind = active === document.body
      ? "body"
      : activeElement?.matches('[data-canvas-stage="continuous"]')
        ? "stage"
        : activeElement?.hasAttribute("data-canvas-file-trigger")
          ? "chrome-opener"
          : activeElement?.closest(".native-canvas-empty")
            ? "empty-opener"
            : activeElement?.closest("[data-canvas-file-panel]")
              ? "file-panel"
              : "other";
    return {
      sameNodeAsActiveElement: active === opener,
      openerConnected: opener.isConnected,
      activeConnected: activeElement?.isConnected ?? false,
      activeIsBody: active === document.body,
      activeKind,
      activeText: activeElement?.textContent?.trim() ?? null,
      activeAriaLabel: activeElement?.getAttribute("aria-label") ?? null,
      openerKind: opener.hasAttribute("data-canvas-file-trigger") ? "chrome-opener" : "empty-opener",
      openerText: opener.textContent?.trim() ?? null,
      openerExpanded: opener.getAttribute("aria-expanded"),
      openerControls: opener.getAttribute("aria-controls"),
      openerControlsExists: Boolean(
        opener.getAttribute("aria-controls")
        && document.getElementById(opener.getAttribute("aria-controls")),
      ),
    };
  });
}

async function runMatrixCase(browser, {
  name,
  openerKind,
  closeAction,
  screenshot = null,
}) {
  const context = await browser.newContext({
    viewport: report.viewport,
    deviceScaleFactor: 1,
    reducedMotion: "no-preference",
  });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  page.on("pageerror", (error) => pageErrors.push(String(error)));
  page.on("request", (request) => {
    const requestUrl = request.url();
    if (
      !requestUrl.startsWith("http://127.0.0.1:5173/")
      && !requestUrl.startsWith("data:")
      && !requestUrl.startsWith("blob:")
    ) {
      externalRequests.push(requestUrl);
    }
  });
  await page.route("**/*", async (route) => {
    const requestUrl = route.request().url();
    if (
      requestUrl.startsWith("http://127.0.0.1:5173/")
      || requestUrl.startsWith("data:")
      || requestUrl.startsWith("blob:")
    ) {
      await route.continue();
      return;
    }
    await route.abort("blockedbyclient");
  });

  const result = {
    name,
    openerKind,
    closeAction,
    passed: false,
    open: null,
    after: null,
    audit: null,
    screenshot,
  };
  try {
    await page.goto(fixtureUrl, { waitUntil: "networkidle" });
    await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
    const initialMetrics = JSON.parse(await page.locator("#knowledge-workbench-visual-metrics").textContent());
    const root = await openEmptyCanvasSurface(page);
    const opener = openerKind === "chrome"
      ? root.locator("[data-canvas-file-trigger]")
      : root.getByRole("button", { name: "选择画布", exact: true });
    await opener.waitFor();
    const openerHandle = await opener.elementHandle();
    if (!openerHandle) throw new Error("matrix_opener_handle_missing");

    await opener.click();
    const panel = root.locator("[data-canvas-file-panel]");
    await panel.waitFor();
    const existingCanvas = panel.locator(`button[title="${canvasPath}"]`);
    await existingCanvas.waitFor();
    await page.waitForFunction(() => document.activeElement?.closest("[data-canvas-file-panel]") !== null);
    const openIdentity = await openerIdentity(openerHandle);
    const openMetrics = await collectMetrics(page, `${name}-open`);
    result.open = {
      identity: openIdentity,
      firstFocusInsidePanel: openIdentity.activeKind === "file-panel",
      panelCount: openMetrics.canvas.filePanel.count,
      panelInteractiveChildren: openMetrics.canvas.filePanel.interactiveChildren,
    };

    if (closeAction === "escape") {
      await page.keyboard.press("Escape");
    } else if (closeAction === "explicit-close") {
      await panel.getByRole("button", { name: "关闭 Canvas 文件面板" }).click();
    } else if (closeAction === "select-existing") {
      await existingCanvas.click();
    } else {
      throw new Error(`unknown_close_action:${closeAction}`);
    }

    await panel.waitFor({ state: "detached" });
    if (closeAction === "select-existing") {
      await root.locator(".native-canvas-flow-stage .react-flow").waitFor();
    }
    await page.waitForTimeout(100);
    const afterIdentity = await openerIdentity(openerHandle);
    const finalMetrics = await collectMetrics(page, `${name}-after`);
    const overflow = noHorizontalOverflow(finalMetrics);
    const calls = Object.keys(finalMetrics.mock.callsByCommand);
    const openAriaComplete = (
      openIdentity.openerExpanded === "true"
      && openIdentity.openerControls === "native-canvas-file-panel"
      && openIdentity.openerControlsExists
    );
    const panelRemoved = (
      finalMetrics.canvas.filePanel.count === 0
      && finalMetrics.canvas.filePanel.interactiveChildren === 0
    );
    const cancelReturnedToSameIdentity = (
      afterIdentity.sameNodeAsActiveElement
      && afterIdentity.openerConnected
      && afterIdentity.activeConnected
      && !afterIdentity.activeIsBody
    );
    const selectionFocusedStage = (
      afterIdentity.activeKind === "stage"
      && afterIdentity.activeConnected
      && !afterIdentity.activeIsBody
      && finalMetrics.canvas.reactFlowCount === 1
    );
    result.passed = (
      openAriaComplete
      && result.open.firstFocusInsidePanel
      && panelRemoved
      && (closeAction === "select-existing" ? selectionFocusedStage : cancelReturnedToSameIdentity)
    );
    result.after = {
      identity: afterIdentity,
      panelCount: finalMetrics.canvas.filePanel.count,
      panelInteractiveChildren: finalMetrics.canvas.filePanel.interactiveChildren,
      canvasLoaded: finalMetrics.canvas.reactFlowCount === 1,
      expectedTarget: closeAction === "select-existing" ? "stage" : `${openerKind}-opener-same-DOM-node`,
    };
    result.audit = {
      localStorageEmptyBeforeMount: initialMetrics.fixture.localStorageEmptyBeforeMount,
      localStorage: finalMetrics.localStorage,
      readAllowlist: finalMetrics.mock.allowedReadCommands,
      callsByCommand: finalMetrics.mock.callsByCommand,
      writeCallCount: finalMetrics.mock.writeCallCount,
      unrecognizedCallCount: finalMetrics.mock.unrecognizedCallCount,
      externalRequestCount: externalRequests.length,
      consoleErrorCount: consoleErrors.length,
      pageErrorCount: pageErrors.length,
      overflow,
      exactReadAllowlist: JSON.stringify(finalMetrics.mock.allowedReadCommands) === JSON.stringify(expectedReadAllowlist),
      callsStayInsideReadAllowlist: calls.every((command) => expectedReadAllowlist.includes(command)),
      localStorageChromeOnly: finalMetrics.localStorage.keys.length <= 1
        && storageContainsChromeOnly(finalMetrics.localStorage),
    };
    if (screenshot) {
      await page.screenshot({
        path: fileURLToPath(new URL(`./${screenshot}`, rawDirectory)),
        fullPage: false,
      });
    }
  } catch (error) {
    result.inspectionError = String(error?.stack ?? error);
  } finally {
    report.assertions += 1;
    if (!result.passed) {
      report.failed += 1;
      report.failures.push(result.name);
    }
    report.matrix.push(result);
    await context.close();
  }
}

const browser = await chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});

try {
  await runMatrixCase(browser, {
    name: "chrome opener -> Escape returns to the same chrome opener",
    openerKind: "chrome",
    closeAction: "escape",
  });
  await runMatrixCase(browser, {
    name: "empty opener -> Escape returns to the same empty opener",
    openerKind: "empty",
    closeAction: "escape",
    screenshot: redScreenshot,
  });
  await runMatrixCase(browser, {
    name: "chrome opener -> explicit close returns to the same chrome opener",
    openerKind: "chrome",
    closeAction: "explicit-close",
  });
  await runMatrixCase(browser, {
    name: "empty opener -> explicit close returns to the same empty opener",
    openerKind: "empty",
    closeAction: "explicit-close",
  });
  await runMatrixCase(browser, {
    name: "chrome opener -> successful selection focuses the continuous stage",
    openerKind: "chrome",
    closeAction: "select-existing",
  });
  await runMatrixCase(browser, {
    name: "empty opener -> successful selection focuses the continuous stage",
    openerKind: "empty",
    closeAction: "select-existing",
  });
} finally {
  await browser.close();
}

const observedFailures = report.matrix.filter((entry) => !entry.passed).map((entry) => entry.name);
const auditsAreClean = report.matrix.every((entry) => (
  !entry.inspectionError
  && entry.audit?.localStorageEmptyBeforeMount === true
  && entry.audit?.exactReadAllowlist === true
  && entry.audit?.callsStayInsideReadAllowlist === true
  && entry.audit?.writeCallCount === 0
  && entry.audit?.unrecognizedCallCount === 0
  && entry.audit?.externalRequestCount === 0
  && entry.audit?.consoleErrorCount === 0
  && entry.audit?.pageErrorCount === 0
  && entry.audit?.overflow.ok === true
  && entry.audit?.localStorageChromeOnly === true
));
const expectedRedMatches = JSON.stringify(observedFailures) === JSON.stringify(expectedFailures);
report.outcome = expectedRedMatches && auditsAreClean ? "EXPECTED_RED" : "UNEXPECTED_RESULT";
report.visualConclusion = "VISUAL_UNCHANGED / FOCUS_PATH_BROKEN";
await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);

console.log(JSON.stringify({
  outcome: report.outcome,
  assertions: report.assertions,
  failed: report.failed,
  failures: report.failures,
  auditsAreClean,
}, null, 2));
process.exitCode = report.outcome === "EXPECTED_RED" ? 0 : 1;
