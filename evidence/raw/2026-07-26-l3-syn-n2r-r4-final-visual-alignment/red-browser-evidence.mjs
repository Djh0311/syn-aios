// N2R-R4 RED：改产品前的现状量尺。纯合成夹具 + 真实 React + 真实生产 CSS。
// 用法：先起 vite（127.0.0.1:5173），再 node red-browser-evidence.mjs
import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./red-browser-evidence.json", rawDirectory));

// R0 §2.3 `984×768` 参考几何——逐条从 R0 原文读出，不从任务包转述照抄。
const R0 = {
  activityRailWidth: 42,
  leftSidebarExpanded: 288,
  rightSidebar: 185,
  centralStatusHeight: 26,
  integratedTopBar: 39,
  viewToolbar: 35,
  leftFooterVault: 41,
  bodyFontSize: 16,
};

const report = {
  phase: "pre-implementation-red",
  fixture: "synthetic-only",
  purpose: "R4 D1 骨架 / D2 正文字号 / D3 悬停提示的现状反例",
  r0Reference: R0,
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
        clientHeight: element.clientHeight,
        scrollHeight: element.scrollHeight,
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
    // 中央 chrome：标签头 + 该组内视图自己的工具条（Markdown 面板 / Graph 工具栏等）
    // Syn 的中央 chrome 是两层：组头（标签 + 组工具 + 源码/预览投影控件）与文档头
    // （路径 + 标题 + 投影标签）。对应 R0 的「集成顶栏 39」+「视图工具栏 35」。
    const viewToolbar = box('[data-active-group="true"] .native-workspace-document-head')
      ?? box('[data-active-group="true"] .native-graph-toolbar');
    const separator = box(".knowledge-workbench-separator");
    // 左栏底部是否存在 vault/footer 常驻带
    const leftFooter = box('[data-knowledge-region="left"] .native-workspace-footer')
      ?? box('[data-knowledge-region="left"] footer');
    return {
      shellGridColumns: shell ? getComputedStyle(shell).gridTemplateColumns : null,
      shellGridRows: shell ? getComputedStyle(shell).gridTemplateRows : null,
      activity,
      left,
      central,
      right,
      status,
      groupHeader,
      viewToolbar,
      separator,
      leftFooter,
      centralChromeTotal: groupHeader && viewToolbar
        ? round(groupHeader.height + viewToolbar.height)
        : (groupHeader?.height ?? null),
      ratio: (activity && left && central && right)
        ? { left: left.width, central: central.width, right: right.width }
        : null,
      overflow: {
        documentElement: scroll("html"),
        body: scroll("body"),
        shell: scroll(".syn-knowledge-shell"),
        central: scroll('[data-knowledge-region="central"]'),
        activeGroupPanel: scroll('[data-active-group="true"] [data-knowledge-group-panel="active"]'),
        right: scroll('[data-knowledge-region="right"]'),
        left: scroll('[data-knowledge-region="left"]'),
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
        lineHeight: style.lineHeight === "normal" ? "normal" : Number.parseFloat(style.lineHeight),
        lineHeightRatio: style.lineHeight === "normal"
          ? null
          : Math.round((Number.parseFloat(style.lineHeight) / Number.parseFloat(style.fontSize)) * 100) / 100,
        fontFamily: style.fontFamily.split(",")[0].replace(/["']/g, ""),
      };
    };
    return {
      // 正文（本包唯一允许提字号的族）
      readingBody: size(".native-workspace-markdown"),
      readingParagraph: size(".native-workspace-markdown p"),
      readingHeading: size(".native-workspace-markdown h1, .native-workspace-markdown h2, .native-workspace-markdown h3"),
      readingPre: size(".native-workspace-markdown pre"),
      editorTextarea: size(".native-workspace-source textarea"),
      // chrome：本包明确不许动，改前改后必须逐项相同
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

async function activityTooltipState(page) {
  return page.evaluate(() => {
    const rail = document.querySelector('[data-knowledge-region="activity"]');
    if (!rail) return null;
    const buttons = [...rail.querySelectorAll("button")].map((button) => ({
      ariaLabel: button.getAttribute("aria-label"),
      title: button.getAttribute("title"),
      svgTitleCount: button.querySelectorAll("svg title").length,
      iconAriaHidden: button.querySelector("svg")?.getAttribute("aria-hidden") ?? null,
      iconFocusable: button.querySelector("svg")?.getAttribute("focusable") ?? null,
    }));
    return {
      buttonCount: buttons.length,
      titleCount: buttons.filter((button) => button.title !== null).length,
      svgTitleTotal: buttons.reduce((sum, button) => sum + button.svgTitleCount, 0),
      buttons,
    };
  });
}

/** 打开第一条 Markdown。默认落在「源码」编辑态，本函数只负责打开。 */
async function openFirstMarkdown(page) {
  const entry = page.locator('[data-knowledge-region="left"] .native-workspace-tree button').first();
  await entry.waitFor();
  await entry.click();
  await page.locator(".native-workspace-source textarea").first().waitFor();
  await page.waitForTimeout(300);
}

/** 切到「预览」阅读态，量渲染正文。 */
async function switchToReading(page) {
  await page.locator('[data-active-group="true"] .knowledge-workbench-projection-controls button', { hasText: "预览" }).first().click();
  await page.locator(".native-workspace-markdown").first().waitFor();
  await page.waitForTimeout(300);
}

async function withFreshContext(browser, { name, viewport, action }) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = { name, viewport, assertions: [], failures: [], evidence: {} };
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
  for (const [label, viewport] of [["1440", { width: 1440, height: 900 }], ["1180", { width: 1180, height: 760 }]]) {
    await withFreshContext(browser, {
      name: `red-${label}-skeleton-and-typography`,
      viewport,
      action: async (page, contextReport) => {
        await openFirstMarkdown(page);
        const editorTypography = await typographyState(page);
        await switchToReading(page);
        const skeleton = await skeletonState(page);
        const typography = { ...(await typographyState(page)), editorTextarea: editorTypography.editorTextarea };
        const tooltip = await activityTooltipState(page);

        // §6①：正文字号应为 16（现状应失败）
        check(contextReport, `RED D2 ${label}: 阅读正文 computed font-size = 16px（现状应失败）`,
          typography.readingBody.fontSize === R0.bodyFontSize, { readingBody: typography.readingBody });
        check(contextReport, `RED D2 ${label}: 编辑正文 computed font-size = 16px（现状应失败）`,
          typography.editorTextarea?.fontSize === R0.bodyFontSize, { editorTextarea: typography.editorTextarea });

        // §6②：活动栏 title 计数（现状应为 0）
        check(contextReport, `RED D3 ${label}: 活动栏 8 个按钮各有 title（现状应失败）`,
          tooltip.titleCount === tooltip.buttonCount && tooltip.buttonCount === 8,
          { titleCount: tooltip.titleCount, buttonCount: tooltip.buttonCount });

        // §6③：§5.1 骨架逐项——现状在带内的如实记为通过，不为凑失败破坏现状
        check(contextReport, `RED D1 ${label}: 活动栏宽 = 42`,
          skeleton.activity.width === R0.activityRailWidth, { measured: skeleton.activity.width, r0: R0.activityRailWidth });
        check(contextReport, `RED D1 ${label}: 中央底部状态区高 = 26`,
          skeleton.status.height === R0.centralStatusHeight, { measured: skeleton.status.height, r0: R0.centralStatusHeight });
        check(contextReport, `RED D1 ${label}: 右栏宽在 185–240 带内`,
          skeleton.right.width >= R0.rightSidebar && skeleton.right.width <= 240, { measured: skeleton.right.width });
        check(contextReport, `RED D1 ${label}: 左栏展开宽在 220–288 带内`,
          skeleton.left.width >= 220 && skeleton.left.width <= 288, { measured: skeleton.left.width });
        check(contextReport, `RED D1 ${label}: 中央 chrome 总高在 74±10`,
          skeleton.centralChromeTotal !== null && Math.abs(skeleton.centralChromeTotal - 74) <= 10,
          { measured: skeleton.centralChromeTotal, r0: `${R0.integratedTopBar}+${R0.viewToolbar}=74` });
        check(contextReport, `RED D1 ${label}: 各层零横向 overflow`,
          Object.values(skeleton.overflow).every((entry) => !entry || !entry.overflowsHorizontally),
          { overflow: skeleton.overflow });

        report.redFindings.push({
          viewport: label,
          skeleton: {
            shellGridColumns: skeleton.shellGridColumns,
            shellGridRows: skeleton.shellGridRows,
            activityWidth: skeleton.activity.width,
            leftWidth: skeleton.left.width,
            centralWidth: skeleton.central.width,
            rightWidth: skeleton.right.width,
            statusHeight: skeleton.status.height,
            groupHeaderHeight: skeleton.groupHeader?.height ?? null,
            viewToolbarHeight: skeleton.viewToolbar?.height ?? null,
            centralChromeTotal: skeleton.centralChromeTotal,
            leftFooterPresent: skeleton.leftFooter !== null,
            separatorBorders: skeleton.separator
              ? [skeleton.separator.borderLeftWidth, skeleton.separator.borderRightWidth]
              : null,
            ratio: skeleton.ratio,
          },
          typography,
          tooltip: { buttonCount: tooltip.buttonCount, titleCount: tooltip.titleCount, svgTitleTotal: tooltip.svgTitleTotal },
        });

        if (label === "1440") {
          await page.screenshot({ path: fileURLToPath(new URL("./red-01-1440-before-alignment.png", rawDirectory)) });
        }
        return { skeleton, typography, tooltip };
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
    failures: context.failures.map((failure) => failure.name),
    inspectionError: context.inspectionError ?? null,
  })),
  redFindings: report.redFindings,
}, null, 2));
