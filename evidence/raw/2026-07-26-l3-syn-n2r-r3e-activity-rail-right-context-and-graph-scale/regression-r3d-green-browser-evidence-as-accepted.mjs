import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./green-browser-evidence.json", rawDirectory));
const preferenceKey = "syn-native-knowledge-workspace-ui-v1";
const taskReadAllowlist = [
  "knowledge_workspace_graph",
  "knowledge_workspace_read_markdown",
  "knowledge_workspace_snapshot",
];
const screenshotNames = [
  "01-1440-global-graph.png",
  "02-1180-filter-disclosure.png",
  "03-1180-local-graph.png",
  "04-900-keyboard-focus.png",
];
const activationEvidence = [];
const report = {
  phase: "post-implementation-green",
  fixture: "synthetic-only",
  fixtureGraphResponseContract: "frozen global projection for every graph request",
  outcome: "PENDING",
  assertions: 0,
  failed: 0,
  failures: [],
  contexts: [],
  screenshots: screenshotNames,
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
    && !serialized.includes("selectedNode")
    && !serialized.includes("nodePosition")
    && !serialized.includes("graphSelection");
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
  await root.locator(".native-graph-node-button").first().waitFor();
  await page.waitForTimeout(180);
  return root;
}

async function graphVisualState(root) {
  return root.evaluate((graphRoot) => {
    const rounded = (value) => Math.round(value * 1000) / 1000;
    const bounds = (element) => {
      if (!(element instanceof HTMLElement || element instanceof SVGElement)) return null;
      const rect = element.getBoundingClientRect();
      return {
        x: rounded(rect.x),
        y: rounded(rect.y),
        right: rounded(rect.right),
        bottom: rounded(rect.bottom),
        width: rounded(rect.width),
        height: rounded(rect.height),
      };
    };
    const styleValue = (element, property) => {
      if (!(element instanceof HTMLElement || element instanceof SVGElement)) return null;
      return getComputedStyle(element)[property] ?? null;
    };
    const isVisible = (element) => {
      if (!(element instanceof HTMLElement || element instanceof SVGElement)) return false;
      const style = getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return style.display !== "none"
        && style.visibility !== "hidden"
        && style.opacity !== "0"
        && rect.width > 1
        && rect.height > 1
        && style.clip !== "rect(0px, 0px, 0px, 0px)";
    };
    const fitsInside = (inner, outer, tolerance = 2) => Boolean(
      inner
      && outer
      && inner.x >= outer.x - tolerance
      && inner.y >= outer.y - tolerance
      && inner.right <= outer.right + tolerance
      && inner.bottom <= outer.bottom + tolerance
    );

    const rootBounds = bounds(graphRoot);
    const chrome = graphRoot.querySelector(".native-graph-toolbar");
    const chromeBounds = bounds(chrome);
    const flow = graphRoot.querySelector(".react-flow");
    const flowBounds = bounds(flow);
    const viewport = graphRoot.querySelector(".react-flow__viewport");
    const viewportBounds = bounds(viewport);
    const disclosure = graphRoot.querySelector("[data-graph-filter-panel]");
    const disclosureBounds = bounds(disclosure);
    const disclosureControls = disclosure
      ? [...disclosure.querySelectorAll("input, select, button")].filter(isVisible).map((control) => ({
          tag: control.tagName.toLowerCase(),
          label: control.getAttribute("aria-label") ?? control.textContent?.trim() ?? "",
          bounds: bounds(control),
        }))
      : [];
    const rootStyle = getComputedStyle(graphRoot);
    const nodes = [...graphRoot.querySelectorAll(".react-flow__node")].map((wrapper) => {
      const node = wrapper.querySelector(".native-graph-node");
      const button = wrapper.querySelector(".native-graph-node-button");
      const title = button?.querySelector("strong");
      const nodeStyle = node ? getComputedStyle(node) : null;
      const buttonStyle = button ? getComputedStyle(button) : null;
      const titleStyle = title ? getComputedStyle(title) : null;
      const nodeBounds = bounds(node);
      const buttonBounds = bounds(button);
      const visibleMetadata = node
        ? [...node.querySelectorAll(":scope > span, :scope > em, :scope > small")].filter(isVisible).map(
            (element) => element.textContent?.trim() ?? "",
          )
        : [];
      return {
        id: wrapper.getAttribute("data-id"),
        selected: wrapper.classList.contains("selected"),
        wrapperBounds: bounds(wrapper),
        nodeBounds,
        buttonBounds,
        buttonTag: button?.tagName.toLowerCase() ?? null,
        buttonType: button?.getAttribute("type") ?? null,
        ariaLabel: button?.getAttribute("aria-label") ?? null,
        ariaCurrent: button?.getAttribute("aria-current") ?? null,
        title: title?.textContent?.trim() ?? null,
        buttonVisibleText: button?.textContent?.trim() ?? null,
        visibleMetadata,
        nodeBoxShadow: nodeStyle?.boxShadow ?? null,
        nodeBorderLeftWidth: nodeStyle?.borderLeftWidth ?? null,
        nodeBackground: nodeStyle?.backgroundColor ?? null,
        buttonBoxShadow: buttonStyle?.boxShadow ?? null,
        buttonBorderLeftWidth: buttonStyle?.borderLeftWidth ?? null,
        buttonBorderColor: buttonStyle?.borderColor ?? null,
        buttonBackground: buttonStyle?.backgroundColor ?? null,
        titleFontSize: titleStyle?.fontSize ?? null,
        titleClipped: title instanceof HTMLElement ? title.scrollWidth > title.clientWidth + 1 : null,
        insideFlow: fitsInside(buttonBounds, flowBounds),
      };
    });
    const edges = [...graphRoot.querySelectorAll(".react-flow__edge")].map((edge) => {
      const path = edge.querySelector(".react-flow__edge-path");
      const style = path ? getComputedStyle(path) : null;
      return {
        id: edge.getAttribute("data-id"),
        stroke: style?.stroke ?? null,
        strokeWidth: style?.strokeWidth ?? null,
        opacity: style?.opacity ?? null,
        pathLength: path instanceof SVGGeometryElement ? rounded(path.getTotalLength()) : null,
      };
    });
    const persistentFilters = [...graphRoot.querySelectorAll(
      ".native-graph-toolbar input, .native-graph-toolbar select",
    )].filter(isVisible);
    const opener = graphRoot.querySelector("[data-graph-filter-opener]");
    const surface = graphRoot.querySelector(".native-graph-surface");
    const activeGroup = graphRoot.closest('[data-knowledge-group-panel="active"]')
      ?? graphRoot.closest('[data-active-group="true"]');
    const overflowEntry = (element) => {
      if (!(element instanceof HTMLElement)) return null;
      return { clientWidth: element.clientWidth, scrollWidth: element.scrollWidth };
    };
    return {
      rootBounds,
      rootStyle: {
        borderTopWidth: rootStyle.borderTopWidth,
        borderLeftWidth: rootStyle.borderLeftWidth,
        borderRadius: rootStyle.borderRadius,
        boxShadow: rootStyle.boxShadow,
        marginTop: rootStyle.marginTop,
        marginBottom: rootStyle.marginBottom,
        backgroundColor: rootStyle.backgroundColor,
      },
      repeatedHeadingCount: graphRoot.querySelectorAll(".native-graph-head h2, h2").length,
      repeatedExplanationCount: graphRoot.querySelectorAll(".native-graph-head > p").length,
      persistentFilterControlCount: persistentFilters.length,
      filterDisclosureOpenerCount: graphRoot.querySelectorAll("[data-graph-filter-opener]").length,
      filterDisclosureCount: graphRoot.querySelectorAll("[data-graph-filter-panel]").length,
      openerAriaExpanded: opener?.getAttribute("aria-expanded") ?? null,
      openerAriaControls: opener?.getAttribute("aria-controls") ?? null,
      chromeBounds,
      flowBounds,
      viewportBounds,
      flowHeightRatio: rootBounds && flowBounds && rootBounds.height > 0
        ? flowBounds.height / rootBounds.height
        : null,
      disclosureBounds,
      disclosureControls,
      disclosureInsideRoot: disclosure ? fitsInside(disclosureBounds, rootBounds, 1) : null,
      nodeCount: nodes.length,
      edgeCount: edges.length,
      nodes,
      edges,
      maxNodeWidth: Math.max(0, ...nodes.map((node) => node.nodeBounds?.width ?? 0)),
      maxNodeHeight: Math.max(0, ...nodes.map((node) => node.nodeBounds?.height ?? 0)),
      visibleMetadataCount: nodes.reduce((count, node) => count + node.visibleMetadata.length, 0),
      shadowCount: nodes.filter((node) => (
        (node.nodeBoxShadow && node.nodeBoxShadow !== "none")
        || (node.buttonBoxShadow && node.buttonBoxShadow !== "none")
      )).length,
      coarseLeftBorderCount: nodes.filter((node) => (
        Number.parseFloat(node.nodeBorderLeftWidth ?? "0") >= 2
        || Number.parseFloat(node.buttonBorderLeftWidth ?? "0") >= 2
      )).length,
      allNodesInsideFlow: nodes.every((node) => node.insideFlow),
      allEdgesVisible: edges.every((edge) => (
        edge.pathLength !== null
        && edge.pathLength > 0
        && edge.stroke !== "none"
        && edge.stroke !== "transparent"
        && Number.parseFloat(edge.opacity ?? "0") >= 0.4
      )),
      overflow: {
        root: overflowEntry(graphRoot),
        surface: overflowEntry(surface),
        flow: overflowEntry(flow),
        activeGroup: overflowEntry(activeGroup),
      },
    };
  });
}

async function activeElementState(page) {
  return page.evaluate(() => {
    const active = document.activeElement;
    const button = active instanceof HTMLElement && active.matches(".native-graph-node-button") ? active : null;
    const wrapper = button?.closest(".react-flow__node") ?? null;
    const style = button ? getComputedStyle(button) : null;
    const wrapperStyle = wrapper ? getComputedStyle(wrapper) : null;
    return {
      tag: active?.tagName.toLowerCase() ?? null,
      ariaLabel: active?.getAttribute("aria-label") ?? null,
      type: active?.getAttribute("type") ?? null,
      dataGraphNodeAction: active?.hasAttribute("data-graph-node-action") ?? false,
      nodeId: wrapper?.getAttribute("data-id") ?? null,
      wrapperSelected: wrapper?.classList.contains("selected") ?? false,
      outlineStyle: style?.outlineStyle ?? null,
      outlineWidth: style?.outlineWidth ?? null,
      outlineColor: style?.outlineColor ?? null,
      buttonBackground: style?.backgroundColor ?? null,
      buttonBorderColor: style?.borderColor ?? null,
      wrapperBackground: wrapperStyle?.backgroundColor ?? null,
      bodyScrollTop: document.body.scrollTop,
      documentScrollTop: document.documentElement.scrollTop,
    };
  });
}

async function tabUntil(page, selector, limit = 160) {
  const path = [];
  for (let index = 0; index < limit; index += 1) {
    await page.keyboard.press("Tab");
    const state = await page.evaluate((targetSelector) => {
      const active = document.activeElement;
      return {
        matched: active?.matches(targetSelector) ?? false,
        tag: active?.tagName.toLowerCase() ?? null,
        ariaLabel: active?.getAttribute("aria-label") ?? null,
        text: active?.textContent?.trim().slice(0, 80) ?? null,
      };
    }, selector);
    path.push(state);
    if (state.matched) return { reached: true, tabs: index + 1, path };
  }
  return { reached: false, tabs: limit, path };
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
  const fixtureValues = {
    documentElement: metrics.overflow.documentElement,
    body: metrics.overflow.body,
    shell: metrics.overflow.shell,
    activeGroupPanel: metrics.overflow.activeGroupPanel,
  };
  const graphValues = graphState?.overflow ?? {};
  const graphLayers = {
    root: graphValues.root,
    surface: graphValues.surface,
    activeGroup: graphValues.activeGroup,
  };
  const allEntries = [...Object.values(fixtureValues), ...Object.values(graphLayers)].filter(Boolean);
  return {
    fixtureValues,
    graphValues,
    note: "ReactFlow's internal transformed canvas scrollWidth is recorded but is not a document, shell, active-group, or Graph-root overflow layer",
    ok: allEntries.every((value) => value.scrollWidth <= value.clientWidth),
  };
}

async function withFreshContext(browser, {
  name,
  viewport,
  reducedMotion = "no-preference",
  action,
}) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1, reducedMotion });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = { name, viewport, reducedMotion, assertions: [], evidence: {}, audit: {} };
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
      prohibitedSearchOrCanvasCalls: (
        (metrics.mock.callsByCommand.knowledge_workspace_search ?? 0)
        + (metrics.mock.callsByCommand.knowledge_workspace_read_canvas ?? 0)
      ),
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
      && contextReport.audit.prohibitedSearchOrCanvasCalls === 0
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
    check(contextReport, `${name}: document shell active group and Graph have no horizontal overflow`, overflow.ok, overflow);
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
    name: "01-1440-global-graph",
    viewport: { width: 1440, height: 900 },
    action: async (page, contextReport) => {
      const root = await openGraph(page);
      const graphState = await graphVisualState(root);
      check(contextReport, "Graph root directly fills the active group without card chrome", (
        graphState.rootStyle.borderTopWidth === "0px"
        && graphState.rootStyle.borderLeftWidth === "0px"
        && graphState.rootStyle.borderRadius === "0px"
        && graphState.rootStyle.boxShadow === "none"
        && graphState.rootStyle.marginTop === "0px"
        && graphState.rootStyle.marginBottom === "0px"
      ), graphState);
      check(contextReport, "Graph has no repeated internal h2 or explanation", (
        graphState.repeatedHeadingCount === 0 && graphState.repeatedExplanationCount === 0
      ), graphState);
      check(contextReport, "default chrome is compact and detailed filters are closed", (
        (graphState.chromeBounds?.height ?? Number.POSITIVE_INFINITY) <= 44
        && graphState.persistentFilterControlCount === 0
        && graphState.filterDisclosureOpenerCount === 1
        && graphState.filterDisclosureCount === 0
        && graphState.openerAriaExpanded === "false"
      ), graphState);
      check(contextReport, "1440 ReactFlow owns at least 74 percent of the Graph root", (
        (graphState.flowHeightRatio ?? 0) >= 0.74
      ), graphState);
      check(contextReport, "all projected nodes are lightweight 144 by 48 or smaller", (
        graphState.maxNodeWidth <= 144 && graphState.maxNodeHeight <= 48
      ), graphState);
      check(contextReport, "nodes keep only the readable title visible", (
        graphState.visibleMetadataCount === 0
        && graphState.nodes.every((node) => (
          node.buttonTag === "button"
          && node.buttonType === "button"
          && node.title === node.buttonVisibleText
          && node.ariaLabel?.includes(node.title)
          && node.ariaLabel?.includes(node.id)
        ))
      ), graphState.nodes);
      check(contextReport, "nodes have no card shadow coarse left accent or clipped title", (
        graphState.shadowCount === 0
        && graphState.coarseLeftBorderCount === 0
        && graphState.nodes.every((node) => node.titleClipped === false)
      ), graphState.nodes);
      check(contextReport, "all six nodes and five visible edges match and fit the returned projection", (
        graphState.nodeCount === 6
        && graphState.edgeCount === 5
        && graphState.allNodesInsideFlow
        && graphState.allEdgesVisible
      ), graphState);
      await page.screenshot({
        path: fileURLToPath(new URL("./01-1440-global-graph.png", rawDirectory)),
        fullPage: false,
      });
      return { graphState };
    },
  });

  await withFreshContext(browser, {
    name: "02-1180-filter-disclosure",
    viewport: { width: 1180, height: 760 },
    action: async (page, contextReport) => {
      const root = await openGraph(page);
      const opener = root.locator("[data-graph-filter-opener]");
      await opener.click();
      const panel = root.locator("[data-graph-filter-panel]");
      await panel.waitFor();
      await page.waitForTimeout(80);
      const openFocus = await activeElementState(page);
      const openState = await graphVisualState(root);
      check(contextReport, "filter opener exposes stable expanded and controls semantics", (
        openState.filterDisclosureCount === 1
        && openState.openerAriaExpanded === "true"
        && Boolean(openState.openerAriaControls)
        && await panel.getAttribute("id") === openState.openerAriaControls
      ), openState);
      check(contextReport, "opening disclosure naturally focuses the first query input", (
        openFocus.tag === "input" && openFocus.ariaLabel === "关系图文字筛选"
      ), openFocus);
      check(contextReport, "query tag local-focus and explicit actions are present on demand", (
        await panel.locator('input[aria-label="关系图文字筛选"]').count() === 1
        && await panel.locator('input[aria-label="关系图标签筛选"]').count() === 1
        && await panel.locator('select[aria-label="关系图局部焦点"]').count() === 1
        && await panel.getByRole("button", { name: "应用" }).count() === 1
        && await panel.getByRole("button", { name: "关闭" }).count() === 1
      ), openState.disclosureControls);
      check(contextReport, "filter disclosure remains fully inside the Graph root", (
        openState.disclosureInsideRoot === true
      ), { root: openState.rootBounds, disclosure: openState.disclosureBounds });
      await page.screenshot({
        path: fileURLToPath(new URL("./02-1180-filter-disclosure.png", rawDirectory)),
        fullPage: false,
      });

      await page.keyboard.press("Escape");
      await panel.waitFor({ state: "detached" });
      await page.waitForTimeout(40);
      const escapedToActualOpener = await opener.evaluate((element) => document.activeElement === element);
      check(contextReport, "Escape closes the disclosure and returns to this actual opener", (
        escapedToActualOpener && await opener.getAttribute("aria-expanded") === "false"
      ), { escapedToActualOpener, expanded: await opener.getAttribute("aria-expanded") });

      await opener.click();
      await panel.waitFor();
      await panel.getByRole("button", { name: "关闭" }).click();
      await panel.waitFor({ state: "detached" });
      await page.waitForTimeout(40);
      const closeReturnedToActualOpener = await opener.evaluate((element) => document.activeElement === element);
      check(contextReport, "explicit close returns to the same actual opener", closeReturnedToActualOpener, {
        closeReturnedToActualOpener,
      });
      return { graphState: await graphVisualState(root), openState, openFocus, escapedToActualOpener, closeReturnedToActualOpener };
    },
  });

  await withFreshContext(browser, {
    name: "03-1180-local-graph",
    viewport: { width: 1180, height: 760 },
    action: async (page, contextReport) => {
      const root = await openGraph(page);
      await installInvokeObserver(page);
      const globalButton = root.getByRole("button", { name: "全局", exact: true });
      const localButton = root.getByRole("button", { name: "局部", exact: true });
      await localButton.click();
      check(contextReport, "global and local scope have real pressed state", (
        await localButton.getAttribute("aria-pressed") === "true"
        && await globalButton.getAttribute("aria-pressed") === "false"
      ), {
        local: await localButton.getAttribute("aria-pressed"),
        global: await globalButton.getAttribute("aria-pressed"),
      });

      const opener = root.locator("[data-graph-filter-opener]");
      await opener.click();
      const panel = root.locator("[data-graph-filter-panel]");
      await panel.locator('input[aria-label="关系图文字筛选"]').fill("合成");
      await panel.locator('input[aria-label="关系图标签筛选"]').fill("synthetic");
      await panel.locator('select[aria-label="关系图局部焦点"]').selectOption("notes/visual-baseline.md");
      await panel.getByRole("button", { name: "应用" }).click();
      await panel.waitFor({ state: "detached" });
      await page.waitForFunction(() => (
        (window.__n2rR3dObservedInvokes ?? []).some((entry) => entry.command === "knowledge_workspace_graph")
      ));
      await page.waitForTimeout(120);
      const invokes = await observedInvokes(page);
      const graphInvokes = invokes.filter((entry) => entry.command === "knowledge_workspace_graph");
      const expectedPayload = {
        scope: "local",
        focusRelativePath: "notes/visual-baseline.md",
        query: "合成",
        tag: "synthetic",
      };
      check(contextReport, "local update sends one exact lower-camel graph payload", (
        graphInvokes.length === 1 && JSON.stringify(graphInvokes[0].payload) === JSON.stringify(expectedPayload)
      ), { graphInvokes, expectedPayload });
      const graphState = await graphVisualState(root);
      check(contextReport, "applied local focus is represented without persistent detail controls", (
        await localButton.getAttribute("aria-pressed") === "true"
        && graphState.filterDisclosureCount === 0
        && graphState.persistentFilterControlCount === 0
        && graphState.nodes.filter((node) => node.ariaCurrent === "page").length === 1
      ), graphState);
      check(contextReport, "frozen response remains a complete six-node five-edge projection", (
        graphState.nodeCount === 6 && graphState.edgeCount === 5 && graphState.allNodesInsideFlow
      ), {
        note: "fixture is frozen to return its global projection; the runner does not rewrite or relabel it",
        graphState,
      });
      const returnedFocus = await opener.evaluate((element) => document.activeElement === element);
      check(contextReport, "successful apply closes and returns to the actual filter opener", returnedFocus, {
        returnedFocus,
      });
      await page.screenshot({
        path: fileURLToPath(new URL("./03-1180-local-graph.png", rawDirectory)),
        fullPage: false,
      });
      return {
        graphState,
        graphInvokes,
        expectedPayload,
        frozenFixtureResponseScope: "global",
        selectedUiScope: "local",
        returnedFocus,
      };
    },
  });

  const activationCases = [
    { name: "04a-1180-click-activation", method: "click" },
    { name: "04b-1180-enter-activation", method: "Enter" },
    { name: "04c-1180-space-activation", method: "Space" },
  ];
  for (const activationCase of activationCases) {
    await withFreshContext(browser, {
      name: activationCase.name,
      viewport: { width: 1180, height: 760 },
      action: async (page, contextReport) => {
        const root = await openGraph(page);
        const relativePath = "notes/visual-baseline.md";
        const selector = `[data-graph-relative-path="${relativePath}"] .native-graph-node-button`;
        const target = root.locator(selector);
        const beforeMetrics = await collectFixtureMetrics(page, `${activationCase.name}-before`);
        const beforeReadCount = beforeMetrics.mock.callsByCommand.knowledge_workspace_read_markdown ?? 0;
        const tabResult = activationCase.method === "click" ? null : await tabUntil(page, selector);
        const focusBefore = await activeElementState(page);
        const scrollBefore = await page.evaluate(() => ({
          body: document.body.scrollTop,
          document: document.documentElement.scrollTop,
        }));
        if (activationCase.method === "click") {
          await target.click();
        } else {
          await page.keyboard.press(activationCase.method);
        }
        await root.waitFor({ state: "detached" });
        await page.waitForFunction(({ beforeCount }) => {
          const metricsNode = document.querySelector("#knowledge-workbench-visual-metrics");
          if (!metricsNode?.textContent) return false;
          const metrics = JSON.parse(metricsNode.textContent);
          return (metrics.mock.callsByCommand.knowledge_workspace_read_markdown ?? 0) === beforeCount + 1;
        }, { beforeCount: beforeReadCount });
        const afterMetrics = await collectFixtureMetrics(page, `${activationCase.name}-after`);
        const afterReadCount = afterMetrics.mock.callsByCommand.knowledge_workspace_read_markdown ?? 0;
        const readCountDelta = afterReadCount - beforeReadCount;
        const selectedSourcePath = await page.locator('[aria-label="合成来源上下文"] strong').textContent();
        const scrollAfter = await page.evaluate(() => ({
          body: document.body.scrollTop,
          document: document.documentElement.scrollTop,
        }));
        check(contextReport, `${activationCase.method} activates exactly one typed Markdown read`, (
          readCountDelta === 1 && selectedSourcePath?.trim() === relativePath
        ), { beforeReadCount, afterReadCount, readCountDelta, relativePath, selectedSourcePath });
        if (activationCase.method !== "click") {
          check(contextReport, `${activationCase.method} reaches the node by natural Tab order`, (
            tabResult?.reached === true
            && focusBefore.tag === "button"
            && focusBefore.dataGraphNodeAction
            && focusBefore.ariaLabel?.includes(relativePath)
          ), { tabResult, focusBefore });
        }
        if (activationCase.method === "Space") {
          check(contextReport, "Space activation does not scroll body or document", (
            JSON.stringify(scrollBefore) === JSON.stringify(scrollAfter)
          ), { scrollBefore, scrollAfter });
        }
        const evidence = {
          method: activationCase.method,
          relativePath,
          command: "knowledge_workspace_read_markdown",
          beforeReadCount,
          afterReadCount,
          readCountDelta,
          selectedSourcePath: selectedSourcePath?.trim() ?? null,
          tabResult,
          focusBefore,
          scrollBefore,
          scrollAfter,
        };
        activationEvidence.push(evidence);
        return evidence;
      },
    });
  }

  await withFreshContext(browser, {
    name: "05-900-keyboard-focus-and-selected",
    viewport: { width: 900, height: 760 },
    action: async (page, contextReport) => {
      const root = await openGraph(page);
      const sidebars = await page.evaluate(() => ({
        leftHidden: document.querySelector('[data-knowledge-region="left"]')?.getAttribute("aria-hidden"),
        rightHidden: document.querySelector('[data-knowledge-region="right"]')?.getAttribute("aria-hidden"),
      }));
      const tabResult = await tabUntil(page, ".native-graph-node-button");
      const focused = await activeElementState(page);
      const selectionState = await root.evaluate(() => {
        const selectedWrapper = document.querySelector(
          '[data-active-group="true"] .native-knowledge-graph .react-flow__node.selected',
        );
        const selectedButton = selectedWrapper?.querySelector(".native-graph-node-button");
        const unselectedButton = document.querySelector(
          '[data-active-group="true"] .native-knowledge-graph .react-flow__node:not(.selected) .native-graph-node-button',
        );
        const selectedStyle = selectedButton ? getComputedStyle(selectedButton) : null;
        const unselectedStyle = unselectedButton ? getComputedStyle(unselectedButton) : null;
        return {
          selectedId: selectedWrapper?.getAttribute("data-id") ?? null,
          selectedBackground: selectedStyle?.backgroundColor ?? null,
          selectedBorderColor: selectedStyle?.borderColor ?? null,
          unselectedBackground: unselectedStyle?.backgroundColor ?? null,
          unselectedBorderColor: unselectedStyle?.borderColor ?? null,
        };
      });
      check(contextReport, "900 starts with both sidebars expanded", (
        sidebars.leftHidden === "false" && sidebars.rightHidden === "false"
      ), sidebars);
      check(contextReport, "Tab naturally reaches a native Graph node action", (
        tabResult.reached
        && focused.tag === "button"
        && focused.type === "button"
        && focused.dataGraphNodeAction
      ), { tabResult, focused });
      check(contextReport, "keyboard focus has a visible focus indicator", (
        focused.outlineStyle !== "none" && Number.parseFloat(focused.outlineWidth ?? "0") >= 1
      ), focused);
      check(contextReport, "focus selects the real ReactFlow wrapper and changes its descendant style", (
        focused.wrapperSelected
        && selectionState.selectedId === focused.nodeId
        && (
          selectionState.selectedBackground !== selectionState.unselectedBackground
          || selectionState.selectedBorderColor !== selectionState.unselectedBorderColor
        )
      ), { focused, selectionState });
      const graphState = await graphVisualState(root);
      check(contextReport, "900 ReactFlow owns at least 68 percent and nodes remain readable and contained", (
        (graphState.flowHeightRatio ?? 0) >= 0.68
        && graphState.allNodesInsideFlow
        && graphState.nodes.every((node) => Number.parseFloat(node.titleFontSize ?? "0") >= 11)
      ), graphState);
      await page.screenshot({
        path: fileURLToPath(new URL("./04-900-keyboard-focus.png", rawDirectory)),
        fullPage: false,
      });
      await page.keyboard.press("Shift+Tab");
      const shiftTabLeftNode = await page.evaluate(() => !document.activeElement?.matches(".native-graph-node-button"));
      check(contextReport, "Shift+Tab naturally leaves the Graph node action", shiftTabLeftNode, {
        active: await page.evaluate(() => ({
          tag: document.activeElement?.tagName.toLowerCase(),
          ariaLabel: document.activeElement?.getAttribute("aria-label"),
        })),
      });
      return { graphState, sidebars, tabResult, focused, selectionState, shiftTabLeftNode };
    },
  });

  await withFreshContext(browser, {
    name: "06-900-reduced-motion-collapsed",
    viewport: { width: 900, height: 760 },
    reducedMotion: "reduce",
    action: async (page, contextReport) => {
      const root = await openGraph(page);
      await page.locator('[data-knowledge-region="left"] button[aria-label="折叠左侧栏"]').click();
      const rightToggle = page.locator('[data-knowledge-region="activity"] button[aria-label="切换右侧上下文"]');
      if (await rightToggle.getAttribute("aria-pressed") === "true") await rightToggle.click();
      const opener = root.locator("[data-graph-filter-opener]");
      await opener.click();
      const panel = root.locator("[data-graph-filter-panel]");
      await panel.waitFor();
      await page.waitForFunction(() => document.activeElement?.getAttribute("aria-label") === "关系图文字筛选");
      const graphState = await graphVisualState(root);
      const reducedMotionState = await root.evaluate(() => (
        [...document.querySelectorAll(
          '[data-active-group="true"] .native-knowledge-graph .native-graph-node, '
          + '[data-active-group="true"] .native-knowledge-graph .native-graph-node-button, '
          + '[data-active-group="true"] .native-knowledge-graph .native-graph-filter-disclosure',
        )].map((element) => {
          const style = getComputedStyle(element);
          return {
            className: element.className,
            animationName: style.animationName,
            animationDuration: style.animationDuration,
            transitionDuration: style.transitionDuration,
          };
        })
      ));
      const collapsed = await page.evaluate(() => ({
        leftHidden: document.querySelector('[data-knowledge-region="left"]')?.getAttribute("aria-hidden"),
        rightHidden: document.querySelector('[data-knowledge-region="right"]')?.getAttribute("aria-hidden"),
      }));
      check(contextReport, "reduced-motion scenario uses a real collapsed sidebar combination", (
        collapsed.leftHidden === "true" && collapsed.rightHidden === "true"
      ), collapsed);
      check(contextReport, "collapsed 900 disclosure stays within Graph and flow keeps at least 68 percent", (
        graphState.disclosureInsideRoot === true && (graphState.flowHeightRatio ?? 0) >= 0.68
      ), graphState);
      check(contextReport, "Graph adds no animation under reduced motion", (
        reducedMotionState.every((entry) => (
          entry.animationName === "none"
          && (entry.animationDuration === "0s" || entry.animationDuration === "0ms")
          && (entry.transitionDuration === "0s" || entry.transitionDuration === "0ms")
        ))
      ), reducedMotionState);
      await page.keyboard.press("Escape");
      await panel.waitFor({ state: "detached" });
      const returnedFocus = await opener.evaluate((element) => document.activeElement === element);
      check(contextReport, "reduced-motion Escape still returns to the actual opener", returnedFocus, {
        returnedFocus,
      });
      return { graphState, reducedMotionState, collapsed, returnedFocus };
    },
  });
} finally {
  await browser.close();
}

const repeatedActivationPaths = activationEvidence.map((entry) => entry.selectedSourcePath);
report.repeatedActivation = {
  methods: activationEvidence.map((entry) => entry.method),
  readCount: activationEvidence.reduce((count, entry) => count + entry.readCountDelta, 0),
  paths: repeatedActivationPaths,
  sameBackendRelativePath: repeatedActivationPaths.length === 3
    && repeatedActivationPaths.every((path) => path === "notes/visual-baseline.md"),
  note: "each modality used a fresh context so the existing parent tab selection could not suppress a first typed read; the formal sequence helper contract separately proves monotonic repeated requests",
};
if (!report.repeatedActivation.sameBackendRelativePath || report.repeatedActivation.readCount !== 3) {
  report.failed += 1;
  report.failures.push({
    context: "cross-context-repeated-activation",
    name: "click Enter and Space each open the same backend relative_path exactly once",
    detail: report.repeatedActivation,
  });
}

report.commandTotals = report.contexts.reduce((totals, context) => {
  for (const [command, count] of Object.entries(context.audit?.callsByCommand ?? {})) {
    totals[command] = (totals[command] ?? 0) + count;
  }
  return totals;
}, {});
report.zeroTotals = {
  write: report.contexts.reduce((count, context) => count + (context.audit?.writeCallCount ?? 0), 0),
  unknown: report.contexts.reduce((count, context) => count + (context.audit?.unrecognizedCallCount ?? 0), 0),
  external: report.contexts.reduce((count, context) => count + (context.audit?.externalRequestCount ?? 0), 0),
  console: report.contexts.reduce((count, context) => count + (context.audit?.consoleErrorCount ?? 0), 0),
  pageError: report.contexts.reduce((count, context) => count + (context.audit?.pageErrorCount ?? 0), 0),
};
report.outcome = report.failed === 0 ? "PASS_SYNTHETIC_GRAPH_EVIDENCE" : "NEEDS_R3D_REWORK";
await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`, "utf8");
console.log(JSON.stringify({
  outcome: report.outcome,
  contexts: report.contexts.length,
  assertions: report.assertions,
  failed: report.failed,
  failures: report.failures.map(({ context, name }) => `${context}: ${name}`),
  commandTotals: report.commandTotals,
  zeroTotals: report.zeroTotals,
}, null, 2));
process.exit(report.failed === 0 ? 0 : 1);
