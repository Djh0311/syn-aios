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
const secondMarkdownPath = "notes/layout-convergence.md";
const preferenceKey = "syn-native-knowledge-workspace-ui-v1";
const expectedReadAllowlist = [
  "knowledge_workspace_graph",
  "knowledge_workspace_read_canvas",
  "knowledge_workspace_read_markdown",
  "knowledge_workspace_search",
  "knowledge_workspace_snapshot",
];
const screenshots = [
  "01-1440-markdown-source-preview.png",
  "02-1180-markdown-graph.png",
  "03-900-double-sidebar-split.png",
  "04-900-right-collapsed-split.png",
  "05-900-both-collapsed-split.png",
  "06-1180-split-ratio-60.png",
  "07-1180-merged-single-group.png",
  "08-1180-quick-open-second-tab.png",
  "09-1180-dirty-close-rejected.png",
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
    window.dispatchEvent(new Event("n2r-r3b-capture"));
  }, scenario);
  await page.waitForTimeout(220);
  return JSON.parse(await page.locator("#knowledge-workbench-visual-metrics").textContent());
}

function horizontalOverflow(metrics) {
  const values = {
    documentElement: metrics.overflow.documentElement,
    body: metrics.overflow.body,
    shell: metrics.overflow.shell,
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

function normalizedPreferenceIsChromeOnly(storage) {
  const content = storage.latestNormalizedContent;
  if (!content || typeof content !== "object") return false;
  const serialized = JSON.stringify(content);
  return content.version === 2
    && serialized.includes('"centralState"')
    && !serialized.includes('"body"')
    && !serialized.includes("这是一段只用于隔离浏览器视觉量尺的合成知识内容")
    && !serialized.includes("R3B-DIRTY-MARKER");
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

async function saveScreenshot(page, filename) {
  await page.screenshot({
    path: fileURLToPath(new URL(`./${filename}`, rawDirectory)),
    fullPage: false,
  });
}

async function withFreshFixture(browser, name, viewport, action) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = { name, viewport, evidence: {}, audit: {} };

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
    const overflow = horizontalOverflow(metrics);
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
    expect(`${name}: document body and shell have no horizontal overflow`, overflow.ok, overflow.values);
    expect(`${name}: central workspace has one tablist per group and at most two groups`, (
      metrics.central.groupCount >= 1
      && metrics.central.groupCount <= 2
      && metrics.central.tablistCount === metrics.central.groupCount
    ), metrics.central);
    expect(`${name}: tab and tabpanel ARIA references are complete`, centralAriaIsLinked(metrics), metrics.central.groups);
    expect(`${name}: at most one textarea and one save action exist`, (
      metrics.central.textareaCount <= 1
      && metrics.central.saveActionCount <= 1
    ), metrics.central);
    expect(`${name}: localStorage writes only normalized disposable chrome`, (
      metrics.localStorage.writeCount > 0
      && metrics.localStorage.keys.length === 1
      && metrics.localStorage.keys[0] === preferenceKey
      && normalizedPreferenceIsChromeOnly(metrics.localStorage)
    ), metrics.localStorage);
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
  await withFreshFixture(browser, "01-1440-markdown-source-preview", { width: 1440, height: 900 }, async (page) => {
    await splitInitialMarkdown(page);
    const textarea = page.locator(".knowledge-workbench-central textarea");
    const originalDraft = await textarea.inputValue();
    await textarea.fill(`${originalDraft}\n\nR3B 即时共享草稿投影。`);
    await page.waitForFunction(() => document.body.textContent?.includes("R3B 即时共享草稿投影"));
    const state = await collectMetrics(page, "01-1440-markdown-source-preview");
    expect("1440 split has two Markdown groups", state.central.groupCount === 2, state.central);
    expect("1440 split has exactly one source textarea and one save action", (
      state.central.textareaCount === 1 && state.central.saveActionCount === 1
    ), state.central);
    expect("1440 split keeps source left and preview right for the same path", (
      state.central.groups[0]?.tabs.find((tab) => tab.selected === "true")?.ariaLabel?.includes(`${initialMarkdownPath}，Markdown 源码`)
      && state.central.groups[1]?.tabs.find((tab) => tab.selected === "true")?.ariaLabel?.includes(`${initialMarkdownPath}，渲染预览`)
    ), state.central.groups);
    await saveScreenshot(page, "01-1440-markdown-source-preview.png");
    return { screenshot: "01-1440-markdown-source-preview.png", sharedDraftMarkerVisible: true };
  });

  await withFreshFixture(browser, "02-1180-markdown-graph", { width: 1180, height: 760 }, async (page) => {
    await splitInitialMarkdown(page);
    await page.locator('[data-knowledge-region="activity"] button[aria-label="关系图"]').click();
    await page.waitForSelector(`[data-knowledge-tab-group="${secondaryGroupId}"] .native-knowledge-graph`);
    const state = await collectMetrics(page, "02-1180-markdown-graph");
    expect("1180 Graph remains inside the right group tabpanel", (
      state.central.groupCount === 2
      && state.central.groups[0]?.panelContainsGraph === false
      && state.central.groups[1]?.panelContainsGraph === true
      && state.central.groups[1]?.active === true
    ), state.central.groups);
    await saveScreenshot(page, "02-1180-markdown-graph.png");
    return { screenshot: "02-1180-markdown-graph.png" };
  });

  await withFreshFixture(browser, "03-900-double-sidebar-split", { width: 900, height: 760 }, async (page) => {
    await splitInitialMarkdown(page);
    const state = await collectMetrics(page, "03-900-double-sidebar-split");
    expect("900 expanded sidebars retain two groups", state.central.groupCount === 2, state.central);
    await saveScreenshot(page, "03-900-double-sidebar-split.png");
    return { screenshot: "03-900-double-sidebar-split.png", sidebarState: "expanded" };
  });

  await withFreshFixture(browser, "04-900-right-collapsed-split", { width: 900, height: 760 }, async (page) => {
    await splitInitialMarkdown(page);
    await page.getByRole("button", { name: "切换右侧上下文" }).click();
    const state = await collectMetrics(page, "04-900-right-collapsed-split");
    expect("900 right sidebar collapses out of the accessibility path", (
      state.regions.right.ariaHidden === "true"
      && state.regions.right.inert === true
      && state.regions.right.interactiveChildren === 0
    ), state.regions.right);
    await saveScreenshot(page, "04-900-right-collapsed-split.png");
    return { screenshot: "04-900-right-collapsed-split.png", sidebarState: "right-collapsed" };
  });

  await withFreshFixture(browser, "05-900-both-collapsed-split", { width: 900, height: 760 }, async (page) => {
    await splitInitialMarkdown(page);
    await page.locator('[data-knowledge-region="left"] button[aria-label="折叠左侧栏"]').click();
    await page.getByRole("button", { name: "切换右侧上下文" }).click();
    const state = await collectMetrics(page, "05-900-both-collapsed-split");
    expect("900 both sidebars collapse out of the accessibility path", (
      state.regions.left.ariaHidden === "true"
      && state.regions.left.inert === true
      && state.regions.left.interactiveChildren === 0
      && state.regions.right.ariaHidden === "true"
      && state.regions.right.inert === true
      && state.regions.right.interactiveChildren === 0
    ), { left: state.regions.left, right: state.regions.right });
    await saveScreenshot(page, "05-900-both-collapsed-split.png");
    return { screenshot: "05-900-both-collapsed-split.png", sidebarState: "both-collapsed" };
  });

  await withFreshFixture(browser, "06-1180-resize-merge-shortcuts-dirty", { width: 1180, height: 760 }, async (page) => {
    await splitInitialMarkdown(page);
    const separator = page.getByRole("separator", { name: "调整标签组分隔比例" });
    await separator.focus();
    await separator.press("ArrowRight");
    await separator.press("ArrowRight");
    const ratioMetrics = await collectMetrics(page, "06a-1180-split-ratio-60");
    expect("separator changes 50 to 60 with complete ARIA", (
      ratioMetrics.central.separator?.min === "30"
      && ratioMetrics.central.separator?.max === "70"
      && ratioMetrics.central.separator?.now === "60"
      && ratioMetrics.central.separator?.orientation === "vertical"
    ), ratioMetrics.central.separator);
    await saveScreenshot(page, "06-1180-split-ratio-60.png");

    const primaryTab = page.locator(`[data-knowledge-tab-group="${primaryGroupId}"] [role="tab"][aria-selected="true"]`);
    await primaryTab.click();
    const mergeButton = page.locator(`[data-knowledge-tab-group="${secondaryGroupId}"] button[aria-label="合并分栏"]`);
    await mergeButton.focus();
    await mergeButton.press("Enter");
    await page.waitForFunction(() => document.querySelectorAll("[data-knowledge-tab-group]").length === 1);
    await page.waitForTimeout(80);
    const afterMerge = await page.evaluate(() => ({
      groupCount: document.querySelectorAll("[data-knowledge-tab-group]").length,
      textareaCount: document.querySelectorAll(".knowledge-workbench-central textarea").length,
      focusedRole: document.activeElement?.getAttribute("role") ?? null,
      focusedAriaLabel: document.activeElement?.getAttribute("aria-label") ?? null,
    }));
    expect("merge returns to one source group and restores tab focus without reread", (
      afterMerge.groupCount === 1
      && afterMerge.textareaCount === 1
      && afterMerge.focusedRole === "tab"
      && afterMerge.focusedAriaLabel?.includes(initialMarkdownPath)
    ), afterMerge);
    await saveScreenshot(page, "07-1180-merged-single-group.png");

    await page.keyboard.press("Meta+t");
    const quickOpen = page.getByRole("dialog", { name: "快速打开" });
    await quickOpen.waitFor();
    const quickInput = quickOpen.getByRole("combobox", { name: "快速打开" });
    await quickInput.fill("layout-convergence");
    await quickOpen.getByRole("button", { name: "查找" }).click();
    await page.waitForSelector('.syn-knowledge-overlay [role="listbox"] [role="option"]');
    await quickInput.press("Enter");
    await page.waitForFunction((relativePath) => (
      document.activeElement?.getAttribute("role") === "tab"
      && document.activeElement?.getAttribute("aria-label")?.includes(relativePath)
    ), secondMarkdownPath);
    const afterQuickOpen = await page.evaluate(() => ({
      tabCount: document.querySelectorAll('[data-knowledge-tab-group] [role="tab"]').length,
      focusedRole: document.activeElement?.getAttribute("role") ?? null,
      focusedAriaLabel: document.activeElement?.getAttribute("aria-label") ?? null,
    }));
    expect("Meta+T selects the second synthetic Markdown as a real focused tab", (
      afterQuickOpen.tabCount === 2
      && afterQuickOpen.focusedRole === "tab"
      && afterQuickOpen.focusedAriaLabel?.includes(secondMarkdownPath)
    ), afterQuickOpen);
    await saveScreenshot(page, "08-1180-quick-open-second-tab.png");

    await page.keyboard.press("Control+Tab");
    await page.waitForFunction((relativePath) => document.activeElement?.getAttribute("aria-label")?.includes(relativePath), initialMarkdownPath);
    const afterNextCycle = await page.evaluate(() => ({
      focusedRole: document.activeElement?.getAttribute("role") ?? null,
      focusedAriaLabel: document.activeElement?.getAttribute("aria-label") ?? null,
    }));
    expect("Control+Tab cycles inside the current group", (
      afterNextCycle.focusedRole === "tab"
      && afterNextCycle.focusedAriaLabel?.includes(initialMarkdownPath)
    ), afterNextCycle);

    await page.keyboard.press("Control+Shift+Tab");
    await page.waitForFunction((relativePath) => document.activeElement?.getAttribute("aria-label")?.includes(relativePath), secondMarkdownPath);
    const textarea = page.locator(".knowledge-workbench-central textarea");
    const draftBeforeDirty = await textarea.inputValue();
    await textarea.fill(`${draftBeforeDirty}\n\nR3B-DIRTY-MARKER`);
    await page.getByRole("button", { name: `关闭 ${secondMarkdownPath}` }).click();
    await page.waitForTimeout(100);
    const dirtyState = await page.evaluate((relativePath) => ({
      tabStillPresent: [...document.querySelectorAll('[role="tab"]')]
        .some((tab) => tab.getAttribute("aria-label")?.includes(relativePath)),
      textareaValue: (document.querySelector(".knowledge-workbench-central textarea") instanceof HTMLTextAreaElement)
        ? document.querySelector(".knowledge-workbench-central textarea").value
        : null,
      noticeVisible: document.body.textContent?.includes("已拒绝关闭标签") ?? false,
      focusedRole: document.activeElement?.getAttribute("role") ?? null,
    }), secondMarkdownPath);
    expect("dirty current Markdown close is rejected and preserves the draft", (
      dirtyState.tabStillPresent
      && dirtyState.textareaValue?.includes("R3B-DIRTY-MARKER")
      && dirtyState.noticeVisible
    ), dirtyState);
    await saveScreenshot(page, "09-1180-dirty-close-rejected.png");

    return {
      screenshots: screenshots.slice(5),
      ratio: ratioMetrics.central.separator,
      afterMerge,
      afterQuickOpen,
      afterNextCycle,
      dirtyState,
    };
  });
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
