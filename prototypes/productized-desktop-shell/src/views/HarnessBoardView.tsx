import { Badge } from "../components/Badge";
import { SummaryTile } from "../components/WorkbenchPrimitives";
import { formatDate } from "../lib/format";
import type { HarnessCandidate, HarnessResource, ProjectRecord } from "../lib/types";

type HarnessBoardViewProps = {
  projects: ProjectRecord[];
};

export function HarnessBoardView({ projects }: HarnessBoardViewProps) {
  const resources = projects.flatMap((project) =>
    project.harness_resources.map((resource) => ({
      ...resource,
      projectName: project.name,
      projectRoot: project.project_root,
    })),
  );
  const candidates = projects.flatMap((project) =>
    project.harness_candidates.map((candidate) => ({
      ...candidate,
      projectName: project.name,
      projectRoot: project.project_root,
    })),
  );
  const configuredResources = resources.filter((resource) => resource.entrypoints.length && !resource.warnings.length);
  const waitingResources = resources.filter((resource) => resource.warnings.length || !resource.entrypoints.length);
  const capabilityNames = Array.from(new Set(resources.flatMap((resource) => resource.capabilities))).filter(Boolean);

  return (
    <section className="view-stack">
      <p className="sr-only">文件夹级运行器资源 · 文件级运行器候选</p>
      <div className="section-heading">
        <div>
          <p className="eyebrow">运行器</p>
          <h2>运行器能力库</h2>
        </div>
        <p className="muted">查看运行器能力、可运行范围和等待配置原因；这里不新增运行按钮，候选不代表可运行或已验证。</p>
      </div>

      <div className="object-summary-grid harness-summary">
        <SummaryTile label="运行器能力" value={`${capabilityNames.length || resources.length} 项`} hint="按可见能力和资源粗略归类" />
        <SummaryTile label="可运行范围" value={`${configuredResources.length} 个`} hint="有入口且暂无警告的资源" />
        <SummaryTile label="最近运行" value="未接入" hint="暂无运行事件，不伪造结果" />
        <SummaryTile label="等待配置" value={`${waitingResources.length + candidates.length} 个`} hint="缺入口、说明、版本或仍是文件候选" />
      </div>

      <div className="board-grid harness-object-grid">
        <ObjectColumn title="运行器能力" tone="candidate">
          {capabilityNames.length ? (
            capabilityNames.slice(0, 8).map((capability) => (
              <div className="board-card" key={capability}>
                <strong>{capability}</strong>
                <span>由运行器资源声明的能力；是否执行仍取决于项目任务和权限。</span>
              </div>
            ))
          ) : resources.length ? (
            resources.slice(0, 4).map((resource) => (
              <div className="board-card" key={`${resource.projectRoot}-${resource.root_path}`}>
                <strong>{resource.display_name || pathName(resource.root_path)}</strong>
                <span>暂未声明具体能力；可在开发者详情查看资源字段。</span>
              </div>
            ))
          ) : (
            <div className="board-card muted-card">
              <strong>暂无运行器资源</strong>
              <span>当前工作台没有可见运行器；可在设置的开发者区检查来源。</span>
            </div>
          )}
        </ObjectColumn>

        <ObjectColumn title="可运行范围" tone={configuredResources.length ? "candidate" : "unknown"}>
          {configuredResources.length ? (
            configuredResources.slice(0, 6).map((resource) => (
              <div className="board-card" key={`${resource.projectRoot}-${resource.root_path}`}>
                <strong>{resource.display_name || pathName(resource.root_path)}</strong>
                <span>{resource.projectName}</span>
                <div className="badge-row">
                  <Badge tone="candidate">{resource.permission_level || "权限待确认"}</Badge>
                  <Badge tone="neutral">{entrypointText(resource)}</Badge>
                </div>
              </div>
            ))
          ) : (
            <div className="board-card muted-card">
              <strong>暂无可运行范围</strong>
              <span>没有资源同时满足入口存在且无警告；不显示运行按钮。</span>
            </div>
          )}
        </ObjectColumn>

        <ObjectColumn title="最近运行" tone="unknown">
          <div className="board-card muted-card">
            <strong>暂无运行记录</strong>
            <span>当前只展示运行器对象；没有运行日志证据时不展示最近运行结果。</span>
          </div>
          {resources.slice(0, 3).map((resource) => (
            <div className="board-card" key={`recent-placeholder-${resource.projectRoot}-${resource.root_path}`}>
              <strong>{resource.display_name || pathName(resource.root_path)}</strong>
              <span>未见最近运行事件。</span>
            </div>
          ))}
        </ObjectColumn>

        <ObjectColumn title="等待配置 / 不可用原因" tone={waitingResources.length || candidates.length ? "warning" : "candidate"}>
          {waitingResources.length || candidates.length ? (
            <>
              {waitingResources.slice(0, 4).map((resource) => (
                <div className="board-card" key={`waiting-${resource.projectRoot}-${resource.root_path}`}>
                  <strong>{resource.display_name || pathName(resource.root_path)}</strong>
                  <span>{resource.warnings.length ? resource.warnings.map(warningNameLabel).join(" / ") : "缺入口或配置未完整"}</span>
                </div>
              ))}
              {candidates.slice(0, 3).map((candidate) => (
                <div className="board-card" key={`candidate-${candidate.projectRoot}-${candidate.path}`}>
                  <strong>{candidate.name || pathName(candidate.path)}</strong>
                  <span>文件候选；需要补充为运行器资源后才能进入可运行范围。</span>
                </div>
              ))}
            </>
          ) : (
            <div className="board-card">
              <strong>暂无阻断配置</strong>
              <span>当前未见等待配置项；仍需通过任务权限后才能运行。</span>
            </div>
          )}
        </ObjectColumn>
      </div>

      <details className="object-detail-panel">
        <summary>开发者详情：资源字段和候选入口</summary>
        <div className="content-grid two">
          <article className="panel">
            <div className="panel-heading">
              <h3>文件夹级运行器资源</h3>
              <Badge tone="warning">候选资源，未验证</Badge>
            </div>
            <div className="resource-list">
              {resources.length ? (
                resources.map((resource) => <ResourceCard resource={resource} key={`${resource.projectRoot}-${resource.root_path}`} />)
              ) : (
                <p className="empty-line">当前没有运行器资源。</p>
              )}
            </div>
          </article>

          <article className="panel">
            <div className="panel-heading">
              <h3>文件级运行器候选</h3>
              <Badge tone="candidate">兼容保留</Badge>
            </div>
            <div className="resource-list compact-resource-list">
              {candidates.length ? (
                candidates.slice(0, 24).map((candidate) => (
                  <CandidateCard candidate={candidate} key={`${candidate.projectRoot}-${candidate.path}`} />
                ))
              ) : (
                <p className="empty-line">当前没有运行器候选。</p>
              )}
            </div>
          </article>
        </div>

        <div className="board-grid harness-board">
          <BoardColumn title="框架 / 类型" tone="candidate">
            {Object.entries(groupResourcesByKind(resources)).map(([kind, items]) => (
              <div className="board-card" key={kind}>
                <strong>{kind || "类型未知"}</strong>
                <span>来自运行器资源类型。</span>
                <div className="badge-row">
                  <Badge tone="neutral">{items.length} 个资源</Badge>
                </div>
              </div>
            ))}
          </BoardColumn>

          <BoardColumn title="智能体 / 适配器" tone="candidate">
            {Object.entries(groupResourcesByAdapter(resources)).map(([adapter, items]) => (
              <div className="board-card" key={adapter}>
                <strong>{adapter}</strong>
                <span>智能体类型 / 适配器编号只作为字段展示，不代表已接入新智能体。</span>
                <div className="badge-row">
                  <Badge tone="neutral">{items.length} 个资源</Badge>
                </div>
              </div>
            ))}
          </BoardColumn>

          <BoardColumn title="缺失警告" tone="warning">
            {warningNames(resources).map((warning) => (
              <div className="board-card" key={warning}>
                <strong>{warning}</strong>
                <span>{warningLabel(warning)}</span>
              </div>
            ))}
          </BoardColumn>

          <BoardColumn title="项目适配" tone="candidate">
            {projects
              .filter((project) => project.harness_resources.length || project.harness_candidates.length)
              .slice(0, 8)
              .map((project) => (
                <div className="board-card" key={project.project_root}>
                  <strong>{project.name}</strong>
                  <span>{project.project_root}</span>
                  <div className="badge-row">
                    <Badge tone="warning">资源 {project.harness_resources.length}</Badge>
                    <Badge tone="candidate">候选 {project.harness_candidates.length}</Badge>
                  </div>
                </div>
              ))}
          </BoardColumn>
        </div>

        <article className="panel">
          <div className="panel-heading">
            <h3>边界</h3>
            <Badge tone="unknown">没有运行能力</Badge>
          </div>
          <div className="gap-grid">
            <GapLine label="已读取字段" value="显示名、根路径、运行器类型、智能体类型、适配器编号、来源类型、能力、清单路径、说明路径、版本、入口、权限级别、警告。" />
            <GapLine label="必须保留的区分" value="运行器资源是文件夹级候选资源；运行器候选是文件级候选入口。" />
            <GapLine label="警告展示" value="缺清单、缺说明、缺版本、缺入口等警告直接展示，不自动降噪。" />
            <GapLine label="当前边界" value="不新增运行按钮，不自动运行运行器，不把资源显示为可用或已验证。" />
          </div>
        </article>
      </details>
    </section>
  );
}

type ProjectHarnessResource = HarnessResource & {
  projectName: string;
  projectRoot: string;
};

type ProjectHarnessCandidate = HarnessCandidate & {
  projectName: string;
  projectRoot: string;
};

function ResourceCard({ resource }: { resource: ProjectHarnessResource }) {
  return (
    <article className="resource-card warning-card">
      <div className="resource-card-head">
        <div>
          <strong>{resource.display_name || pathName(resource.root_path)}</strong>
          <span>{resource.projectName}</span>
        </div>
        <Badge tone="warning">候选资源</Badge>
      </div>
      <div className="resource-fields">
        <Field label="运行器类型" value={resource.harness_kind || "缺失"} />
        <Field label="能力" value={resource.capabilities.length ? resource.capabilities.join(", ") : "缺失"} />
        <Field label="版本" value={resource.version || "缺失"} warning={!resource.version} />
        <Field label="入口" value={entrypointText(resource)} warning={!resource.entrypoints.length} />
        <Field label="权限级别" value={resource.permission_level || "缺失"} />
        <Field label="更新时间" value={formatDate(resource.updated_at_ms)} />
      </div>
      <WarningRow warnings={resource.warnings} />
      <details className="agent-boundary-details">
        <summary className="agent-boundary-summary">开发者详情</summary>
        <div className="resource-fields">
          <Field label="根路径" value={resource.root_path} />
          <Field label="智能体类型" value={resource.agent_type || "缺失"} />
          <Field label="适配器编号" value={resource.adapter_id || "缺失"} />
          <Field label="来源类型" value={resource.source_kind || "缺失"} />
          <Field label="清单路径" value={resource.manifest_path || "缺失"} warning={!resource.manifest_path} />
          <Field label="说明路径" value={resource.readme_path || "缺失"} warning={!resource.readme_path} />
        </div>
      </details>
    </article>
  );
}

function CandidateCard({ candidate }: { candidate: ProjectHarnessCandidate }) {
  return (
    <article className="resource-card candidate-card">
      <div className="resource-card-head">
        <div>
          <strong>{candidate.name || pathName(candidate.path)}</strong>
          <span>{candidate.projectName}</span>
        </div>
        <Badge tone="candidate">文件候选</Badge>
      </div>
      <div className="resource-fields">
        <Field label="更新时间" value={formatDate(candidate.updated_at_ms)} />
      </div>
      <WarningRow warnings={candidate.warnings} />
      <details className="agent-boundary-details">
        <summary className="agent-boundary-summary">开发者详情</summary>
        <div className="resource-fields">
          <Field label="路径" value={candidate.path} />
          <Field label="入口类型" value={candidate.entry_type || "缺失"} />
          <Field label="来源" value={candidate.source || "缺失"} />
        </div>
      </details>
    </article>
  );
}

function Field({ label, value, warning = false }: { label: string; value: string; warning?: boolean }) {
  return (
    <div className={`resource-field ${warning ? "warning" : ""}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function WarningRow({ warnings }: { warnings: string[] }) {
  if (!warnings.length) {
    return (
      <div className="warning-row neutral-warning">
        <Badge tone="neutral">无警告</Badge>
        <span>这仍只是候选，不代表已验证。</span>
      </div>
    );
  }
  return (
    <div className="warning-row">
      {warnings.map((warning) => (
        <Badge tone="warning" key={warning}>
          {warningNameLabel(warning)}
        </Badge>
      ))}
    </div>
  );
}

function ObjectColumn({ title, tone, children }: { title: string; tone: "candidate" | "unknown" | "warning"; children: React.ReactNode }) {
  return (
    <article className="board-column">
      <div className="panel-heading">
        <h3>{title}</h3>
        <Badge tone={tone}>{tone === "candidate" ? "可查看" : tone === "warning" ? "需配置" : "待补充"}</Badge>
      </div>
      <div className="list-stack">{children}</div>
    </article>
  );
}

function BoardColumn({ title, tone, children }: { title: string; tone: "candidate" | "unknown" | "warning"; children: React.ReactNode }) {
  return (
    <article className="board-column">
      <div className="panel-heading">
        <h3>{title}</h3>
        <Badge tone={tone}>{tone === "candidate" ? "索引支持" : "需注意"}</Badge>
      </div>
      <div className="list-stack">{children}</div>
    </article>
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

function groupResourcesByKind(resources: ProjectHarnessResource[]) {
  return resources.reduce<Record<string, ProjectHarnessResource[]>>((groups, resource) => {
    const key = resource.harness_kind || "未知";
    groups[key] = [...(groups[key] ?? []), resource];
    return groups;
  }, {});
}

function groupResourcesByAdapter(resources: ProjectHarnessResource[]) {
  return resources.reduce<Record<string, ProjectHarnessResource[]>>((groups, resource) => {
    const key = `${resource.agent_type || "未知"} / ${resource.adapter_id || "未知"}`;
    groups[key] = [...(groups[key] ?? []), resource];
    return groups;
  }, {});
}

function warningNames(resources: ProjectHarnessResource[]) {
  const names = Array.from(new Set(resources.flatMap((resource) => resource.warnings)));
  return names.length ? names : ["无资源警告"];
}

function warningNameLabel(warning: string) {
  if (warning === "missing_manifest") return "缺清单";
  if (warning === "missing_readme") return "缺说明";
  if (warning === "missing_version") return "缺版本";
  if (warning === "missing_entrypoints") return "缺入口";
  if (warning === "weak_harness_signal") return "弱运行器信号";
  if (warning === "无资源警告") return warning;
  return warning;
}

function warningLabel(warning: string) {
  if (warning === "missing_manifest") return "缺清单，不能当成规范化运行器。";
  if (warning === "missing_readme") return "缺说明，不能展示说明正文。";
  if (warning === "missing_version") return "缺版本，不能判断版本。";
  if (warning === "missing_entrypoints") return "缺入口，不能提供运行入口。";
  if (warning === "weak_harness_signal") return "弱信号候选，需要后续人工或索引规则确认。";
  return "索引没有更多解释。";
}

function entrypointText(resource: ProjectHarnessResource) {
  return resource.entrypoints.length
    ? resource.entrypoints.map((entrypoint) => `${entrypoint.entry_type || "入口"}:${entrypoint.name || pathName(entrypoint.path)}`).join(", ")
    : "缺失";
}

function pathName(path: string) {
  return path.split("/").filter(Boolean).at(-1) || path || "未知";
}
