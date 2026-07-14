// P1 · 工作流自动连环 controller（决策 decisions/2026-06-23-test-project-auto-chain-light-tier-v1.md）
//
// 实质 = 「乙·自动连环，圈死在固定测试项目」：按真实 nodes+edges 拓扑序，自动逐节点连跑，
// 一次启动跑到底、无逐步审批（用户 2026-06-23 拍板「直接全跑」）。这是跨高危#4 的下放，
// 仅在固定测试项目成立，靠下面四条硬护栏兜住（删了逐步闸后仅剩的安全，必须全在）：
//   ① runaway 上限：单链最多派发 min(节点数, 硬顶 50)；拓扑序每节点最多一次；失败即停、不自动重试。
//   ② 可中断：stop_requested 标志，每个节点边界检查 → 停。
//   ③ 审计：链起/续、每节点 start·done·fail、链停/完成/失败 都进 audit_events。
//   ④ 可回滚：起链前 backup_workflow_state_file（+ 依赖测试项目 git）。
//
// 每节点真派发复用已 gated 的 execute_project_workflow_node_at（双闸 + 沙箱 + resume 会话 + 自动临时
// work_item），不旁路、不新开闸、codex_local_runner / 安全闸 / codex 审批 / manual_relay 字节不动。
// path-lock 闸（workflow_engine_test_project_unsealed）在 _at 顶部复用 —— 非测试项目直接拒，连环更不碰真实仓。

const WORKFLOW_CHAIN_MAX_NODES_HARD_CAP: usize = 50;

#[derive(Deserialize)]
struct ProjectWorkflowChainRunRequest {
    project_root: String,
    workflow_id: String,
    // 可选 runaway 上限；只能比 min(节点数,50) 更小，越不过硬顶。
    #[serde(default)]
    max_nodes: Option<usize>,
}

#[derive(Deserialize)]
struct ProjectWorkflowChainStopRequest {
    project_root: String,
    workflow_id: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct ProjectWorkflowChainRunResult {
    message: String,
    path: String,
    chain_run_id: String,
    workflow_id: String,
    state: String,
    dispatched_count: usize,
    max_nodes: usize,
    nodes: Vec<Value>,
}

// 拓扑序（确定性 Kahn 变体）：edge from→to 表示 from 先于 to。按 node_ids 输入顺序做 tiebreak，
// 保证同图同序。返回排好的 node_id；存在环 → Err（P1 链只支持无环 DAG）。
fn workflow_chain_topological_order(
    node_ids: &[String],
    edges: &[(String, String)],
) -> Result<Vec<String>, String> {
    use std::collections::BTreeSet;
    let node_set: BTreeSet<&str> = node_ids.iter().map(String::as_str).collect();
    let mut emitted: BTreeSet<String> = BTreeSet::new();
    let mut order: Vec<String> = Vec::new();
    while order.len() < node_ids.len() {
        let mut progressed = false;
        for n in node_ids {
            if emitted.contains(n) {
                continue;
            }
            // 本工作流内所有前驱都已 emitted？（指向工作流外节点的边忽略）
            let ready = edges.iter().all(|(from, to)| {
                to != n || !node_set.contains(from.as_str()) || emitted.contains(from)
            });
            if ready {
                emitted.insert(n.clone());
                order.push(n.clone());
                progressed = true;
            }
        }
        if !progressed {
            return Err("工作流图存在环（cycle），P1 链只支持无环 DAG；请去掉回边再起链".to_string());
        }
    }
    Ok(order)
}

fn chain_run_record<'a>(value: &'a Value, chain_run_id: &str) -> Option<&'a Value> {
    value
        .get("workflow_chain_runs")
        .and_then(Value::as_array)?
        .iter()
        .find(|r| optional_string_from(r, "chain_run_id").as_deref() == Some(chain_run_id))
}

fn chain_run_dispatched_count(value: &Value, chain_run_id: &str) -> usize {
    chain_run_record(value, chain_run_id)
        .and_then(|r| r.get("nodes"))
        .and_then(Value::as_array)
        .map(|ns| {
            ns.iter()
                .filter(|n| optional_string_from(n, "state").as_deref() == Some("completed"))
                .count()
        })
        .unwrap_or(0)
}

fn chain_node_state(value: &Value, chain_run_id: &str, node_id: &str) -> Option<String> {
    chain_run_record(value, chain_run_id)?
        .get("nodes")?
        .as_array()?
        .iter()
        .find(|n| optional_string_from(n, "node_id").as_deref() == Some(node_id))
        .and_then(|n| optional_string_from(n, "state"))
}

fn chain_run_stop_requested(value: &Value, chain_run_id: &str) -> bool {
    chain_run_record(value, chain_run_id)
        .and_then(|r| r.get("stop_requested"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn chain_run_max_nodes(value: &Value, chain_run_id: &str) -> usize {
    chain_run_record(value, chain_run_id)
        .and_then(|r| r.get("max_nodes"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

// 找未完成的链运行（running/stopped）→ 续；否则新建。返回 chain_run_id。
fn ensure_chain_run_record(
    value: &mut Value,
    project_id_value: &str,
    workflow_id: &str,
    order: &[String],
    max_nodes: usize,
    timestamp: &str,
) -> Result<String, String> {
    let runs = ensure_array_mut(value, "workflow_chain_runs")?;
    let existing_idx = runs.iter().position(|r| {
        optional_string_from(r, "workflow_id").as_deref() == Some(workflow_id)
            && optional_string_from(r, "project_id").as_deref() == Some(project_id_value)
            && matches!(
                optional_string_from(r, "state").as_deref(),
                Some("running") | Some("stopped")
            )
    });
    if let Some(idx) = existing_idx {
        let run = &mut runs[idx];
        run["state"] = json!("running");
        run["stop_requested"] = json!(false);
        run["max_nodes"] = json!(max_nodes);
        run["ended_at"] = Value::Null;
        // 图可能改过：补齐缺失节点为 pending（已有节点状态保留 → 断点续）。
        let present: std::collections::BTreeSet<String> = run
            .get("nodes")
            .and_then(Value::as_array)
            .map(|ns| {
                ns.iter()
                    .filter_map(|n| optional_string_from(n, "node_id"))
                    .collect()
            })
            .unwrap_or_default();
        let mut nodes = run
            .get("nodes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for nid in order {
            if !present.contains(nid) {
                nodes.push(json!({
                  "node_id": nid, "state": "pending", "dispatch_id": Value::Null, "message": Value::Null
                }));
            }
        }
        run["nodes"] = Value::Array(nodes);
        return Ok(optional_string_from(run, "chain_run_id").unwrap_or_default());
    }
    let chain_run_id = format!("workflow-chain-run:{}:{timestamp}", stable_id(workflow_id));
    runs.push(json!({
      "chain_run_id": chain_run_id.clone(),
      "project_id": project_id_value,
      "workflow_id": workflow_id,
      "state": "running",
      "max_nodes": max_nodes,
      "stop_requested": false,
      "started_at": timestamp,
      "ended_at": Value::Null,
      "nodes": order.iter().map(|nid| json!({
        "node_id": nid, "state": "pending", "dispatch_id": Value::Null, "message": Value::Null
      })).collect::<Vec<_>>()
    }));
    Ok(chain_run_id)
}

fn set_chain_node_state(
    value: &mut Value,
    chain_run_id: &str,
    node_id: &str,
    state: &str,
    dispatch_id: Option<&str>,
    message: Option<&str>,
) {
    let Some(runs) = value
        .get_mut("workflow_chain_runs")
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    let Some(run) = runs
        .iter_mut()
        .find(|r| optional_string_from(r, "chain_run_id").as_deref() == Some(chain_run_id))
    else {
        return;
    };
    let Some(nodes) = run.get_mut("nodes").and_then(Value::as_array_mut) else {
        return;
    };
    if let Some(node) = nodes
        .iter_mut()
        .find(|n| optional_string_from(n, "node_id").as_deref() == Some(node_id))
    {
        node["state"] = json!(state);
        if let Some(d) = dispatch_id {
            node["dispatch_id"] = json!(d);
        }
        if let Some(m) = message {
            node["message"] = json!(m);
        }
    }
}

fn finalize_chain_run(value: &mut Value, chain_run_id: &str, final_state: &str, timestamp: &str) {
    if let Some(runs) = value
        .get_mut("workflow_chain_runs")
        .and_then(Value::as_array_mut)
    {
        if let Some(run) = runs
            .iter_mut()
            .find(|r| optional_string_from(r, "chain_run_id").as_deref() == Some(chain_run_id))
        {
            run["state"] = json!(final_state);
            run["ended_at"] = json!(timestamp);
            run["stop_requested"] = json!(false);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_chain_audit(
    value: &mut Value,
    chain_run_id: &str,
    workflow_id: &str,
    event_type: &str,
    before_state: &str,
    after_state: &str,
    timestamp: &str,
    reason: &str,
) -> Result<(), String> {
    ensure_array_mut(value, "audit_events")?.push(json!({
      "event_id": crate::workflow_audit::audit_event_identity(event_type, chain_run_id, timestamp),
      "event_type": event_type,
      "target_ref": chain_run_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state_and_codex_resume",
      "permission_level": "user_confirmed_write",
      "before_state": before_state,
      "after_state": after_state,
      "workflow_id": workflow_id,
      "created_at": timestamp,
      "reason": reason
    }));
    Ok(())
}

fn chain_run_result(
    value: &Value,
    chain_run_id: &str,
    workflow_id: &str,
    max_nodes: usize,
    message: String,
    path: &Path,
) -> ProjectWorkflowChainRunResult {
    let record = chain_run_record(value, chain_run_id);
    let state = record
        .and_then(|r| optional_string_from(r, "state"))
        .unwrap_or_else(|| "unknown".to_string());
    let nodes = record
        .and_then(|r| r.get("nodes"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    ProjectWorkflowChainRunResult {
        message,
        path: path.display().to_string(),
        chain_run_id: chain_run_id.to_string(),
        workflow_id: workflow_id.to_string(),
        state,
        dispatched_count: chain_run_dispatched_count(value, chain_run_id),
        max_nodes,
        nodes,
    }
}

// 链驱动核心（可测，不依赖 tauri::State）。
fn run_project_workflow_chain_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &ProjectWorkflowChainRunRequest,
) -> Result<ProjectWorkflowChainRunResult, String> {
    // path-lock 闸复用（不改闸本身，高危#3 不动）：非测试项目直接拒——自动连环更不能碰真实仓。
    if !workflow_engine_test_project_unsealed(&request.project_root) {
        return Err(legacy_product_command_blocked_message(
            "start_project_workflow_chain",
        ));
    }
    let workflow_id = request.workflow_id.trim().to_string();
    if workflow_id.is_empty() {
        return Err("workflow_id 不能为空；无法起链".to_string());
    }
    if !path.exists() {
        return Err("工作流状态文件不存在；无法起链".to_string());
    }
    let mut value = read_workflow_state_value(path)?;
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目下找不到该 workflow；无法起链".to_string());
    }

    // 1) 取本工作流的 node_ids + edges
    let node_ids: Vec<String> = value
        .get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter(|n| {
                    optional_string_from(n, "workflow_id").as_deref() == Some(workflow_id.as_str())
                })
                .filter_map(|n| optional_string_from(n, "node_id"))
                .collect()
        })
        .unwrap_or_default();
    if node_ids.is_empty() {
        return Err("当前 workflow 没有任何节点；无法起链".to_string());
    }
    let edges: Vec<(String, String)> = value
        .get("edges")
        .and_then(Value::as_array)
        .map(|edges| {
            edges
                .iter()
                .filter(|e| {
                    optional_string_from(e, "workflow_id").as_deref() == Some(workflow_id.as_str())
                })
                .filter_map(|e| {
                    Some((
                        optional_string_from(e, "from_node_id")?,
                        optional_string_from(e, "to_node_id")?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    // 2) 拓扑序（有环即拒，P1 不支持 loop）
    let order = workflow_chain_topological_order(&node_ids, &edges)?;

    // 3) runaway 上限：min(请求, 节点数, 硬顶 50)，至少 1
    let requested_cap = request.max_nodes.unwrap_or(node_ids.len());
    let max_nodes = requested_cap
        .min(node_ids.len())
        .min(WORKFLOW_CHAIN_MAX_NODES_HARD_CAP)
        .max(1);

    let pid = project_id(&request.project_root);
    let timestamp = unix_timestamp_string();

    // 4) find-or-create 链运行记录（断点续：复用未完成的） + 起链前 backup（可回滚） + 审计「链起/续」
    let chain_run_id =
        ensure_chain_run_record(&mut value, &pid, &workflow_id, &order, max_nodes, &timestamp)?;
    backup_workflow_state_file(path, &timestamp)?;
    append_chain_audit(
        &mut value,
        &chain_run_id,
        &workflow_id,
        "workflow_chain_run_started",
        "ready",
        "running",
        &timestamp,
        "用户授权启动工作流自动连环（圈固定测试项目，决策 2026-06-23）：按拓扑序逐节点自动派发，失败即停、可中断、有 runaway 上限。",
    )?;
    write_m5b_batch1_workflow_state(path, "workflow_chain_run_started", &value)?;

    // 5) 拓扑序逐节点连跑
    let mut dispatched = chain_run_dispatched_count(&value, &chain_run_id);
    let mut final_state = "completed".to_string();
    let mut closing_message = "工作流自动连环完成：所有节点按拓扑序真派发成功。".to_string();

    for node_id in &order {
        // 5a) runaway 上限
        if dispatched >= max_nodes {
            final_state = "stopped".to_string();
            closing_message = format!("达到 runaway 上限（{max_nodes} 个节点），已停链。");
            break;
        }
        // 5b) 每次重读（execute_* 会写文件；同时拿 stop 标志最新值）
        let mut current = read_workflow_state_value(path)?;
        // 断点续：本节点已 completed → 跳过
        if chain_node_state(&current, &chain_run_id, node_id).as_deref() == Some("completed") {
            continue;
        }
        // 可中断：stop_requested → 在节点边界停
        if chain_run_stop_requested(&current, &chain_run_id) {
            final_state = "stopped".to_string();
            closing_message =
                "收到停链请求，已在节点边界停下（已完成节点保留，可断点续）。".to_string();
            break;
        }
        // 标 running + 审计 node-start
        let ts_start = unix_timestamp_string();
        set_chain_node_state(&mut current, &chain_run_id, node_id, "running", None, None);
        append_chain_audit(
            &mut current,
            &chain_run_id,
            &workflow_id,
            "workflow_chain_node_started",
            "pending",
            "running",
            &ts_start,
            &format!("自动连环：派发节点 {node_id}"),
        )?;
        write_m5b_batch1_workflow_state(path, "workflow_chain_node_started", &current)?;

        // 真派发（复用 gated 的 _at：resume 会话、自动临时 work_item、双闸 + 沙箱）
        let node_request = ProjectWorkflowNodeRunRequest {
            project_root: request.project_root.clone(),
            node_id: node_id.clone(),
            work_item_id: String::new(),
            workflow_id: Some(workflow_id.clone()),
        };
        let outcome =
            execute_project_workflow_node_at(path, index, readback_db_path, runner, &node_request);

        // 重读（execute_* 写过文件，避免覆盖它的写入）
        let mut after = read_workflow_state_value(path)?;
        let ts_done = unix_timestamp_string();
        let node_ok = matches!(&outcome, Ok(result) if result.dispatch.state == "completed");
        if node_ok {
            let dispatch_id = match &outcome {
                Ok(result) => result.dispatch.dispatch_id.clone(),
                Err(_) => String::new(),
            };
            set_chain_node_state(
                &mut after,
                &chain_run_id,
                node_id,
                "completed",
                Some(&dispatch_id),
                None,
            );
            append_chain_audit(
                &mut after,
                &chain_run_id,
                &workflow_id,
                "workflow_chain_node_completed",
                "running",
                "completed",
                &ts_done,
                &format!("自动连环：节点 {node_id} 真派发成功（dispatch {dispatch_id}）"),
            )?;
            write_m5b_batch1_workflow_state(path, "workflow_chain_node_completed", &after)?;
            dispatched += 1;
        } else {
            // 失败即停（不自动重试 / 不跳过，防在老失败节点上打转）
            let (dispatch_id, fail_msg) = match &outcome {
                Ok(result) => (
                    Some(result.dispatch.dispatch_id.clone()),
                    format!("节点 codex 派发未完成（state={}）", result.dispatch.state),
                ),
                Err(e) => (None, e.clone()),
            };
            set_chain_node_state(
                &mut after,
                &chain_run_id,
                node_id,
                "failed",
                dispatch_id.as_deref(),
                Some(&fail_msg),
            );
            append_chain_audit(
                &mut after,
                &chain_run_id,
                &workflow_id,
                "workflow_chain_node_failed",
                "running",
                "failed",
                &ts_done,
                &format!("自动连环：节点 {node_id} 失败即停——{fail_msg}"),
            )?;
            finalize_chain_run(&mut after, &chain_run_id, "failed", &ts_done);
            let stop_message =
                format!("节点 {node_id} 失败，已停链（失败即停、不自动重试）：{fail_msg}");
            append_chain_audit(
                &mut after,
                &chain_run_id,
                &workflow_id,
                "workflow_chain_run_failed",
                "running",
                "failed",
                &ts_done,
                &stop_message,
            )?;
            write_m5b_batch1_workflow_state(path, "workflow_chain_node_failed", &after)?;
            return Ok(chain_run_result(
                &after,
                &chain_run_id,
                &workflow_id,
                max_nodes,
                stop_message,
                path,
            ));
        }
    }

    // 6) 收尾（completed 或 stopped）
    let mut closing = read_workflow_state_value(path)?;
    let ts_close = unix_timestamp_string();
    finalize_chain_run(&mut closing, &chain_run_id, &final_state, &ts_close);
    let event_type = if final_state == "stopped" {
        "workflow_chain_run_stopped"
    } else {
        "workflow_chain_run_completed"
    };
    append_chain_audit(
        &mut closing,
        &chain_run_id,
        &workflow_id,
        event_type,
        "running",
        &final_state,
        &ts_close,
        &closing_message,
    )?;
    write_m5b_batch1_workflow_state(path, "workflow_chain_run_finalized", &closing)?;
    Ok(chain_run_result(
        &closing,
        &chain_run_id,
        &workflow_id,
        max_nodes,
        closing_message,
        path,
    ))
}

// 停链：置 stop 标志，运行中的链在下个节点边界看到 → 停。停链本身不跑 codex、不需 path-lock 闸。
fn stop_project_workflow_chain_at(
    path: &Path,
    request: &ProjectWorkflowChainStopRequest,
) -> Result<ProjectWorkflowChainRunResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法停链".to_string());
    }
    let workflow_id = request.workflow_id.trim().to_string();
    let pid = project_id(&request.project_root);
    let mut value = read_workflow_state_value(path)?;
    let chain_run_id = value
        .get("workflow_chain_runs")
        .and_then(Value::as_array)
        .and_then(|runs| {
            runs.iter().find(|r| {
                optional_string_from(r, "workflow_id").as_deref() == Some(workflow_id.as_str())
                    && optional_string_from(r, "project_id").as_deref() == Some(pid.as_str())
                    && optional_string_from(r, "state").as_deref() == Some("running")
            })
        })
        .and_then(|r| optional_string_from(r, "chain_run_id"))
        .ok_or_else(|| "没有正在运行的链可停（该工作流当前无 running 链）".to_string())?;
    let timestamp = unix_timestamp_string();
    if let Some(runs) = value
        .get_mut("workflow_chain_runs")
        .and_then(Value::as_array_mut)
    {
        if let Some(run) = runs
            .iter_mut()
            .find(|r| optional_string_from(r, "chain_run_id").as_deref() == Some(chain_run_id.as_str()))
        {
            run["stop_requested"] = json!(true);
        }
    }
    append_chain_audit(
        &mut value,
        &chain_run_id,
        &workflow_id,
        "workflow_chain_stop_requested",
        "running",
        "running",
        &timestamp,
        "用户请求停链；将在下个节点边界停下，已完成节点保留、可断点续。",
    )?;
    write_m5b_batch1_workflow_state(path, "workflow_chain_stop_requested", &value)?;
    let max_nodes = chain_run_max_nodes(&value, &chain_run_id);
    Ok(chain_run_result(
        &value,
        &chain_run_id,
        &workflow_id,
        max_nodes,
        "已请求停链（将在下个节点边界生效）。".to_string(),
        path,
    ))
}

// async + spawn_blocking：链同步阻塞跑（每节点真起 codex，长耗时）。Tauri 的同步命令跑在主线程，
// 整条链会把 UI 冻死（转圈、连「停链」都点不动），且停链命令也抢不到主线程 → 可中断形同虚设。
// 改成 async 命令 + 把阻塞链体丢进 spawn_blocking 线程池：主线程空出来、UI 可响应、停链可点并在节点
// 边界生效。index/path 在 await 前从 state 取出（State 不能跨进 'static 闭包）。
#[tauri::command]
async fn start_project_workflow_chain(
    request: ProjectWorkflowChainRunRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowChainRunResult, String> {
    let path = state.workflow_state_path.clone();
    let index = read_index(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let readback_db_path = codex_db::default_state_db_path();
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        run_project_workflow_chain_at(&path, &index, &readback_db_path, &runner, &request)
    })
    .await
    .map_err(|error| format!("链执行线程异常：{error}"))?
}

#[tauri::command]
fn stop_project_workflow_chain(
    request: ProjectWorkflowChainStopRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowChainRunResult, String> {
    stop_project_workflow_chain_at(&state.workflow_state_path, &request)
}

// #19 实时进度：读该工作流最新一条链运行记录（state + 每节点状态），供画布轮询显示。只读、无副作用。
fn latest_chain_run_for(value: &Value, project_root: &str, workflow_id: &str) -> Option<Value> {
    let wid = workflow_id.trim();
    let pid = project_id(project_root);
    value
        .get("workflow_chain_runs")
        .and_then(Value::as_array)
        .and_then(|runs| {
            runs.iter()
                .filter(|r| {
                    optional_string_from(r, "workflow_id").as_deref() == Some(wid)
                        && optional_string_from(r, "project_id").as_deref() == Some(pid.as_str())
                })
                .max_by_key(|r| {
                    optional_string_from(r, "started_at")
                        .and_then(|s| s.parse::<i64>().ok())
                        .unwrap_or(0)
                })
                .cloned()
        })
}

#[tauri::command]
fn get_project_workflow_chain_status(
    project_root: String,
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<Value>, String> {
    if !state.workflow_state_path.exists() {
        return Ok(None);
    }
    let value = read_workflow_state_value(&state.workflow_state_path)?;
    Ok(latest_chain_run_for(&value, &project_root, &workflow_id))
}

#[cfg(test)]
mod workflow_chain_controller_topo_tests {
    use super::*;

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }
    fn edge(from: &str, to: &str) -> (String, String) {
        (from.to_string(), to.to_string())
    }

    #[test]
    fn topological_order_respects_edges_and_is_deterministic() {
        let nodes = ids(&["a", "b", "c", "d"]);
        // a→b, a→c, b→d, c→d
        let edges = vec![edge("a", "b"), edge("a", "c"), edge("b", "d"), edge("c", "d")];
        let order = workflow_chain_topological_order(&nodes, &edges).expect("dag");
        let pos = |x: &str| order.iter().position(|n| n == x).unwrap();
        assert!(pos("a") < pos("b"));
        assert!(pos("a") < pos("c"));
        assert!(pos("b") < pos("d"));
        assert!(pos("c") < pos("d"));
        // 确定性：tiebreak 按输入顺序 → b 在 c 前
        assert!(pos("b") < pos("c"));
        // 同输入再跑一次应完全一致
        let again = workflow_chain_topological_order(&nodes, &edges).expect("dag");
        assert_eq!(order, again);
    }

    #[test]
    fn topological_order_detects_cycle() {
        let nodes = ids(&["a", "b", "c"]);
        let edges = vec![edge("a", "b"), edge("b", "c"), edge("c", "a")];
        let err = workflow_chain_topological_order(&nodes, &edges).unwrap_err();
        assert!(err.contains("环"), "cycle should be rejected, got: {err}");
    }

    #[test]
    fn topological_order_ignores_edges_to_outside_nodes() {
        let nodes = ids(&["a", "b"]);
        // 指向工作流外节点 z 的边不该卡住 b
        let edges = vec![edge("a", "b"), edge("z", "b")];
        let order = workflow_chain_topological_order(&nodes, &edges).expect("dag");
        assert_eq!(order, ids(&["a", "b"]));
    }

    #[test]
    fn isolated_nodes_keep_input_order() {
        let nodes = ids(&["x", "y", "z"]);
        let order = workflow_chain_topological_order(&nodes, &[]).expect("dag");
        assert_eq!(order, ids(&["x", "y", "z"]));
    }
}
