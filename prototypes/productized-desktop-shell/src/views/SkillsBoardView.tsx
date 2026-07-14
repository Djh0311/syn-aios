// ⑥ I 定稿(hifi `I · 技能 / harness(回顾面 B1 同构·全量不截断)`)：与记忆中心完全同构 ——
// 工具条(真实总数 + 过滤 chip)+ 三元素行全量列表 + 右详情。样板 = MemoryCenterView 的 B1 双栏。
//
// 治体检 P0：旧四列看板把 90 条技能 `.slice(0, 6)` 砍到 6 条**且无展开入口**——84 条永久不可达。
// 本次删掉全部硬截断，列表全量可滚(宪法 §六 回顾面：找一条已知存在的记录不许超 3 步/10 秒)。
//
// hooks 约定：本组件只经 `visibleText`(真 SSR)消费(见 tests/offline-permission-dialog.test.tsx:3498)，
// 不被 renderComposite 裸调 → 可以用 hooks。(对比 ProjectOverview 必须零 hooks。)
import { useMemo, useState } from "react";
import { Badge } from "../components/Badge";
import { EmptyState, FactRow, ListRow, SegTitle } from "../components/SpecPrimitives";
import type { PluginRecord, ProjectRecord, SkillRecord } from "../lib/types";

type SkillsBoardViewProps = {
  skills: SkillRecord[];
  plugins: PluginRecord[];
  projects: ProjectRecord[];
};

type SkillFilter = "all" | "plugin";

export function SkillsBoardView({ skills, plugins }: SkillsBoardViewProps) {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<SkillFilter>("all");
  const [selectedSkillId, setSelectedSkillId] = useState<string | null>(null);

  const pluginSkillCount = useMemo(() => skills.filter((skill) => skill.source_type === "plugin").length, [skills]);

  const rows = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return skills
      .filter((skill) => (filter === "plugin" ? skill.source_type === "plugin" : true))
      .filter((skill) => {
        if (!normalized) return true;
        return [skill.title, skill.description, skill.path, skill.plugin_name, sourceLabel(skill.source_type)]
          .some((value) => (value ?? "").toLowerCase().includes(normalized));
      });
  }, [skills, filter, query]);

  const selectedSkill = rows.find((skill) => skill.skill_id === selectedSkillId) ?? rows[0] ?? null;

  return (
    <section className="view-stack skills-board" aria-label="技能">
      <div className="sr-only">
        <p>技能</p>
        <h1>技能</h1>
        <p>查看可见技能的来源和适用场景；这里不加载、不编辑、不自动推荐。</p>
      </div>

      <div className="memory-b1-grid">
        <section className="memory-center-panel" aria-label="技能列表">
          <div className="memory-b1-toolbar">
            <input
              type="text"
              className="jiaoban-session-search"
              placeholder="搜技能…"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              aria-label="搜索技能"
            />
          </div>
          <div className="memory-b1-toolbar" role="group" aria-label="技能过滤">
            {/* 计数取实数(skills.length / 插件来源技能数)，不写死。 */}
            <button className={`jiaoban-chip ${filter === "all" ? "on" : ""}`} type="button" onClick={() => setFilter("all")}>
              全部 {skills.length}
            </button>
            <button className={`jiaoban-chip ${filter === "plugin" ? "on" : ""}`} type="button" onClick={() => setFilter("plugin")}>
              插件 {pluginSkillCount}
            </button>
          </div>
          <div className="spec-scroll memory-b1-list" aria-label="技能条目">
            {rows.map((skill) => (
              <ListRow
                key={skill.skill_id}
                badge={<Badge tone={skill.source_type === "plugin" ? "candidate" : "neutral"}>{sourceLabel(skill.source_type)}</Badge>}
                claim={skill.description ? `${skill.title} — ${skill.description}` : skill.title}
                time={skill.plugin_name ?? undefined}
                selected={selectedSkill?.skill_id === skill.skill_id}
                onSelect={() => setSelectedSkillId(skill.skill_id)}
              />
            ))}
            {!rows.length ? (
              <EmptyState
                what={query.trim() || filter === "plugin" ? "没有匹配的技能" : "当前工作台还没有可见技能"}
                next={query.trim() || filter === "plugin" ? "换个词或切回「全部」" : "到设置的开发者区查看技能来源和接入边界"}
              />
            ) : null}
          </div>
          {/* 全量零截断(治体检 P0)：这一行是实数对账，不是「还有更多」的省略号。 */}
          <p className="muted small-note">
            共 {skills.length} 条技能，全量可滚，零截断{rows.length === skills.length ? "" : `；当前筛出 ${rows.length} 条`}。
          </p>
        </section>

        <section className="memory-center-panel memory-detail-panel" aria-label="技能详情">
          {selectedSkill ? <SkillDetail skill={selectedSkill} plugins={plugins} /> : (
            <EmptyState what="暂无可展示详情" next="先在左侧列表选一条技能(列表为空时，到设置的开发者区查看技能来源)" />
          )}
        </section>
      </div>
    </section>
  );
}

function SkillDetail({ skill, plugins }: { skill: SkillRecord; plugins: PluginRecord[] }) {
  const plugin = skill.plugin_name ? plugins.find((item) => item.plugin_name === skill.plugin_name) ?? null : null;
  return (
    <article>
      <SegTitle>{skill.title}</SegTitle>
      <div>
        {/* 来源 / 适用 = 索引真有的字段(path / plugin_name / description)。 */}
        <FactRow k="来源">{skill.plugin_name ? `${pathTail(skill.path)} · ${skill.plugin_name}` : pathTail(skill.path)}</FactRow>
        <FactRow k="适用">{skill.description || "索引没有能力描述"}</FactRow>
        {/* 定稿这里有一行「状态：候选(未登记)——登记前不注入任务包」+ 动作行[登记为正式技能][查看 SKILL.md]。
            SkillRecord(workbenchCoreTypes.ts:174-183)**没有 registration/status 字段**，全仓也没有
            register_skill / skill_registration 命令；`src/lib/tauri.ts` 只有 reveal_indexed_rollout 和
            open_indexed_project(后者按索引项目根白名单校验，SKILL.md 路径会被拒)。
            → 状态行留位「接线中」，两个按钮**不做**(宪法 §四.3：没有后端命令的按钮就别做)。 */}
        <FactRow k="状态">接线中——索引还没有技能登记状态</FactRow>
        <FactRow k="类型">{sourceLabel(skill.source_type)}</FactRow>
        {plugin ? <FactRow k="插件版本">{plugin.plugin_version}</FactRow> : null}
        <FactRow k="路径">{skill.path}</FactRow>
        {skill.warnings.length ? (
          <FactRow k="警告" bad>
            {skill.warnings.join(" / ")}
          </FactRow>
        ) : null}
      </div>
      <p className="muted small-note">
        这里只显示索引看到的技能；可见不等于已加载、已推荐或已绑定项目。登记和打开 SKILL.md 都还没有后端命令，所以先不放按钮。
      </p>
    </article>
  );
}

function pathTail(path: string) {
  return path.split("/").filter(Boolean).at(-1) || path || "未知";
}

function sourceLabel(sourceType: string) {
  if (sourceType === "plugin") return "插件技能";
  if (sourceType === "system") return "系统技能";
  if (sourceType === "user") return "本地技能";
  return "来源未知";
}
