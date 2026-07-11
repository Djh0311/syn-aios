# 实现任务包:第四波——授权锁竞态重试+运行态同图+删节点文案 · 总指导 → 执行线 v1(可派)

日期:2026-07-11 · 来源:用户真机三报:「只会用于这一步…」删/开始后画布变回老样子/只读单卡住。**卡住已由总指导破案**:`plan_authorization_store_locked` = M1 批准链路多步写授权 store 的瞬时锁竞态(锁文件 create_new 互斥·用完即删·账本 07:45:45 事件+现场锁已消失=transient),非死锁非只读单专属。

## 1. 建什么

### 1.1 锁竞态根修(后端·小而关键)
- `plan_authorization_store.rs` `StoreLock::acquire`(:1051 一带)撞 `AlreadyExists` → **有限重试**(如 5 次×80-120ms 退避)再放弃;放弃时错误话术加「稍等几秒再点一次就好」;
- **不改锁语义本身**(create_new 互斥/Drop 释放照旧·只加获取重试);不改审批逻辑(高危#3 不碰——这是锁获取健壮性不是审批规则);
- 回归:并发双写单测(一个持锁一个重试成功)+ 案发式(锁存在短暂后释放→重试拿到);
- **前端死脸补丁(案发追加)**:合流命令 post_confirm 段失败返回时,前端**必须刷新方案店**再显示错误(实案:record_decision 成功[方案已 user_confirmed]→边界批准撞锁→前端快照仍 pending→界面既非待批也不给[接着跑]=两头不沾死脸·用户 Cmd+R 才自愈)。失败路径 onProposalStoreRefresh + 若方案已 user_confirmed 直接置 continueHint(出[接着跑]),不许让确认态躺在旧快照里;
- **悬挂授权自愈**:授权 `pending_global_boundary_review` 且方案 user_confirmed 时,[接着跑]/auto_advance 路径应能补记边界批准转 active(复用现成 record_global_boundary_review·actor=user·与合流同语义)或经 control_core 既有的 pending 并列放行(执行线亲核 :408/lib.rs:2187 两处「Active|Pending」并列判定的真实语义后择一·不许新造第三条转正路)。

### 1.2 运行态同图(前端为主·核心)
- **现状**:authorize/binding 显纵向工序图(previewCanvas),`running` 起切回 workflowPanel(旧 ReactFlow 运行视图)=「变回老样子」;
- **目标**:开始后**同一张纵向工序图不换脸**,节点按运行态点亮——数据源换成真实链态(chainStatus/run 节点状态:pending/running/completed/needs_rework/failed),卡样式沿预演卡加运行态色(复用现有状态色语义);跑到哪亮到哪;节点上会话标仍显(已绑定的显实际会话);
- 运行态节点**只读**(不再可改会话·点开只看);旧 workflowPanel(ReactFlow)保留在「工作流」tab 完整视图,合一页 running 相不再用它;
- 简单活单节点同理;done/blocked 相图保持终态显示。

### 1.3 删文案(用户点名)
- 节点 details 内「只会用于这一步;其余节点默认各开新会话。」删(SessionPicker label 保留)。

## 2. 安全死线

- 审批逻辑/人闸/冻结核/runner/h5 0-diff(1.1 只动锁获取重试);五态逻辑不改(1.2 是画布区渲染换数据源);blocked 死配对零改;`WorkflowCanvasEngine` 本体不动(合一页不再嵌它=少一个依赖,不是改它)。

## 3. 验收

- 1.1:并发单测+案发式重试测;`cargo test --lib` 计数不降(780 基线);
- 1.2:离线 DOM——running 相位画布 = 纵向图+运行态类(非 workflowPanel);节点状态映射断言(completed/running/pending 各一);done 相终态图;
- 1.3:负断言;
- typecheck+离线全套;真机:用户走一单看「开始后图不换脸·跑到哪亮到哪」(待真机口径)。

## 4. 回交

锁重试参数+竞态测试证据 / 运行态数据源接法 / §3 证据 → 总指导核实物。**子线不 commit。**

## 5. 不接受为

- 改锁语义或审批逻辑 / 为运行态改画布引擎本体 / 运行态节点可改会话 / 删 SessionPicker label / 动后端链驱动。
