# L2 改下单逻辑写入 status

阶段：stage-02 库存联动
目标：下单时给订单写入 status，默认 pending
干完的标准：新订单能查到状态，老数据不受影响

允许动：
- src/order/create.ts
- src/models/order.ts

## 步骤
1. 写一个失败的测试：`test/order/create.test.ts`，跑 `npm test -- create`，期望 FAIL "status is undefined"
2. 改 `src/models/order.ts` 加字段，跑同一条命令，期望仍 FAIL
3. 改 `src/order/create.ts` 写入默认值，跑同一条命令，期望 PASS
4. 跑 `npm test -- order`，期望全绿

<!-- 先落 plan/stage/leaf 再执行。整阶段已经授权时，不逐 leaf 重复询问。
     "允许动"每条要么能指回阶段文件，要么标 [新增]。
     未完成用 hl park；完成才用 hl done，后者代表完成声明并归档。 -->
