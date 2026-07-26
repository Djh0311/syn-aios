# S1B-H2-R3 canonical 三消息矩阵（脱敏）

三条均为 target workflow 的 recorded ordinal `9–11/11`；文本 SHA-256 均为
`8576e1e7036849ced5bd435247b60ba5e28b0a640acea573ef05556ea0d72079`，与 R2 固定首句 hash 一致。
不保存原文。

| # | message_id | recorded event | 时间（+0800） | client SHA-256（截断） | injected | reply |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | `user:1784666691878362000` | `supervisor-resident-message:user:1784666691878379000` | 07-22 04:44:51.878 | `ceed5246…21df95`（尾 `872523ab`） | 0 | 0 |
| 2 | `user:1784666696842452000` | `supervisor-resident-message:user:1784666696842456000` | 07-22 04:44:56.842 | `eca80860…adede0`（尾 `879c0847`） | 0 | 0 |
| 3 | `user:1784666700190151000` | `supervisor-resident-message:user:1784666700190154000` | 07-22 04:45:00.190 | `0d75c566…c954f2`（尾 `7f0f9ba4`） | 0 | 0 |

逐条以 `message_id` 检索 injected、以 `reply_to_message_id` 检索 reply、再以同一 `target_ref` 交叉检索；每条均只关联自己的 recorded event。三个 message/client identity 均不同，因此这三笔与用户三次独立发送相符，不能判作重复落账。

SQLite 的 workflow audit projection 逐条与上表一致；R2 时间窗的 injected/reply 投影均为零。
