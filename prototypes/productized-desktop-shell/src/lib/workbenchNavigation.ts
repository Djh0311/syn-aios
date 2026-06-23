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
  | "settings";

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
  { key: "skills", label: "Skill", glyph: "✦" },
  { key: "harness", label: "Harness", glyph: "⬡" },
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
      { key: "skills", label: "Skill", glyph: "✦" },
      { key: "harness", label: "Harness", glyph: "⬡" },
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
