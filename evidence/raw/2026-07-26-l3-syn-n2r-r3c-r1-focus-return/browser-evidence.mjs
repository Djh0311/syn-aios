import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./browser-evidence.json", rawDirectory));
const canvasPath = "canvas/visual-baseline.canvas";
const preferenceKey = "syn-native-knowledge-workspace-ui-v1";
const expectedReadAllowlist = [
  "knowledge_workspace_graph",
  "knowledge_workspace_read_canvas",
  "knowledge_workspace_read_markdown",
  "knowledge_workspace_search",
  "knowledge_workspace_snapshot",
];
const screenshots = [
  "01-1180-empty-opener-panel-open.png",
  "02-1180-empty-opener-escape-focus-return.png",
  "03-1180-empty-opener-selection-stage-focus.png",
];
const report = {
  phase: "post-implementation-synthetic-browser-evidence",
  fixture: "synthetic-only",
  visualConclusion: "VISUAL_UNCHANGED / FOCUS_PATH_FIXED",
  contexts: [],
  assertions: 0,
  failed: 0,
  failures: [],
  screenshots,
  outcome: "PENDING",
};

function check(contextReport, name, condition, detail = {}) {
  const passed = Boolean(condition);
  report.assertions += 1;
  contextReport.assertions.push({ name, passed, ...(passed ? {} : { detail }) });
  if (!passed) {
    report.failed += 1;
    report.failures.push({ context: contextReport.name, name, detail });
  }
}

function storageContainsChromeOnly(storage) {
  const serialized = JSON.stringify(storage.latestNormalizedContent);
  return storage.keys.every((key) => key === preferenceKey)
    && !serialized.includes('"body"')
    && !serialized.includes("合成视觉基线")
    && !serialized.includes("这是一段只用于隔离浏览器视觉量尺的合成知识内容");
}

function horizontalOverflowState(metrics) {
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

async function saveScreenshot(page, filename) {
  await page.screenshot({
    path: fileURLToPath(new URL(`./${filename}`, rawDirectory)),
    fullPage: false,
  });
}

async function openEmptyCanvasSurface(page) {
  await page.locator('[data-knowledge-region="activity"] button[aria-label="Canvas"]').click();
  const root = page.locator('[data-active-group="true"] .native-knowledge-canvas');
  await root.waitFor();
  await root.locator(".native-canvas-empty").waitFor();
  return root;
}

function openerLocator(root, openerKind) {
  return openerKind === "chrome"
    ? root.locator('[data-canvas-file-opener="chrome"]')
    : root.locator('[data-canvas-file-opener="empty"]');
}

async function identityState(handle) {
  return handle.evaluate((opener) => {
    const active = document.activeElement;
    const activeElement = active instanceof HTMLElement ? active : null;
    const activeKind = active === document.body
      ? "body"
      : activeElement?.matches('[data-canvas-stage="continuous"]')
        ? "stage"
        : activeElement?.getAttribute("data-canvas-file-opener") === "chrome"
          ? "chrome-opener"
          : activeElement?.getAttribute("data-canvas-file-opener") === "empty"
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
      openerKind: opener.getAttribute("data-canvas-file-opener"),
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

async function openPanelRound(page, root, contextReport, label, openerKind, screenshot = null) {
  const opener = openerLocator(root, openerKind);
  await opener.waitFor();
  const openerHandle = await opener.elementHandle();
  if (!openerHandle) throw new Error(`${label}:opener_handle_missing`);
  await opener.click();
  const panel = root.locator("[data-canvas-file-panel]");
  await panel.waitFor();
  const existingCanvas = panel.locator(`button[title="${canvasPath}"]`);
  await existingCanvas.waitFor();
  await page.waitForFunction(() => document.activeElement?.closest("[data-canvas-file-panel]") !== null);
  const identity = await identityState(openerHandle);
  const metrics = await collectMetrics(page, `${contextReport.name}-${label}-open`);
  check(contextReport, `${label}: first focus enters the file panel`, (
    identity.activeKind === "file-panel"
    && identity.activeConnected
    && !identity.activeIsBody
  ), identity);
  check(contextReport, `${label}: actual opener exposes expanded controls linked to the live panel`, (
    identity.openerExpanded === "true"
    && identity.openerControls === "native-canvas-file-panel"
    && identity.openerControlsExists
  ), identity);
  check(contextReport, `${label}: file panel remains one contained absolute Canvas surface`, (
    metrics.canvas.filePanel.count === 1
    && metrics.canvas.filePanel.withinRoot === true
    && metrics.canvas.filePanel.position === "absolute"
    && metrics.canvas.filePanel.interactiveChildren === 5
  ), metrics.canvas.filePanel);
  if (screenshot) await saveScreenshot(page, screenshot);
  return { opener, openerHandle, panel, existingCanvas, identity, metrics };
}

async function cancelRound(page, root, contextReport, {
  label,
  openerKind,
  action,
  openScreenshot = null,
  closedScreenshot = null,
}) {
  const opened = await openPanelRound(page, root, contextReport, label, openerKind, openScreenshot);
  if (action === "escape") {
    await page.keyboard.press("Escape");
  } else if (action === "explicit-close") {
    await opened.panel.getByRole("button", { name: "关闭 Canvas 文件面板" }).click();
  } else if (action === "opener-toggle") {
    await opened.opener.click();
  } else {
    throw new Error(`${label}:unknown_cancel_action:${action}`);
  }
  await opened.panel.waitFor({ state: "detached" });
  await page.waitForFunction((opener) => document.activeElement === opener, opened.openerHandle);
  const identity = await identityState(opened.openerHandle);
  const metrics = await collectMetrics(page, `${contextReport.name}-${label}-closed`);
  check(contextReport, `${label}: closing removes panel controls from DOM Tab and accessibility paths`, (
    metrics.canvas.filePanel.count === 0
    && metrics.canvas.filePanel.interactiveChildren === 0
    && identity.openerExpanded === "false"
    && identity.openerControls === "native-canvas-file-panel"
    && !identity.openerControlsExists
  ), { identity, panel: metrics.canvas.filePanel });
  check(contextReport, `${label}: activeElement is the exact clicked opener DOM identity`, (
    identity.sameNodeAsActiveElement
    && identity.openerConnected
    && identity.activeConnected
    && !identity.activeIsBody
    && identity.activeKind === `${openerKind}-opener`
  ), identity);
  if (closedScreenshot) await saveScreenshot(page, closedScreenshot);
  return {
    label,
    openerKind,
    action,
    open: { identity: opened.identity, canvas: opened.metrics.canvas },
    closed: { identity, canvas: metrics.canvas },
  };
}

async function selectionRound(page, root, contextReport, {
  label,
  openerKind,
  screenshot = null,
}) {
  const opened = await openPanelRound(page, root, contextReport, label, openerKind);
  await opened.existingCanvas.click();
  await opened.panel.waitFor({ state: "detached" });
  await root.locator(".native-canvas-flow-stage .react-flow").waitFor();
  await page.waitForFunction(() => document.activeElement?.matches('[data-canvas-stage="continuous"]'));
  const identity = await identityState(opened.openerHandle);
  const metrics = await collectMetrics(page, `${contextReport.name}-${label}-selected`);
  check(contextReport, `${label}: successful selection unloads the panel and loads one Canvas`, (
    metrics.canvas.filePanel.count === 0
    && metrics.canvas.filePanel.interactiveChildren === 0
    && metrics.canvas.reactFlowCount === 1
    && metrics.canvas.chrome.currentPathLabel === `当前 Canvas：${canvasPath}`
  ), metrics.canvas);
  check(contextReport, `${label}: successful selection focuses the connected continuous stage`, (
    identity.activeKind === "stage"
    && identity.activeConnected
    && !identity.activeIsBody
    && !identity.sameNodeAsActiveElement
  ), identity);
  check(contextReport, `${label}: successful selection performs one allowed Canvas read and no write`, (
    metrics.mock.callsByCommand.knowledge_workspace_read_canvas === 1
    && metrics.mock.writeCallCount === 0
    && metrics.mock.unrecognizedCallCount === 0
  ), metrics.mock);
  if (screenshot) await saveScreenshot(page, screenshot);
  return {
    label,
    openerKind,
    action: "select-existing",
    open: { identity: opened.identity, canvas: opened.metrics.canvas },
    selected: { identity, canvas: metrics.canvas },
  };
}

async function withFreshContext(browser, {
  name,
  viewport,
  reducedMotion = "no-preference",
  action,
}) {
  const context = await browser.newContext({
    viewport,
    deviceScaleFactor: 1,
    reducedMotion,
  });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = {
    name,
    viewport,
    reducedMotion,
    assertions: [],
    evidence: {},
    audit: {},
  };
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

  try {
    await page.goto(fixtureUrl, { waitUntil: "networkidle" });
    await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
    const initialMetrics = JSON.parse(await page.locator("#knowledge-workbench-visual-metrics").textContent());
    const root = await openEmptyCanvasSurface(page);
    contextReport.evidence = await action(page, root, contextReport);
    const metrics = await collectMetrics(page, `${name}-final`);
    const overflow = horizontalOverflowState(metrics);
    const callNames = Object.keys(metrics.mock.callsByCommand);
    contextReport.metrics = metrics;
    contextReport.audit = {
      localStorageEmptyBeforeMount: initialMetrics.fixture.localStorageEmptyBeforeMount,
      localStorage: metrics.localStorage,
      readAllowlist: metrics.mock.allowedReadCommands,
      callsByCommand: metrics.mock.callsByCommand,
      writeCallCount: metrics.mock.writeCallCount,
      unrecognizedCallCount: metrics.mock.unrecognizedCallCount,
      externalRequestCount: externalRequests.length,
      consoleErrorCount: consoleErrors.length,
      pageErrorCount: pageErrors.length,
      externalRequests,
      consoleErrors,
      pageErrors,
      overflow,
    };
    check(contextReport, `${name}: localStorage is empty before mount`, (
      initialMetrics.fixture.localStorageEmptyBeforeMount === true
    ), initialMetrics.fixture);
    check(contextReport, `${name}: exact read allowlist and observed calls stay read-only`, (
      JSON.stringify(metrics.mock.allowedReadCommands) === JSON.stringify(expectedReadAllowlist)
      && callNames.every((command) => expectedReadAllowlist.includes(command))
    ), metrics.mock);
    check(contextReport, `${name}: write unknown external console and page-error counts are all zero`, (
      metrics.mock.writeCallCount === 0
      && metrics.mock.unrecognizedCallCount === 0
      && externalRequests.length === 0
      && consoleErrors.length === 0
      && pageErrors.length === 0
    ), contextReport.audit);
    check(contextReport, `${name}: localStorage contains only disposable R3B chrome preference`, (
      metrics.localStorage.keys.length <= 1 && storageContainsChromeOnly(metrics.localStorage)
    ), metrics.localStorage);
    check(contextReport, `${name}: document body shell group and Canvas have no horizontal overflow`, (
      overflow.ok
    ), overflow.values);
    check(contextReport, `${name}: frozen R3C Canvas geometry and internal tools remain intact`, (
      metrics.canvas.rootCount === 1
      && metrics.canvas.chrome.count === 1
      && metrics.canvas.stage.heightRatio >= 0.75
      && metrics.canvas.floatingTools.count === 1
      && metrics.canvas.floatingTools.withinStage === true
    ), metrics.canvas);
  } catch (error) {
    contextReport.inspectionError = String(error?.stack ?? error);
    check(contextReport, `${name}: browser evidence completed without inspection error`, false, {
      error: contextReport.inspectionError,
    });
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
  await withFreshContext(browser, {
    name: "01-1180-chrome-opener-cancel-paths",
    viewport: { width: 1180, height: 760 },
    action: async (page, root, contextReport) => ({
      rounds: [
        await cancelRound(page, root, contextReport, {
          label: "chrome Escape",
          openerKind: "chrome",
          action: "escape",
        }),
        await cancelRound(page, root, contextReport, {
          label: "chrome explicit close",
          openerKind: "chrome",
          action: "explicit-close",
        }),
        await cancelRound(page, root, contextReport, {
          label: "chrome toggle close",
          openerKind: "chrome",
          action: "opener-toggle",
        }),
      ],
    }),
  });

  await withFreshContext(browser, {
    name: "02-1180-empty-opener-cancel-paths",
    viewport: { width: 1180, height: 760 },
    action: async (page, root, contextReport) => ({
      rounds: [
        await cancelRound(page, root, contextReport, {
          label: "empty Escape",
          openerKind: "empty",
          action: "escape",
          openScreenshot: "01-1180-empty-opener-panel-open.png",
          closedScreenshot: "02-1180-empty-opener-escape-focus-return.png",
        }),
        await cancelRound(page, root, contextReport, {
          label: "empty explicit close",
          openerKind: "empty",
          action: "explicit-close",
        }),
      ],
    }),
  });

  await withFreshContext(browser, {
    name: "03-1180-chrome-opener-selection",
    viewport: { width: 1180, height: 760 },
    action: async (page, root, contextReport) => selectionRound(page, root, contextReport, {
      label: "chrome selection",
      openerKind: "chrome",
    }),
  });

  await withFreshContext(browser, {
    name: "04-1180-empty-opener-selection",
    viewport: { width: 1180, height: 760 },
    action: async (page, root, contextReport) => selectionRound(page, root, contextReport, {
      label: "empty selection",
      openerKind: "empty",
      screenshot: "03-1180-empty-opener-selection-stage-focus.png",
    }),
  });

  await withFreshContext(browser, {
    name: "05-900-reduced-motion-empty-opener-escape",
    viewport: { width: 900, height: 760 },
    reducedMotion: "reduce",
    action: async (page, root, contextReport) => {
      const reducedMotionMatches = await page.evaluate(() => (
        window.matchMedia("(prefers-reduced-motion: reduce)").matches
      ));
      check(contextReport, "900 reduced-motion media query is active", reducedMotionMatches, {
        reducedMotionMatches,
      });
      return cancelRound(page, root, contextReport, {
        label: "900 reduced-motion empty Escape",
        openerKind: "empty",
        action: "escape",
      });
    },
  });
} finally {
  await browser.close();
}

report.outcome = report.failed === 0 ? "PASS" : "FAIL";
await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({
  outcome: report.outcome,
  contexts: report.contexts.length,
  assertions: report.assertions,
  failed: report.failed,
  failures: report.failures.map((failure) => `${failure.context}: ${failure.name}`),
}, null, 2));
process.exitCode = report.failed === 0 ? 0 : 1;
