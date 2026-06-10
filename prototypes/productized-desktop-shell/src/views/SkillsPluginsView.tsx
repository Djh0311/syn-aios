import { Badge } from "../components/Badge";
import type { PluginRecord, SkillRecord } from "../lib/types";

type SkillsPluginsViewProps = {
  skills: SkillRecord[];
  plugins: PluginRecord[];
};

export function SkillsPluginsView({ skills, plugins }: SkillsPluginsViewProps) {
  return (
    <section className="view-stack">
      <div className="section-heading">
        <div>
          <p className="eyebrow">能力来源</p>
          <h2>技能 / 插件页</h2>
        </div>
        <p className="muted">只显示路径和清单元数据，不读取技能正文。</p>
      </div>

      <div className="content-grid two">
        <article className="panel">
          <div className="panel-heading">
            <h3>技能</h3>
            <Badge tone="candidate">{skills.length} 个</Badge>
          </div>
          <div className="list-stack">
            {skills.map((skill) => (
              <div className="mini-row" key={skill.skill_id}>
                <strong>{skill.title}</strong>
                <span>{skill.path}</span>
                <div className="badge-row">
                  <Badge tone={skill.source_type === "plugin" ? "candidate" : "unknown"}>{sourceLabel(skill.source_type)}</Badge>
                  <Badge>{skill.plugin_name || "无插件名"}</Badge>
                </div>
              </div>
            ))}
          </div>
        </article>

        <article className="panel">
          <div className="panel-heading">
            <h3>插件</h3>
            <Badge tone="candidate">{plugins.length} 个</Badge>
          </div>
          <div className="list-stack">
            {plugins.map((plugin) => (
              <div className="mini-row" key={`${plugin.plugin_name}-${plugin.plugin_version}`}>
                <strong>{plugin.plugin_name}</strong>
                <span>{plugin.homepage || "无主页元数据"}</span>
                <div className="badge-row">
                  <Badge>版本 {plugin.plugin_version}</Badge>
                  <Badge>技能 {plugin.skill_count}</Badge>
                  {plugin.has_mcp_servers ? <Badge tone="candidate">MCP</Badge> : null}
                </div>
              </div>
            ))}
          </div>
        </article>
      </div>
    </section>
  );
}

function sourceLabel(sourceType: string) {
  if (sourceType === "plugin") return "插件技能";
  if (sourceType === "system") return "系统技能";
  if (sourceType === "user") return "本地技能";
  return "来源未知";
}
