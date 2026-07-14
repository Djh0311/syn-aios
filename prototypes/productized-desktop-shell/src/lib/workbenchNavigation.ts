export type ViewKey =
  | "home"
  | "projects"
  | "ideas"
  | "proposal"
  | "workflow"
  | "command-console"
  | "agents"
  | "knowledge"
  | "memory"
  | "skills"
  | "harness"
  | "tools"
  | "models"
  | "settings"
  // Part②·秘书看板视图（经铃 / 右栏「打开看板」可达；不进左导航——红线）。
  | "secretary_board"
  // ④·审计账本页（宪法 §二「审计/账本：任何态都不是主角；常驻"可查"位」→ 不进左导航；
  //   可达路径 = 右栏 rail「管」抽屉点行直达）。
  | "audit-ledger";

// 通用导航焦点（「点击带上下文直达」）：调用方给出「跳哪一页 + 落在哪一条」，
// 目标页据 focus 选中那一条。刻意不绑右栏——首页「每条可点直达」等任何列表调用点同样复用。
// kind 由调用方与目标页约定（如 audit-event / runtime-log / degraded-state），不在此枚举，
// 免得每加一种可点行就要改本文件。
export type NavigationFocus = {
  kind: string;
  id: string;
};

// 导航句柄统一签名：focus 可省 = 只切页不选行（既有调用点零破坏）。
export type NavigateHandler = (view: ViewKey, focus?: NavigationFocus) => void;

export type RightPanelKey = "notifications" | "todos" | "audit" | "running" | "ideas" | "secretary";

export type WorkbenchNavItem = {
  key: ViewKey;
  label: string;
  glyph: string;
};

export type WorkbenchNavGroup = {
  key: string;
  label: string;
  items: WorkbenchNavItem[];
};

export const homeNavItem: WorkbenchNavItem = { key: "home", label: "首页", glyph: "⌂" };

export const primaryNavItems: WorkbenchNavItem[] = [
  { key: "projects", label: "项目", glyph: "▤" },
  { key: "agents", label: "智能体", glyph: "◍" },
  { key: "ideas", label: "想法箱", glyph: "✎" },
  { key: "knowledge", label: "知识库", glyph: "▢" },
  { key: "memory", label: "记忆层", glyph: "◐" },
  { key: "skills", label: "技能", glyph: "✦" },
  { key: "harness", label: "运行器", glyph: "⬡" },
  { key: "workflow", label: "实验画布", glyph: "⊹" },
];

export const primaryNavGroups: WorkbenchNavGroup[] = [
  {
    key: "main",
    label: "主入口",
    items: [
      { key: "projects", label: "项目", glyph: "▤" },
      { key: "agents", label: "智能体", glyph: "◍" },
    ],
  },
  {
    key: "flow",
    label: "流转",
    items: [
      { key: "ideas", label: "想法箱", glyph: "✎" },
      { key: "workflow", label: "实验画布", glyph: "⊹" },
      { key: "command-console", label: "发令台", glyph: "►" },
    ],
  },
  {
    key: "memory",
    label: "积累",
    items: [
      { key: "knowledge", label: "知识库", glyph: "▢" },
      { key: "memory", label: "记忆层", glyph: "◐" },
    ],
  },
  {
    key: "system",
    label: "中枢",
    items: [
      { key: "skills", label: "技能", glyph: "✦" },
      { key: "harness", label: "运行器", glyph: "⬡" },
    ],
  },
];

export const settingsNavItem: WorkbenchNavItem = { key: "settings", label: "设置", glyph: "…" };

export const devNavItems: WorkbenchNavItem[] = [
  { key: "proposal", label: "建议方案", glyph: "≣" },
  { key: "tools", label: "工具", glyph: "⚙" },
  { key: "models", label: "模型/凭据", glyph: "◇" },
];

export const navItems: WorkbenchNavItem[] = [
  homeNavItem,
  ...primaryNavItems,
  settingsNavItem,
];

export const workspaceRailItems = [
  { key: "secretary", label: "秘书", glyph: "秘" },
  { key: "notifications", label: "通知", glyph: "知" },
  { key: "todos", label: "待办", glyph: "待" },
  { key: "ideas", label: "想法", glyph: "想" },
  { key: "running", label: "运行中", glyph: "行" },
  { key: "audit", label: "管理", glyph: "管" },
] as const;
