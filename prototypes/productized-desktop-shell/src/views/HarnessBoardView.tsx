// ⑥ I 定稿(hifi `I · 技能 / harness`：「harness 页同款，不再单画」)：与技能页/记忆中心同构的 B1 双栏 ——
// 工具条(真实总数 + 过滤 chip)+ 三元素行全量列表 + 右详情。旧的四列看板 + 底部 <details> 开发者详情退场。
//
// 全量零截断：旧版 8 处 `.slice(0, N)`(能力 8 / 资源 4 / 可运行 6 / 最近 3 / 等待 4 / 候选 3 / 项目 8 /
// 候选 24)全无展开入口 → 全删。列表全量可滚(宪法 §六 回顾面)。
//
// 词表(2026-07-14 拍板)：产品域 UI 名 = `harness`，不译；「运行器」译名已废止 → 本页新写文案一律用 harness。
// (左导航 workbenchNavigation.ts:42 的 label 仍是「运行器」= 别的范围，本包不动。)
//
// hooks 约定：本组件只经 `visibleText`(真 SSR)消费(tests/offline-permission-dialog.test.tsx:3503) → 可用 hooks。
import { useMemo, useState } from "react";
import { Badge } from "../components/Badge";
import { EmptyState, FactRow, ListRow, SegTitle } from "../components/SpecPrimitives";
import { formatDate } from "../lib/format";
import type { HarnessCandidate, HarnessResource, ProjectRecord } from "../lib/types";

type HarnessBoardViewProps = {
  projects: ProjectRecord[];
};

type ProjectHarnessResource = HarnessResource & { projectName: string; projectRoot: string };
type ProjectHarnessCandidate = HarnessCandidate & { projectName: string; projectRoot: string };

// 「资源 = 文件夹级」/「候选 = 文件级」这个区分是既有边界声明里明确要求保留的，B1 化后靠 kind + chip 保住。
type HarnessRow =
  | { kind: "resource"; key: string; resource: ProjectHarnessResource }
  | { kind: "candidate"; key: string; candidate: ProjectHarnessCandidate };

type HarnessFilter = "all" | "resource" | "candidate";

export function HarnessBoardView({ projects }: HarnessBoardViewProps) {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<HarnessFilter>("all");
  const [selectedKey, setSelectedKey] = useState<string | null>(null);

  const allRows = useMemo<HarnessRow[]>(() => {
    const resourceRows = projects.flatMap((project) =>
      project.harness_resources.map<HarnessRow>((resource) => ({
        kind: "resource",
        key: `resource:${project.project_root}:${resource.root_path}`,
        resource: { ...resource, projectName: project.name, projectRoot: project.project_root },
      })),
    );
    const candidateRows = projects.flatMap((project) =>
      project.harness_candidates.map<HarnessRow>((candidate) => ({
        kind: "candidate",
        key: `candidate:${project.project_root}:${candidate.path}`,
        candidate: { ...candidate, projectName: project.name, projectRoot: project.project_root },
      })),
    );
    return [...resourceRows, ...candidateRows];
  }, [projects]);

  const resourceCount = allRows.filter((row) => row.kind === "resource").length;
  const candidateCount = allRows.filter((row) => row.kind === "candidate").length;

  const rows = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return allRows
      .filter((row) => (filter === "all" ? true : row.kind === filter))
      .filter((row) => {
        if (!normalized) return true;
        const values =
          row.kind === "resource"
            ? [
                row.resource.display_name,
                row.resource.root_path,
                row.resource.harness_kind,
                row.resource.projectName,
                row.resource.version,
                ...row.resource.capabilities,
                ...row.resource.warnings,
              ]
            : [row.candidate.name, row.candidate.path, row.candidate.projectName, row.candidate.entry_type, row.candidate.source];
        return values.some((value) => (value ?? "").toLowerCase().includes(normalized));
      });
  }, [allRows, filter, query]);

  const selectedRow = rows.find((row) => row.key === selectedKey) ?? rows[0] ?? null;

  return (
    <section className="view-stack harness-board" aria-label="harness">
      <div className="sr-only">
        <p>harness</p>
        <h1>harness</h1>
        <p>文件夹级 harness 资源 · 文件级 harness 候选；这里不新增运行按钮，候选不代表可运行或已验证。</p>
      </div>

      <div className="memory-b1-grid">
        <section className="memory-center-panel" aria-label="harness 列表">
          <div className="memory-b1-toolbar">
            <input
              type="text"
              className="jiaoban-session-search"
              placeholder="搜 harness…"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              aria-label="搜索 harness"
            />
          </div>
          <div className="memory-b1-toolbar" role="group" aria-label="harness 过滤">
            <button className={`jiaoban-chip ${filter === "all" ? "on" : ""}`} type="button" onClick={() => setFilter("all")}>
              全部 {allRows.length}
            </button>
            <button className={`jiaoban-chip ${filter === "resource" ? "on" : ""}`} type="button" onClick={() => setFilter("resource")}>
              文件夹级 harness 资源 {resourceCount}
            </button>
            <button className={`jiaoban-chip ${filter === "candidate" ? "on" : ""}`} type="button" onClick={() => setFilter("candidate")}>
              文件级 harness 候选 {candidateCount}
            </button>
          </div>
          <div className="spec-scroll memory-b1-list" aria-label="harness 条目">
            {rows.map((row) => (
              <ListRow
                key={row.key}
                badge={rowBadge(row)}
                claim={rowClaim(row)}
                time={formatDate(row.kind === "resource" ? row.resource.updated_at_ms : row.candidate.updated_at_ms)}
                selected={selectedRow?.key === row.key}
                onSelect={() => setSelectedKey(row.key)}
              />
            ))}
            {!rows.length ? (
              <EmptyState
                what={query.trim() || filter !== "all" ? "没有匹配的 harness" : "当前工作台没有可见 harness"}
                next={query.trim() || filter !== "all" ? "换个词或切回「全部」" : "到设置的开发者区检查 harness 来源"}
              />
            ) : null}
          </div>
          <p className="muted small-note">
            共 {allRows.length} 条，全量可滚，零截断{rows.length === allRows.length ? "" : `；当前筛出 ${rows.length} 条`}。
          </p>
        </section>

        <section className="memory-center-panel memory-detail-panel" aria-label="harness 详情">
          {selectedRow ? (
            selectedRow.kind === "resource" ? (
              <HarnessResourceDetail resource={selectedRow.resource} />
            ) : (
              <HarnessCandidateDetail candidate={selectedRow.candidate} />
            )
          ) : (
            <EmptyState what="暂无可展示详情" next="先在左侧列表选一条 harness(列表为空时，到设置的开发者区检查来源)" />
          )}
        </section>
      </div>
    </section>
  );
}

function rowBadge(row: HarnessRow) {
  if (row.kind === "candidate") return <Badge tone="candidate">文件候选</Badge>;
  const ready = row.resource.entrypoints.length > 0 && !row.resource.warnings.length;
  // 「有入口」只说入口存在，不说「可运行 / 已验证」——那是索引给不出的判断。
  return ready ? <Badge tone="candidate">有入口</Badge> : <Badge tone="warning">缺配置</Badge>;
}

function rowClaim(row: HarnessRow) {
  if (row.kind === "candidate") {
    return `${row.candidate.name || pathName(row.candidate.path)} — ${row.candidate.projectName} · 文件级候选`;
  }
  const name = row.resource.display_name || pathName(row.resource.root_path);
  const capabilities = row.resource.capabilities.length ? row.resource.capabilities.join(" / ") : "未声明能力";
  return `${name} — ${capabilities}`;
}

function HarnessResourceDetail({ resource }: { resource: ProjectHarnessResource }) {
  return (
    <article>
      <SegTitle>{resource.display_name || pathName(resource.root_path)}</SegTitle>
      <div>
        <FactRow k="项目">{resource.projectName}</FactRow>
        <FactRow k="显示名">{resource.display_name || "缺失"}</FactRow>
        <FactRow k="harness 类型">{resource.harness_kind || "缺失"}</FactRow>
        <FactRow k="能力">{resource.capabilities.length ? resource.capabilities.join(", ") : "缺失"}</FactRow>
        <FactRow k="版本" bad={!resource.version}>
          {resource.version || "缺失"}
        </FactRow>
        <FactRow k="入口" bad={!resource.entrypoints.length}>
          {entrypointText(resource)}
        </FactRow>
        <FactRow k="权限级别">{resource.permission_level || "缺失"}</FactRow>
        <FactRow k="更新时间">{formatDate(resource.updated_at_ms)}</FactRow>
        <FactRow k="根路径">{resource.root_path}</FactRow>
        <FactRow k="智能体类型">{resource.agent_type || "缺失"}</FactRow>
        <FactRow k="适配器编号">{resource.adapter_id || "缺失"}</FactRow>
        <FactRow k="来源类型">{resource.source_kind || "缺失"}</FactRow>
        <FactRow k="清单路径" bad={!resource.manifest_path}>
          {resource.manifest_path || "缺失"}
        </FactRow>
        <FactRow k="说明路径" bad={!resource.readme_path}>
          {resource.readme_path || "缺失"}
        </FactRow>
        <FactRow k="警告" bad={resource.warnings.length > 0}>
          {resource.warnings.length ? resource.warnings.map(warningNameLabel).join(" / ") : "无警告"}
        </FactRow>
      </div>
      <p className="muted small-note">
        文件夹级 harness 资源。缺清单、缺说明、缺版本、缺入口等警告直接展示，不自动降噪。这里不新增运行按钮，不自动运行 harness，也不把资源显示为可用或已验证。
      </p>
    </article>
  );
}

function HarnessCandidateDetail({ candidate }: { candidate: ProjectHarnessCandidate }) {
  return (
    <article>
      <SegTitle>{candidate.name || pathName(candidate.path)}</SegTitle>
      <div>
        <FactRow k="项目">{candidate.projectName}</FactRow>
        <FactRow k="路径">{candidate.path}</FactRow>
        <FactRow k="入口类型">{candidate.entry_type || "缺失"}</FactRow>
        <FactRow k="来源">{candidate.source || "缺失"}</FactRow>
        <FactRow k="更新时间">{formatDate(candidate.updated_at_ms)}</FactRow>
        <FactRow k="警告" bad={candidate.warnings.length > 0}>
          {candidate.warnings.length ? candidate.warnings.map(warningNameLabel).join(" / ") : "无警告"}
        </FactRow>
      </div>
      <p className="muted small-note">
        文件级 harness 候选：需要补充为 harness 资源后才能进入可运行范围。这里不新增运行按钮，候选不代表可运行或已验证。
      </p>
    </article>
  );
}

function warningNameLabel(warning: string) {
  if (warning === "missing_manifest") return "缺清单";
  if (warning === "missing_readme") return "缺说明";
  if (warning === "missing_version") return "缺版本";
  if (warning === "missing_entrypoints") return "缺入口";
  if (warning === "weak_harness_signal") return "弱 harness 信号";
  return warning;
}

function entrypointText(resource: ProjectHarnessResource) {
  return resource.entrypoints.length
    ? resource.entrypoints.map((entrypoint) => `${entrypoint.entry_type || "入口"}:${entrypoint.name || pathName(entrypoint.path)}`).join(", ")
    : "缺失";
}

function pathName(path: string) {
  return path.split("/").filter(Boolean).at(-1) || path || "未知";
}
