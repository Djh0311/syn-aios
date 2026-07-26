import fs from "node:fs/promises";
import playwrightCore from "/Users/yoyi/.npm/_npx/e41f203b7505f1fb/node_modules/playwright-core/index.js";

const { chromium } = playwrightCore;
const fixtureUrl = "http://127.0.0.1:5173/tests/knowledge-workbench-visual-fixture.html";
const outputPath = new URL("./red-contracts.json", import.meta.url);
const report = {
  phase: "pre-implementation-red-contract",
  fixture: "synthetic-only",
  assertions: 0,
  failed: 0,
  passNames: [],
  failures: [],
  overflowRows: [],
  contextAudits: [],
};

function pass(name) {
  report.assertions += 1;
  report.passNames.push(name);
}

function fail(name, detail = {}) {
  report.assertions += 1;
  report.failed += 1;
  report.failures.push({ name, detail });
}

async function collectMetrics(page) {
  await page.evaluate(() => window.dispatchEvent(new Event("n2r-r2-capture")));
  await page.waitForTimeout(180);
  return JSON.parse(await page.locator("#knowledge-workbench-visual-metrics").textContent());
}

async function withFreshFixture(browser, name, action) {
  const context = await browser.newContext({
    viewport: { width: 900, height: 760 },
    deviceScaleFactor: 1,
  });
  const page = await context.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  const externalRequests = [];
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
    await action(page);
    const after = await collectMetrics(page);
    report.contextAudits.push({
      name,
      localStorageEmptyBeforeMount: before.fixture.localStorageEmptyBeforeMount,
      mock: after.mock,
      externalRequests,
      consoleErrors,
      pageErrors,
    });
  } catch (error) {
    fail(`${name}: browser inspection error`, { error: String(error?.stack ?? error) });
    report.contextAudits.push({ name, inspectionError: String(error?.stack ?? error), externalRequests, consoleErrors, pageErrors });
  } finally {
    await context.close();
  }
}

const browser = await chromium.launch({
  headless: true,
  executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  args: ["--no-first-run", "--no-default-browser-check"],
});

try {
  for (const scenario of ["both-expanded", "left-collapsed", "right-collapsed", "both-collapsed"]) {
    await withFreshFixture(browser, `overflow-${scenario}`, async (page) => {
      if (scenario === "left-collapsed" || scenario === "both-collapsed") {
        await page.locator('[data-knowledge-region="left"] button[aria-label="折叠左侧栏"]').click();
      }
      if (scenario === "right-collapsed" || scenario === "both-collapsed") {
        await page.getByRole("button", { name: "切换右侧上下文" }).click();
      }
      const metrics = await collectMetrics(page);
      const values = {
        document: metrics.overflow.documentElement,
        body: metrics.overflow.body,
        shell: metrics.overflow.shell,
      };
      const ok = Object.values(values).every((metric) => metric && metric.scrollWidth <= metric.clientWidth);
      report.overflowRows.push({ scenario, ok, values });
      if (ok) pass(`900×760 ${scenario} has no horizontal overflow`);
      else fail(`900×760 ${scenario} has horizontal overflow`, values);
    });
  }

  await withFreshFixture(browser, "left-search", async (page) => {
    await page.getByRole("button", { name: "搜索" }).click();
    const state = await page.evaluate(() => ({
      inputs: document.querySelectorAll('[data-knowledge-region="left"] input[role="combobox"]').length,
      listboxes: document.querySelectorAll('[data-knowledge-region="left"] [role="listbox"]').length,
      options: document.querySelectorAll('[data-knowledge-region="left"] [role="option"]').length,
    }));
    if (state.inputs === 1) pass("left Search owns one in-place combobox");
    else fail("left Search lacks its required in-place combobox", state);
    if (state.listboxes === 1) pass("left Search owns one result listbox");
    else fail("left Search lacks its required result listbox", state);
  });

  await withFreshFixture(browser, "quick-open", async (page) => {
    await page.getByRole("button", { name: "搜索" }).click();
    await page.getByRole("button", { name: "打开快速搜索" }).click();
    await page.locator("#native-knowledge-search").fill("合成");
    await page.getByRole("button", { name: "查找" }).click();
    await page.waitForTimeout(160);
    const state = await page.evaluate(() => ({
      comboboxes: document.querySelectorAll('.syn-knowledge-overlay input[role="combobox"]').length,
      listboxes: document.querySelectorAll('.syn-knowledge-overlay [role="listbox"]').length,
      options: document.querySelectorAll('.syn-knowledge-overlay [role="option"]').length,
      current: document.querySelectorAll('.syn-knowledge-overlay [aria-selected="true"]').length,
      hints: [...document.querySelectorAll(".syn-knowledge-overlay *")].filter((element) => element.textContent?.includes("↑↓")).length,
    }));
    if (state.comboboxes === 1) pass("quick-open renders a combobox");
    else fail("quick-open lacks combobox contract", state);
    if (state.listboxes === 1) pass("quick-open renders a listbox");
    else fail("quick-open lacks listbox contract", state);
    if (state.options > 0) pass("quick-open renders result options after synthetic query");
    else fail("quick-open lacks result options after synthetic query", state);
    if (state.current === 1) pass("quick-open renders exactly one current option");
    else fail("quick-open lacks exactly one current option", state);
    if (state.hints > 0) pass("quick-open renders keyboard hints");
    else fail("quick-open lacks visible keyboard hints", state);
  });

  await withFreshFixture(browser, "command", async (page) => {
    await page.getByRole("button", { name: "Syn 命令" }).click();
    const state = await page.evaluate(() => ({
      comboboxes: document.querySelectorAll('.syn-knowledge-overlay input[role="combobox"]').length,
      listboxes: document.querySelectorAll('.syn-knowledge-overlay [role="listbox"]').length,
      options: document.querySelectorAll('.syn-knowledge-overlay [role="option"]').length,
      current: document.querySelectorAll('.syn-knowledge-overlay [aria-selected="true"]').length,
      hints: [...document.querySelectorAll(".syn-knowledge-overlay *")].filter((element) => element.textContent?.includes("↑↓")).length,
    }));
    if (state.comboboxes === 1) pass("command renders a filter combobox");
    else fail("command lacks filter combobox", state);
    if (state.listboxes === 1) pass("command renders a listbox");
    else fail("command lacks command listbox", state);
    if (state.options >= 2) pass("command exposes existing Markdown/directory entries");
    else fail("command lacks existing new Markdown/new directory command entries", state);
    if (state.current === 1) pass("command renders exactly one current option");
    else fail("command lacks exactly one current option", state);
    if (state.hints > 0) pass("command renders keyboard hints");
    else fail("command lacks visible keyboard hints", state);
  });

  await withFreshFixture(browser, "shortcuts", async (page) => {
    for (const [name, key, expected] of [
      ["quick-open", "Meta+o", "快速打开"],
      ["command", "Meta+p", "Syn 命令"],
      ["left-search", "Meta+Shift+f", "left-search"],
    ]) {
      await page.keyboard.press(key);
      await page.waitForTimeout(90);
      const overlay = page.locator(".syn-knowledge-overlay");
      const observed = expected === "left-search"
        ? await page.getByRole("button", { name: "搜索" }).getAttribute("aria-pressed")
        : (await overlay.count()) === 1 ? await overlay.getAttribute("aria-label") : null;
      const ok = expected === "left-search" ? observed === "true" : observed === expected;
      if (ok) pass(`${key} routes to ${name}`);
      else fail(`shortcut ${key} did not route to ${name}`, { observed });
      if (expected !== "left-search" && observed) await page.keyboard.press("Escape");
    }
  });

  const allContextsHealthy = report.contextAudits.every((audit) => (
    audit.localStorageEmptyBeforeMount === true
    && audit.mock?.writeCallCount === 0
    && audit.mock?.unrecognizedCallCount === 0
    && audit.externalRequests?.length === 0
    && audit.consoleErrors?.length === 0
    && audit.pageErrors?.length === 0
  ));
  if (allContextsHealthy) pass("all red-contract contexts stayed fresh, synthetic, zero-write, and error-free");
  else fail("a red-contract context was not fresh/synthetic/zero-write/error-free", report.contextAudits);
} finally {
  await browser.close();
  report.outcome = report.failed === 0 ? "UNEXPECTED_GREEN" : "EXPECTED_RED";
  await fs.writeFile(outputPath, `${JSON.stringify(report, null, 2)}\n`);
}

console.log(JSON.stringify({
  outcome: report.outcome,
  assertions: report.assertions,
  failed: report.failed,
  failures: report.failures.map((failure) => failure.name),
}, null, 2));
process.exitCode = report.failed === 0 ? 0 : 1;
