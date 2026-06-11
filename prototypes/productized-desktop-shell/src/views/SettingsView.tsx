import { Badge } from "../components/Badge";
import type { WorkbenchSnapshot, WorkflowStateSnapshot } from "../lib/types";
import type { ViewKey, WorkbenchNavItem } from "../lib/workbenchNavigation";

type SettingsViewProps = {
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateError: string | null;
  hasRealSnapshot: boolean;
  developerItems: WorkbenchNavItem[];
  onNavigate: (view: ViewKey) => void;
};

const developerDescriptions: Partial<Record<ViewKey, string>> = {
  proposal: "收纳建议方案入口；用于查看方案草案和确认边界。",
  workflow: "实验、模板、草图和后置画布资料；不是项目工作流事实源。",
  tools: "工具和 Harness 资源的只读索引说明；不提供直接运行按钮。",
  models: "模型、凭据、供应方和适配器边界说明；不读取密钥、令牌或认证授权材料。",
};

export function SettingsView({
  snapshot,
  workflowState,
  workflowStateError,
  hasRealSnapshot,
  developerItems,
  onNavigate,
}: SettingsViewProps) {
  const adapterCount = snapshot.agent_adapters.length;
  const providerCount = snapshot.provider_availability.length;
  const diagnosticCount = snapshot.diagnostic_summary.degraded_states.length;
  const runtimeLogCount = snapshot.runtime_log_store.entries.length;
  const pageReadModelInventory = snapshot.page_read_model_inventory;
  const pageContractCount = pageReadModelInventory.contracts.length;

  return (
    <section className="stage-pad settings-view">
      <div className="pg-head">
        <div>
          <p className="pg-sub">设置</p>
          <h1 className="pg-title">工作台设置</h1>
        </div>
        <div className="pg-meta">
          <div className="big">{hasRealSnapshot ? "索引已读取" : "未接真实数据"}</div>
          <div>设置页只整理入口和边界，不读取凭据、不触发执行。</div>
        </div>
      </div>

      <div className="settings-grid">
        <section className="panel settings-section">
          <div className="panel-h">
            常规
            <Badge tone={workflowStateError ? "warning" : "neutral"}>{workflowStateError ? "事实层异常" : "只读"}</Badge>
          </div>
          <div className="settings-fact-grid">
            <SettingFact label="项目" value={`${snapshot.summary.project_count}`} />
            <SettingFact label="智能体会话" value={`${snapshot.summary.session_count}`} />
            <SettingFact label="Skill" value={`${snapshot.summary.skill_count}`} />
            <SettingFact label="工作流" value={`${workflowState?.counts.workflows ?? 0}`} />
          </div>
          <p className="muted small-note">
            普通主导航展示项目、智能体、想法箱、知识库、记忆层、Skill、Harness、运行中工作流；开发和内部边界统一从本页进入。
          </p>
          {workflowStateError ? <p className="rail-error">事实层读取失败：{workflowStateError}</p> : null}
        </section>

        <section className="panel settings-section developer-settings-section">
          <div className="panel-h">
            开发者
            <Badge tone="unknown">主动进入</Badge>
          </div>
          <p className="muted small-note">
            这里收纳开发、内部边界和诊断入口：建议方案、实验画布、工具、模型/凭据、适配器、供应方、边车文件、原始状态、诊断等只读材料。
          </p>
          <div className="settings-link-grid">
            {developerItems.map((item) => (
              <button
                className="settings-link-card"
                key={item.key}
                type="button"
                onClick={() => onNavigate(item.key)}
              >
                <span className="settings-link-glyph" aria-hidden="true">{item.glyph}</span>
                <strong>{item.label}</strong>
                <em>{developerDescriptions[item.key] ?? "开发者只读入口；不进入普通主导航。"}</em>
              </button>
            ))}
          </div>
        </section>
      </div>

      <section className="panel settings-section developer-boundary-section">
        <div className="panel-h">
          内部边界摘要
          <Badge tone="unknown">不展开原始材料</Badge>
        </div>
        <div className="developer-boundary-grid">
          <BoundaryItem
            title="适配器 / 供应方"
            value={`${adapterCount} 个适配器 · ${providerCount} 个供应方摘要`}
            note="这里只说明可见边界；不验证模型、不读取密钥、不发起供应方调用。"
          />
          <BoundaryItem
            title="边车文件 / 原始状态"
            value={`${runtimeLogCount} 条运行日志索引 · ${diagnosticCount} 条诊断状态`}
            note="详细原始状态、诊断和日志摘要放在开发者区或右侧管理；普通首页不铺开。"
          />
          <BoundaryItem
            title="凭据"
            value="不可在 UI 中读取或展示"
            note="模型/凭据入口只用于说明配置边界，不显示令牌、密钥、认证授权或系统钥匙串内容。"
          />
          <BoundaryItem
            title="真实执行"
            value="不从设置页触发"
            note="设置页不运行真实执行或会话恢复命令，不写工作流状态，不替代权限弹层。"
          />
        </div>
      </section>

      <section className="panel settings-section developer-boundary-section">
        <div className="panel-h">
          页面读模型合同
          <Badge tone="unknown">R4-A1</Badge>
        </div>
        <p className="muted small-note">
          当前只冻结每个页面该读取什么、哪些内部材料不能放到首屏；页面仍使用既有 WorkbenchSnapshot，尚未切到按页查询。
        </p>
        <div className="developer-boundary-grid">
          <BoundaryItem
            title="合同数量"
            value={`${pageContractCount} 个页面合同`}
            note={`状态：${pageReadModelInventory.status}；来源：${pageReadModelInventory.source_policy}`}
          />
          <BoundaryItem
            title="迁移边界"
            value="contract only"
            note="R4-A1 不新增 Tauri command，不拆页面大组件，不重做视觉或布局。"
          />
        </div>
        <div className="developer-boundary-grid" aria-label="页面读模型合同清单">
          {pageReadModelInventory.contracts.map((contract) => (
            <BoundaryItem
              key={contract.page_id}
              title={contract.page_label}
              value={contract.planned_read_model}
              note={`用户数据：${contract.user_facing_data.join(" / ")}；首屏禁止：${contract.must_not_show_as_primary.join(" / ")}`}
            />
          ))}
        </div>
      </section>
    </section>
  );
}

function SettingFact({ label, value }: { label: string; value: string }) {
  return (
    <div className="settings-fact">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function BoundaryItem({ title, value, note }: { title: string; value: string; note: string }) {
  return (
    <article className="developer-boundary-item">
      <strong>{title}</strong>
      <span>{value}</span>
      <em>{note}</em>
    </article>
  );
}
