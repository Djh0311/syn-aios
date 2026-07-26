// N2R-R4 GREEN：改后量尺。纯合成夹具 + 真实 React + 真实生产 CSS，每场景 fresh context。
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
const railNames = ["文件", "搜索", "关系图", "Canvas", "Syn 命令", "设置与维护", "来源", "切换右侧上下文"];

// R0 §2.3 参考几何（逐条读自 R0 原文）。R0 自身写明"不应按比例盲目拉伸"，
// 因此这些是参照带，不是像素契约。
const R0 = {
  activityRailWidth: 42,
  leftSidebarExpanded: 288,
  rightSidebar: 185,
  centralStatusHeight: 26,
  centralChrome: 39 + 35,
  bodyFontSize: 16,
};
// 改前 chrome 字号（取自本包 red-browser-evidence.json，1440 档）
const RED_CHROME = {
  activityButton: 10,
  leftTree: 13,
  leftSidebarTab: 11,
  centralTab: 13,
  rightSectionSummary: 10,
  rightSectionBody: 13,
  rightHeader: 11,
  statusBar: 10,
};

const report = {
  phase: "post-implementation-green",
  fixture: "synthetic-only",
  scope: "D1 骨架对照 / D2 正文字号 / D3 活动栏悬停提示",
  notRealApp: "NOT_REAL_APP：只跑合成夹具，不进真实 App / store / vault",
  r0Reference: R0,
  redChromeBaseline: RED_CHROME,
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
  const outside = Object.keys(mock.callsByCommand).filter((command) => !readAllowlist.includes(command));
  check(contextReport, `${contextReport.name}: command 全在精确 read allowlist 内`, outside.length === 0, {
    callsByCommand: mock.callsByCommand, outside,
  });
  check(contextReport, `${contextReport.name}: 写/未知 command 为 0`,
    mock.writeCallCount === 0 && mock.unrecognizedCallCount === 0, mock);
  check(contextReport, `${contextReport.name}: mount 前 localStorage 为空`,
    metrics.localStorage.emptyBeforeMount === true, metrics.localStorage);
  check(contextReport, `${contextReport.name}: localStorage 只含既有可丢弃 UI chrome 偏好`,
    metrics.localStorage.keys.every((key) => key === preferenceKey), metrics.localStorage);
  return { callsByCommand: mock.callsByCommand, localStorage: metrics.localStorage };
}

async function skeletonState(page) {
  return page.evaluate(() => {
    const round = (value) => Math.round(value * 100) / 100;
    const box = (selector) => {
      const element = document.querySelector(selector);
      if (!element) return null;
      const rect = element.getBoundingClientRect();
      const style = getComputedStyle(element);
      return {
        width: round(rect.width),
        height: round(rect.height),
        borderTopWidth: style.borderTopWidth,
        borderRightWidth: style.borderRightWidth,
        borderBottomWidth: style.borderBottomWidth,
        borderLeftWidth: style.borderLeftWidth,
      };
    };
    const scroll = (selector) => {
      const element = document.querySelector(selector);
      if (!element) return null;
      return {
        clientWidth: element.clientWidth,
        scrollWidth: element.scrollWidth,
        overflowsHorizontally: element.scrollWidth > element.clientWidth + 1,
      };
    };
    const shell = document.querySelector(".syn-knowledge-shell");
    const activity = box('[data-knowledge-region="activity"]');
    const left = box('[data-knowledge-region="left"]');
    const central = box('[data-knowledge-region="central"]');
    const right = box('[data-knowledge-region="right"]');
    const status = box('[data-knowledge-region="status"]');
    const groupHeader = box(".knowledge-workbench-group__header");
    const documentHead = box('[data-active-group="true"] .native-workspace-document-head');
    const separator = box(".knowledge-workbench-separator");
    const leftFooter = box('[data-knowledge-region="left"] footer');
    // 文字截断扫描：任何可见文本节点容器 scrollWidth 明显超过 clientWidth
    const truncated = [...document.querySelectorAll(
      '[data-knowledge-region] button, [data-knowledge-region] span, [data-knowledge-region] strong, .native-workspace-markdown p',
    )].filter((element) => element.scrollWidth > element.clientWidth + 1 && element.clientWidth > 0)
      .slice(0, 8)
      .map((element) => ({
        text: element.textContent?.trim().slice(0, 24) ?? "",
        clientWidth: element.clientWidth,
        scrollWidth: element.scrollWidth,
        className: typeof element.className === "string" ? element.className : "",
      }));
    return {
      shellGridColumns: shell ? getComputedStyle(shell).gridTemplateColumns : null,
      activity, left, central, right, status, groupHeader, documentHead, separator,
      leftFooterPresent: leftFooter !== null,
      centralChromeTotal: groupHeader && documentHead ? round(groupHeader.height + documentHead.height) : (groupHeader?.height ?? null),
      ratio: activity && left && central && right
        ? { left: left.width, central: central.width, right: right.width, order: left.width < central.width && right.width < left.width }
        : null,
      truncated,
      overflow: {
        documentElement: scroll("html"),
        body: scroll("body"),
        shell: scroll(".syn-knowledge-shell"),
        central: scroll('[data-knowledge-region="central"]'),
        activeGroupPanel: scroll('[data-active-group="true"] [data-knowledge-group-panel="active"]'),
        left: scroll('[data-knowledge-region="left"]'),
        right: scroll('[data-knowledge-region="right"]'),
      },
    };
  });
}

async function typographyState(page) {
  return page.evaluate(() => {
    const size = (selector) => {
      const element = document.querySelector(selector);
      if (!element) return null;
      const style = getComputedStyle(element);
      return {
        fontSize: Number.parseFloat(style.fontSize),
        lineHeightRatio: style.lineHeight === "normal"
          ? null
          : Math.round((Number.parseFloat(style.lineHeight) / Number.parseFloat(style.fontSize)) * 100) / 100,
      };
    };
    return {
      readingBody: size(".native-workspace-markdown"),
      readingParagraph: size(".native-workspace-markdown p"),
      readingHeading: size(".native-workspace-markdown h1, .native-workspace-markdown h2, .native-workspace-markdown h3"),
      readingListItem: size(".native-workspace-markdown li"),
      readingPre: size(".native-workspace-markdown pre"),
      readingInlineCode: size(".native-workspace-markdown code"),
      editorTextarea: size(".native-workspace-source textarea"),
      chrome: {
        activityButton: size('[data-knowledge-region="activity"] button'),
        leftTree: size('[data-knowledge-region="left"] .native-workspace-tree button'),
        leftSidebarTab: size('[data-knowledge-region="left"] .syn-knowledge-sidebar-tabs button'),
        centralTab: size(".knowledge-workbench-tab"),
        rightSectionSummary: size(".native-context-summary"),
        rightSectionBody: size(".native-context-body"),
        rightHeader: size(".syn-knowledge-sidebar-tabs--right span"),
        statusBar: size('[data-knowledge-region="status"]'),
        graphNodeTitle: size(".native-graph-node-button strong"),
      },
    };
  });
}

async function railState(page) {
  return page.evaluate(() => {
    const rail = document.querySelector('[data-knowledge-region="activity"]');
    return [...rail.querySelectorAll("button")].map((button) => ({
      ariaLabel: button.getAttribute("aria-label"),
      title: button.getAttribute("title"),
      ariaPressed: button.getAttribute("aria-pressed"),
      svgTitleCount: button.querySelectorAll("svg title").length,
      iconAriaHidden: button.querySelector("svg")?.getAttribute("aria-hidden") ?? null,
      iconFocusable: button.querySelector("svg")?.getAttribute("focusable") ?? null,
    }));
  });
}

/**
 * 用 CDP 的完整无障碍树取**浏览器真实计算出来**的可访问名称与它的来源，
 * 而不是读属性自己推断。这样才能证明 title 是附加、没有顶替 aria-label。
 */
async function accessibleNameSources(context, page) {
  const cdp = await context.newCDPSession(page);
  await cdp.send("Accessibility.enable");
  const { nodes } = await cdp.send("Accessibility.getFullAXTree");
  await cdp.detach();
  return nodes
    .filter((node) => node.role?.value === "button" && node.name?.value)
    .map((node) => {
      const sources = node.name.sources ?? [];
      const effective = sources.find((source) => source.value && !source.superseded);
      return {
        name: node.name.value,
        effectiveSource: effective?.attribute ?? effective?.type ?? null,
        allSources: sources
          .filter((source) => source.value)
          .map((source) => ({ type: source.type, attribute: source.attribute ?? null, superseded: Boolean(source.superseded) })),
      };
    });
}

async function openFirstMarkdown(page) {
  const entry = page.locator('[data-knowledge-region="left"] .native-workspace-tree button').first();
  await entry.waitFor();
  await entry.click();
  await page.locator(".native-workspace-source textarea").first().waitFor();
  await page.waitForTimeout(300);
}

async function switchToReading(page) {
  await page.locator('[data-active-group="true"] .knowledge-workbench-projection-controls button', { hasText: "预览" }).first().click();
  await page.locator(".native-workspace-markdown").first().waitFor();
  await page.waitForTimeout(300);
}

async function withFreshContext(browser, { name, viewport, action }) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
  // action 需要 context 才能开 CDP session，透传下去
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = { name, viewport, assertions: [], evidence: {} };
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
    contextReport.evidence = await action(page, contextReport, context);
    const metrics = await fixtureMetrics(page);
    contextReport.audit = {
      ...auditMetrics(contextReport, metrics),
      externalRequestCount: externalRequests.length,
      consoleErrorCount: consoleErrors.length,
      pageErrorCount: pageErrors.length,
      externalRequests, consoleErrors, pageErrors,
    };
    check(contextReport, `${name}: 外部请求 / console error / page error 三项零值`,
      externalRequests.length === 0 && consoleErrors.length === 0 && pageErrors.length === 0,
      { externalRequests, consoleErrors, pageErrors });
  } catch (error) {
    contextReport.inspectionError = String(error?.stack ?? error);
    report.failed += 1;
    report.failures.push({ context: name, name: "context threw", detail: contextReport.inspectionError });
  } finally {
    report.contexts.push(contextReport);
    await context.close();
  }
}

function assertSkeleton(contextReport, skeleton, label) {
  check(contextReport, `${label} D1: 活动栏宽 = 42（定值）`,
    skeleton.activity.width === R0.activityRailWidth, { measured: skeleton.activity.width });
  check(contextReport, `${label} D1: 中央底部状态区高 = 26（定值）`,
    skeleton.status.height === R0.centralStatusHeight, { measured: skeleton.status.height });
  check(contextReport, `${label} D1: 左栏展开宽落在 220–288 带内`,
    skeleton.left.width >= 220 && skeleton.left.width <= 288, { measured: skeleton.left.width });
  check(contextReport, `${label} D1: 右栏宽落在 185–240 带内`,
    skeleton.right.width >= R0.rightSidebar && skeleton.right.width <= 240, { measured: skeleton.right.width });
  check(contextReport, `${label} D1: 侧栏秩序为「中央最宽 > 左 > 右」`,
    skeleton.ratio.order === true, { ratio: skeleton.ratio });
  check(contextReport, `${label} D1: 各层零横向 overflow`,
    Object.values(skeleton.overflow).every((entry) => !entry || !entry.overflowsHorizontally), skeleton.overflow);
  check(contextReport, `${label} D1: 零文字截断`, skeleton.truncated.length === 0, { truncated: skeleton.truncated });
}

const browser = await chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});

try {
  // ---- 1 / 2：1440 与 1180 全景（阅读态） ---------------------------------
  for (const [label, viewport, shot] of [
    ["1440", { width: 1440, height: 900 }, "01-1440-final-alignment.png"],
    ["1180", { width: 1180, height: 760 }, "02-1180-final-alignment.png"],
  ]) {
    await withFreshContext(browser, {
      name: `${label}-final-alignment`,
      viewport,
      action: async (page, contextReport) => {
        await openFirstMarkdown(page);
        await switchToReading(page);
        const skeleton = await skeletonState(page);
        const typography = await typographyState(page);
        assertSkeleton(contextReport, skeleton, label);
        check(contextReport, `${label} D2: 阅读正文 computed font-size = 16px`,
          typography.readingBody.fontSize === R0.bodyFontSize, { readingBody: typography.readingBody });
        check(contextReport, `${label} D2: 阅读正文行高比落在 1.5–1.8`,
          typography.readingBody.lineHeightRatio >= 1.5 && typography.readingBody.lineHeightRatio <= 1.8,
          { lineHeightRatio: typography.readingBody.lineHeightRatio });
        check(contextReport, `${label} D2: chrome 各级字号与改前逐项相同`,
          Object.entries(RED_CHROME).every(([key, value]) => typography.chrome[key]?.fontSize === value),
          { before: RED_CHROME, after: Object.fromEntries(Object.entries(typography.chrome).map(([k, v]) => [k, v?.fontSize ?? null])) });
        check(contextReport, `${label} D2: 内联 code 不得与正文倒挂`,
          typography.readingInlineCode === null || typography.readingInlineCode.fontSize <= typography.readingBody.fontSize,
          { code: typography.readingInlineCode, body: typography.readingBody });

        report.measurements.D1[label] = {
          shellGridColumns: skeleton.shellGridColumns,
          activityWidth: skeleton.activity.width,
          leftWidth: skeleton.left.width,
          centralWidth: skeleton.central.width,
          rightWidth: skeleton.right.width,
          statusHeight: skeleton.status.height,
          groupHeaderHeight: skeleton.groupHeader?.height ?? null,
          documentHeadHeight: skeleton.documentHead?.height ?? null,
          centralChromeTotal: skeleton.centralChromeTotal,
          centralChromeR0: R0.centralChrome,
          centralChromeVerdict: Math.abs(skeleton.centralChromeTotal - R0.centralChrome) <= 10
            ? "在带内" : "超出参照带（未修：所需 selector 不在 §4.2 白名单）",
          leftFooterPresent: skeleton.leftFooterPresent,
          ratio: skeleton.ratio,
          truncated: skeleton.truncated,
          overflow: skeleton.overflow,
        };
        report.measurements.D2[label] = {
          readingBody: typography.readingBody,
          readingParagraph: typography.readingParagraph,
          readingHeading: typography.readingHeading,
          readingListItem: typography.readingListItem,
          readingPre: typography.readingPre,
          readingInlineCode: typography.readingInlineCode,
          chromeBefore: RED_CHROME,
          chromeAfter: Object.fromEntries(Object.entries(typography.chrome).map(([k, v]) => [k, v?.fontSize ?? null])),
        };
        await page.screenshot({ path: fileURLToPath(new URL(`./${shot}`, rawDirectory)) });
        if (label === "1440") {
          await page.screenshot({ path: fileURLToPath(new URL("./03-1440-reading-16px.png", rawDirectory)) });
        }
        return { skeleton, typography };
      },
    });
  }

  // ---- 3：1440 编辑态 -----------------------------------------------------
  await withFreshContext(browser, {
    name: "1440-editing",
    viewport: { width: 1440, height: 900 },
    action: async (page, contextReport) => {
      await openFirstMarkdown(page);
      const typography = await typographyState(page);
      check(contextReport, "1440 D2: 编辑正文 computed font-size = 16px",
        typography.editorTextarea.fontSize === R0.bodyFontSize, { editorTextarea: typography.editorTextarea });
      check(contextReport, "1440 D2: 编辑正文行高比落在 1.5–1.8",
        typography.editorTextarea.lineHeightRatio >= 1.5 && typography.editorTextarea.lineHeightRatio <= 1.8,
        { lineHeightRatio: typography.editorTextarea.lineHeightRatio });
      // 光标可用 + 保存语义未变：真实键入一个字符再看状态栏与保存按钮
      const textarea = page.locator(".native-workspace-source textarea").first();
      await textarea.click();
      await textarea.press("End");
      await textarea.type("。");
      await page.waitForTimeout(250);
      const editing = await page.evaluate(() => {
        const area = document.querySelector(".native-workspace-source textarea");
        const save = [...document.querySelectorAll("button")].find((button) => button.textContent?.trim() === "保存 Markdown");
        return {
          focused: document.activeElement === area,
          selectionStart: area.selectionStart,
          valueEndsWithTyped: area.value.endsWith("。"),
          saveButtonPresent: Boolean(save),
          saveButtonDisabled: save?.disabled ?? null,
          statusText: document.querySelector('[data-knowledge-region="status"]')?.textContent ?? "",
        };
      });
      check(contextReport, "1440 D2: 提字号后光标仍可用、键入生效", (
        editing.focused && editing.valueEndsWithTyped && editing.selectionStart > 0
      ), editing);
      check(contextReport, "1440 D2: 保存/草稿语义未变（保存按钮在、状态栏仍报草稿态）", (
        editing.saveButtonPresent && editing.statusText.includes("草稿")
      ), editing);
      const skeleton = await skeletonState(page);
      check(contextReport, "1440 编辑态: 零横向 overflow",
        Object.values(skeleton.overflow).every((entry) => !entry || !entry.overflowsHorizontally), skeleton.overflow);
      report.measurements.D2.editing = { ...typography.editorTextarea, editing };
      return { typography, editing };
    },
  });

  // ---- 4：1440 活动栏悬停提示 ---------------------------------------------
  await withFreshContext(browser, {
    name: "1440-activity-tooltip",
    viewport: { width: 1440, height: 900 },
    action: async (page, contextReport, context) => {
      const rail = await railState(page);
      const axEntries = await accessibleNameSources(context, page);
      const railAx = axEntries.filter((entry) => railNames.includes(entry.name));
      const axNames = axEntries.map((entry) => entry.name);
      await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').hover();
      await page.waitForTimeout(200);
      const hovered = await page.evaluate(() => {
        const button = document.querySelector('[data-knowledge-region="activity"] button[aria-label="关系图"]');
        return { title: button.getAttribute("title"), ariaLabel: button.getAttribute("aria-label") };
      });
      check(contextReport, "D3: 8 个入口 title 与 aria-label 逐字同值",
        rail.length === 8 && rail.every((button) => button.title !== null && button.title === button.ariaLabel),
        { rail: rail.map((b) => ({ ariaLabel: b.ariaLabel, title: b.title })) });
      check(contextReport, "D3: title 值即八个既有可访问名称，顺序不变",
        JSON.stringify(rail.map((b) => b.title)) === JSON.stringify(railNames),
        { titles: rail.map((b) => b.title) });
      check(contextReport, "D3: 浏览器算出的可访问名称逐字等于 aria-label，且生效来源就是 aria-label",
        railAx.length === 8
          && railAx.every((entry) => entry.effectiveSource === "aria-label")
          && JSON.stringify(railAx.map((entry) => entry.name).sort()) === JSON.stringify([...railNames].sort()),
        { railAx });
      check(contextReport, "D3: title 只作附加来源，被 aria-label 压过而不是叠加（无双读）",
        railAx.every((entry) => {
          const titleSource = entry.allSources.find((source) => source.attribute === "title");
          return titleSource === undefined || titleSource.superseded === true;
        }),
        { railAx });
      check(contextReport, "D3: svg 仍 aria-hidden / focusable=false 且内部无 <title>",
        rail.every((button) => button.iconAriaHidden === "true" && button.iconFocusable === "false" && button.svgTitleCount === 0),
        { rail });
      check(contextReport, "D3: aria-pressed 行为零变化（初始 2 按下 / 5 未按下 / Syn 命令无该属性）",
        rail.filter((b) => b.ariaPressed === "true").length === 2
          && rail.filter((b) => b.ariaPressed === "false").length === 5
          && rail.filter((b) => b.ariaPressed === null).length === 1,
        { pressed: rail.map((b) => ({ ariaLabel: b.ariaLabel, ariaPressed: b.ariaPressed })) });
      // 键盘路径未变
      await page.locator('[data-knowledge-region="activity"] button[aria-label="搜索"]').focus();
      await page.keyboard.press("Enter");
      await page.waitForTimeout(250);
      const keyboard = await page.evaluate(() => ({
        searchPanelOpen: Boolean(document.querySelector('[data-knowledge-region="left"] .syn-knowledge-search-panel')),
        activeAriaLabel: document.activeElement?.getAttribute("aria-label") ?? null,
      }));
      check(contextReport, "D3: 键盘 Enter 激活路径未变", keyboard.searchPanelOpen === true, keyboard);
      report.measurements.D3 = {
        rail: rail.map((b) => ({ ariaLabel: b.ariaLabel, title: b.title, same: b.title === b.ariaLabel, ariaPressed: b.ariaPressed })),
        accessibleNamesFromAxTree: railAx,
        hovered,
        keyboard,
      };
      await page.screenshot({ path: fileURLToPath(new URL("./04-1440-activity-tooltip.png", rawDirectory)) });
      return { rail, axNames, hovered, keyboard };
    },
  });

  // ---- 5：1180 双栏分栏（Markdown + Graph） --------------------------------
  await withFreshContext(browser, {
    name: "1180-split-after-typography",
    viewport: { width: 1180, height: 760 },
    action: async (page, contextReport) => {
      await openFirstMarkdown(page);
      await switchToReading(page);
      await page.locator('[data-active-group="true"] .knowledge-workbench-group__tools button', { hasText: "分栏" }).first().click();
      await page.waitForTimeout(400);
      await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
      await page.locator(".native-graph-node-button").first().waitFor();
      await page.waitForTimeout(600);
      const skeleton = await skeletonState(page);
      const typography = await typographyState(page);
      const groups = await page.evaluate(() => document.querySelectorAll("[data-knowledge-tab-group]").length);
      check(contextReport, "1180 分栏: 真实两个标签组", groups === 2, { groups });
      check(contextReport, "1180 分栏: 提字号后仍零横向 overflow",
        Object.values(skeleton.overflow).every((entry) => !entry || !entry.overflowsHorizontally), skeleton.overflow);
      check(contextReport, "1180 分栏: 分隔器为单像素 hairline（无双线/粗边）",
        skeleton.separator !== null
          && Number.parseFloat(skeleton.separator.borderLeftWidth) <= 1
          && Number.parseFloat(skeleton.separator.borderRightWidth) <= 1,
        { separator: skeleton.separator });
      check(contextReport, "1180 分栏: Graph 节点标题字号未被正文档影响（仍 12px）",
        typography.chrome.graphNodeTitle?.fontSize === 12, { graphNodeTitle: typography.chrome.graphNodeTitle });
      // 在这套排布里 Graph 顶掉了本组的阅读面，仍在场的 Markdown 面是另一组的编辑面。
      // 断言落在「仍在场的正文面」上，并把这条排布事实一起记进证据。
      const survivingBody = typography.readingBody ?? typography.editorTextarea;
      check(contextReport, "1180 分栏: 仍在场的 Markdown 正文面仍 16px",
        survivingBody?.fontSize === R0.bodyFontSize,
        { survivingBody, readingBody: typography.readingBody, editorTextarea: typography.editorTextarea });
      report.measurements.D1.split1180 = {
        groups,
        separator: skeleton.separator,
        overflow: skeleton.overflow,
        graphNodeTitle: typography.chrome.graphNodeTitle,
        readingBody: typography.readingBody,
        editorTextarea: typography.editorTextarea,
        arrangementNote: "分栏后点开关系图会顶掉本组阅读面；仍在场的 Markdown 正文面是另一组的编辑面",
      };
      await page.screenshot({ path: fileURLToPath(new URL("./05-1180-split-after-typography.png", rawDirectory)) });
      return { skeleton, typography, groups };
    },
  });

  // ---- 6：900 不回归 -------------------------------------------------------
  await withFreshContext(browser, {
    name: "900-no-regression",
    viewport: { width: 900, height: 760 },
    action: async (page, contextReport) => {
      await openFirstMarkdown(page);
      await switchToReading(page);
      await page.locator('[data-knowledge-region="left"] button[aria-label="折叠左侧栏"]').click();
      await page.waitForTimeout(200);
      const rightToggle = page.locator('[data-knowledge-region="activity"] button[aria-label="切换右侧上下文"]');
      if (await rightToggle.getAttribute("aria-pressed") === "true") await rightToggle.click();
      await page.waitForTimeout(250);
      const skeleton = await skeletonState(page);
      const typography = await typographyState(page);
      const collapsed = await page.evaluate(() => ({
        leftHidden: document.querySelector('[data-knowledge-region="left"]')?.getAttribute("aria-hidden"),
        rightHidden: document.querySelector('[data-knowledge-region="right"]')?.getAttribute("aria-hidden"),
      }));
      check(contextReport, "900 不回归: 真实折叠组合", collapsed.leftHidden === "true" && collapsed.rightHidden === "true", collapsed);
      check(contextReport, "900 不回归: 折叠态零横向 overflow",
        Object.values(skeleton.overflow).every((entry) => !entry || !entry.overflowsHorizontally), skeleton.overflow);
      check(contextReport, "900 不回归: 活动栏仍 42px",
        skeleton.activity.width === R0.activityRailWidth, { measured: skeleton.activity.width });
      check(contextReport, "900 不回归: 正文仍 16px",
        typography.readingBody.fontSize === R0.bodyFontSize, { readingBody: typography.readingBody });
      report.measurements.D1["900"] = {
        collapsed, activityWidth: skeleton.activity.width, overflow: skeleton.overflow,
        readingBody: typography.readingBody,
        note: "本档只作不回归，不做 R0 对照",
      };
      return { skeleton, collapsed };
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
  failures: report.failures.slice(0, 10),
  contexts: report.contexts.map((context) => ({
    name: context.name,
    assertions: context.assertions.length,
    failed: context.assertions.filter((assertion) => !assertion.passed).length,
    inspectionError: context.inspectionError ?? null,
  })),
}, null, 2));
process.exitCode = report.failed === 0 ? 0 : 1;
