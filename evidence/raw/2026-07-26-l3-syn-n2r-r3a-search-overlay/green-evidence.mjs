import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const rawDirectory = new URL("./", import.meta.url);
const reportPath = fileURLToPath(new URL("./green-browser-evidence.json", import.meta.url));
const expectedReadAllowlist = [
  "knowledge_workspace_graph",
  "knowledge_workspace_read_canvas",
  "knowledge_workspace_read_markdown",
  "knowledge_workspace_search",
  "knowledge_workspace_snapshot",
];
const report = {
  phase: "post-implementation-green-browser-evidence",
  fixture: "synthetic-only",
  assertions: 0,
  failed: 0,
  failures: [],
  contexts: [],
  screenshots: [
    "01-1180-left-search-results.png",
    "02-1180-quick-open-results.png",
    "03-900-command-filter-results.png",
  ],
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
    window.dispatchEvent(new Event("n2r-r2-capture"));
  }, scenario);
  await page.waitForTimeout(180);
  return JSON.parse(await page.locator("#knowledge-workbench-visual-metrics").textContent());
}

function noHorizontalOverflow(metrics) {
  const values = {
    document: metrics.overflow.documentElement,
    body: metrics.overflow.body,
    shell: metrics.overflow.shell,
  };
  return {
    values,
    ok: Object.values(values).every((metric) => metric && metric.scrollWidth <= metric.clientWidth),
  };
}

async function withFreshFixture(browser, name, viewport, action) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
  const contextReport = { name, viewport, audit: {}, evidence: {} };
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
  try {
    await page.goto(fixtureUrl, { waitUntil: "networkidle" });
    await page.waitForFunction(() => document.documentElement.dataset.fixtureReady === "true");
    const before = JSON.parse(await page.locator("#knowledge-workbench-visual-metrics").textContent());
    contextReport.audit.localStorageEmptyBeforeMount = before.fixture.localStorageEmptyBeforeMount;
    expect(`${name}: localStorage was empty before mount`, before.fixture.localStorageEmptyBeforeMount, { observed: before.fixture.localStorageEmptyBeforeMount });
    contextReport.evidence = await action(page);
    const metrics = await collectMetrics(page, name);
    const overflow = noHorizontalOverflow(metrics);
    contextReport.metrics = metrics;
    contextReport.overflow = overflow;
    contextReport.audit.readAllowlist = metrics.mock.allowedReadCommands;
    contextReport.audit.callsByCommand = metrics.mock.callsByCommand;
    contextReport.audit.writeCallCount = metrics.mock.writeCallCount;
    contextReport.audit.unrecognizedCallCount = metrics.mock.unrecognizedCallCount;
    contextReport.audit.externalRequestCount = externalRequests.length;
    contextReport.audit.consoleErrorCount = consoleErrors.length;
    contextReport.audit.pageErrorCount = pageErrors.length;
    contextReport.audit.externalRequests = externalRequests;
    contextReport.audit.consoleErrors = consoleErrors;
    contextReport.audit.pageErrors = pageErrors;
    expect(`${name}: fixture exposes the exact read allowlist`, JSON.stringify(metrics.mock.allowedReadCommands) === JSON.stringify(expectedReadAllowlist), { observed: metrics.mock.allowedReadCommands });
    expect(`${name}: no unexpected mock command is called`, Object.keys(metrics.mock.callsByCommand).every((command) => expectedReadAllowlist.includes(command)), { callsByCommand: metrics.mock.callsByCommand });
    expect(`${name}: mock write calls stay zero`, metrics.mock.writeCallCount === 0, { observed: metrics.mock.writeCallCount });
    expect(`${name}: mock unrecognized calls stay zero`, metrics.mock.unrecognizedCallCount === 0, { observed: metrics.mock.unrecognizedCallCount });
    expect(`${name}: external request count stays zero`, externalRequests.length === 0, { externalRequests });
    expect(`${name}: console error count stays zero`, consoleErrors.length === 0, { consoleErrors });
    expect(`${name}: page error count stays zero`, pageErrors.length === 0, { pageErrors });
    expect(`${name}: document/body/shell have no horizontal overflow`, overflow.ok, overflow.values);
  } catch (error) {
    expect(`${name}: browser evidence completed without inspection errors`, false, { error: String(error?.stack ?? error) });
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
  await withFreshFixture(browser, "01-1180-left-search-results", { width: 1180, height: 760 }, async (page) => {
    await page.getByRole("button", { name: "搜索" }).click();
    const left = page.locator('[data-knowledge-region="left"]');
    const input = left.getByRole("combobox", { name: "搜索固定 Syn vault" });
    await input.fill("合成");
    await left.getByRole("button", { name: "搜索" }).click();
    await page.waitForFunction(() => document.querySelectorAll('[data-knowledge-region="left"] [role="listbox"] [role="option"]').length > 0);
    const beforeOpen = await page.evaluate(() => ({
      optionCount: document.querySelectorAll('[data-knowledge-region="left"] [role="listbox"] [role="option"]').length,
      currentCount: document.querySelectorAll('[data-knowledge-region="left"] [role="listbox"] [role="option"][aria-selected="true"]').length,
      hasLoading: Boolean(document.querySelector('[data-knowledge-region="left"] [role="status"]')),
    }));
    expect("left Search has results after an actual synthetic query", beforeOpen.optionCount > 0, beforeOpen);
    expect("left Search has exactly one current result", beforeOpen.currentCount === 1, beforeOpen);
    await page.screenshot({ path: fileURLToPath(new URL("./01-1180-left-search-results.png", rawDirectory)) });
    await input.press("ArrowDown");
    const activeDescendant = await input.getAttribute("aria-activedescendant");
    expect("left Search ArrowDown advances the current result", activeDescendant === "native-knowledge-left-search-option-1", { activeDescendant });
    await input.press("Enter");
    await page.waitForTimeout(180);
    const afterOpen = await page.evaluate(() => ({
      focusRegion: document.activeElement?.closest("[data-knowledge-region]")?.getAttribute("data-knowledge-region") ?? null,
      focusedRole: document.activeElement?.getAttribute("role") ?? null,
      focusedText: document.activeElement?.textContent?.trim() ?? null,
    }));
    expect("left Search Enter opens and focuses a central workspace tab", afterOpen.focusRegion === "central" && afterOpen.focusedRole === "tab", afterOpen);
    return { query: "合成", beforeOpen, activeDescendant, afterOpen, screenshot: "01-1180-left-search-results.png" };
  });

  await withFreshFixture(browser, "02-1180-quick-open-results", { width: 1180, height: 760 }, async (page) => {
    const trigger = page.getByRole("button", { name: "搜索" });
    await trigger.focus();
    await page.keyboard.press("Meta+o");
    const dialog = page.getByRole("dialog", { name: "快速打开" });
    await dialog.waitFor();
    const input = dialog.getByRole("combobox", { name: "快速打开" });
    const initialFocus = await page.evaluate(() => document.activeElement?.id ?? null);
    expect("⌘O opens quick-open with its input as primary focus", initialFocus === "native-knowledge-quick-open", { initialFocus });
    await input.fill("合成");
    await dialog.getByRole("button", { name: "查找" }).click();
    await page.waitForFunction(() => document.querySelectorAll('.syn-knowledge-overlay [role="listbox"] [role="option"]').length > 0);
    await input.press("ArrowDown");
    const state = await page.evaluate(() => ({
      optionCount: document.querySelectorAll('.syn-knowledge-overlay [role="listbox"] [role="option"]').length,
      currentCount: document.querySelectorAll('.syn-knowledge-overlay [role="listbox"] [role="option"][aria-selected="true"]').length,
      activeDescendant: document.activeElement?.getAttribute("aria-activedescendant") ?? null,
      hints: [...document.querySelectorAll('.syn-knowledge-overlay .native-workspace-overlay-hints span')].map((element) => element.textContent),
    }));
    expect("quick-open has results, one current item, and all keyboard hints", state.optionCount > 0 && state.currentCount === 1 && state.hints.join(" ").includes("↑↓") && state.hints.join(" ").includes("Enter") && state.hints.join(" ").includes("Esc"), state);
    await page.screenshot({ path: fileURLToPath(new URL("./02-1180-quick-open-results.png", rawDirectory)) });
    await input.press("Escape");
    await page.waitForTimeout(100);
    const afterEscape = await page.evaluate(() => ({
      dialogCount: document.querySelectorAll('.syn-knowledge-overlay').length,
      focusAriaLabel: document.activeElement?.getAttribute("aria-label") ?? null,
    }));
    expect("quick-open Escape closes and restores the trigger focus", afterEscape.dialogCount === 0 && afterEscape.focusAriaLabel === "搜索", afterEscape);
    return { query: "合成", initialFocus, state, afterEscape, screenshot: "02-1180-quick-open-results.png" };
  });

  await withFreshFixture(browser, "03-900-command-filter-results", { width: 900, height: 760 }, async (page) => {
    const trigger = page.getByRole("button", { name: "Syn 命令" });
    await trigger.focus();
    await page.keyboard.press("Meta+p");
    const dialog = page.getByRole("dialog", { name: "Syn 命令" });
    await dialog.waitFor();
    const input = dialog.getByRole("combobox", { name: "筛选 Syn 命令" });
    const initialCount = await dialog.locator('[role="listbox"] [role="option"]').count();
    expect("command starts with both existing safe creation commands", initialCount === 2, { initialCount });
    await input.fill("目录");
    await page.waitForFunction(() => document.querySelectorAll('.syn-knowledge-overlay [role="listbox"] [role="option"]').length === 1);
    const state = await page.evaluate(() => ({
      optionCount: document.querySelectorAll('.syn-knowledge-overlay [role="listbox"] [role="option"]').length,
      currentCount: document.querySelectorAll('.syn-knowledge-overlay [role="listbox"] [role="option"][aria-selected="true"]').length,
      currentText: document.querySelector('.syn-knowledge-overlay [role="listbox"] [role="option"][aria-selected="true"]')?.textContent?.trim() ?? null,
      hints: [...document.querySelectorAll('.syn-knowledge-overlay .native-workspace-overlay-hints span')].map((element) => element.textContent),
    }));
    expect("command filtering preserves a current result and hints", state.optionCount === 1 && state.currentCount === 1 && state.currentText?.includes("新建目录") && state.hints.join(" ").includes("↑↓") && state.hints.join(" ").includes("Enter") && state.hints.join(" ").includes("Esc"), state);
    await page.screenshot({ path: fileURLToPath(new URL("./03-900-command-filter-results.png", rawDirectory)) });
    await input.press("Enter");
    await page.waitForTimeout(80);
    const safeForm = await page.evaluate(() => ({
      pathInputCount: document.querySelectorAll('.syn-knowledge-overlay input[aria-label="新建条目的相对路径"]').length,
      createButtonDisabled: (document.querySelector('.syn-knowledge-overlay button.secondary-button') instanceof HTMLButtonElement)
        ? document.querySelector('.syn-knowledge-overlay button.secondary-button').disabled
        : null,
    }));
    expect("command Enter only enters the existing safe relative-path form", safeForm.pathInputCount === 1 && safeForm.createButtonDisabled === true, safeForm);
    await page.keyboard.press("Escape");
    await page.waitForTimeout(100);
    const afterEscape = await page.evaluate(() => ({
      dialogCount: document.querySelectorAll('.syn-knowledge-overlay').length,
      focusAriaLabel: document.activeElement?.getAttribute("aria-label") ?? null,
    }));
    expect("command Escape restores its trigger focus", afterEscape.dialogCount === 0 && afterEscape.focusAriaLabel === "Syn 命令", afterEscape);
    return { query: "目录", initialCount, state, safeForm, afterEscape, screenshot: "03-900-command-filter-results.png" };
  });

  for (const scenario of ["04-900-left-collapsed", "05-900-right-collapsed", "06-900-both-collapsed"]) {
    await withFreshFixture(browser, scenario, { width: 900, height: 760 }, async (page) => {
      if (scenario === "04-900-left-collapsed" || scenario === "06-900-both-collapsed") {
        await page.locator('[data-knowledge-region="left"] button[aria-label="折叠左侧栏"]').click();
      }
      if (scenario === "05-900-right-collapsed" || scenario === "06-900-both-collapsed") {
        await page.getByRole("button", { name: "切换右侧上下文" }).click();
      }
      const state = await page.evaluate(() => ({
        left: {
          ariaHidden: document.querySelector('[data-knowledge-region="left"]')?.getAttribute("aria-hidden") ?? null,
          inert: document.querySelector('[data-knowledge-region="left"]')?.hasAttribute("inert") ?? false,
          interactive: document.querySelectorAll('[data-knowledge-region="left"] button, [data-knowledge-region="left"] input, [data-knowledge-region="left"] [tabindex]').length,
        },
        right: {
          ariaHidden: document.querySelector('[data-knowledge-region="right"]')?.getAttribute("aria-hidden") ?? null,
          inert: document.querySelector('[data-knowledge-region="right"]')?.hasAttribute("inert") ?? false,
          interactive: document.querySelectorAll('[data-knowledge-region="right"] button, [data-knowledge-region="right"] input, [data-knowledge-region="right"] [tabindex]').length,
        },
        activityControls: document.querySelectorAll('[data-knowledge-region="activity"] button').length,
      }));
      const leftShouldBeCollapsed = scenario !== "05-900-right-collapsed";
      const rightShouldBeCollapsed = scenario !== "04-900-left-collapsed";
      expect(`${scenario}: collapsed left exits the interactive/AT path`, !leftShouldBeCollapsed || (state.left.ariaHidden === "true" && state.left.inert && state.left.interactive === 0), state);
      expect(`${scenario}: collapsed right exits the interactive/AT path`, !rightShouldBeCollapsed || (state.right.ariaHidden === "true" && state.right.inert && state.right.interactive === 0), state);
      expect(`${scenario}: activity rail remains available to restore sidebars`, state.activityControls > 0, state);
      return state;
    });
  }
} finally {
  await browser.close();
  report.outcome = report.failed === 0 ? "PASS" : "FAIL";
  await fs.writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

console.log(JSON.stringify({
  outcome: report.outcome,
  assertions: report.assertions,
  failed: report.failed,
  failures: report.failures.map((failure) => failure.name),
}, null, 2));
process.exitCode = report.failed === 0 ? 0 : 1;
