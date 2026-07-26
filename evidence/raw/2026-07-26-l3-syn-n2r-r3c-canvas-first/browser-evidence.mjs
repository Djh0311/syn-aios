import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./browser-evidence.json", rawDirectory));
const primaryGroupId = "knowledge-group-primary";
const secondaryGroupId = "knowledge-group-secondary";
const initialMarkdownPath = "notes/visual-baseline.md";
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
  "01-1180-canvas-continuous.png",
  "02-1180-file-panel-open.png",
  "03-1180-file-panel-escaped.png",
  "04-1180-node-inspector-open.png",
  "05-1180-node-inspector-closed.png",
  "06-1180-canvas-graph-split.png",
  "07-900-double-sidebar-canvas.png",
  "08-900-right-collapsed-canvas.png",
  "09-900-both-collapsed-canvas.png",
  "10-1180-reduced-motion-inspector.png",
];
const report = {
  phase: "post-implementation-synthetic-browser-evidence",
  fixture: "synthetic-only",
  assertions: 0,
  failed: 0,
  failures: [],
  contexts: [],
  screenshots,
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
    window.dispatchEvent(new Event("n2r-r3c-capture"));
  }, scenario);
  await page.waitForTimeout(220);
  return JSON.parse(await page.locator("#knowledge-workbench-visual-metrics").textContent());
}

function metricsHaveNoHorizontalOverflow(metrics) {
  const values = {
    documentElement: metrics.overflow.documentElement,
    body: metrics.overflow.body,
    shell: metrics.overflow.shell,
    activeGroupPanel: metrics.overflow.activeGroupPanel,
    ...Object.fromEntries(metrics.central.groups.map((group, index) => [`groupPanel${index + 1}`, group.scroll])),
  };
  return {
    values,
    ok: Object.values(values).every((metric) => metric && metric.scrollWidth <= metric.clientWidth),
  };
}

function centralAriaIsLinked(metrics) {
  return metrics.central.groups.every((group) => {
    if (group.tabCount === 0) return group.activeTabId === null && group.panelId === null;
    const selectedTabs = group.tabs.filter((tab) => tab.selected === "true");
    return group.tablistCount === 1
      && selectedTabs.length === 1
      && group.activeTabId === selectedTabs[0]?.id
      && group.panelLabelledBy === group.activeTabId
      && selectedTabs[0]?.controls === group.panelId
      && group.tabs.every((tab) => tab.controlsExists);
  });
}

function storageContainsChromeOnly(storage) {
  const serialized = JSON.stringify(storage.latestNormalizedContent);
  return storage.keys.every((key) => key === preferenceKey)
    && !serialized.includes('"body"')
    && !serialized.includes("合成视觉基线")
    && !serialized.includes("这是一段只用于隔离浏览器视觉量尺的合成知识内容");
}

function visualComparisonFor(name) {
  const comparisons = {
    "01-1180-canvas-continuous": {
      r0_04: { result: "GAP", difference: "连续 Canvas 与内嵌控件已收敛，但此图是单组，没有 R0 04 的右组 Graph 构图。" },
      r0_08: { result: "GAP", difference: "左右侧栏均展开，没有 R0 08 的左栏折叠状态。" },
    },
    "02-1180-file-panel-focus-escape": {
      r0_04: { result: "GAP", difference: "按需文件面板符合冻结检查点，但 R0 04 没有对应的 Canvas 内浮层状态。" },
      r0_08: { result: "GAP", difference: "焦点/Escape 图不对应 R0 08 的侧栏折叠构图。" },
    },
    "03-1180-node-inspector-focus": {
      r0_04: { result: "GAP", difference: "selection inspector 局限在 Canvas 根内，但 R0 04 的右侧是全局链接上下文，不是节点字段面板。" },
      r0_08: { result: "GAP", difference: "节点检查图没有复现 R0 08 的左栏折叠构图。" },
    },
    "04-1180-canvas-graph-split": {
      r0_04: { result: "PASS", difference: "左 Canvas、右 Graph、外侧目录/上下文与连续中央空间关系一致；Syn 组壳、节点尺度和既有 Graph 工具仍不同。" },
      r0_08: { result: "GAP", difference: "此图保留左侧目录，不是 R0 08 的折叠态。" },
    },
    "05-900-double-sidebar-canvas": {
      r0_04: { result: "GAP", difference: "窄宽下 Canvas chrome 和内部工具保持可达，但没有 R0 04 的 Canvas/Graph 双组。" },
      r0_08: { result: "GAP", difference: "双侧栏均展开，不是 R0 08 的折叠态。" },
    },
    "06-900-sidebar-collapse-canvas": {
      r0_04: { result: "GAP", difference: "此场景验证单 Canvas 接管空间，没有 R0 04 的右组 Graph。" },
      r0_08: { result: "PASS", difference: "左活动 rail 保留且折叠侧栏把宽度归还中央舞台；Syn 顶栏、节点卡和底部状态栏仍不同。" },
    },
    "07-1180-reduced-motion-canvas": {
      r0_04: { result: "GAP", difference: "reduced-motion inspector 是可用性证据，不对应 R0 04 的静态 Canvas/Graph 构图。" },
      r0_08: { result: "GAP", difference: "左右侧栏均展开，没有 R0 08 的折叠构图。" },
    },
  };
  return comparisons[name];
}

async function saveScreenshot(page, filename) {
  await page.screenshot({
    path: fileURLToPath(new URL(`./${filename}`, rawDirectory)),
    fullPage: false,
  });
}

async function openInitialMarkdown(page) {
  const treeTab = page.locator(`.native-workspace-tree-note[title="${initialMarkdownPath}"]`);
  await treeTab.waitFor();
  await treeTab.click();
  await page.waitForFunction((relativePath) => (
    [...document.querySelectorAll('[role="tab"]')]
      .some((tab) => tab.getAttribute("aria-label")?.includes(relativePath))
  ), initialMarkdownPath);
}

async function splitInitialMarkdown(page) {
  await openInitialMarkdown(page);
  await page.getByRole("button", { name: "向右分栏" }).click();
  await page.waitForFunction(() => document.querySelectorAll("[data-knowledge-tab-group]").length === 2);
}

async function openCanvasInActiveGroup(page) {
  await page.locator('[data-knowledge-region="activity"] button[aria-label="Canvas"]').click();
  const activeGroup = page.locator('[data-active-group="true"]');
  await activeGroup.locator(".native-knowledge-canvas").waitFor();
  const trigger = activeGroup.locator("[data-canvas-file-trigger]");
  await trigger.click();
  const panel = activeGroup.locator("[data-canvas-file-panel]");
  await panel.waitFor();
  await panel.locator(`button[title="${canvasPath}"]`).click();
  await activeGroup.locator(".native-canvas-flow-stage .react-flow").waitFor();
  await panel.waitFor({ state: "detached" });
}

function assertCanvasCore(name, metrics) {
  expect(`${name}: exactly one Canvas root and one React Flow projection`, (
    metrics.canvas.rootCount === 1 && metrics.canvas.reactFlowCount === 1
  ), metrics.canvas);
  expect(`${name}: compact chrome and full current-path label remain visible`, (
    metrics.canvas.chrome.count === 1
    && metrics.canvas.chrome.currentPathLabel === `当前 Canvas：${canvasPath}`
  ), metrics.canvas.chrome);
  expect(`${name}: continuous stage owns at least 75 percent of Canvas height`, (
    metrics.canvas.stage.heightRatio >= 0.75
  ), metrics.canvas.stage);
  expect(`${name}: floating tools are named and contained by the stage`, (
    metrics.canvas.floatingTools.count === 1
    && metrics.canvas.floatingTools.withinStage === true
    && metrics.canvas.floatingTools.ariaLabel === "Canvas 节点工具"
  ), metrics.canvas.floatingTools);
  expect(`${name}: Canvas root and stage have no horizontal overflow`, (
    metrics.canvas.root.horizontalOverflow === false
    && metrics.canvas.stage.horizontalOverflow === false
  ), { root: metrics.canvas.root, stage: metrics.canvas.stage });
}

async function withFreshFixture(browser, name, viewport, action, options = {}) {
  const context = await browser.newContext({
    viewport,
    deviceScaleFactor: 1,
    reducedMotion: options.reducedMotion ?? "no-preference",
  });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = {
    name,
    viewport,
    reducedMotion: options.reducedMotion ?? "no-preference",
    evidence: {},
    audit: {},
    visualComparison: visualComparisonFor(name),
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
    expect(`${name}: localStorage is empty before mount`, initialMetrics.fixture.localStorageEmptyBeforeMount === true, {
      observed: initialMetrics.fixture.localStorageEmptyBeforeMount,
    });

    contextReport.evidence = await action(page);
    const metrics = await collectMetrics(page, name);
    const overflow = metricsHaveNoHorizontalOverflow(metrics);
    const calls = Object.keys(metrics.mock.callsByCommand);
    contextReport.metrics = metrics;
    contextReport.overflow = overflow;
    contextReport.audit = {
      localStorageEmptyBeforeMount: metrics.fixture.localStorageEmptyBeforeMount,
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
    };

    expect(`${name}: exact read allowlist`, JSON.stringify(metrics.mock.allowedReadCommands) === JSON.stringify(expectedReadAllowlist), {
      observed: metrics.mock.allowedReadCommands,
    });
    expect(`${name}: calls stay inside the read allowlist`, calls.every((command) => expectedReadAllowlist.includes(command)), {
      calls,
    });
    expect(`${name}: real Tauri write call count is zero`, metrics.mock.writeCallCount === 0, {
      observed: metrics.mock.writeCallCount,
    });
    expect(`${name}: unrecognized call count is zero`, metrics.mock.unrecognizedCallCount === 0, {
      observed: metrics.mock.unrecognizedCallCount,
    });
    expect(`${name}: external request count is zero`, externalRequests.length === 0, { externalRequests });
    expect(`${name}: console error count is zero`, consoleErrors.length === 0, { consoleErrors });
    expect(`${name}: page error count is zero`, pageErrors.length === 0, { pageErrors });
    expect(`${name}: document body shell and group panels have no horizontal overflow`, overflow.ok, overflow.values);
    expect(`${name}: central workspace has one tablist per group and at most two groups`, (
      metrics.central.groupCount >= 1
      && metrics.central.groupCount <= 2
      && metrics.central.tablistCount === metrics.central.groupCount
    ), metrics.central);
    expect(`${name}: tab and tabpanel ARIA references are complete`, centralAriaIsLinked(metrics), metrics.central.groups);
    expect(`${name}: localStorage contains only disposable R3B chrome preference`, (
      metrics.localStorage.keys.length <= 1 && storageContainsChromeOnly(metrics.localStorage)
    ), metrics.localStorage);
    assertCanvasCore(name, metrics);
  } catch (error) {
    expect(`${name}: browser evidence completed without inspection error`, false, {
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
  await withFreshFixture(browser, "01-1180-canvas-continuous", { width: 1180, height: 760 }, async (page) => {
    await openCanvasInActiveGroup(page);
    const state = await collectMetrics(page, "01-1180-canvas-continuous");
    expect("1180 Canvas defaults to closed file panel and inspector", (
      state.canvas.filePanel.count === 0
      && state.canvas.filePanel.interactiveChildren === 0
      && state.canvas.inspector.count === 0
      && state.canvas.inspector.interactiveChildren === 0
      && state.canvas.fileTrigger.expanded === "false"
    ), state.canvas);
    await saveScreenshot(page, "01-1180-canvas-continuous.png");
    return { screenshot: "01-1180-canvas-continuous.png", canvas: state.canvas };
  });

  await withFreshFixture(browser, "02-1180-file-panel-focus-escape", { width: 1180, height: 760 }, async (page) => {
    await openCanvasInActiveGroup(page);
    const trigger = page.locator('[data-active-group="true"] [data-canvas-file-trigger]');
    await trigger.click();
    await page.waitForFunction(() => document.activeElement?.closest("[data-canvas-file-panel]") !== null);
    const openState = await collectMetrics(page, "02a-1180-file-panel-open");
    const openFocus = await page.evaluate(() => ({
      insidePanel: document.activeElement?.closest("[data-canvas-file-panel]") !== null,
      tag: document.activeElement?.tagName.toLowerCase() ?? null,
      title: document.activeElement?.getAttribute("title") ?? null,
      ariaLabel: document.activeElement?.getAttribute("aria-label") ?? null,
    }));
    expect("file panel opens inside Canvas with complete trigger ARIA and first focus", (
      openState.canvas.filePanel.count === 1
      && openState.canvas.filePanel.withinRoot === true
      && openState.canvas.filePanel.position === "absolute"
      && openState.canvas.fileTrigger.expanded === "true"
      && openState.canvas.fileTrigger.controls === "native-canvas-file-panel"
      && openState.canvas.fileTrigger.controlsExists === true
      && openFocus.insidePanel
    ), { canvas: openState.canvas, focus: openFocus });
    expect("file panel is the only raised Canvas panel", openState.canvas.inspector.count === 0, openState.canvas);
    await saveScreenshot(page, "02-1180-file-panel-open.png");

    await page.keyboard.press("Escape");
    await page.waitForSelector("[data-canvas-file-panel]", { state: "detached" });
    await page.waitForFunction(() => document.activeElement?.hasAttribute("data-canvas-file-trigger"));
    const closedState = await collectMetrics(page, "02b-1180-file-panel-escaped");
    const returnedToTrigger = await page.evaluate(() => document.activeElement?.hasAttribute("data-canvas-file-trigger") ?? false);
    expect("Escape removes file panel from focus and accessibility paths and returns focus", (
      closedState.canvas.filePanel.count === 0
      && closedState.canvas.filePanel.interactiveChildren === 0
      && closedState.canvas.fileTrigger.expanded === "false"
      && returnedToTrigger
    ), { canvas: closedState.canvas, returnedToTrigger });
    await saveScreenshot(page, "03-1180-file-panel-escaped.png");
    return {
      screenshots: ["02-1180-file-panel-open.png", "03-1180-file-panel-escaped.png"],
      open: { canvas: openState.canvas, focus: openFocus },
      closed: { canvas: closedState.canvas, returnedToTrigger },
    };
  });

  await withFreshFixture(browser, "03-1180-node-inspector-focus", { width: 1180, height: 760 }, async (page) => {
    await openCanvasInActiveGroup(page);
    const node = page.locator('.react-flow__node[data-id="baseline"]');
    await node.click();
    await page.waitForSelector("[data-canvas-inspector]");
    const openState = await collectMetrics(page, "03a-1180-node-inspector-open");
    expect("selected node opens one associated inspector inside Canvas", (
      openState.canvas.inspector.count === 1
      && openState.canvas.inspector.withinRoot === true
      && openState.canvas.inspector.position === "absolute"
      && openState.canvas.filePanel.count === 0
      && openState.canvas.selectedNode.id === "baseline"
      && openState.canvas.selectedNode.controls === "native-canvas-node-inspector"
      && openState.canvas.selectedNode.controlsExists === true
      && openState.canvas.selectedNode.expanded === "true"
    ), openState.canvas);
    await saveScreenshot(page, "04-1180-node-inspector-open.png");

    await page.getByRole("button", { name: "关闭节点属性" }).click();
    await page.waitForSelector("[data-canvas-inspector]", { state: "detached" });
    await page.waitForTimeout(100);
    const closedState = await collectMetrics(page, "03b-1180-node-inspector-closed");
    const focusAfterClose = await page.evaluate(() => ({
      tag: document.activeElement?.tagName.toLowerCase() ?? null,
      nodeId: document.activeElement?.getAttribute("data-id") ?? null,
      isStage: document.activeElement?.matches('[data-canvas-stage="continuous"]') ?? false,
      isBody: document.activeElement === document.body,
    }));
    expect("closing inspector restores a non-body Canvas focus target", (
      closedState.canvas.inspector.count === 0
      && closedState.canvas.inspector.interactiveChildren === 0
      && !focusAfterClose.isBody
      && (focusAfterClose.nodeId === "baseline" || focusAfterClose.isStage)
    ), { canvas: closedState.canvas, focusAfterClose });
    await saveScreenshot(page, "05-1180-node-inspector-closed.png");
    return {
      screenshots: ["04-1180-node-inspector-open.png", "05-1180-node-inspector-closed.png"],
      open: openState.canvas,
      closed: { canvas: closedState.canvas, focusAfterClose },
    };
  });

  await withFreshFixture(browser, "04-1180-canvas-graph-split", { width: 1180, height: 760 }, async (page) => {
    await splitInitialMarkdown(page);
    await page.locator(`[data-knowledge-tab-group="${primaryGroupId}"] [role="tab"][aria-selected="true"]`).click();
    await openCanvasInActiveGroup(page);
    await page.locator(`[data-knowledge-tab-group="${secondaryGroupId}"] [role="tab"][aria-selected="true"]`).click();
    await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
    await page.waitForSelector(`[data-knowledge-tab-group="${secondaryGroupId}"] .native-knowledge-graph`);
    const state = await collectMetrics(page, "04-1180-canvas-graph-split");
    expect("1180 split keeps Canvas left and Graph right in real tab groups", (
      state.central.groupCount === 2
      && state.central.groups[0]?.panelContainsCanvas === true
      && state.central.groups[0]?.panelContainsGraph === false
      && state.central.groups[1]?.panelContainsCanvas === false
      && state.central.groups[1]?.panelContainsGraph === true
    ), state.central.groups);
    await saveScreenshot(page, "06-1180-canvas-graph-split.png");
    return { screenshot: "06-1180-canvas-graph-split.png", central: state.central, canvas: state.canvas };
  });

  await withFreshFixture(browser, "05-900-double-sidebar-canvas", { width: 900, height: 760 }, async (page) => {
    await openCanvasInActiveGroup(page);
    const state = await collectMetrics(page, "05-900-double-sidebar-canvas");
    expect("900 expanded sidebars keep Canvas chrome and floating tools reachable", (
      state.regions.left.ariaHidden === "false"
      && state.regions.right.ariaHidden === "false"
      && state.canvas.chrome.bounds.width > 0
      && state.canvas.floatingTools.withinStage === true
    ), { left: state.regions.left, right: state.regions.right, canvas: state.canvas });
    await saveScreenshot(page, "07-900-double-sidebar-canvas.png");
    return { screenshot: "07-900-double-sidebar-canvas.png", canvas: state.canvas };
  });

  await withFreshFixture(browser, "06-900-sidebar-collapse-canvas", { width: 900, height: 760 }, async (page) => {
    await openCanvasInActiveGroup(page);
    await page.getByRole("button", { name: "切换右侧上下文" }).click();
    const rightCollapsed = await collectMetrics(page, "06a-900-right-collapsed-canvas");
    expect("900 right sidebar collapse removes its interactive accessibility path", (
      rightCollapsed.regions.right.ariaHidden === "true"
      && rightCollapsed.regions.right.inert === true
      && rightCollapsed.regions.right.interactiveChildren === 0
    ), rightCollapsed.regions.right);
    await saveScreenshot(page, "08-900-right-collapsed-canvas.png");

    await page.locator('[data-knowledge-region="left"] button[aria-label="折叠左侧栏"]').click();
    const bothCollapsed = await collectMetrics(page, "06b-900-both-collapsed-canvas");
    expect("900 both sidebars collapse and Canvas takes the returned width", (
      bothCollapsed.regions.left.ariaHidden === "true"
      && bothCollapsed.regions.left.inert === true
      && bothCollapsed.regions.left.interactiveChildren === 0
      && bothCollapsed.regions.right.ariaHidden === "true"
      && bothCollapsed.regions.right.inert === true
      && bothCollapsed.regions.right.interactiveChildren === 0
      && bothCollapsed.canvas.root.bounds.width > rightCollapsed.canvas.root.bounds.width
    ), { rightCollapsed, bothCollapsed });
    await saveScreenshot(page, "09-900-both-collapsed-canvas.png");
    return {
      screenshots: ["08-900-right-collapsed-canvas.png", "09-900-both-collapsed-canvas.png"],
      rightCollapsed,
      bothCollapsed,
    };
  });

  await withFreshFixture(
    browser,
    "07-1180-reduced-motion-canvas",
    { width: 1180, height: 760 },
    async (page) => {
      await openCanvasInActiveGroup(page);
      const reducedMotionMatches = await page.evaluate(() => window.matchMedia("(prefers-reduced-motion: reduce)").matches);
      const trigger = page.locator('[data-canvas-file-trigger]');
      await trigger.click();
      await page.waitForFunction(() => document.activeElement?.closest("[data-canvas-file-panel]") !== null);
      await page.keyboard.press("Escape");
      await page.waitForFunction(() => document.activeElement?.hasAttribute("data-canvas-file-trigger"));
      await page.locator('.react-flow__node[data-id="baseline"]').click();
      await page.waitForSelector("[data-canvas-inspector]");
      const inspectorState = await collectMetrics(page, "07a-1180-reduced-motion-inspector");
      expect("reduced-motion keeps panel Escape focus return and associated inspector usable", (
        reducedMotionMatches
        && inspectorState.canvas.filePanel.count === 0
        && inspectorState.canvas.inspector.count === 1
        && inspectorState.canvas.selectedNode.controlsExists === true
      ), { reducedMotionMatches, canvas: inspectorState.canvas });
      await saveScreenshot(page, "10-1180-reduced-motion-inspector.png");
      await page.getByRole("button", { name: "关闭节点属性" }).click();
      await page.waitForSelector("[data-canvas-inspector]", { state: "detached" });
      await page.waitForTimeout(100);
      const focusAfterClose = await page.evaluate(() => ({
        nodeId: document.activeElement?.getAttribute("data-id") ?? null,
        isStage: document.activeElement?.matches('[data-canvas-stage="continuous"]') ?? false,
        isBody: document.activeElement === document.body,
      }));
      expect("reduced-motion inspector close keeps focus inside Canvas", (
        !focusAfterClose.isBody && (focusAfterClose.nodeId === "baseline" || focusAfterClose.isStage)
      ), focusAfterClose);
      return {
        screenshot: "10-1180-reduced-motion-inspector.png",
        reducedMotionMatches,
        inspector: inspectorState.canvas,
        focusAfterClose,
      };
    },
    { reducedMotion: "reduce" },
  );
} finally {
  await browser.close();
  report.outcome = report.failed === 0 ? "PASS" : "FAIL";
  await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

console.log(JSON.stringify({
  outcome: report.outcome,
  contexts: report.contexts.length,
  assertions: report.assertions,
  failed: report.failed,
  failures: report.failures.map((failure) => failure.name),
}, null, 2));
process.exitCode = report.failed === 0 ? 0 : 1;
