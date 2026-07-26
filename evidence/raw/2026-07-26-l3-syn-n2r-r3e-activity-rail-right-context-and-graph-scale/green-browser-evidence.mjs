// N2R-R3E green：纯合成夹具 + 真实 React + 真实生产 CSS，每个场景 fresh context。
// 用法：先起 vite（127.0.0.1:5173），再 node green-browser-evidence.mjs
import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./green-browser-evidence.json", rawDirectory));

const preferenceKey = "syn-native-knowledge-workspace-ui-v1";
const readAllowlist = [
  "knowledge_workspace_snapshot",
  "knowledge_workspace_graph",
  "knowledge_workspace_read_markdown",
  "knowledge_workspace_search",
  "knowledge_workspace_read_canvas",
];
const activityRailAccessibleNames = [
  "文件",
  "搜索",
  "关系图",
  "Canvas",
  "Syn 命令",
  "设置与维护",
  "来源",
  "切换右侧上下文",
];
const contextSectionTitles = ["属性", "反向引用", "来源上下文"];
/** 达标缩放口径：节点盒实测 ≥ 28×28 CSS px。与 KNOWLEDGE_GRAPH_READABLE_ZOOM 同源。 */
const READABLE_NODE_BOX = 28;

const report = {
  phase: "post-implementation-green",
  fixture: "synthetic-only",
  scope: "D1 活动栏 ribbon / D2 右栏三区折叠 / D3 Graph 布局规模化",
  notRealApp: "NOT_REAL_APP：只跑合成夹具，不进入真实 App / store / vault",
  outcome: "PENDING",
  assertions: 0,
  failed: 0,
  failures: [],
  readAllowlist,
  measurements: { D1: {}, D2: {}, D3: {} },
  contexts: [],
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

async function fixtureMetrics(page) {
  await page.evaluate(() => window.dispatchEvent(new Event("n2r-r3c-capture")));
  await page.waitForTimeout(120);
  const raw = await page.locator("#knowledge-workbench-visual-metrics").textContent();
  return raw && raw !== "pending" ? JSON.parse(raw) : null;
}

function auditMetrics(contextReport, metrics) {
  const mock = metrics.mock;
  const outsideAllowlist = Object.keys(mock.callsByCommand).filter((command) => !readAllowlist.includes(command));
  check(contextReport, `${contextReport.name}: 观察到的 command 全在精确 read allowlist 内`, outsideAllowlist.length === 0, {
    callsByCommand: mock.callsByCommand,
    outsideAllowlist,
  });
  check(contextReport, `${contextReport.name}: 写/未知 command 计数为 0`, mock.writeCallCount === 0 && mock.unrecognizedCallCount === 0, {
    writeCallCount: mock.writeCallCount,
    unrecognizedCallCount: mock.unrecognizedCallCount,
    unrecognizedCommandNames: mock.unrecognizedCommandNames,
  });
  check(contextReport, `${contextReport.name}: mount 前目标 origin localStorage 为空`, metrics.localStorage.emptyBeforeMount === true, metrics.localStorage);
  check(
    contextReport,
    `${contextReport.name}: localStorage 只允许既有可丢弃 UI chrome 偏好键`,
    metrics.localStorage.keys.every((key) => key === preferenceKey),
    metrics.localStorage,
  );
  const overflow = metrics.overflow;
  const horizontallyOverflowing = Object.entries(overflow)
    .filter(([, value]) => value && value.scrollWidth > value.clientWidth + 1)
    .map(([layer, value]) => ({ layer, ...value }));
  check(contextReport, `${contextReport.name}: 各层零横向 overflow`, horizontallyOverflowing.length === 0, { horizontallyOverflowing, overflow });
  return { callsByCommand: mock.callsByCommand, localStorage: metrics.localStorage, overflow };
}

async function activityRailState(page) {
  return page.evaluate((expectedNames) => {
    const round = (value) => Math.round(value * 100) / 100;
    const rail = document.querySelector('[data-knowledge-region="activity"]');
    if (!rail) return null;
    const buttons = [...rail.querySelectorAll("button")].map((button) => {
      const style = getComputedStyle(button);
      const rect = button.getBoundingClientRect();
      let textLineBoxes = 0;
      for (const child of button.childNodes) {
        if (child.nodeType !== Node.TEXT_NODE) continue;
        if (!child.textContent?.trim()) continue;
        const range = document.createRange();
        range.selectNodeContents(child);
        textLineBoxes += range.getClientRects().length;
      }
      const icon = button.querySelector("svg");
      return {
        ariaLabel: button.getAttribute("aria-label"),
        visibleText: button.textContent?.trim() ?? "",
        ariaPressed: button.getAttribute("aria-pressed"),
        title: button.getAttribute("title"),
        className: button.className,
        bounds: { x: round(rect.x), y: round(rect.y), width: round(rect.width), height: round(rect.height) },
        clientHeight: button.clientHeight,
        scrollHeight: button.scrollHeight,
        clientWidth: button.clientWidth,
        scrollWidth: button.scrollWidth,
        fontSize: Number.parseFloat(style.fontSize),
        textLineBoxes,
        wrapped: textLineBoxes > 1,
        noScrollOverflow: button.scrollHeight <= button.clientHeight && button.scrollWidth <= button.clientWidth,
        hitTargetOk: rect.width >= 28 && rect.height >= 28,
        svgCount: button.querySelectorAll("svg").length,
        iconAriaHidden: icon?.getAttribute("aria-hidden") ?? null,
        iconFocusable: icon?.getAttribute("focusable") ?? null,
        iconStroke: icon?.getAttribute("stroke") ?? null,
        iconTabbable: icon ? icon.hasAttribute("tabindex") : null,
      };
    });
    const railRect = rail.getBoundingClientRect();
    return {
      railWidth: round(railRect.width),
      railBounds: { x: round(railRect.x), y: round(railRect.y), width: round(railRect.width), height: round(railRect.height) },
      railScroll: {
        clientWidth: rail.clientWidth,
        scrollWidth: rail.scrollWidth,
        clientHeight: rail.clientHeight,
        scrollHeight: rail.scrollHeight,
      },
      buttonCount: buttons.length,
      svgCountTotal: rail.querySelectorAll("svg").length,
      buttons,
      accessibleNames: buttons.map((button) => button.ariaLabel),
      accessibleNamesMatchExpected:
        JSON.stringify(buttons.map((button) => button.ariaLabel)) === JSON.stringify(expectedNames),
      wrappedButtons: buttons.filter((button) => button.wrapped).map((button) => button.ariaLabel),
      totalTextLineBoxes: buttons.reduce((sum, button) => sum + button.textLineBoxes, 0),
      pressedNames: buttons.filter((button) => button.ariaPressed === "true").map((button) => button.ariaLabel),
      buttonsWithoutPressed: buttons.filter((button) => button.ariaPressed === null).map((button) => button.ariaLabel),
    };
  }, activityRailAccessibleNames);
}

async function rightSidebarState(page) {
  return page.evaluate(() => {
    const round = (value) => Math.round(value * 100) / 100;
    const bounds = (element) => {
      if (!element) return null;
      const rect = element.getBoundingClientRect();
      return { x: round(rect.x), y: round(rect.y), width: round(rect.width), height: round(rect.height) };
    };
    const tabbable = (root) => [...root.querySelectorAll('button, a[href], input, select, textarea, [tabindex]:not([tabindex="-1"])')]
      .filter((element) => element.offsetParent !== null || element === document.activeElement);
    const right = document.querySelector('[data-knowledge-region="right"]');
    if (!right) return null;
    const sections = [...right.querySelectorAll(".native-context-section")].map((section) => {
      const summary = section.querySelector(".native-context-summary");
      const body = section.querySelector(".native-context-body");
      return {
        title: section.getAttribute("data-context-section"),
        summaryText: summary?.querySelector(".native-context-title")?.textContent?.trim() ?? null,
        badge: summary?.querySelector(".native-context-badge")?.textContent?.trim() ?? null,
        ariaExpanded: summary?.getAttribute("aria-expanded") ?? null,
        ariaControls: summary?.getAttribute("aria-controls") ?? null,
        bodyId: body?.id ?? null,
        controlsResolves: Boolean(summary && document.getElementById(summary.getAttribute("aria-controls") ?? "")),
        bodyHidden: body?.hasAttribute("hidden") ?? null,
        bodyDisplay: body ? getComputedStyle(body).display : null,
        bodyTabbableCount: body ? tabbable(body).length : null,
        summaryBounds: bounds(summary),
        bodyBounds: bounds(body),
      };
    });
    const sourceContextSection = right.querySelector('section[aria-label="合成来源上下文"]');
    const sourceContextSpans = sourceContextSection
      ? [...sourceContextSection.querySelectorAll(".knowledge-document-detail > span")].map((span) => span.textContent?.trim() ?? "")
      : [];
    const headerSpan = right.querySelector(".syn-knowledge-sidebar-tabs--right span");
    return {
      headerText: headerSpan?.textContent?.trim() ?? null,
      headerDeclaresOutline: (headerSpan?.textContent ?? "").includes("大纲"),
      rightTextDeclaresOutline: (right.textContent ?? "").includes("大纲"),
      sectionCount: sections.length,
      sections,
      sectionTitles: sections.map((section) => section.title),
      sectionsWithCollapseControl: sections.filter((section) => section.ariaExpanded !== null).length,
      expandedTitles: sections.filter((section) => section.ariaExpanded === "true").map((section) => section.title),
      sourceContextPresent: Boolean(sourceContextSection),
      sourceContextInsideProductContainer: Boolean(sourceContextSection?.closest(".native-context-body")),
      sourceContextItemCount: sourceContextSpans.length,
      sourceContextFirstItem: sourceContextSpans[0] ?? null,
      sourceContextLastItem: sourceContextSpans.at(-1) ?? null,
      sourceContextHeading: sourceContextSection?.querySelector("strong")?.textContent?.trim() ?? null,
      emptyStateText: right.textContent?.includes("打开笔记后会显示安全属性、标签和反向引用。") ?? false,
      rightBounds: bounds(right),
      rightScroll: {
        clientHeight: right.clientHeight,
        scrollHeight: right.scrollHeight,
        clientWidth: right.clientWidth,
        scrollWidth: right.scrollWidth,
      },
      rightTabbableCount: tabbable(right).length,
    };
  });
}

async function graphScaleState(page) {
  return page.evaluate(() => {
    const round = (value) => Math.round(value * 100) / 100;
    const graphRoot = document.querySelector('[data-active-group="true"] .native-knowledge-graph');
    if (!graphRoot) return null;
    const stage = graphRoot.querySelector(".native-graph-flow-stage");
    const viewport = graphRoot.querySelector(".react-flow__viewport");
    const transform = viewport ? getComputedStyle(viewport).transform : "none";
    let zoom = null;
    if (transform && transform !== "none") {
      const parts = transform.replace(/matrix\(|\)/g, "").split(",").map((value) => Number.parseFloat(value));
      if (parts.length >= 4 && Number.isFinite(parts[0])) zoom = Math.round(parts[0] * 10000) / 10000;
    }
    const nodes = [...graphRoot.querySelectorAll(".react-flow__node")].map((wrapper) => {
      const rect = wrapper.getBoundingClientRect();
      return {
        id: wrapper.getAttribute("data-id"),
        x: round(rect.x),
        y: round(rect.y),
        right: round(rect.right),
        bottom: round(rect.bottom),
        width: round(rect.width),
        height: round(rect.height),
      };
    });
    let intersectingPairs = 0;
    const examples = [];
    for (let i = 0; i < nodes.length; i += 1) {
      for (let j = i + 1; j < nodes.length; j += 1) {
        const a = nodes[i];
        const b = nodes[j];
        if (a.x < b.right && b.x < a.right && a.y < b.bottom && b.y < a.bottom) {
          intersectingPairs += 1;
          if (examples.length < 4) examples.push({ a, b });
        }
      }
    }
    const stageRect = (graphRoot.querySelector(".react-flow") ?? stage)?.getBoundingClientRect() ?? null;
    const insideStage = stageRect
      ? nodes.filter((node) => (
        node.x >= stageRect.x - 2
        && node.y >= stageRect.y - 2
        && node.right <= stageRect.right + 2
        && node.bottom <= stageRect.bottom + 2
      )).length
      : null;
    const buttons = [...graphRoot.querySelectorAll(".native-graph-node-button")];
    const titles = buttons.map((button) => button.querySelector("strong")).filter(Boolean);
    const firstTitle = titles[0] ?? null;
    const nodeBoxes = nodes.map((node) => ({ width: node.width, height: node.height }));
    const minBox = nodeBoxes.reduce(
      (accumulator, box) => ({ width: Math.min(accumulator.width, box.width), height: Math.min(accumulator.height, box.height) }),
      { width: Infinity, height: Infinity },
    );
    return {
      nodeCount: nodes.length,
      edgeCount: graphRoot.querySelectorAll(".react-flow__edge").length,
      zoom,
      zoomTier: stage?.getAttribute("data-graph-zoom-tier") ?? null,
      stageNodeCountAttribute: stage?.getAttribute("data-graph-node-count") ?? null,
      intersectingPairs,
      examples,
      stageBounds: stageRect
        ? { x: round(stageRect.x), y: round(stageRect.y), width: round(stageRect.width), height: round(stageRect.height) }
        : null,
      nodesInsideStage: insideStage,
      nodeBoxSample: nodes.slice(0, 3),
      measuredNodeBox: Number.isFinite(minBox.width) ? { width: round(minBox.width), height: round(minBox.height) } : null,
      ariaLabelCount: buttons.filter((button) => (button.getAttribute("aria-label") ?? "").length > 0).length,
      firstNodeAriaLabel: buttons[0]?.getAttribute("aria-label") ?? null,
      titleElementCount: titles.length,
      titleVisible: firstTitle ? getComputedStyle(firstTitle).display !== "none" : null,
      titleClipped: firstTitle ? firstTitle.scrollWidth > firstTitle.clientWidth + 1 : null,
      nodeButtonDisabledCount: buttons.filter((button) => button.disabled).length,
    };
  });
}

async function openGraph(page) {
  await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
  const root = page.locator('[data-active-group="true"] .native-knowledge-graph');
  await root.waitFor();
  await root.locator(".native-graph-node-button").first().waitFor();
  await page.waitForTimeout(700);
}

/** 用真实的 zoom-in 控件一步步放大，直到节点盒实测 ≥ 28×28，报告首个达标缩放。 */
async function zoomToReadable(page, maxClicks = 40) {
  const zoomIn = page.locator('[data-active-group="true"] .react-flow__controls-zoomin');
  const steps = [];
  for (let click = 0; click <= maxClicks; click += 1) {
    const state = await graphScaleState(page);
    steps.push({ click, zoom: state.zoom, nodeBox: state.measuredNodeBox, titleVisible: state.titleVisible });
    if (state.measuredNodeBox && state.measuredNodeBox.width >= READABLE_NODE_BOX && state.measuredNodeBox.height >= READABLE_NODE_BOX) {
      return { reached: true, clicks: click, readableZoom: state.zoom, state, steps };
    }
    if (click === maxClicks) break;
    await zoomIn.click();
    await page.waitForTimeout(120);
  }
  const state = await graphScaleState(page);
  return { reached: false, clicks: maxClicks, readableZoom: null, state, steps };
}

async function withFreshContext(browser, { name, viewport, scenario = null, reducedMotion = "no-preference", action }) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1, reducedMotion });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = { name, viewport, scenario, reducedMotion, assertions: [], evidence: {} };
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  page.on("pageerror", (error) => pageErrors.push(String(error)));
  page.on("request", (request) => {
    const requestUrl = request.url();
    if (!requestUrl.startsWith("http://127.0.0.1:5173/") && !requestUrl.startsWith("data:") && !requestUrl.startsWith("blob:")) {
      externalRequests.push(requestUrl);
    }
  });
  await page.route("**/*", async (route) => {
    const requestUrl = route.request().url();
    if (requestUrl.startsWith("http://127.0.0.1:5173/") || requestUrl.startsWith("data:") || requestUrl.startsWith("blob:")) {
      await route.continue();
      return;
    }
    await route.abort("blockedbyclient");
  });
  try {
    await page.goto(fixtureUrl, { waitUntil: "networkidle" });
    await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
    if (scenario) {
      await page.evaluate((nextScenario) => {
        document.documentElement.dataset.fixtureScenario = nextScenario;
      }, scenario);
    }
    contextReport.evidence = await action(page, contextReport);
    const metrics = await fixtureMetrics(page);
    contextReport.audit = {
      ...auditMetrics(contextReport, metrics),
      externalRequestCount: externalRequests.length,
      consoleErrorCount: consoleErrors.length,
      pageErrorCount: pageErrors.length,
      externalRequests,
      consoleErrors,
      pageErrors,
    };
    check(contextReport, `${name}: 外部请求 / console error / page error 三项零值`, (
      externalRequests.length === 0 && consoleErrors.length === 0 && pageErrors.length === 0
    ), { externalRequests, consoleErrors, pageErrors });
  } catch (error) {
    contextReport.inspectionError = String(error?.stack ?? error);
    report.failed += 1;
    report.failures.push({ context: name, name: "context threw", detail: contextReport.inspectionError });
  } finally {
    report.contexts.push(contextReport);
    await context.close();
  }
}

function assertRibbon(contextReport, rail, label) {
  check(contextReport, `${label}: 八个入口可访问名称逐字相符`, rail.accessibleNamesMatchExpected, {
    accessibleNames: rail.accessibleNames,
    expected: activityRailAccessibleNames,
  });
  check(contextReport, `${label}: 活动栏宽度稳定在 36-48px`, rail.railWidth >= 36 && rail.railWidth <= 48, { railWidth: rail.railWidth });
  check(contextReport, `${label}: 八入口零断行`, rail.wrappedButtons.length === 0 && rail.totalTextLineBoxes === 0, {
    wrappedButtons: rail.wrappedButtons,
    totalTextLineBoxes: rail.totalTextLineBoxes,
  });
  check(contextReport, `${label}: 每个按钮 scrollHeight/scrollWidth 未溢出`, rail.buttons.every((button) => button.noScrollOverflow), {
    overflowing: rail.buttons.filter((button) => !button.noScrollOverflow),
  });
  check(contextReport, `${label}: 每个入口 hit target ≥ 28×28`, rail.buttons.every((button) => button.hitTargetOk), {
    bounds: rail.buttons.map((button) => ({ ariaLabel: button.ariaLabel, bounds: button.bounds })),
  });
  check(contextReport, `${label}: 每个入口恰好一枚 inline SVG`, rail.svgCountTotal === rail.buttonCount && rail.buttons.every((button) => button.svgCount === 1), {
    svgCountTotal: rail.svgCountTotal,
    buttonCount: rail.buttonCount,
  });
  check(contextReport, `${label}: 图标 aria-hidden + focusable=false + currentColor，且不用 title 替代名称`, rail.buttons.every((button) => (
    button.iconAriaHidden === "true" && button.iconFocusable === "false" && button.iconStroke === "currentColor" && button.title === null
  )), { icons: rail.buttons.map((button) => ({ ariaLabel: button.ariaLabel, iconAriaHidden: button.iconAriaHidden, iconFocusable: button.iconFocusable, iconStroke: button.iconStroke, title: button.title })) });
  check(contextReport, `${label}: aria-pressed 只出现在既有有状态入口`, (
    JSON.stringify(rail.buttonsWithoutPressed) === JSON.stringify(["Syn 命令"])
  ), { buttonsWithoutPressed: rail.buttonsWithoutPressed, pressedNames: rail.pressedNames });
  check(contextReport, `${label}: 活动栏整体无横向/纵向 overflow`, (
    rail.railScroll.scrollWidth <= rail.railScroll.clientWidth + 1 && rail.railScroll.scrollHeight <= rail.railScroll.clientHeight + 1
  ), rail.railScroll);
}

const browser = await chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});

try {
  // ---- 1. D1 1440 常态 -----------------------------------------------------
  await withFreshContext(browser, {
    name: "01-1440-activity-ribbon",
    viewport: { width: 1440, height: 900 },
    action: async (page, contextReport) => {
      const rail = await activityRailState(page);
      assertRibbon(contextReport, rail, "1440 D1");
      const focusRing = await page.evaluate(() => {
        const button = document.querySelector('[data-knowledge-region="activity"] button[aria-label="搜索"]');
        button.focus();
        const style = getComputedStyle(button);
        return {
          matchesFocusVisible: button.matches(":focus-visible"),
          outlineWidth: style.outlineWidth,
          outlineStyle: style.outlineStyle,
          activeAriaLabel: document.activeElement?.getAttribute("aria-label") ?? null,
        };
      });
      check(contextReport, "1440 D1: :focus-visible 有可见 outline", (
        focusRing.matchesFocusVisible && focusRing.outlineStyle !== "none" && Number.parseFloat(focusRing.outlineWidth) >= 1
      ), focusRing);
      const pressedAfterGraph = await (async () => {
        await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
        await page.waitForTimeout(300);
        return activityRailState(page);
      })();
      check(contextReport, "1440 D1: 点开关系图后 aria-pressed 与真实状态一致", (
        pressedAfterGraph.pressedNames.includes("关系图")
      ), { pressedNames: pressedAfterGraph.pressedNames });
      report.measurements.D1["1440"] = {
        railWidth: rail.railWidth,
        buttonCount: rail.buttonCount,
        svgCountTotal: rail.svgCountTotal,
        totalTextLineBoxes: rail.totalTextLineBoxes,
        accessibleNames: rail.accessibleNames,
        perButton: rail.buttons.map((button) => ({
          ariaLabel: button.ariaLabel,
          bounds: button.bounds,
          clientHeight: button.clientHeight,
          scrollHeight: button.scrollHeight,
          textLineBoxes: button.textLineBoxes,
          hitTargetOk: button.hitTargetOk,
          ariaPressed: button.ariaPressed,
        })),
        focusRing,
      };
      await page.screenshot({ path: fileURLToPath(new URL("./01-1440-activity-ribbon.png", rawDirectory)) });
      return { rail, focusRing, pressedAfterGraph: pressedAfterGraph.pressedNames };
    },
  });

  // ---- 2. D1 900 窄宽 + 折叠后恢复面板 --------------------------------------
  await withFreshContext(browser, {
    name: "02-900-activity-ribbon-narrow",
    viewport: { width: 900, height: 760 },
    action: async (page, contextReport) => {
      const rail = await activityRailState(page);
      assertRibbon(contextReport, rail, "900 D1");
      const activity = page.locator('[data-knowledge-region="activity"]');
      await activity.locator('button[aria-label="切换右侧上下文"]').click();
      await page.waitForTimeout(200);
      const rightCollapsed = await page.evaluate(() => document.querySelector(".syn-knowledge-shell").classList.contains("is-right-collapsed"));
      await activity.locator('button[aria-label="切换右侧上下文"]').click();
      await page.waitForTimeout(200);
      const rightRestored = await page.evaluate(() => !document.querySelector(".syn-knowledge-shell").classList.contains("is-right-collapsed"));
      check(contextReport, "900 D1: 上下文入口可折叠并恢复右栏", rightCollapsed && rightRestored, { rightCollapsed, rightRestored });
      await page.locator('[data-knowledge-region="left"] button[aria-label="折叠左侧栏"]').click();
      await page.waitForTimeout(200);
      const leftCollapsed = await page.evaluate(() => document.querySelector(".syn-knowledge-shell").classList.contains("is-left-collapsed"));
      await activity.locator('button[aria-label="搜索"]').click();
      await page.waitForTimeout(250);
      const leftRestoredToSearch = await page.evaluate(() => ({
        expanded: !document.querySelector(".syn-knowledge-shell").classList.contains("is-left-collapsed"),
        hasSearchPanel: Boolean(document.querySelector('[data-knowledge-region="left"] .syn-knowledge-search-panel')),
      }));
      check(contextReport, "900 D1: 折叠左栏后活动栏入口仍能恢复对应面板", (
        leftCollapsed && leftRestoredToSearch.expanded && leftRestoredToSearch.hasSearchPanel
      ), { leftCollapsed, leftRestoredToSearch });
      await activity.locator('button[aria-label="Canvas"]').click();
      await page.waitForTimeout(400);
      const canvasOpened = await page.evaluate(() => Boolean(document.querySelector('[data-active-group="true"] .native-knowledge-canvas')));
      check(contextReport, "900 D1: Canvas 入口仍打开 Canvas 面板", canvasOpened, { canvasOpened });
      report.measurements.D1["900"] = {
        railWidth: rail.railWidth,
        totalTextLineBoxes: rail.totalTextLineBoxes,
        wrappedButtons: rail.wrappedButtons,
        perButton: rail.buttons.map((button) => ({
          ariaLabel: button.ariaLabel,
          bounds: button.bounds,
          clientHeight: button.clientHeight,
          scrollHeight: button.scrollHeight,
          textLineBoxes: button.textLineBoxes,
          hitTargetOk: button.hitTargetOk,
        })),
        panelRestore: { rightCollapsed, rightRestored, leftCollapsed, leftRestoredToSearch, canvasOpened },
      };
      await page.screenshot({ path: fileURLToPath(new URL("./02-900-activity-ribbon-narrow.png", rawDirectory)) });
      return { rail, rightCollapsed, rightRestored, leftCollapsed, leftRestoredToSearch, canvasOpened };
    },
  });

  // ---- 3. D2 1440 三区块 + 折叠展开 + Tab 路径 ------------------------------
  await withFreshContext(browser, {
    name: "03-1440-right-context-sections",
    viewport: { width: 1440, height: 900 },
    action: async (page, contextReport) => {
      const initial = await rightSidebarState(page);
      check(contextReport, "1440 D2: 右栏恰好三个稳定区块", (
        initial.sectionCount === 3 && JSON.stringify(initial.sectionTitles) === JSON.stringify(contextSectionTitles)
      ), { sectionCount: initial.sectionCount, sectionTitles: initial.sectionTitles });
      check(contextReport, "1440 D2: 每区都有 aria-expanded 明确的折叠控件且 aria-controls 可解析", (
        initial.sectionsWithCollapseControl === 3 && initial.sections.every((section) => section.controlsResolves)
      ), initial.sections);
      check(contextReport, "1440 D2: 默认展开集合固定为三区全展开", (
        JSON.stringify(initial.expandedTitles) === JSON.stringify(contextSectionTitles)
      ), { expandedTitles: initial.expandedTitles });
      check(contextReport, "1440 D2: 右栏标题不再声明大纲，右栏全文零命中", (
        initial.headerText === "属性 / 反向引用 / 来源上下文" && !initial.headerDeclaresOutline && !initial.rightTextDeclaresOutline
      ), { headerText: initial.headerText, rightTextDeclaresOutline: initial.rightTextDeclaresOutline });
      check(contextReport, "1440 D2: 注入的 SyntheticSourceContext 被产品侧容器包住且内容逐字保留", (
        initial.sourceContextPresent
        && initial.sourceContextInsideProductContainer
        && initial.sourceContextItemCount === 28
        && initial.sourceContextFirstItem === "上下文量尺 01"
        && initial.sourceContextLastItem === "上下文量尺 28"
        && initial.sourceContextHeading === "尚未选择合成笔记"
      ), {
        sourceContextItemCount: initial.sourceContextItemCount,
        sourceContextFirstItem: initial.sourceContextFirstItem,
        sourceContextLastItem: initial.sourceContextLastItem,
        sourceContextHeading: initial.sourceContextHeading,
        sourceContextInsideProductContainer: initial.sourceContextInsideProductContainer,
      });
      check(contextReport, "1440 D2: 未选择文件的空态文案仍在", initial.emptyStateText, { emptyStateText: initial.emptyStateText });

      // 逐区折叠 → 展开
      const perSection = [];
      for (const title of contextSectionTitles) {
        const summary = page.locator(`.native-context-section[data-context-section="${title}"] .native-context-summary`);
        await summary.click();
        await page.waitForTimeout(150);
        const collapsed = await rightSidebarState(page);
        const collapsedSection = collapsed.sections.find((section) => section.title === title);
        await summary.click();
        await page.waitForTimeout(150);
        const expanded = await rightSidebarState(page);
        const expandedSection = expanded.sections.find((section) => section.title === title);
        perSection.push({ title, collapsed: collapsedSection, expanded: expandedSection });
        check(contextReport, `1440 D2: 「${title}」折叠后 aria-expanded=false、内容 display:none 且退出 Tab 路径`, (
          collapsedSection.ariaExpanded === "false"
          && collapsedSection.bodyHidden === true
          && collapsedSection.bodyDisplay === "none"
          && collapsedSection.bodyTabbableCount === 0
        ), collapsedSection);
        check(contextReport, `1440 D2: 「${title}」重新展开后 aria-expanded=true 且内容回到 Tab 路径`, (
          expandedSection.ariaExpanded === "true" && expandedSection.bodyHidden === false && expandedSection.bodyDisplay !== "none"
        ), expandedSection);
      }

      // 真实 Tab 走一遍：折叠「属性」后，从它的标题按 Tab 必须直接落到「反向引用」标题
      await page.locator('.native-context-section[data-context-section="属性"] .native-context-summary').click();
      await page.waitForTimeout(150);
      await page.locator('.native-context-section[data-context-section="属性"] .native-context-summary').focus();
      await page.keyboard.press("Tab");
      const tabTarget = await page.evaluate(() => {
        const active = document.activeElement;
        return {
          title: active?.closest(".native-context-section")?.getAttribute("data-context-section") ?? null,
          className: active?.className ?? null,
          text: active?.textContent?.trim().slice(0, 20) ?? null,
        };
      });
      check(contextReport, "1440 D2: 折叠区块的内容真的不在 Tab 路径上", tabTarget.title === "反向引用", tabTarget);
      await page.locator('.native-context-section[data-context-section="属性"] .native-context-summary').click();
      await page.waitForTimeout(150);

      const final = await rightSidebarState(page);
      report.measurements.D2["1440"] = {
        headerText: final.headerText,
        sectionTitles: final.sectionTitles,
        expandedTitles: final.expandedTitles,
        sourceContext: {
          itemCount: final.sourceContextItemCount,
          firstItem: final.sourceContextFirstItem,
          lastItem: final.sourceContextLastItem,
          insideProductContainer: final.sourceContextInsideProductContainer,
        },
        rightScroll: final.rightScroll,
        rightBounds: final.rightBounds,
        perSection,
        tabTarget,
      };
      await page.screenshot({ path: fileURLToPath(new URL("./03-1440-right-context-sections.png", rawDirectory)) });
      return { initial, perSection, tabTarget, final };
    },
  });

  // ---- 4. D2 900 双栏展开 + 折叠态 ------------------------------------------
  await withFreshContext(browser, {
    name: "04-900-right-context-collapsed",
    viewport: { width: 900, height: 760 },
    action: async (page, contextReport) => {
      const expanded = await rightSidebarState(page);
      check(contextReport, "900 D2: 双栏展开时右栏零横向 overflow", (
        expanded.rightScroll.scrollWidth <= expanded.rightScroll.clientWidth + 1
      ), expanded.rightScroll);
      check(contextReport, "900 D2: 纵向滚动限制在右栏容器内", (
        expanded.rightScroll.scrollHeight > expanded.rightScroll.clientHeight
      ), expanded.rightScroll);
      check(contextReport, "900 D2: 空态文案在窄宽下仍可见", expanded.emptyStateText, { emptyStateText: expanded.emptyStateText });
      check(contextReport, "900 D2: 三区块在窄宽下仍稳定", expanded.sectionCount === 3, { sectionTitles: expanded.sectionTitles });
      // 折叠「来源上下文」，量折叠态
      await page.locator('.native-context-section[data-context-section="来源上下文"] .native-context-summary').click();
      await page.waitForTimeout(200);
      const collapsed = await rightSidebarState(page);
      const collapsedSource = collapsed.sections.find((section) => section.title === "来源上下文");
      check(contextReport, "900 D2: 折叠来源上下文后内容退出 Tab/AT 路径，其余两区仍在", (
        collapsedSource.bodyHidden === true
        && collapsedSource.bodyTabbableCount === 0
        && collapsed.sectionCount === 3
        && collapsed.expandedTitles.length === 2
      ), { collapsedSource, expandedTitles: collapsed.expandedTitles });
      check(contextReport, "900 D2: 折叠后右栏仍零横向 overflow", (
        collapsed.rightScroll.scrollWidth <= collapsed.rightScroll.clientWidth + 1
      ), collapsed.rightScroll);
      report.measurements.D2["900"] = {
        expandedRightScroll: expanded.rightScroll,
        collapsedRightScroll: collapsed.rightScroll,
        collapsedSource,
        expandedTitles: collapsed.expandedTitles,
        emptyStateText: collapsed.emptyStateText,
      };
      await page.screenshot({ path: fileURLToPath(new URL("./04-900-right-context-collapsed.png", rawDirectory)) });
      return { expanded, collapsed };
    },
  });

  // ---- 5/6/7. D3 n = 6 / 40 / 512 -----------------------------------------
  for (const total of [6, 40, 512]) {
    await withFreshContext(browser, {
      name: `graph-n${total}`,
      viewport: { width: 1440, height: 900 },
      scenario: total === 6 ? null : `graph-scale-${total}`,
      action: async (page, contextReport) => {
        await openGraph(page);
        const fitView = await graphScaleState(page);
        check(contextReport, `D3 n=${total}: DOM 节点数与投影一致`, fitView.nodeCount === total, {
          nodeCount: fitView.nodeCount,
          expected: total,
        });
        check(contextReport, `D3 n=${total}: fitView 后零节点矩形相交`, fitView.intersectingPairs === 0, {
          intersectingPairs: fitView.intersectingPairs,
          examples: fitView.examples,
        });
        check(contextReport, `D3 n=${total}: fitView 后全部节点 bounds 在舞台内`, fitView.nodesInsideStage === fitView.nodeCount, {
          nodesInsideStage: fitView.nodesInsideStage,
          nodeCount: fitView.nodeCount,
          stageBounds: fitView.stageBounds,
        });
        check(contextReport, `D3 n=${total}: 任何缩放下 aria-label 完整`, (
          fitView.ariaLabelCount === fitView.nodeCount && (fitView.firstNodeAriaLabel ?? "").length > 0
        ), { ariaLabelCount: fitView.ariaLabelCount, nodeCount: fitView.nodeCount, firstNodeAriaLabel: fitView.firstNodeAriaLabel });

        const fitViewMeets28 = Boolean(
          fitView.measuredNodeBox
          && fitView.measuredNodeBox.width >= READABLE_NODE_BOX
          && fitView.measuredNodeBox.height >= READABLE_NODE_BOX,
        );
        if (total === 6) {
          check(contextReport, "D3 n=6: fitView 即达标缩放（节点可点、标题可见、未被裁切）", (
            fitViewMeets28 && fitView.titleVisible === true && fitView.titleClipped === false && fitView.zoomTier === "readable"
          ), {
            measuredNodeBox: fitView.measuredNodeBox,
            zoom: fitView.zoom,
            titleVisible: fitView.titleVisible,
            titleClipped: fitView.titleClipped,
            zoomTier: fitView.zoomTier,
          });
          check(contextReport, "D3 n=6: 6 节点 / 5 边全可见（R3D 不变量不回归）", (
            fitView.nodeCount === 6 && fitView.edgeCount === 5
          ), { nodeCount: fitView.nodeCount, edgeCount: fitView.edgeCount });
          check(contextReport, "D3 n=6: 节点盒仍是 136×40", (
            Math.abs(fitView.measuredNodeBox.width - 136 * fitView.zoom) < 2
            && Math.abs(fitView.measuredNodeBox.height - 40 * fitView.zoom) < 2
          ), { measuredNodeBox: fitView.measuredNodeBox, zoom: fitView.zoom });
        }

        // 俯瞰图必须拍在放大之前：它要佐证的是 fitView 的「全在舞台内 + 零相交」。
        if (total === 40) {
          await page.screenshot({ path: fileURLToPath(new URL("./05-1440-graph-n40.png", rawDirectory)) });
        }
        if (total === 512) {
          await page.screenshot({ path: fileURLToPath(new URL("./06-1440-graph-n512.png", rawDirectory)) });
        }

        // 俯瞰口径：大 n 允许 fitView 不达标；再实测放大到 readableZoom 后的达标情况。
        const readable = await zoomToReadable(page);
        check(contextReport, `D3 n=${total}: 放大到 readableZoom 后节点盒 ≥ 28×28`, readable.reached, {
          readableZoom: readable.readableZoom,
          clicks: readable.clicks,
          measuredNodeBox: readable.state.measuredNodeBox,
        });
        check(contextReport, `D3 n=${total}: readableZoom 下仍零相交`, readable.state.intersectingPairs === 0, {
          intersectingPairs: readable.state.intersectingPairs,
          examples: readable.state.examples,
        });
        check(contextReport, `D3 n=${total}: readableZoom 下 aria-label 仍完整`, (
          readable.state.ariaLabelCount === readable.state.nodeCount
        ), { ariaLabelCount: readable.state.ariaLabelCount, nodeCount: readable.state.nodeCount });

        report.measurements.D3[`n${total}`] = {
          fitViewZoom: fitView.zoom,
          fitViewNodeBox: fitView.measuredNodeBox,
          fitViewMeets28,
          fitViewTitleVisible: fitView.titleVisible,
          fitViewTitleClipped: fitView.titleClipped,
          fitViewZoomTier: fitView.zoomTier,
          fitViewIntersectingPairs: fitView.intersectingPairs,
          fitViewNodesInsideStage: fitView.nodesInsideStage,
          stageBounds: fitView.stageBounds,
          nodeCount: fitView.nodeCount,
          edgeCount: fitView.edgeCount,
          readableZoom: readable.readableZoom,
          readableZoomClicks: readable.clicks,
          readableNodeBox: readable.state.measuredNodeBox,
          readableTitleVisible: readable.state.titleVisible,
          readableZoomTier: readable.state.zoomTier,
          readableIntersectingPairs: readable.state.intersectingPairs,
          ariaLabelCount: readable.state.ariaLabelCount,
          firstNodeAriaLabel: fitView.firstNodeAriaLabel,
          zoomSteps: readable.steps,
          overviewOnly: total !== 6,
          scaleScenarioLimitation: total === 6 ? null : "规模场景只量布局：生成节点不激活、不断言可读性",
        };
        // 放大后的补图：佐证「达标缩放下节点可读、仍零相交」，与俯瞰图分开存。
        if (total === 40) {
          await page.screenshot({ path: fileURLToPath(new URL("./05b-1440-graph-n40-readable-zoom.png", rawDirectory)) });
        }
        if (total === 512) {
          await page.screenshot({ path: fileURLToPath(new URL("./06b-1440-graph-n512-readable-zoom.png", rawDirectory)) });
        }
        return { fitView, readable: { reached: readable.reached, readableZoom: readable.readableZoom, clicks: readable.clicks } };
      },
    });
  }

  // ---- 8. 900 reduced-motion ------------------------------------------------
  await withFreshContext(browser, {
    name: "08-900-reduced-motion",
    viewport: { width: 900, height: 760 },
    reducedMotion: "reduce",
    action: async (page, contextReport) => {
      const rail = await activityRailState(page);
      assertRibbon(contextReport, rail, "900 reduced-motion D1");
      await page.locator('.native-context-section[data-context-section="来源上下文"] .native-context-summary').click();
      await page.waitForTimeout(200);
      await openGraph(page);
      const motion = await page.evaluate(() => {
        const animated = [];
        const scope = [
          ...document.querySelectorAll('[data-knowledge-region="activity"] *'),
          ...document.querySelectorAll(".native-context-section, .native-context-section *"),
          ...document.querySelectorAll(".native-graph-flow-stage, .native-graph-node, .native-graph-node-button"),
        ];
        for (const element of scope) {
          const style = getComputedStyle(element);
          const hasTransition = style.transitionDuration !== "0s" && style.transitionProperty !== "none";
          const hasAnimation = style.animationName !== "none" && style.animationDuration !== "0s";
          if (hasTransition || hasAnimation) {
            animated.push({
              className: typeof element.className === "string" ? element.className : String(element.className?.baseVal ?? ""),
              transitionProperty: style.transitionProperty,
              transitionDuration: style.transitionDuration,
              animationName: style.animationName,
              animationDuration: style.animationDuration,
            });
          }
        }
        return { scanned: scope.length, animated };
      });
      check(contextReport, "900 reduced-motion: ribbon + 右栏折叠 + Graph 均零新增动画", motion.animated.length === 0, motion);
      const right = await rightSidebarState(page);
      const graph = await graphScaleState(page);
      check(contextReport, "900 reduced-motion: 折叠态与 Graph 均无横向 overflow", (
        right.rightScroll.scrollWidth <= right.rightScroll.clientWidth + 1
      ), right.rightScroll);
      check(contextReport, "900 reduced-motion: Graph 仍零相交", graph.intersectingPairs === 0, {
        intersectingPairs: graph.intersectingPairs,
      });
      report.measurements.D2.reducedMotion = {
        expandedTitles: right.expandedTitles,
        rightScroll: right.rightScroll,
        motionScanned: motion.scanned,
        animatedCount: motion.animated.length,
      };
      return { rail: { railWidth: rail.railWidth }, motion, right: right.rightScroll, graph: { intersectingPairs: graph.intersectingPairs } };
    },
  });
} finally {
  await browser.close();
}

report.outcome = report.failed === 0 ? "GREEN_ALL_ASSERTIONS_PASSED" : "GREEN_HAS_FAILURES";
await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`, "utf8");
console.log(JSON.stringify({
  outcome: report.outcome,
  assertions: report.assertions,
  failed: report.failed,
  failures: report.failures.slice(0, 8),
  contexts: report.contexts.map((context) => ({
    name: context.name,
    assertions: context.assertions.length,
    failed: context.assertions.filter((assertion) => !assertion.passed).length,
    inspectionError: context.inspectionError ?? null,
  })),
  D3: report.measurements.D3,
}, null, 2));
process.exitCode = report.failed === 0 ? 0 : 1;
