import { Badge } from "../components/Badge";
import { SummaryTile } from "../components/WorkbenchPrimitives";
import type { PluginRecord, ProjectRecord, SkillRecord } from "../lib/types";

type SkillsBoardViewProps = {
  skills: SkillRecord[];
  plugins: PluginRecord[];
  projects: ProjectRecord[];
};

export function SkillsBoardView({ skills, plugins, projects }: SkillsBoardViewProps) {
  const bySource = groupSkillsBySource(skills);
  const pluginSkills = skills.filter((skill) => skill.source_type === "plugin").slice(0, 8);
  const reusableSkills = skills.slice(0, 6);
  const projectCount = projects.length;

  return (
    <section className="view-stack">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Skill</p>
          <h2>Skill 能力库</h2>
        </div>
        <p className="muted">查看可复用能力、适用场景和当前可用性；这里不加载、不编辑、不自动推荐。</p>
      </div>

      <div className="object-summary-grid">
        <SummaryTile label="可复用能力" value={`${skills.length} 个`} hint="来自当前工作台可见 Skill" />
        <SummaryTile label="适用场景" value={`${Object.keys(bySource).length} 类`} hint="按系统、本地和插件来源粗分" />
        <SummaryTile label="最近使用" value="未接入" hint="暂无使用事件，不伪造热度" />
        <SummaryTile label="当前可用性" value={skills.length ? "可查看" : "待配置"} hint="仅代表可见，不代表已加载到智能体" />
      </div>

      <div className="board-grid skill-object-grid">
        <ObjectColumn title="可复用能力" tone="candidate">
          {reusableSkills.length ? (
            reusableSkills.map((skill) => (
              <SkillObjectCard skill={skill} key={skill.skill_id} />
            ))
          ) : (
            <div className="board-card muted-card">
              <strong>暂无 Skill</strong>
              <span>当前工作台还没有可见 Skill；可到设置的开发者区查看来源和接入边界。</span>
            </div>
          )}
        </ObjectColumn>

        <ObjectColumn title="适用场景" tone="candidate">
          {Object.entries(bySource).length ? (
            Object.entries(bySource).map(([source, items]) => (
              <div className="board-card" key={source}>
                <strong>{sourceLabel(source)}</strong>
                <span>{scenarioLabel(source)}</span>
                <div className="badge-row">
                  <Badge tone="neutral">{items.length} 个 Skill</Badge>
                </div>
              </div>
            ))
          ) : (
            <div className="board-card muted-card">
              <strong>场景待补充</strong>
              <span>没有可见 Skill 时不推断适用场景。</span>
            </div>
          )}
        </ObjectColumn>

        <ObjectColumn title="最近使用" tone="unknown">
          <div className="board-card muted-card">
            <strong>暂无使用事件</strong>
            <span>当前只知道哪些 Skill 可见；不知道最近由谁、在哪个项目里使用过。</span>
          </div>
          {pluginSkills.slice(0, 3).map((skill) => (
            <div className="board-card" key={skill.skill_id}>
              <strong>{skill.title}</strong>
              <span>插件 Skill，可作为后续使用记录的候选对象。</span>
              <div className="badge-row">
                <Badge tone="unknown">未见最近使用</Badge>
              </div>
            </div>
          ))}
        </ObjectColumn>

        <ObjectColumn title="当前可用性" tone="unknown">
          <div className="board-card">
            <strong>{skills.length ? "可查看，未声明已加载" : "待配置"}</strong>
            <span>这里不把可见 Skill 等同于已加载、已推荐或已绑定项目。</span>
            <div className="badge-row">
              <Badge tone="candidate">项目 {projectCount}</Badge>
              <Badge tone="unknown">使用关系待补</Badge>
            </div>
          </div>
          <div className="board-card muted-card">
            <strong>设置 &gt; 开发者</strong>
            <span>来源路径、插件清单和内部字段缺口收纳到详情或开发者区。</span>
          </div>
        </ObjectColumn>
      </div>

      <details className="object-detail-panel">
        <summary>开发者详情：来源和字段缺口</summary>
        <article className="panel">
          <div className="panel-heading">
            <h3>来源和缺字段</h3>
            <Badge tone="unknown">设置 &gt; 开发者</Badge>
          </div>
          <div className="gap-grid">
            <GapLine label="已用字段" value="技能编号、标题、描述、路径、来源类型、插件名、插件版本、插件清单元数据。" />
            <GapLine label="缺少字段" value="被哪个智能体使用、能在哪个智能体使用、被哪些项目使用、推荐关系、加载状态。" />
            <GapLine label="插件信息" value={`当前只读插件元数据；索引内插件候选 ${plugins.length} 条。`} />
          </div>
        </article>
      </details>
    </section>
  );
}

function ObjectColumn({ title, tone, children }: { title: string; tone: "candidate" | "unknown"; children: React.ReactNode }) {
  return (
    <article className="board-column">
      <div className="panel-heading">
        <h3>{title}</h3>
        <Badge tone={tone}>{tone === "candidate" ? "可查看" : "待补充"}</Badge>
      </div>
      <div className="list-stack">{children}</div>
    </article>
  );
}

function SkillObjectCard({ skill }: { skill: SkillRecord }) {
  return (
    <div className="board-card">
      <strong>{skill.title}</strong>
      <span>{skill.description || "暂无能力描述"}</span>
      <div className="badge-row">
        <Badge tone="neutral">{sourceLabel(skill.source_type)}</Badge>
        {skill.plugin_name ? <Badge tone="candidate">{skill.plugin_name}</Badge> : null}
      </div>
    </div>
  );
}

function GapLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="gap-line">
      <strong>{label}</strong>
      <span>{value}</span>
    </div>
  );
}

function groupSkillsBySource(skills: SkillRecord[]) {
  return skills.reduce<Record<string, SkillRecord[]>>((groups, skill) => {
    const key = skill.source_type || "unknown";
    groups[key] = [...(groups[key] ?? []), skill];
    return groups;
  }, {});
}

function sourceLabel(sourceType: string) {
  if (sourceType === "plugin") return "插件 Skill";
  if (sourceType === "system") return "系统 Skill";
  if (sourceType === "user") return "本地 Skill";
  return "来源未知";
}

function scenarioLabel(sourceType: string) {
  if (sourceType === "plugin") return "适合插件提供的专项能力，使用前仍需确认项目语境。";
  if (sourceType === "system") return "适合基础工作台能力；不代表自动注入当前任务。";
  if (sourceType === "user") return "适合本地自定义工作流；是否使用由项目任务决定。";
  return "当前没有足够信息判断适用场景。";
}
