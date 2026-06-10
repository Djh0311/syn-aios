const invoke = window.__TAURI__?.core?.invoke;

const state = {
  summary: null,
};

const elements = {
  notice: document.querySelector("#notice"),
  generatedAt: document.querySelector("#generatedAt"),
  indexPath: document.querySelector("#indexPath"),
  projectCount: document.querySelector("#projectCount"),
  threadCount: document.querySelector("#threadCount"),
  skillCount: document.querySelector("#skillCount"),
  pluginCount: document.querySelector("#pluginCount"),
  projectActionCount: document.querySelector("#projectActionCount"),
  rolloutActionCount: document.querySelector("#rolloutActionCount"),
  warnings: document.querySelector("#warnings"),
  projectPath: document.querySelector("#projectPath"),
  rolloutPath: document.querySelector("#rolloutPath"),
  reloadButton: document.querySelector("#reloadButton"),
  copyProjectButton: document.querySelector("#copyProjectButton"),
  openProjectButton: document.querySelector("#openProjectButton"),
  copyRolloutButton: document.querySelector("#copyRolloutButton"),
  revealRolloutButton: document.querySelector("#revealRolloutButton"),
};

document.addEventListener("DOMContentLoaded", () => {
  bindActions();
  loadSummary();
});

function bindActions() {
  elements.reloadButton.addEventListener("click", loadSummary);
  elements.copyProjectButton.addEventListener("click", () => copyPath(state.summary?.first_project_path));
  elements.copyRolloutButton.addEventListener("click", () => copyPath(state.summary?.first_rollout_path));
  elements.openProjectButton.addEventListener("click", () => openProject(state.summary?.first_project_path));
  elements.revealRolloutButton.addEventListener("click", () => revealRollout(state.summary?.first_rollout_path));
}

async function loadSummary() {
  setNotice("正在读取静态索引。");
  try {
    ensureTauri();
    state.summary = await invoke("load_probe_summary");
    renderSummary();
    setNotice("已读取索引。桌面动作仍需用户点击触发。");
  } catch (error) {
    state.summary = null;
    renderSummary();
    setNotice(`读取失败：${messageOf(error)}`, true);
  }
}

function renderSummary() {
  const summary = state.summary;
  elements.generatedAt.textContent = summary?.generated_at || "生成时间未知";
  elements.indexPath.textContent = summary?.index_path || "未知";
  elements.projectCount.textContent = numberText(summary?.project_count);
  elements.threadCount.textContent = numberText(summary?.thread_count);
  elements.skillCount.textContent = numberText(summary?.skill_count);
  elements.pluginCount.textContent = numberText(summary?.plugin_count);
  elements.projectActionCount.textContent = numberText(summary?.project_action_count);
  elements.rolloutActionCount.textContent = numberText(summary?.rollout_action_count);
  elements.warnings.textContent = summary?.warnings?.length ? summary.warnings.join(", ") : "无顶层 warning";
  elements.projectPath.textContent = summary?.first_project_path || "索引未提供项目路径";
  elements.rolloutPath.textContent = summary?.first_rollout_path || "索引未提供 rollout 路径";

  const hasProject = Boolean(summary?.first_project_path);
  const hasRollout = Boolean(summary?.first_rollout_path);
  elements.copyProjectButton.disabled = !hasProject;
  elements.openProjectButton.disabled = !hasProject;
  elements.copyRolloutButton.disabled = !hasRollout;
  elements.revealRolloutButton.disabled = !hasRollout;
}

async function copyPath(path) {
  if (!path) return;
  try {
    ensureTauri();
    const result = await invoke("copy_indexed_path", { path });
    setNotice(result);
  } catch (error) {
    setNotice(`复制失败：${messageOf(error)}`, true);
  }
}

async function openProject(path) {
  if (!path) return;
  try {
    ensureTauri();
    const result = await invoke("open_indexed_project", { path });
    setNotice(result);
  } catch (error) {
    setNotice(`打开失败：${messageOf(error)}`, true);
  }
}

async function revealRollout(path) {
  if (!path) return;
  try {
    ensureTauri();
    const result = await invoke("reveal_indexed_rollout", { path });
    setNotice(result);
  } catch (error) {
    setNotice(`定位失败：${messageOf(error)}`, true);
  }
}

function ensureTauri() {
  if (!invoke) {
    throw new Error("当前页面不在 Tauri 窗口中运行");
  }
}

function setNotice(text, isError = false) {
  elements.notice.textContent = text;
  elements.notice.classList.toggle("error", isError);
}

function numberText(value) {
  return Number.isFinite(value) ? String(value) : "-";
}

function messageOf(error) {
  return error?.message || String(error);
}
