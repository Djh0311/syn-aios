# M4R01 修正合同、生产调用图与红灯验收

阶段：stage-07 阶段7 M4 独立修正与再验收
目标：不改写 M4 v1 冻结合同，逐项冻结五项 P1 的普通产品生产调用图、owner、单写者、失败反例、红灯探针和证据层级。
干完的标准：新增增补合同；至少点名一个真实内部 source owner 及普通 command/event 入口；五个 red probe 可重复复现旧缺口并保存旧基线 receipt；默认测试套件不留下永久失败或 ignored；冻结合同 hash exact。

允许动：

- docs/contracts/m4-independent-remediation-addendum-v1.md [新增]
- docs/harness/
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/

## 步骤

1. 固定 M1/M3/M4 合同、当前源码与总线复核输入 hash。
2. 定位普通 source owner、command/event 入口和五项 P1 的真实调用断点。
3. 新建不改写 v1 的增补合同，冻结 production call graph、旁路禁令、失败反例和验证层级。
4. 建立可重复 red probes，保存基线 red receipt；默认测试套件不得永久失败或 ignored。
5. 机械校验合同、调用图、hash 和测试入口，独立审查后精确提交并归档。
