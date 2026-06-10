const INDEX_URL = "../index-kernel/codex-index.json";
const TASKS_URL = "../../tasks/README.md";

const state = {
  index: null,
  tasks: null,
  view: "home",
  query: "",
};

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => Array.from(document.querySelectorAll(selector));

document.addEventListener("DOMContentLoaded", () => {
  bindChrome();
  loadAll();
});

function bindChrome() {
  $$(".nav-item").forEach((button) => {
    button.addEventListener("click", () => {
      setView(button.dataset.view);
    });
  });

  $("#globalSearch").addEventListener("input", (event) => {
    state.query = event.target.value.trim().toLowerCase();
    render();
  });

  $("#reloadButton").addEventListener("click", () => {
    loadAll();
  });
}

async function loadAll() {
  setNotice("加载中", "正在读取静态索引和任务队列。");
  try {
    const [index, tasksText] = await Promise.all([
      fetchJson(INDEX_URL),
      fetchText(TASKS_URL).catch(() => ""),
    ]);
    state.index = index;
    state.tasks = parseTasks(tasksText);
    $("#sourcePath").textContent = INDEX_URL;
    setNotice("已加载", "页面只使用静态索引和任务队列 Markdown，不读取 Codex 会话正文。");
    render();
  } catch (error) {
    state.index = null;
    setNotice("读取失败", `${error.message}。请从 product-line 目录启动本地 server。`, true);
    render();
  }
}

async function fetchJson(url) {
  const response = await fetch(url, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`无法读取 ${url}`);
  }
  return response.json();
}

async function fetchText(url) {
  const response = await fetch(url, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`无法读取 ${url}`);
  }
  return response.text();
}

function setView(view) {
  state.view = view;
  $$(".nav-item").forEach((button) => {
    button.classList.toggle("active", button.dataset.view === view);
  });
  $$(".view").forEach((section) => {
    section.classList.toggle("active", section.id === `view-${view}`);
  });
}

function setNotice(title, text, isError = false) {
  const panel = $("#noticePanel");
  panel.classList.toggle("error", isError);
  panel.innerHTML = "";
  panel.append(el("strong", {}, title), el("span", {}, text));
}

function render() {
  if (!state.index) {
    renderEmptyShell();
    return;
  }

  renderHome();
  renderProjects();
  renderSessions();
  renderSkills();
  renderHarness();
  renderTasks();
}

function renderEmptyShell() {
  $("#metricGrid").innerHTML = "";
  $("#warningSummary").innerHTML = emptyHtml();
  $("#recentProjects").innerHTML = emptyHtml();
  $("#projectsList").innerHTML = emptyHtml();
  $("#sessionsList").innerHTML = emptyHtml();
  $("#skillsList").innerHTML = emptyHtml();
  $("#harnessList").innerHTML = emptyHtml();
  $("#tasksList").innerHTML = emptyHtml();
}

function renderHome() {
  const index = state.index;
  const projects = index.projects || [];
  const threads = index.threads || [];
  const skills = index.skills || [];
  const plugins = index.plugins || [];
  const warnings = collectWarnings(index);
  const sourceStats = index.source_stats || {};
  const projectContext = sourceStats.project_context || {};

  $("#generatedAt").textContent = `数据生成时间：${formatDate(index.generated_at)}`;

  const activeProjects = projects.filter((project) => project.active_hint).length;
  const archivedThreads = threads.filter((thread) => thread.archived).length;
  const localSkills = skills.filter((skill) => skill.source_type !== "plugin").length;
  const pluginSkills = skills.filter((skill) => skill.source_type === "plugin").length;
  const harnessCount = projects.reduce((sum, project) => sum + list(project.harness_candidates).length, 0);

  const metrics = [
    ["项目", projects.length, `${activeProjects} 个 active_hint 为真`],
    ["会话", threads.length, `${archivedThreads} 个已归档`],
    ["Skills", skills.length, `${localSkills} 个本地或系统，${pluginSkills} 个插件`],
    ["Plugins", plugins.length, "来自插件 manifest 元数据"],
    ["Harness 候选", harnessCount, `${projectContext.projects_missing ?? "未知"} 个项目根缺失`],
  ];

  $("#metricGrid").replaceChildren(
    ...metrics.map(([label, value, hint]) =>
      el("article", { class: "metric" }, el("span", {}, label), el("strong", {}, String(value)), el("small", {}, hint))
    )
  );

  const warningRows = warnings.slice(0, 12).map((warning) =>
    miniRow(warning.label, warning.detail, warning.level === "warning" ? "warning" : "unknown")
  );
  $("#warningSummary").replaceChildren(...orEmpty(warningRows));

  const recent = [...projects]
    .sort((a, b) => value(b.latest_updated_at_ms) - value(a.latest_updated_at_ms))
    .slice(0, 8)
    .map((project) =>
      miniRow(projectName(project.project_root), project.project_root || "未知项目路径", "candidate", [
        badge(`会话 ${project.thread_count ?? "未知"}`, "neutral"),
        badge(`更新 ${formatDate(project.latest_updated_at_ms)}`, "neutral"),
      ])
    );
  $("#recentProjects").replaceChildren(...orEmpty(recent));
}

function renderProjects() {
  const projects = filterItems(state.index.projects || [], (project) => [
    project.project_root,
    projectName(project.project_root),
    list(project.context_warnings).join(" "),
  ]);

  const cards = projects.map((project) => {
    const warnings = [...list(project.context_warnings), ...list(project.warnings)];
    const authority = list(project.authority_files);
    const handoff = list(project.handoff_files);
    const evidence = list(project.evidence_files);
    const harness = list(project.harness_candidates);

    return el(
      "article",
      { class: "item-card" },
      el(
        "div",
        { class: "item-head" },
        el(
          "div",
          {},
          el("h3", { class: "item-title" }, projectName(project.project_root)),
          el("p", { class: "path-text" }, project.project_root || "未知项目路径")
        ),
        el(
          "div",
          { class: "badge-row" },
          badge("项目类型未知", "unknown"),
          badge(project.active_hint ? "active_hint" : "未标 active_hint", project.active_hint ? "candidate" : "unknown"),
          warnings.length ? badge(`warning ${warnings.length}`, "warning") : badge("warning 0", "neutral")
        )
      ),
      el(
        "div",
        { class: "detail-grid" },
        detail("会话数", project.thread_count ?? "未知"),
        detail("活跃会话", project.active_thread_count ?? "未知"),
        detail("归档会话", project.archived_thread_count ?? "未知"),
        detail("最近更新时间", formatDate(project.latest_updated_at_ms))
      ),
      candidateSection("Authority 候选", authority, "kind"),
      candidateSection("Handoff 候选", handoff, "kind"),
      candidateSection("Evidence 候选", evidence, "kind"),
      candidateSection("Harness 候选", harness.slice(0, 8), "entry_type"),
      warningSection(warnings)
    );
  });

  $("#projectsList").replaceChildren(...orEmpty(cards));
}

function renderSessions() {
  const threads = filterItems(state.index.threads || [], (thread) => [
    thread.title,
    thread.thread_id,
    thread.project_root,
    thread.model,
    list(thread.warnings).join(" "),
  ]).sort((a, b) => value(b.updated_at_ms) - value(a.updated_at_ms));

  if (!threads.length) {
    $("#sessionsList").innerHTML = emptyHtml();
    return;
  }

  const rows = threads.slice(0, 300).map((thread) =>
    el(
      "tr",
      {},
      el("td", { class: "session-title" }, thread.title || "未知标题"),
      el("td", {}, codeText(shortId(thread.thread_id))),
      el("td", {}, codeText(thread.project_root || "未知项目")),
      el("td", {}, formatDate(thread.updated_at_ms)),
      el("td", {}, badge(thread.archived ? "已归档" : "未归档", thread.archived ? "neutral" : "candidate")),
      el("td", {}, badge(thread.rollout_exists ? "rollout 存在" : "rollout 未知/缺失", thread.rollout_exists ? "candidate" : "warning")),
      el("td", {}, `${thread.model || "未知"} / ${thread.reasoning_effort || "未知"}`),
      el("td", {}, warningBadges(thread.warnings))
    )
  );

  const table = el(
    "table",
    {},
    el(
      "thead",
      {},
      el(
        "tr",
        {},
        ...["标题", "编号", "项目路径", "更新时间", "归档", "Rollout", "模型/推理", "Warning"].map((label) => el("th", {}, label))
      )
    ),
    el("tbody", {}, ...rows)
  );

  $("#sessionsList").replaceChildren(table);
}

function renderSkills() {
  const skills = filterItems(state.index.skills || [], (skill) => [
    skill.title,
    skill.skill_id,
    skill.path,
    skill.plugin_name,
    skill.source_type,
    list(skill.warnings).join(" "),
  ]);

  const cards = skills.map((skill) =>
    el(
      "article",
      { class: "item-card" },
      el(
        "div",
        { class: "item-head" },
        el("div", {}, el("h3", { class: "item-title" }, skill.title || skill.skill_id || "未知 skill"), el("p", { class: "path-text" }, skill.path || "未知路径")),
        el(
          "div",
          { class: "badge-row" },
          badge(skillSourceLabel(skill.source_type), skill.source_type === "plugin" ? "candidate" : "unknown"),
          skill.plugin_name ? badge(skill.plugin_name, "neutral") : badge("无插件名", "unknown")
        )
      ),
      el(
        "div",
        { class: "detail-grid" },
        detail("Skill ID", skill.skill_id || "未知"),
        detail("来源", skill.source_type || "未知"),
        detail("插件版本", skill.plugin_version || "不适用"),
        detail("Warning", list(skill.warnings).length)
      ),
      warningSection(skill.warnings)
    )
  );

  $("#skillsList").replaceChildren(...orEmpty(cards));
}

function renderHarness() {
  const projects = state.index.projects || [];
  const candidates = projects.flatMap((project) =>
    list(project.harness_candidates).map((candidate) => ({
      ...candidate,
      project_root: project.project_root,
    }))
  );

  const filtered = filterItems(candidates, (candidate) => [
    candidate.name,
    candidate.path,
    candidate.project_root,
    candidate.entry_type,
    candidate.source,
    list(candidate.warnings).join(" "),
  ]);

  const cards = filtered.map((candidate) =>
    el(
      "article",
      { class: "item-card" },
      el(
        "div",
        { class: "item-head" },
        el("div", {}, el("h3", { class: "item-title" }, candidate.name || "未知入口"), el("p", { class: "path-text" }, candidate.path || "未知路径")),
        el("div", { class: "badge-row" }, badge("候选", "candidate"), badge(candidate.entry_type || "未知类型", "unknown"))
      ),
      el(
        "div",
        { class: "detail-grid" },
        detail("所属项目", projectName(candidate.project_root)),
        detail("来源", candidate.source || "未知"),
        detail("更新时间", formatDate(candidate.updated_at_ms)),
        detail("大小", formatBytes(candidate.size_bytes))
      ),
      el("p", { class: "muted" }, "静态壳只登记候选入口，不展示命令正文，也不提供运行按钮。"),
      warningSection(candidate.warnings)
    )
  );

  $("#harnessList").replaceChildren(...orEmpty(cards));
}

function renderTasks() {
  const tasks = state.tasks || parseTasks("");
  const labels = [
    ["pending", "待派发"],
    ["active", "进行中"],
    ["done", "已回收"],
    ["paused", "暂停"],
  ];

  const columns = labels.map(([key, label]) =>
    el(
      "article",
      { class: "panel task-column" },
      el("h3", {}, label),
      el(
        "div",
        { class: "list-stack" },
        ...orEmpty(
          list(tasks[key]).map((item) =>
            miniRow(item.title, "任务队列候选入口；不展开任务说明正文。", key === "done" ? "candidate" : key === "paused" ? "warning" : "unknown")
          )
        )
      )
    )
  );

  $("#tasksList").replaceChildren(...columns);
}

function parseTasks(markdown) {
  const result = {
    pending: [],
    active: [],
    done: [],
    paused: [],
  };

  const sectionMap = {
    "待派发": "pending",
    "进行中": "active",
    "已回收": "done",
    "暂停": "paused",
  };

  let current = null;
  for (const rawLine of markdown.split(/\r?\n/)) {
    const heading = rawLine.match(/^##\s+(.+?)\s*$/);
    if (heading) {
      current = sectionMap[heading[1].trim()] || null;
      continue;
    }
    if (!current) continue;
    const item = rawLine.match(/^-\s+(.+?)\s*$/);
    if (!item) continue;
    const text = item[1].trim();
    const [title] = text.split("：");
    result[current].push({
      title: stripMarkdown(title),
      detail: "",
    });
  }

  return result;
}

function collectWarnings(index) {
  const rows = [];
  for (const warning of list(index.warnings)) {
    rows.push({ level: "warning", label: "全局 warning", detail: warning });
  }
  for (const project of list(index.projects)) {
    for (const warning of [...list(project.context_warnings), ...list(project.warnings)]) {
      rows.push({ level: "warning", label: projectName(project.project_root), detail: warning });
    }
  }
  for (const thread of list(index.threads)) {
    for (const warning of list(thread.warnings)) {
      rows.push({ level: "warning", label: `会话 ${shortId(thread.thread_id)}`, detail: warning });
    }
  }
  for (const skill of list(index.skills)) {
    for (const warning of list(skill.warnings)) {
      rows.push({ level: "warning", label: skill.title || skill.skill_id || "skill", detail: warning });
    }
  }
  if (!rows.length) {
    rows.push({ level: "unknown", label: "全局 warning", detail: "索引没有提供 warning。这个结论只代表当前静态样例。" });
  }
  return rows;
}

function candidateSection(title, files, kindKey) {
  const items = list(files);
  return el(
    "div",
    { class: "candidate-block" },
    el("h4", {}, `${title}：${items.length ? `${items.length} 个` : "未知或暂无"}`),
    items.length
      ? el(
          "ul",
          { class: "file-list" },
          ...items.slice(0, 10).map((file) =>
            el(
              "li",
              {},
              el("b", {}, file[kindKey] || file.name || "候选"),
              el("span", {}, file.path || "未知路径")
            )
          )
        )
      : emptyNode()
  );
}

function warningSection(warnings) {
  const items = list(warnings);
  if (!items.length) {
    return el("div", { class: "badge-row" }, badge("warning 0", "neutral"));
  }
  return el("div", { class: "badge-row" }, ...items.map((warning) => badge(warning, "warning")));
}

function warningBadges(warnings) {
  const items = list(warnings);
  if (!items.length) return badge("无", "neutral");
  return el("div", { class: "badge-row" }, ...items.map((warning) => badge(warning, "warning")));
}

function detail(label, valueText) {
  return el("div", { class: "detail" }, el("span", {}, label), el("strong", {}, String(valueText ?? "未知")));
}

function miniRow(title, detailText, tone = "neutral", extra = []) {
  return el(
    "div",
    { class: "mini-row" },
    el("strong", {}, title || "未知"),
    el("span", {}, detailText || "未知"),
    extra.length ? el("div", { class: "badge-row" }, badge(toneLabel(tone), tone), ...extra) : el("div", { class: "badge-row" }, badge(toneLabel(tone), tone))
  );
}

function toneLabel(tone) {
  if (tone === "warning") return "warning";
  if (tone === "unknown") return "未知";
  if (tone === "candidate") return "候选";
  return "元数据";
}

function badge(text, tone = "neutral") {
  return el("span", { class: `badge ${tone}` }, text);
}

function codeText(text) {
  return el("span", { class: "path-text" }, text || "未知");
}

function el(tag, attrs = {}, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (value === false || value === null || value === undefined) continue;
    if (key === "class") node.className = value;
    else node.setAttribute(key, value);
  }
  for (const child of children.flat()) {
    if (child === null || child === undefined || child === false) continue;
    node.append(child instanceof Node ? child : document.createTextNode(String(child)));
  }
  return node;
}

function emptyNode() {
  return $("#emptyTemplate").content.firstElementChild.cloneNode(true);
}

function emptyHtml() {
  const wrapper = document.createElement("div");
  wrapper.append(emptyNode());
  return wrapper.innerHTML;
}

function orEmpty(nodes) {
  return nodes.length ? nodes : [emptyNode()];
}

function filterItems(items, fieldsFn) {
  const query = state.query;
  if (!query) return items;
  return items.filter((item) => fieldsFn(item).some((field) => String(field || "").toLowerCase().includes(query)));
}

function projectName(path) {
  if (!path) return "未知项目";
  const parts = path.split("/").filter(Boolean);
  return parts[parts.length - 1] || path;
}

function shortId(id) {
  if (!id) return "未知";
  return id.length > 12 ? id.slice(0, 8) : id;
}

function skillSourceLabel(sourceType) {
  if (sourceType === "plugin") return "插件 skill";
  if (sourceType === "system") return "系统 skill";
  if (sourceType === "user") return "本地 skill";
  return "来源未知";
}

function formatDate(value) {
  if (!value) return "未知";
  const date = typeof value === "number" ? new Date(value) : new Date(value);
  if (Number.isNaN(date.getTime())) return "未知";
  return date.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatBytes(value) {
  if (typeof value !== "number") return "未知";
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${Math.round(value / 1024)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

function stripMarkdown(text) {
  return String(text || "")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .trim();
}

function list(value) {
  return Array.isArray(value) ? value : [];
}

function value(raw) {
  return typeof raw === "number" ? raw : 0;
}
