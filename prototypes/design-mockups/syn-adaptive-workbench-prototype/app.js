(() => {
  "use strict";

  const pages = new Map(
    [...document.querySelectorAll("[data-page]")].map((page) => [page.dataset.page, page])
  );
  const main = document.querySelector(".main");
  const sidebar = document.getElementById("sidebar");
  const toast = document.getElementById("toast");
  const menuButton = document.getElementById("mobile-menu-button");
  const scrim = document.getElementById("sidebar-scrim");
  const decisionTopButton = document.getElementById("decision-top-button");
  const agentPage = document.querySelector(".agent-page");
  const conversationList = document.getElementById("conversation-list");
  const conversationMain = document.querySelector(".conversation-main");
  const mobileRoleButton = document.getElementById("mobile-role-button");
  const mobileListClose = document.getElementById("mobile-list-close");
  const agentListScrim = document.getElementById("agent-list-scrim");
  let toastTimer;
  let decisionState = "pending";
  let runningCount = 3;
  let currentRole = "manager";

  const roles = {
    manager: { name: "项目主管", mark: "主", tone: "role", meta: "Syn 工作台 · 会影响方向的角色" },
    advisor: { name: "项目咨询", mark: "询", tone: "role", meta: "Syn 工作台 · 会影响方向的角色" },
    frontend: { name: "前端开发", mark: "前", tone: "worker", meta: "Syn 工作台 · 受限执行角色" },
    review: { name: "体验检查", mark: "测", tone: "worker", meta: "Syn 工作台 · 受限执行角色" },
  };

  const simplePageRows = {
    ideas: [
      ["记", "随手记下", "一句不完整的想法也可以"],
      ["看", "等你确认", "Syn 整理后再问你要不要推进"],
      ["归", "自动归类", "确认后放进合适的项目"],
    ],
    canvas: [
      ["现", "当前实验", "正在验证新手首页是否足够简单"],
      ["下", "下一步", "请一个第一次使用的人完成主要流程"],
      ["停", "停止条件", "看不懂或需要猜，就先停下来修改"],
    ],
    command: [
      ["令", "明确指令", "适合已经知道要做什么时使用"],
      ["界", "权限边界", "范围变化会在执行前单独确认"],
      ["验", "完成标准", "Syn 会把结果和证据一起交给你"],
    ],
    knowledge: [
      ["权", "当前权威", "Syn 工作台产品蓝图"],
      ["研", "研究资料", "工作接续与自动恢复研究"],
      ["史", "历史资料", "旧方案仍可查，但不会混入当前结论"],
    ],
    memory: [
      ["候", "候选记忆", "2 条等待你确认"],
      ["稳", "长期事实", "只保存稳定、可复用的信息"],
      ["删", "随时可撤回", "你可以查看、修正或删除"],
    ],
    skills: [
      ["常", "常用能力", "研究、设计检查、前端开发"],
      ["项", "项目专用", "只在对应项目范围内启用"],
      ["审", "启用前检查", "高风险能力不会静默开启"],
    ],
    settings: [
      ["项", "项目设置", "成员、资料来源和默认工作方式"],
      ["隐", "隐私与权限", "查看 Syn 可以读取和发送什么"],
      ["自", "自动化边界", "决定哪些小故障可以自动恢复"],
    ],
  };

  function createIcon(symbolId) {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("aria-hidden", "true");
    const use = document.createElementNS("http://www.w3.org/2000/svg", "use");
    use.setAttribute("href", `#${symbolId}`);
    svg.append(use);
    return svg;
  }

  function buildSimplePages() {
    document.querySelectorAll(".simple-page").forEach((page) => {
      const content = document.createElement("div");
      content.className = "simple-content";

      const hero = document.createElement("section");
      hero.className = "simple-hero";
      const eyebrow = document.createElement("p");
      eyebrow.className = "eyebrow";
      eyebrow.textContent = page.dataset.kicker;
      const title = document.createElement("h1");
      title.id = `${page.dataset.page}-title`;
      title.textContent = page.dataset.title;
      const copy = document.createElement("p");
      copy.textContent = page.dataset.copy;
      hero.append(eyebrow, title, copy);
      page.setAttribute("aria-labelledby", title.id);

      const list = document.createElement("div");
      list.className = "simple-placeholder";
      (simplePageRows[page.dataset.page] || []).forEach(([mark, name, detail]) => {
        const row = document.createElement("div");
        row.className = "simple-row";
        const icon = document.createElement("span");
        icon.setAttribute("aria-hidden", "true");
        icon.textContent = mark;
        const text = document.createElement("span");
        const strong = document.createElement("strong");
        strong.textContent = name;
        const small = document.createElement("small");
        small.textContent = detail;
        text.append(strong, small);
        row.append(icon, text);
        list.append(row);
      });

      content.append(hero, list);
      page.append(content);
    });
  }

  function showToast(message) {
    window.clearTimeout(toastTimer);
    toast.textContent = message;
    toast.classList.add("show");
    toastTimer = window.setTimeout(() => toast.classList.remove("show"), 3600);
  }

  function closeSidebar({ restoreFocus = false } = {}) {
    document.body.classList.remove("sidebar-open");
    main.inert = false;
    main.removeAttribute("aria-hidden");
    menuButton.setAttribute("aria-expanded", "false");
    menuButton.setAttribute("aria-label", "打开导航");
    menuButton.querySelector("use").setAttribute("href", "#i-menu");
    if (restoreFocus) menuButton.focus();
  }

  function openSidebar() {
    closeRoleList();
    document.body.classList.add("sidebar-open");
    main.inert = true;
    main.setAttribute("aria-hidden", "true");
    menuButton.setAttribute("aria-expanded", "true");
    menuButton.setAttribute("aria-label", "关闭导航");
    menuButton.querySelector("use").setAttribute("href", "#i-close");
    const activeItem = document.querySelector(".sidebar .nav-item.active") || document.querySelector(".sidebar .nav-item");
    activeItem?.focus();
  }

  function closeRoleList({ restoreFocus = false } = {}) {
    agentPage.classList.remove("role-list-open");
    conversationMain.inert = false;
    conversationMain.removeAttribute("aria-hidden");
    mobileRoleButton.setAttribute("aria-expanded", "false");
    if (restoreFocus) mobileRoleButton.focus();
  }

  function openRoleList() {
    closeSidebar();
    agentPage.classList.add("role-list-open");
    conversationMain.inert = true;
    conversationMain.setAttribute("aria-hidden", "true");
    mobileRoleButton.setAttribute("aria-expanded", "true");
    const selectedRole = conversationList.querySelector(".conversation-row.selected");
    (selectedRole || mobileListClose).focus();
  }

  function trapFocus(event, elements) {
    const focusable = elements.filter((element) => element && !element.disabled && element.offsetParent !== null);
    if (!focusable.length) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function showPage(pageName, { focus = true, updateHash = true } = {}) {
    const nextPage = pages.get(pageName) || pages.get("home");
    const resolvedName = nextPage.dataset.page;

    pages.forEach((page) => page.classList.toggle("active", page === nextPage));
    document.querySelectorAll(".sidebar .nav-item[data-nav]").forEach((button) => {
      const isActive = button.dataset.nav === resolvedName;
      button.classList.toggle("active", isActive);
      if (isActive) button.setAttribute("aria-current", "page");
      else button.removeAttribute("aria-current");
    });

    main.scrollTop = 0;
    closeSidebar();
    closeRoleList();
    if (updateHash && window.location.hash !== `#${resolvedName}`) {
      history.replaceState(null, "", `#${resolvedName}`);
    }
    if (focus) {
      window.requestAnimationFrame(() => main.focus({ preventScroll: true }));
    }
  }

  function updateDecisionCounters({ label = "没有待决定事项", pending = false } = {}) {
    document.getElementById("status-decision-count").textContent = pending ? "1" : "0";
    document.getElementById("decision-count-badge").textContent = pending ? "1" : "0";
    document.getElementById("decision-top-label").textContent = label;
    decisionTopButton.classList.toggle("resolved", !pending);
    decisionTopButton.setAttribute("aria-label", label);
    refreshProjectSummaries(pending);
  }

  function updateDecisionSection(mode) {
    const kicker = document.getElementById("decision-section-kicker");
    const heading = document.getElementById("decision-heading");
    const count = document.getElementById("decision-count-badge");
    count.hidden = mode !== "pending";
    if (mode === "processing") {
      kicker.textContent = "Syn 会先核对，再继续";
      heading.textContent = "正在处理";
      return;
    }
    if (mode === "complete" || mode === "paused") {
      kicker.textContent = "刚刚处理";
      heading.textContent = "处理结果";
      return;
    }
    kicker.textContent = "只在必须由你决定时出现";
    heading.textContent = "需要你决定";
  }

  function refreshProjectSummaries(pending = decisionState === "pending") {
    document.getElementById("status-running-count").textContent = String(runningCount);
    document.getElementById("project-running-count").textContent = `${runningCount} 件进行中`;
    document.getElementById("recent-syn-status").textContent = `${runningCount} 件推进中 · ${pending ? "1 件等你" : "没有待决定"}`;
  }

  function replaceTimelineTags(container, labels) {
    container.replaceChildren();
    labels.forEach((label) => {
      const tag = document.createElement("span");
      tag.textContent = label;
      container.append(tag);
    });
  }

  function updateKeyRoleReferences(mode) {
    const conversationStatus = document.getElementById("key-role-conversation-status");
    const projectStatus = document.getElementById("project-decision-status");
    const projectStatusText = projectStatus.querySelector("span");
    const history = document.querySelector("[data-key-role-history]");
    const historyDot = history.querySelector(".timeline-dot");
    const historyIcon = historyDot.querySelector("use");
    const historyTitle = history.querySelector(".timeline-head strong");
    const historyCopy = history.querySelector(":scope > div > p");
    const historyTags = history.querySelector(".timeline-tags");
    document.getElementById("key-role-attention")?.remove();

    if (mode === "processing") {
      conversationStatus.textContent = "Syn 工作台 · 正在核对交接";
      projectStatus.className = "project-next ok";
      projectStatusText.textContent = "正在核对交接";
      historyTitle.textContent = "项目主管正在核对交接";
      historyCopy.textContent = "Syn 正在确认目标、当前方案、权限和验收；全部一致后才会继续。";
      replaceTimelineTags(historyTags, ["关键角色", "核对中"]);
      return;
    }

    if (mode === "complete") {
      conversationStatus.textContent = "Syn 工作台 · 已完成交接，正在工作";
      projectStatus.className = "project-next ok";
      projectStatusText.textContent = "没有需要你处理的事";
      historyDot.className = "timeline-dot ok";
      historyIcon.setAttribute("href", "#i-check");
      historyTitle.textContent = "项目主管已完成交接并继续";
      historyCopy.textContent = "目标、当前方案、权限和验收核对一致；原对话保留，工作已在干净环境中接续。";
      replaceTimelineTags(historyTags, ["关键角色", "交接通过", "范围未变"]);
      return;
    }

    conversationStatus.textContent = "Syn 工作台 · 已安全暂停";
    projectStatus.className = "project-next muted";
    projectStatusText.textContent = "关键工作已暂停，没有待决定";
    historyTitle.textContent = "你已选择暂停项目主管";
    historyCopy.textContent = "没有启动新对话，也没有改变项目范围；只有出现新证据或你主动回来时才会提醒。";
    replaceTimelineTags(historyTags, ["关键角色", "安全暂停"]);
  }

  function createDecisionResult({ title, copy, paused = false }) {
    const result = document.createElement("div");
    result.className = `decision-success${paused ? " paused" : ""}`;
    result.setAttribute("role", "status");
    result.tabIndex = -1;
    const iconWrap = document.createElement("span");
    iconWrap.append(createIcon(paused ? "i-pause" : "i-check"));
    const text = document.createElement("div");
    const strong = document.createElement("strong");
    strong.textContent = title;
    const paragraph = document.createElement("p");
    paragraph.textContent = copy;
    text.append(strong, paragraph);
    result.append(iconWrap, text);
    return result;
  }

  function replaceDecisionSurfaces(result) {
    const activeElement = document.activeElement;
    let focusTarget;
    document.querySelectorAll("[data-decision-surface]").forEach((surface) => {
      const replacement = createDecisionResult(result);
      replacement.dataset.decisionSurface = "";
      if (surface.contains(activeElement)) focusTarget = replacement;
      surface.replaceWith(replacement);
    });
    if (focusTarget) window.requestAnimationFrame(() => focusTarget.focus({ preventScroll: true }));
  }

  function updateWaitingWork({ state, title, detail, progress }) {
    const row = document.querySelector("[data-key-role-work]");
    if (!row) return;
    const workStatus = row.querySelector(".work-status");
    workStatus.className = `work-status ${state}`;
    workStatus.replaceChildren(document.createElement("i"), document.createTextNode(title));
    row.querySelector(".work-copy small").textContent = detail;
    row.querySelector(".work-progress").textContent = progress;
  }

  function addQuietRecoveryRecord() {
    const list = document.getElementById("quiet-list");
    if (list.querySelector("[data-new-handoff]")) return;
    const details = document.createElement("details");
    details.className = "quiet-item";
    details.dataset.newHandoff = "";
    const summary = document.createElement("summary");
    const icon = document.createElement("span");
    icon.className = "quiet-icon";
    icon.append(createIcon("i-check"));
    const copy = document.createElement("span");
    const strong = document.createElement("strong");
    strong.textContent = "项目主管已完成干净交接";
    const small = document.createElement("small");
    small.textContent = "目标和权限核对通过，已继续当前工作";
    copy.append(strong, small);
    const time = document.createElement("time");
    time.textContent = "刚刚";
    summary.append(icon, copy, time);
    const body = document.createElement("div");
    const paragraph = document.createElement("p");
    paragraph.textContent = "原对话已保留；新工作核对了目标、当前方案、权限和验收，没有扩大范围。";
    body.append(paragraph);
    details.append(summary, body);
    list.prepend(details);
  }

  function handleDecision(action) {
    if (decisionState !== "pending" && action !== "later") return;

    if (action === "later") {
      document.querySelectorAll('[data-decision-action="later"]').forEach((button) => {
        button.textContent = "会稍后提醒";
      });
      showToast("已记下稍后提醒；项目主管会保持暂停，不会先继续。");
      return;
    }

    if (action === "pause") {
      decisionState = "paused";
      replaceDecisionSurfaces({
        paused: true,
        title: "已暂停，不会继续",
        copy: "项目主管和原对话都已保持原样。只有出现新证据，或你主动回来时，Syn 才会再次提醒。",
      });
      updateDecisionSection("paused");
      updateDecisionCounters();
      updateKeyRoleReferences("paused");
      updateWaitingWork({ state: "waiting", title: "已暂停", detail: "等你主动回来，不会自行继续", progress: "安全停住" });
      showToast("已安全暂停。没有启动新对话，也没有改变项目范围。");
      return;
    }

    if (action === "accept") {
      decisionState = "processing";
      replaceDecisionSurfaces({
        title: "正在核对交接",
        copy: "Syn 正在逐项确认目标、当前方案、权限和验收；全部一致后才会继续。",
      });
      updateDecisionSection("processing");
      updateDecisionCounters({ label: "正在核对交接" });
      updateKeyRoleReferences("processing");
      updateWaitingWork({ state: "checking", title: "核对中", detail: "正在确认目标、权限和验收", progress: "约 20 秒" });
      showToast("已收到。Syn 正在做干净交接，核对通过前不会继续。");

      window.setTimeout(() => {
        decisionState = "complete";
        replaceDecisionSurfaces({
          title: "交接核对完成，工作已接续",
          copy: "原对话完整保留；目标、当前方案、权限和验收均一致，没有改变范围或使用能力。",
        });
        updateDecisionSection("complete");
        updateDecisionCounters();
        updateKeyRoleReferences("complete");
        updateWaitingWork({ state: "running", title: "已接续", detail: "干净交接核对完成，正在继续当前工作", progress: "刚刚" });
        addQuietRecoveryRecord();
        showToast("交接检查通过，项目主管已继续当前工作。");
      }, 1300);
    }
  }

  function createWorkRow(goal) {
    const row = document.createElement("button");
    row.className = "work-row";
    row.type = "button";
    row.dataset.nav = "projects";
    const status = document.createElement("span");
    status.className = "work-status checking";
    status.append(document.createElement("i"), document.createTextNode("整理中"));
    const copy = document.createElement("span");
    copy.className = "work-copy";
    const strong = document.createElement("strong");
    strong.textContent = goal;
    const small = document.createElement("small");
    small.textContent = "Syn 正在整理目标和完成标准";
    copy.append(strong, small);
    const progress = document.createElement("span");
    progress.className = "work-progress";
    progress.textContent = "刚刚收到";
    row.append(status, copy, progress, createIcon("i-chevron"));
    return row;
  }

  function createChatMessage(text, type = "user", roleKey = currentRole) {
    const role = roles[roleKey];
    const message = document.createElement("div");
    message.className = `message ${type}`;
    const avatar = document.createElement("span");
    avatar.className = `avatar ${type === "user" ? "user" : role.tone}`;
    avatar.textContent = type === "user" ? "你" : role.mark;
    const body = document.createElement("div");
    const meta = document.createElement("small");
    meta.textContent = type === "user" ? "你 · 刚刚" : `${role.name} · 刚刚`;
    const paragraph = document.createElement("p");
    paragraph.textContent = text;
    body.append(meta, paragraph);
    message.append(avatar, body);
    return message;
  }

  function selectRole(roleKey, { restoreFocus = true } = {}) {
    const role = roles[roleKey];
    if (!role) return;
    currentRole = roleKey;

    document.querySelectorAll(".conversation-row[data-role]").forEach((row) => {
      const isSelected = row.dataset.role === roleKey;
      row.classList.toggle("selected", isSelected);
      row.setAttribute("aria-pressed", String(isSelected));
    });
    document.querySelectorAll("[data-conversation-view]").forEach((view) => {
      view.classList.toggle("active", view.dataset.conversationView === roleKey);
    });

    const avatar = document.getElementById("conversation-role-avatar");
    avatar.className = `avatar ${role.tone}`;
    avatar.textContent = role.mark;
    document.getElementById("conversation-role-name").textContent = role.name;
    document.getElementById("conversation-role-meta").textContent = role.meta;
    document.getElementById("chat-input").placeholder = `告诉${role.name}你想怎么做`;
    document.querySelector(".conversation-body").scrollTop = 0;
    closeRoleList({ restoreFocus: restoreFocus && window.matchMedia("(max-width: 680px)").matches });
  }

  buildSimplePages();

  document.addEventListener("click", (event) => {
    const navTarget = event.target.closest("[data-nav]");
    if (navTarget) {
      showPage(navTarget.dataset.nav);
      return;
    }

    const decisionAction = event.target.closest("[data-decision-action]");
    if (decisionAction) {
      handleDecision(decisionAction.dataset.decisionAction);
      return;
    }

    const roleTarget = event.target.closest(".conversation-row[data-role]");
    if (roleTarget) {
      selectRole(roleTarget.dataset.role);
      return;
    }

    const feedbackTarget = event.target.closest("[data-demo-feedback]");
    if (feedbackTarget) showToast(feedbackTarget.dataset.demoFeedback);
  });

  menuButton.addEventListener("click", () => {
    if (document.body.classList.contains("sidebar-open")) closeSidebar({ restoreFocus: true });
    else openSidebar();
  });
  scrim.addEventListener("click", () => closeSidebar({ restoreFocus: true }));
  mobileRoleButton.addEventListener("click", () => {
    if (agentPage.classList.contains("role-list-open")) closeRoleList({ restoreFocus: true });
    else openRoleList();
  });
  mobileListClose.addEventListener("click", () => closeRoleList({ restoreFocus: true }));
  agentListScrim.addEventListener("click", () => closeRoleList({ restoreFocus: true }));

  document.querySelector(".list-search input").addEventListener("input", (event) => {
    const query = event.target.value.trim().toLocaleLowerCase("zh-CN");
    document.querySelectorAll(".conversation-row[data-role]").forEach((row) => {
      row.hidden = Boolean(query) && !row.textContent.toLocaleLowerCase("zh-CN").includes(query);
    });
  });

  decisionTopButton.addEventListener("click", () => {
    showPage("home");
    window.requestAnimationFrame(() => {
      const surface = document.querySelector("[data-page='home'] [data-decision-surface]");
      surface?.classList.add("highlight");
      surface?.scrollIntoView({ behavior: "smooth", block: "center" });
      window.setTimeout(() => surface?.classList.remove("highlight"), 1000);
    });
  });

  document.getElementById("command-form").addEventListener("submit", (event) => {
    event.preventDefault();
    const input = document.getElementById("command-input");
    const goal = input.value.trim();
    if (!goal) {
      input.setAttribute("aria-invalid", "true");
      input.focus();
      showToast("先随便说一句你想完成什么，不需要写成专业需求。");
      return;
    }
    input.removeAttribute("aria-invalid");
    document.getElementById("work-list").prepend(createWorkRow(goal));
    runningCount += 1;
    refreshProjectSummaries();
    input.value = "";
    showToast("Syn 已收到，会先整理方案；需要你决定时再来找你。");
  });

  document.getElementById("chat-form").addEventListener("submit", (event) => {
    event.preventDefault();
    const input = document.getElementById("chat-input");
    const text = input.value.trim();
    if (!text) {
      input.focus();
      return;
    }
    const body = document.querySelector(".conversation-body");
    const roleKey = currentRole;
    const activeView = document.querySelector(`[data-conversation-view="${roleKey}"]`);
    activeView.append(createChatMessage(text, "user", roleKey));
    input.value = "";
    body.scrollTop = body.scrollHeight;
    window.setTimeout(() => {
      activeView.append(createChatMessage("收到。我会先守住现有边界，再继续整理下一步。", "assistant", roleKey));
      if (currentRole === roleKey) body.scrollTop = body.scrollHeight;
    }, 500);
  });

  const globalSearch = document.querySelector(".global-search input");
  globalSearch.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      const query = globalSearch.value.trim();
      showToast(query ? `正在原型中搜索“${query}”` : "输入项目、对话或记忆的名字即可搜索。");
    }
  });

  document.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      if (globalSearch.offsetParent !== null) {
        event.preventDefault();
        globalSearch.focus();
      }
    }
    if (event.key === "Tab" && document.body.classList.contains("sidebar-open")) {
      trapFocus(event, [menuButton, ...sidebar.querySelectorAll("button")]);
    }
    if (event.key === "Tab" && agentPage.classList.contains("role-list-open")) {
      trapFocus(event, [...conversationList.querySelectorAll("button, input")]);
    }
    if (event.key === "Escape") {
      if (document.body.classList.contains("sidebar-open")) closeSidebar({ restoreFocus: true });
      else if (agentPage.classList.contains("role-list-open")) closeRoleList({ restoreFocus: true });
    }
  });

  window.addEventListener("resize", () => {
    if (window.innerWidth > 900 && document.body.classList.contains("sidebar-open")) closeSidebar();
    if (window.innerWidth > 680 && agentPage.classList.contains("role-list-open")) closeRoleList();
  });

  window.addEventListener("hashchange", () => {
    showPage(window.location.hash.slice(1), { updateHash: false });
  });

  const initialPage = window.location.hash.slice(1);
  selectRole("manager", { restoreFocus: false });
  showPage(pages.has(initialPage) ? initialPage : "home", { focus: false });
})();
