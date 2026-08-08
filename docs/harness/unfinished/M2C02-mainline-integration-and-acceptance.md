# M2C02 main 集成与完整验收

阶段：stage-03 M2 主线收口与交接
目标：把 M2 干净候选集成到 main，并在主线树上重跑与 M2 退出条件直接对应的完整验收。
干完的标准：main 只新增已审查的 M2 提交；完整 Rust 库测、R4 隔离 App、聚焦测试、Harness 检查和 Git 检查通过；M3 未启动。

允许动：

- refs/heads/main
- /Users/yoyi/workspace/product-line-syn-integration-main
- docs/harness/audit/ [新增]
- docs/harness/reports/ [新增]

## 步骤

1. 核对候选与 main 的 ancestry、路径集、产品 diff 和工作树清洁度。
2. 以 fast-forward 或明确的无歧义合并把候选集成到 main。
3. 在 main 运行 M2 focused、完整 Rust 库测、R4 隔离 App、Harness task/quick 与 Git 验证。
4. 固化 OID、tree、receipt 和边界结论。
