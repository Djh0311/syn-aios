// K · 设置(轻页)——设计定稿 `docs/design/2026-07-14-stage-b-hifi-fullapp-v1.html` 的 K 段。
// 首卡=人话四行(通知/数据/知识库/高级);开发者入口与内部边界摘要留在下方(=「高级」的第二跳)。
//
// ⚠️ 零 hooks:本文件被 `tests/r4-page-read-model-settings.test.tsx` 与
// `tests/offline-permission-dialog.test.tsx:2400` 以 `visibleText(<SettingsView …/>)` 走真 SSR 消费,
// 当前不在 renderComposite 裸调路径上。仍不引 hooks——四行全是纯投影,不需要。
import { FactRow, Pill } from "../components/SpecPrimitives";
import { deriveSettingsPageReadModelFromParts } from "../lib/pageSelectors";
import type { HomeSystemStatusReadModel } from "./HomeView";
import type { WorkbenchSnapshot, WorkflowStateSnapshot } from "../lib/types";
import type { ViewKey, WorkbenchNavItem } from "../lib/workbenchNavigation";

// 未接线口径与 ⑤ 首页统一一句人话(宪法 §四.3 禁机器术语上脸)。
const NOT_WIRED = "接线中";

type SettingsViewProps = {
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateError: string | null;
  hasRealSnapshot: boolean;
  developerItems: WorkbenchNavItem[];
  onNavigate: (view: ViewKey) => void;
  // 后端「系统状态读模型」(`tasks/2026-07-15-backend-ui-support-readmodels-package-v1.md` §A)未落地前恒为 null
  // →「数据」行显示「接线中」。契约类型复用 ⑤ 首页已立的同款,不另造。
  systemStatus?: HomeSystemStatusReadModel | null;
};

const developerDescriptions: Partial<Record<ViewKey, string>> = {
  proposal: "收纳建议方案入口；用于查看方案草案和确认边界。",
  workflow: "实验、模板、草图和后置画布资料；不是项目工作流事实源。",
  tools: "工具和运行器资源的只读索引说明；不提供直接运行按钮。",
  models: "模型、凭据、供应方和适配器边界说明；不读取密钥、令牌或认证授权材料。",
};

export function SettingsView({
  snapshot,
  workflowState,
  workflowStateError,
  hasRealSnapshot,
  developerItems,
  onNavigate,
  systemStatus = null,
}: SettingsViewProps) {
  const adapterCount = snapshot.agent_adapters.length;
  const providerCount = snapshot.provider_availability.length;
  const diagnosticCount = snapshot.diagnostic_summary.degraded_states.length;
  const runtimeLogCount = snapshot.runtime_log_store.entries.length;
  const pageReadModelInventory = snapshot.page_read_model_inventory;
  const pageReadModel = deriveSettingsPageReadModelFromParts({
    summary: snapshot.summary,
    workflowCount: workflowState?.counts.workflows ?? 0,
    workflowStateError,
    hasRealSnapshot,
    developerItems,
    adapterCount,
    providerCount,
    diagnosticCount,
    runtimeLogCount,
    pageReadModelInventory,
  });

  return (
    <section className="stage-pad settings-view">
      <div className="sr-only">
        <p>设置</p>
        <h1>工作台设置</h1>
        <p>{pageReadModel.snapshot_status_label}；{pageReadModel.boundary_text}</p>
      </div>

      {/* K 定稿首卡·人话四行。拿不到的值一律「接线中」+ 一句人话原因,不立假控件(宪法 §四.3 零假按钮)。 */}
      <section className="panel settings-section settings-plain-card">
        <div className="panel-h">
          设置
          <Pill tone="unknown">还没接上开关</Pill>
        </div>
        <FactRow k="通知">{NOT_WIRED}</FactRow>
        <FactRow k="数据">{storageLine(systemStatus)}</FactRow>
        <FactRow k="知识库">还没连知识库文件夹</FactRow>
        <FactRow k="高级">给开发用的内部页面（不用管）</FactRow>
        <p className="muted small-note">
          「通知」还没接上系统通知，暂时开不了关；
          {systemStatus ? null : "「数据」要等存储状态接上才说得准；"}
          「知识库」还没登记文件夹位置。「高级」= 下面这些开发者入口，平时不用管。
        </p>
        <div className="settings-plain-actions">
          <button className="secondary-button" type="button" onClick={() => onNavigate("knowledge")}>
            去知识库看看
          </button>
        </div>
      </section>

      <div className="settings-grid">
        <section className="panel settings-section">
          <div className="panel-h">
            常规
            <Pill tone={workflowStateError ? "warn" : "plain"}>{workflowStateError ? "事实层异常" : "只读"}</Pill>
          </div>
          <FactRow k="项目">{`${pageReadModel.general.project_count}`}</FactRow>
          <FactRow k="智能体会话">{`${pageReadModel.general.session_count}`}</FactRow>
          <FactRow k="技能">{`${pageReadModel.general.skill_count}`}</FactRow>
          <FactRow k="工作流">{`${pageReadModel.general.workflow_count}`}</FactRow>
          <p className="muted small-note">
            普通主导航展示项目、智能体、想法箱、知识库、记忆层、技能、运行器、运行中工作流；开发和内部边界统一从本页进入。
          </p>
          {workflowStateError ? <p className="rail-error">事实层读取失败：{workflowStateError}</p> : null}
        </section>

        <section className="panel settings-section developer-settings-section">
          <div className="panel-h">
            开发者
            <Pill tone="unknown">主动进入</Pill>
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
          <Pill tone="unknown">不展开原始材料</Pill>
        </div>
        <div className="developer-boundary-grid">
          <BoundaryItem
            title="适配器 / 供应方"
            value={`${pageReadModel.developer_boundary.adapter_count} 个适配器 · ${pageReadModel.developer_boundary.provider_count} 个供应方摘要`}
            note="这里只说明可见边界；不验证模型、不读取密钥、不发起供应方调用。"
          />
          <BoundaryItem
            title="边车文件 / 原始状态"
            value={`${pageReadModel.developer_boundary.runtime_log_count} 条运行日志索引 · ${pageReadModel.developer_boundary.diagnostic_count} 条诊断状态`}
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
          <Pill tone="unknown">R4-A1</Pill>
        </div>
        <p className="muted small-note">
          当前只冻结每个页面该读取什么、哪些内部材料不能放到首屏；页面仍使用既有 WorkbenchSnapshot，尚未切到按页查询。
        </p>
        <div className="developer-boundary-grid">
          <BoundaryItem
            title="合同数量"
            value={`${pageReadModel.page_contract.count} 个页面合同`}
            note={`状态：${pageReadModel.page_contract.status}；来源：${pageReadModel.page_contract.source_policy}`}
          />
          <BoundaryItem
            title="迁移边界"
            value="仅冻结合同"
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

// 「数据」行:定稿写「存得很稳(新引擎试运行中,一切正常)」,真源=后端 §A 的 storage_mode/storage_healthy/
// observation_day。读模型缺席 =「接线中」,不冒充「存得很稳」,也不拿 diagnostic_summary.degraded_states
// 顶替(那是通用降级诊断、不是存储健康,混用=编数据源)。
function storageLine(status: HomeSystemStatusReadModel | null): string {
  if (!status) return NOT_WIRED;
  if (!status.storage_healthy) {
    const reason = status.last_degradation?.reason_human?.trim();
    return reason ? `存得不太稳：${reason}` : "存得不太稳（原因还没读到）";
  }
  return status.storage_mode === "db_primary"
    ? `存得很稳（新引擎试运行中，第 ${status.observation_day} 天，一切正常）`
    : "存得很稳";
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
