import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./red-browser-evidence.json", rawDirectory));

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

const report = {
  phase: "pre-implementation-red",
  fixture: "synthetic-only",
  purpose: "R3E D1/D2/D3 现状反例：文字活动栏 + 平铺右栏 + 恒定半轴环形布局",
  outcome: "PENDING",
  assertions: 0,
  failed: 0,
  redFindings: [],
  contexts: [],
};

function check(contextReport, name, condition, detail = {}) {
  const passed = Boolean(condition);
  report.assertions += 1;
  contextReport.assertions.push({ name, passed, ...(passed ? {} : { detail }) });
  if (!passed) {
    report.failed += 1;
    contextReport.failures.push({ name, detail });
  }
}

function redFinding(dimension, claim, observed) {
  report.redFindings.push({ dimension, claim, observed });
}

async function activityRailState(page) {
  return page.evaluate((expectedNames) => {
    const round = (value) => Math.round(value * 100) / 100;
    const rail = document.querySelector('[data-knowledge-region="activity"]');
    if (!rail) return null;
    const railStyle = getComputedStyle(rail);
    const buttons = [...rail.querySelectorAll("button")].map((button) => {
      const style = getComputedStyle(button);
      const rect = button.getBoundingClientRect();
      const lineHeight = Number.parseFloat(style.lineHeight);
      const fontSize = Number.parseFloat(style.fontSize);
      const paddingBlock = Number.parseFloat(style.paddingTop) + Number.parseFloat(style.paddingBottom);
      const borderBlock = Number.parseFloat(style.borderTopWidth) + Number.parseFloat(style.borderBottomWidth);
      const singleLineBudget = (Number.isFinite(lineHeight) ? lineHeight : fontSize * 1.4) + paddingBlock + borderBlock;
      // 真断行判据：数文本节点实际生成的行盒数量，不用 clientHeight 推断
      // （按钮有 min-height:32px，会把未断行的按钮也顶高，clientHeight 判据不可用）
      let textLineBoxes = 0;
      for (const child of button.childNodes) {
        if (child.nodeType !== Node.TEXT_NODE) continue;
        if (!child.textContent?.trim()) continue;
        const range = document.createRange();
        range.selectNodeContents(child);
        textLineBoxes += range.getClientRects().length;
      }
      return {
        textLineBoxes,
        textWraps: textLineBoxes > 1,
        ariaLabel: button.getAttribute("aria-label"),
        visibleText: button.textContent?.trim() ?? "",
        ariaPressed: button.getAttribute("aria-pressed"),
        className: button.className,
        bounds: { x: round(rect.x), y: round(rect.y), width: round(rect.width), height: round(rect.height) },
        clientHeight: button.clientHeight,
        scrollHeight: button.scrollHeight,
        clientWidth: button.clientWidth,
        scrollWidth: button.scrollWidth,
        fontSize,
        lineHeight: Number.isFinite(lineHeight) ? lineHeight : null,
        singleLineBudget: round(singleLineBudget),
        wrapped: textLineBoxes > 1,
        svgCount: button.querySelectorAll("svg").length,
        hitTargetOk: rect.width >= 28 && rect.height >= 28,
      };
    });
    const railRect = rail.getBoundingClientRect();
    return {
      railBounds: { x: round(railRect.x), y: round(railRect.y), width: round(railRect.width), height: round(railRect.height) },
      railWidth: round(railRect.width),
      railFlexDirection: railStyle.flexDirection,
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
      accessibleNamesMatchExpected: JSON.stringify(buttons.map((button) => button.ariaLabel)) === JSON.stringify(expectedNames),
      wrappedButtons: buttons.filter((button) => button.wrapped).map((button) => button.ariaLabel),
      totalTextLineBoxes: buttons.reduce((sum, button) => sum + button.textLineBoxes, 0),
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
    const right = document.querySelector('[data-knowledge-region="right"]');
    if (!right) return null;
    const sections = [...right.querySelectorAll("section")].map((section) => ({
      ariaLabel: section.getAttribute("aria-label"),
      className: section.className,
      bounds: bounds(section),
      directChildren: section.children.length,
      hasExpandedControl: Boolean(section.querySelector("[aria-expanded]")),
    }));
    const sourceContextSection = right.querySelector('section[aria-label="合成来源上下文"]');
    const sourceContextSpans = sourceContextSection
      ? [...sourceContextSection.querySelectorAll(".knowledge-document-detail > span")].map((span) => span.textContent?.trim() ?? "")
      : [];
    return {
      headerText: right.querySelector(".syn-knowledge-sidebar-tabs--right span")?.textContent?.trim() ?? null,
      headerDeclaresOutline: (right.querySelector(".syn-knowledge-sidebar-tabs--right span")?.textContent ?? "").includes("大纲"),
      ariaExpandedTotal: right.querySelectorAll("[aria-expanded]").length,
      ariaExpandedLabels: [...right.querySelectorAll("[aria-expanded]")].map((element) => element.getAttribute("aria-label")),
      sectionCount: sections.length,
      sections,
      sectionsWithCollapseControl: sections.filter((section) => section.hasExpandedControl).length,
      sourceContextPresent: Boolean(sourceContextSection),
      sourceContextItemCount: sourceContextSpans.length,
      sourceContextFirstItem: sourceContextSpans[0] ?? null,
      sourceContextLastItem: sourceContextSpans.at(-1) ?? null,
      sourceContextBounds: bounds(sourceContextSection),
      rightBounds: bounds(right),
      rightScroll: {
        clientHeight: right.clientHeight,
        scrollHeight: right.scrollHeight,
        clientWidth: right.clientWidth,
        scrollWidth: right.scrollWidth,
      },
      flatDirectChildren: right.children.length,
    };
  });
}

async function graphScaleState(page) {
  return page.evaluate(() => {
    const round = (value) => Math.round(value * 100) / 100;
    const graphRoot = document.querySelector('[data-active-group="true"] .native-knowledge-graph');
    if (!graphRoot) return null;
    const viewport = graphRoot.querySelector(".react-flow__viewport");
    const transform = viewport ? getComputedStyle(viewport).transform : "none";
    let zoom = null;
    if (transform && transform !== "none") {
      const parts = transform.replace(/matrix\(|\)/g, "").split(",").map((value) => Number.parseFloat(value));
      if (parts.length >= 4 && Number.isFinite(parts[0])) zoom = round(parts[0]);
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
    const stage = graphRoot.querySelector(".react-flow");
    const stageRect = stage?.getBoundingClientRect() ?? null;
    const insideStage = stageRect
      ? nodes.filter((node) => (
        node.x >= stageRect.x - 2
        && node.y >= stageRect.y - 2
        && node.right <= stageRect.right + 2
        && node.bottom <= stageRect.bottom + 2
      )).length
      : null;
    const firstButton = graphRoot.querySelector(".native-graph-node-button");
    const firstTitle = firstButton?.querySelector("strong") ?? null;
    return {
      nodeCount: nodes.length,
      edgeCount: graphRoot.querySelectorAll(".react-flow__edge").length,
      zoom,
      intersectingPairs,
      examples,
      stageBounds: stageRect
        ? { x: round(stageRect.x), y: round(stageRect.y), width: round(stageRect.width), height: round(stageRect.height) }
        : null,
      nodesInsideStage: insideStage,
      nodeBoxSample: nodes.slice(0, 3),
      firstNodeAriaLabel: firstButton?.getAttribute("aria-label") ?? null,
      firstTitleVisible: firstTitle ? getComputedStyle(firstTitle).display !== "none" : null,
    };
  });
}

async function openGraph(page) {
  await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
  const root = page.locator('[data-active-group="true"] .native-knowledge-graph');
  await root.waitFor();
  await root.locator(".native-graph-node-button").first().waitFor();
  await page.waitForTimeout(600);
}

async function withFreshContext(browser, { name, viewport, scenario = null, action }) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = { name, viewport, scenario, assertions: [], failures: [], evidence: {} };
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
    contextReport.audit = {
      externalRequestCount: externalRequests.length,
      consoleErrorCount: consoleErrors.length,
      pageErrorCount: pageErrors.length,
      externalRequests,
      consoleErrors,
      pageErrors,
    };
  } catch (error) {
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
  await withFreshContext(browser, {
    name: "red-01-1440-activity-rail",
    viewport: { width: 1440, height: 900 },
    action: async (page, contextReport) => {
      const rail = await activityRailState(page);
      check(contextReport, "RED D1: 活动栏入口为紧凑 icon（现状应失败）", rail.svgCountTotal >= rail.buttonCount, rail);
      check(contextReport, "RED D1: 活动栏按钮不显示中文文字（现状应失败）", rail.buttons.every((button) => button.visibleText === ""), {
        visibleTexts: rail.buttons.map((button) => button.visibleText),
      });
      check(contextReport, "RED D1: 活动栏宽度在 36-48px 区间（现状应失败）", rail.railWidth >= 36 && rail.railWidth <= 48, {
        railWidth: rail.railWidth,
      });
      redFinding("D1", "1440 下活动栏是中文文字按钮、零 inline SVG", {
        railWidth: rail.railWidth,
        buttonCount: rail.buttonCount,
        svgCountTotal: rail.svgCountTotal,
        visibleTexts: rail.buttons.map((button) => button.visibleText),
        accessibleNames: rail.accessibleNames,
        perButton: rail.buttons.map((button) => ({
          ariaLabel: button.ariaLabel,
          bounds: button.bounds,
          clientHeight: button.clientHeight,
          scrollHeight: button.scrollHeight,
          fontSize: button.fontSize,
          singleLineBudget: button.singleLineBudget,
          wrapped: button.wrapped,
        })),
      });
      await page.screenshot({ path: fileURLToPath(new URL("./red-01-1440-activity-rail.png", rawDirectory)) });
      return { rail };
    },
  });

  await withFreshContext(browser, {
    name: "red-02-900-activity-rail-wrap",
    viewport: { width: 900, height: 760 },
    action: async (page, contextReport) => {
      const rail = await activityRailState(page);
      check(contextReport, "RED D1: 900 下八入口零断行（现状应失败）", rail.wrappedButtons.length === 0, {
        wrappedButtons: rail.wrappedButtons,
      });
      check(contextReport, "RED D1: 900 下活动栏宽度 36-48px（现状应失败）", rail.railWidth >= 36 && rail.railWidth <= 48, {
        railWidth: rail.railWidth,
      });
      redFinding("D1", "900 下活动栏文字按钮断行", {
        railWidth: rail.railWidth,
        wrappedButtons: rail.wrappedButtons,
        perButton: rail.buttons.map((button) => ({
          ariaLabel: button.ariaLabel,
          visibleText: button.visibleText,
          bounds: button.bounds,
          clientHeight: button.clientHeight,
          scrollHeight: button.scrollHeight,
          fontSize: button.fontSize,
          lineHeight: button.lineHeight,
          singleLineBudget: button.singleLineBudget,
          wrapped: button.wrapped,
        })),
      });
      await page.screenshot({ path: fileURLToPath(new URL("./red-02-900-activity-rail-wrap.png", rawDirectory)) });
      return { rail };
    },
  });

  await withFreshContext(browser, {
    name: "red-03-1440-right-context-flat",
    viewport: { width: 1440, height: 900 },
    action: async (page, contextReport) => {
      const right = await rightSidebarState(page);
      check(contextReport, "RED D2: 右栏每个区块有折叠控件（现状应失败）", right.sectionsWithCollapseControl >= 3, right);
      check(contextReport, "RED D2: 右栏标题不再声明大纲（现状应失败）", right.headerDeclaresOutline === false, {
        headerText: right.headerText,
      });
      redFinding("D2", "右栏为平铺长列表：零区块折叠控件、标题挂空大纲", {
        headerText: right.headerText,
        headerDeclaresOutline: right.headerDeclaresOutline,
        ariaExpandedTotal: right.ariaExpandedTotal,
        ariaExpandedLabels: right.ariaExpandedLabels,
        sectionCount: right.sectionCount,
        sectionsWithCollapseControl: right.sectionsWithCollapseControl,
        sections: right.sections,
        sourceContextItemCount: right.sourceContextItemCount,
        sourceContextBounds: right.sourceContextBounds,
        rightBounds: right.rightBounds,
        rightScroll: right.rightScroll,
      });
      await page.screenshot({ path: fileURLToPath(new URL("./red-03-1440-right-context-flat.png", rawDirectory)) });
      return { right };
    },
  });

  for (const total of [6, 40, 512]) {
    await withFreshContext(browser, {
      name: `red-04-1440-graph-n${total}`,
      viewport: { width: 1440, height: 900 },
      scenario: total === 6 ? null : `graph-scale-${total}`,
      action: async (page, contextReport) => {
        await openGraph(page);
        const graph = await graphScaleState(page);
        check(contextReport, `RED D3: n=${total} 零节点矩形相交（现状 n>6 应失败）`, graph.intersectingPairs === 0, {
          nodeCount: graph.nodeCount,
          intersectingPairs: graph.intersectingPairs,
          examples: graph.examples,
        });
        check(contextReport, `RED D3: n=${total} 全部节点在舞台内（现状 n>6 应失败）`, graph.nodesInsideStage === graph.nodeCount, {
          nodeCount: graph.nodeCount,
          nodesInsideStage: graph.nodesInsideStage,
        });
        redFinding("D3", `n=${total} 浏览器实测节点矩形相交对数`, {
          nodeCount: graph.nodeCount,
          edgeCount: graph.edgeCount,
          zoom: graph.zoom,
          intersectingPairs: graph.intersectingPairs,
          nodesInsideStage: graph.nodesInsideStage,
          stageBounds: graph.stageBounds,
          nodeBoxSample: graph.nodeBoxSample,
          examples: graph.examples,
        });
        if (total !== 6) {
          await page.screenshot({ path: fileURLToPath(new URL(`./red-04-1440-graph-n${total}.png`, rawDirectory)) });
        }
        return { graph };
      },
    });
  }
} finally {
  await browser.close();
}

report.outcome = report.failed > 0 ? "RED_ESTABLISHED" : "RED_NOT_ESTABLISHED";
await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`, "utf8");
console.log(JSON.stringify({
  outcome: report.outcome,
  assertions: report.assertions,
  failed: report.failed,
  contexts: report.contexts.map((context) => ({
    name: context.name,
    failed: context.failures.length,
    inspectionError: context.inspectionError ?? null,
  })),
}, null, 2));
