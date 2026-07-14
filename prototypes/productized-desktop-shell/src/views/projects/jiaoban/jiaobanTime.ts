// 交办面共用时间工具(阶段3拆巨石·自 ProjectJiaobanPanel.tsx 原样迁出,零逻辑改动)。

const DAY_MS = 24 * 60 * 60 * 1000;

// 方案生成时间 → 「今天/几天前」。用日历日判「不是今天」（避免刚过午夜的边界误判）。
export function proposalAgeDays(createdAtMs: number): number {
  const created = new Date(createdAtMs);
  const now = new Date();
  const createdDay = new Date(created.getFullYear(), created.getMonth(), created.getDate()).getTime();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  return Math.max(0, Math.round((today - createdDay) / DAY_MS));
}

export function formatProposalTime(createdAtMs: number): string {
  const d = new Date(createdAtMs);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
