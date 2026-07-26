import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./red-browser-evidence.json", rawDirectory));
const screenshotName = "red-1440-card-array.png";
const preferenceKey = "syn-native-knowledge-workspace-ui-v1";
const taskReadAllowlist = [
  "knowledge_workspace_graph",
  "knowledge_workspace_read_markdown",
  "knowledge_workspace_snapshot",
];
const report = {
  phase: "pre-implementation-red-first",
  fixture: "synthetic-only",
  outcome: "PENDING",
  assertions: 0,
  failed: 0,
  failures: [],
  contexts: [],
  screenshot: screenshotName,
  taskReadAllowlist,
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
    && !serialized.includes("graphPosition")
    && !serialized.includes("selectedNode");
}

async function collectFixtureMetrics(page, scenario) {
  await page.evaluate((nextScenario) => {
    document.documentElement.dataset.fixtureScenario = nextScenario;
    window.dispatchEvent(new Event("n2r-r3c-capture"));
  }, scenario);
  await page.waitForTimeout(180);
  return JSON.parse(await page.locator("#knowledge-workbench-visual-metrics").textContent());
}

async function openGraph(page) {
  await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
  const root = page.locator('[data-active-group="true"] .native-knowledge-graph');
  await root.waitFor();
  await root.locator(".react-flow__node").first().waitFor();
  return root;
}

async function graphVisualState(root) {
  return root.evaluate((graphRoot) => {
    const bounds = (element) => {
      if (!(element instanceof HTMLElement || element instanceof SVGElement)) return null;
      const rect = element.getBoundingClientRect();
      return {
        x: Math.round(rect.x * 1000) / 1000,
        y: Math.round(rect.y * 1000) / 1000,
        width: Math.round(rect.width * 1000) / 1000,
        height: Math.round(rect.height * 1000) / 1000,
      };
    };
    const isVisible = (element) => {
      if (!(element instanceof HTMLElement || element instanceof SVGElement)) return false;
      const style = getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
    };
    const rootBounds = bounds(graphRoot);
    const flow = graphRoot.querySelector(".react-flow");
    const flowBounds = bounds(flow);
    const nodes = [...graphRoot.querySelectorAll(".react-flow__node")].map((wrapper) => {
      const card = wrapper.querySelector(".native-graph-node");
      const style = card ? getComputedStyle(card) : null;
      const path = card?.querySelector(":scope > span") ?? null;
      const tag = card?.querySelector(":scope > em") ?? null;
      const isolated = card?.querySelector(":scope > small") ?? null;
      return {
        id: wrapper.getAttribute("data-id"),
        wrapperRole: wrapper.getAttribute("role"),
        wrapperAriaLabel: wrapper.getAttribute("aria-label"),
        wrapperBounds: bounds(wrapper),
        cardBounds: bounds(card),
        title: card?.querySelector("strong")?.textContent?.trim() ?? null,
        visiblePath: path && isVisible(path) ? path.textContent?.trim() ?? "" : null,
        visibleTag: tag && isVisible(tag) ? tag.textContent?.trim() ?? "" : null,
        visibleIsolated: isolated && isVisible(isolated) ? isolated.textContent?.trim() ?? "" : null,
        boxShadow: style?.boxShadow ?? null,
        borderLeftWidth: style?.borderLeftWidth ?? null,
        borderLeftColor: style?.borderLeftColor ?? null,
        backgroundColor: style?.backgroundColor ?? null,
      };
    });
    const rootStyle = getComputedStyle(graphRoot);
    const persistentFilters = [...graphRoot.querySelectorAll(
      ".native-graph-toolbar input, .native-graph-toolbar select",
    )].filter(isVisible);
    return {
      rootBounds,
      rootStyle: {
        borderTopWidth: rootStyle.borderTopWidth,
        borderLeftWidth: rootStyle.borderLeftWidth,
        borderRadius: rootStyle.borderRadius,
        boxShadow: rootStyle.boxShadow,
        marginTop: rootStyle.marginTop,
        marginBottom: rootStyle.marginBottom,
      },
      repeatedHeadingCount: graphRoot.querySelectorAll(".native-graph-head h2").length,
      repeatedExplanationCount: graphRoot.querySelectorAll(".native-graph-head > p").length,
      persistentFilterControlCount: persistentFilters.length,
      filterDisclosureOpenerCount: graphRoot.querySelectorAll("[data-graph-filter-opener]").length,
      filterDisclosureCount: graphRoot.querySelectorAll("[data-graph-filter-panel]").length,
      chromeBounds: bounds(graphRoot.querySelector(".native-graph-toolbar")),
      flowBounds,
      flowHeightRatio: rootBounds && flowBounds && rootBounds.height > 0
        ? flowBounds.height / rootBounds.height
        : null,
      nodeCount: nodes.length,
      edgeCount: graphRoot.querySelectorAll(".react-flow__edge").length,
      nodes,
      maxNodeWidth: Math.max(0, ...nodes.map((node) => node.cardBounds?.width ?? 0)),
      maxNodeHeight: Math.max(0, ...nodes.map((node) => node.cardBounds?.height ?? 0)),
      visiblePathCount: nodes.filter((node) => node.visiblePath).length,
      visibleTagOrIsolatedCount: nodes.filter((node) => node.visibleTag || node.visibleIsolated).length,
      cardShadowCount: nodes.filter((node) => node.boxShadow && node.boxShadow !== "none").length,
      coarseLeftBorderCount: nodes.filter((node) => Number.parseFloat(node.borderLeftWidth ?? "0") >= 2).length,
    };
  });
}

async function tabUntil(page, selector, limit = 140) {
  for (let index = 0; index < limit; index += 1) {
    await page.keyboard.press("Tab");
    const matched = await page.evaluate((targetSelector) => document.activeElement?.matches(targetSelector) ?? false, selector);
    if (matched) return { reached: true, tabs: index + 1 };
  }
  return { reached: false, tabs: limit };
}

async function activeNodeState(page) {
  return page.evaluate(() => {
    const active = document.activeElement;
    const wrapper = active instanceof HTMLElement && active.matches(".react-flow__node") ? active : null;
    const card = wrapper?.querySelector(".native-graph-node") ?? null;
    const wrapperStyle = wrapper ? getComputedStyle(wrapper) : null;
    const cardStyle = card ? getComputedStyle(card) : null;
    const activeGroup = active?.closest('[data-knowledge-group-panel="active"]');
    return {
      activeTag: active?.tagName.toLowerCase() ?? null,
      role: active?.getAttribute("role") ?? null,
      ariaLabel: active?.getAttribute("aria-label") ?? null,
      dataId: wrapper?.dataset.id ?? null,
      selected: wrapper?.classList.contains("selected") ?? false,
      outlineStyle: wrapperStyle?.outlineStyle ?? null,
      outlineWidth: wrapperStyle?.outlineWidth ?? null,
      cardBackground: cardStyle?.backgroundColor ?? null,
      cardBorderColor: cardStyle?.borderColor ?? null,
      cardBoxShadow: cardStyle?.boxShadow ?? null,
      activeGroupScrollTop: activeGroup?.scrollTop ?? null,
      bodyScrollTop: document.body.scrollTop,
      documentScrollTop: document.documentElement.scrollTop,
    };
  });
}

async function installInvokeObserver(page) {
  await page.evaluate(() => {
    const internals = window.__TAURI_INTERNALS__;
    const originalInvoke = internals.invoke;
    window.__n2rR3dObservedInvokes = [];
    internals.invoke = async (command, payload, options) => {
      window.__n2rR3dObservedInvokes.push({ command, payload });
      return originalInvoke(command, payload, options);
    };
  });
}

async function observedInvokes(page) {
  return page.evaluate(() => window.__n2rR3dObservedInvokes ?? []);
}

function overflowAudit(metrics, graphState = null) {
  const values = {
    documentElement: metrics.overflow.documentElement,
    body: metrics.overflow.body,
    shell: metrics.overflow.shell,
    activeGroupPanel: metrics.overflow.activeGroupPanel,
  };
  return {
    values,
    graph: graphState?.rootBounds ?? null,
    ok: Object.values(values).every((value) => value && value.scrollWidth <= value.clientWidth),
  };
}

async function withFreshContext(browser, { name, viewport, action }) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = { name, viewport, assertions: [], evidence: {}, audit: {} };
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
    check(contextReport, `${name}: mount starts with empty localStorage`, initialMetrics.fixture.localStorageEmptyBeforeMount === true, {
      observed: initialMetrics.fixture.localStorageEmptyBeforeMount,
    });
    contextReport.evidence = await action(page, contextReport);
    const metrics = await collectFixtureMetrics(page, `${name}-final`);
    const commands = Object.keys(metrics.mock.callsByCommand);
    const overflow = overflowAudit(metrics, contextReport.evidence.graphState ?? null);
    contextReport.audit = {
      taskReadAllowlist,
      fixtureGlobalReadAllowlist: metrics.mock.allowedReadCommands,
      callsByCommand: metrics.mock.callsByCommand,
      callsStayInsideTaskAllowlist: commands.every((command) => taskReadAllowlist.includes(command)),
      writeCallCount: metrics.mock.writeCallCount,
      unrecognizedCallCount: metrics.mock.unrecognizedCallCount,
      externalRequestCount: externalRequests.length,
      consoleErrorCount: consoleErrors.length,
      pageErrorCount: pageErrors.length,
      localStorage: metrics.localStorage,
      overflow,
      externalRequests,
      consoleErrors,
      pageErrors,
    };
    check(contextReport, `${name}: observed commands stay inside exact R3D read allowlist`, (
      contextReport.audit.callsStayInsideTaskAllowlist
    ), contextReport.audit);
    check(contextReport, `${name}: write unknown external console and page-error counts are zero`, (
      metrics.mock.writeCallCount === 0
      && metrics.mock.unrecognizedCallCount === 0
      && externalRequests.length === 0
      && consoleErrors.length === 0
      && pageErrors.length === 0
    ), contextReport.audit);
    check(contextReport, `${name}: localStorage contains only disposable workbench chrome`, (
      storageContainsChromeOnly(metrics.localStorage)
    ), metrics.localStorage);
    check(contextReport, `${name}: shell and active Graph group have no horizontal overflow`, overflow.ok, overflow);
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
    name: "01-1440-current-card-array",
    viewport: { width: 1440, height: 900 },
    action: async (page, contextReport) => {
      const root = await openGraph(page);
      const graphState = await graphVisualState(root);
      check(contextReport, "Graph does not repeat a large internal heading or explanation", (
        graphState.repeatedHeadingCount === 0 && graphState.repeatedExplanationCount === 0
      ), graphState);
      check(contextReport, "query tag and local-focus controls are absent from the default surface", (
        graphState.persistentFilterControlCount === 0
      ), graphState);
      check(contextReport, "default compact chrome is no taller than 44px", (
        (graphState.chromeBounds?.height ?? Number.POSITIVE_INFINITY) <= 44
      ), graphState.chromeBounds);
      check(contextReport, "ReactFlow owns at least 74 percent of the Graph root", (
        (graphState.flowHeightRatio ?? 0) >= 0.74
      ), graphState);
      check(contextReport, "all Graph nodes are at most 144px wide", graphState.maxNodeWidth <= 144, graphState);
      check(contextReport, "all Graph nodes are at most 48px high", graphState.maxNodeHeight <= 48, graphState);
      check(contextReport, "vault-relative paths are not persistent visible node metadata", (
        graphState.visiblePathCount === 0
      ), graphState.nodes);
      check(contextReport, "tags and isolated-note words are not persistent visible node metadata", (
        graphState.visibleTagOrIsolatedCount === 0
      ), graphState.nodes);
      check(contextReport, "Graph nodes have no card shadow or coarse left accent border", (
        graphState.cardShadowCount === 0 && graphState.coarseLeftBorderCount === 0
      ), graphState.nodes);
      check(contextReport, "DOM node and edge counts match the frozen synthetic projection", (
        graphState.nodeCount === 6 && graphState.edgeCount === 5
      ), graphState);
      check(contextReport, "a filter disclosure with an actual opener exists", (
        graphState.filterDisclosureOpenerCount === 1
      ), graphState);
      await page.screenshot({
        path: fileURLToPath(new URL(`./${screenshotName}`, rawDirectory)),
        fullPage: false,
      });
      return { graphState };
    },
  });

  await withFreshContext(browser, {
    name: "02-1180-current-keyboard-and-selection",
    viewport: { width: 1180, height: 760 },
    action: async (page, contextReport) => {
      const root = await openGraph(page);
      await installInvokeObserver(page);
      const tabResult = await tabUntil(page, ".react-flow__node");
      const before = await activeNodeState(page);
      check(contextReport, "Tab naturally reaches a Graph node", tabResult.reached, { tabResult, before });
      check(contextReport, "focused Graph node has button or link equivalent semantics", (
        before.role === "button" || before.role === "link"
      ), before);
      check(contextReport, "keyboard-focused Graph node has a visible focus indicator", (
        before.outlineStyle !== "none" && Number.parseFloat(before.outlineWidth ?? "0") >= 1
      ), before);

      const beforeInvokes = await observedInvokes(page);
      await page.keyboard.press("Enter");
      await page.waitForTimeout(180);
      const afterEnter = await activeNodeState(page);
      const enterInvokes = await observedInvokes(page);
      const enterReads = enterInvokes.slice(beforeInvokes.length).filter(
        (entry) => entry.command === "knowledge_workspace_read_markdown",
      );
      check(contextReport, "selected style changes on the real selected ReactFlow wrapper", (
        afterEnter.selected
        && (
          afterEnter.cardBackground !== before.cardBackground
          || afterEnter.cardBorderColor !== before.cardBorderColor
          || afterEnter.cardBoxShadow !== before.cardBoxShadow
        )
      ), { before, afterEnter });
      check(contextReport, "Enter produces exactly one typed Markdown open", enterReads.length === 1, {
        beforeInvokes,
        enterInvokes,
      });

      const scrollBeforeSpace = await activeNodeState(page);
      await page.keyboard.press("Space");
      await page.waitForTimeout(180);
      const afterSpace = await activeNodeState(page);
      const spaceInvokes = await observedInvokes(page);
      const spaceReads = spaceInvokes.slice(enterInvokes.length).filter(
        (entry) => entry.command === "knowledge_workspace_read_markdown",
      );
      check(contextReport, "Space produces exactly one typed Markdown open", spaceReads.length === 1, {
        enterInvokes,
        spaceInvokes,
      });
      check(contextReport, "Space does not scroll the page or active group", (
        afterSpace.bodyScrollTop === scrollBeforeSpace.bodyScrollTop
        && afterSpace.documentScrollTop === scrollBeforeSpace.documentScrollTop
        && afterSpace.activeGroupScrollTop === scrollBeforeSpace.activeGroupScrollTop
      ), { scrollBeforeSpace, afterSpace });
      return {
        graphState: await graphVisualState(root),
        tabResult,
        before,
        afterEnter,
        afterSpace,
        observedInvokes: spaceInvokes,
      };
    },
  });

  await withFreshContext(browser, {
    name: "03-1180-current-click-handoff",
    viewport: { width: 1180, height: 760 },
    action: async (page, contextReport) => {
      const root = await openGraph(page);
      await installInvokeObserver(page);
      const node = root.locator(".react-flow__node").first();
      const nodeState = await node.evaluate((wrapper) => ({
        id: wrapper.getAttribute("data-id"),
        title: wrapper.querySelector(".native-graph-node strong")?.textContent?.trim() ?? null,
        relativePath: wrapper.querySelector(".native-graph-node > span")?.textContent?.trim() ?? null,
      }));
      await node.click();
      await root.waitFor({ state: "detached" });
      await page.waitForTimeout(180);
      const invokes = await observedInvokes(page);
      const reads = invokes.filter((entry) => entry.command === "knowledge_workspace_read_markdown");
      check(contextReport, "mouse click still performs exactly one typed Markdown open", reads.length === 1, {
        nodeState,
        invokes,
      });
      check(contextReport, "click handoff uses the projected relative path payload", (
        reads[0]?.payload?.relativePath === nodeState.relativePath
      ), { nodeState, reads });
      return { nodeState, observedInvokes: invokes };
    },
  });
} finally {
  await browser.close();
}

report.outcome = report.failed > 0 ? "EXPECTED_RED" : "UNEXPECTED_GREEN";
await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({
  outcome: report.outcome,
  contexts: report.contexts.length,
  assertions: report.assertions,
  failed: report.failed,
  failures: report.failures.map((failure) => `${failure.context}: ${failure.name}`),
}, null, 2));
process.exitCode = report.outcome === "EXPECTED_RED" ? 0 : 1;
