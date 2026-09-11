# CORE-001：Cue 新增／替换统一校验编号唯一性

状态：done。负责人：Sol；由 Qwen 实现、Sol 独立验收。依赖：DEV-001、DEV-002（均已完成）。

执行基线：`d24751c915cd9c3578a03ea964c9651b49cd778d`。
工作区／分支：`/Users/sunqi/projects/stagemaster-worktrees/core-001`／
`core-001-cue-uniqueness`。写入范围保持本工单定义。
本契约已由 Astra 确定，无需再次申请架构评估。对应历史审查 R01。

验收提交：`49ddeadeb88eaf6448b1bf954298027157b915d3`；结果提交：
`22c24e24ab69bd26a7a9b2aadfb9602374c29abf`。基线定向测试按预期失败，
结果已通过定向测试、工作区测试、格式检查与 Clippy；详见交付记录。
集成提交：`df64f98603ca28462cf76a515b65fb39dda9b26d`。

## 范围

读取 `crates/stagemaster-show/src/lib.rs` 及相关领域类型。
实现仅允许改 `Sequence::upsert_cue` 附近的逻辑及必要注释，不改公共签名、CueNumber、其他编程器和 Tracking 语义。
Sol 创建并保护 `crates/stagemaster-show/tests/cue_number_uniqueness.rs`；Qwen 无权改该验收文件。
不新增第三方依赖。

## 固定行为

`upsert_cue(cue)` 在新增和替换时，都要求候选 `number` 不被另一个稳定 ID 的 Cue 使用。
相同 ID 可以沿用自己的编号，也可以改成尚未占用的编号。
碰撞统一返回现有 `ShowError::DuplicateCueNumber(candidate.number)`。

必须在修改序列前完成校验。任何失败均保持已有 Cue 的 ID、编号、名称、值与顺序不变。
成功后按既有编号排序规则排序；替换不增加 Cue 数量，同 ID 只保留一个。
不改变 `tracked_state` 的既有行为，不增加新的工程文件或永久 API。

## Sol 独立验收

1. 新 ID 插入空闲编号成功并排序。
2. 新 ID 使用已占用编号失败；整个 Sequence 与失败前 clone 相等。
3. 已有 ID 更换为另一 Cue 的编号失败；名称、值和顺序均未改变。
4. 已有 ID 保持自身编号但修改名称／值成功，数量不变。
5. 已有 ID 改为新的较小／较大编号成功，顺序正确。
6. 同一 Cue 再次提交保持唯一，不重复插入。

先在基线上证明第 3 项失败，再委派修复；已有测试仍须通过。
针对性命令：`cargo test -p stagemaster-show --locked --offline`；集成验证按根 AGENTS.md。
若格式／Clippy 报告的是 Sol 编写的测试问题，由 Sol 修正并记录，不要求 Qwen 越界改测试。

## 交付

按模板记录契约、基线／验收／候选／集成版本、实际测试、请求数及人工逻辑修改。
Sol 更新历史问题的解决状态与证据，不改写旧审查的历史事实。
