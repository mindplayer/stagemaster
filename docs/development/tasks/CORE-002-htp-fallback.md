# CORE-002：HTP 默认值仅作无有效贡献时的回退

状态：done。负责人：Sol；Qwen 候选、Sol 收尾并独立验收。依赖：CORE-001。
本契约已由 Astra 确定，无需再次申请架构评估。对应历史审查 R02。

执行基线：`8ef3024d7a5fde3a03312a6927f202b6b12e84b9`。
工作区／分支：`/Users/sunqi/projects/stagemaster-worktrees/core-002`／
`core-002-htp-fallback`。写入范围保持本工单定义。

验收提交：`0dda451c7428bec970de87395cc3da6fe03977be`；结果提交：
`9c87cbe559245a8f033101c70da4fe39532a97ce`。基线定向测试按预期失败；
结果已通过定向测试、工作区测试、格式检查与 Clippy，详见交付记录。
集成提交：`81ed816fff5d8a358d5e1ecf933057a148d02e8f`。

## 范围

读取 `crates/stagemaster-engine/src/lib.rs` 和领域里的 `NormalizedValue`、`AttributeDescriptor`。
Qwen 只允许改 Mixer 的 HTP 分支及直接相关注释。Sol 创建并保护
`crates/stagemaster-engine/tests/htp_default_fallback.rs`。
保持所有公共类型和函数签名，不新增依赖，不修改 LTP 或 Playback 控制模式。

## 固定行为

按当前 A0 输入契约，weight 为零的贡献不参与求值；这只是本次兼容规则，不代表已经定下未来所有专业推杆模式。

对每个已知属性：

1. 没有有效贡献：输出 descriptor.default，trace 为空。
2. 有有效贡献：先选最高 priority 层，再在该层各项 `value.scale(weight)` 中取最大值。
3. 有效贡献可以给出显式零值；此时零仍是有效结果，不得回退到 default。
4. default 不参加有贡献时的最大值比较，也不作为隐式下限。
5. trace 继续记录参与求值的最高优先级层贡献，顺序沿用已有确定性排序。默认回退通过无有效贡献及空 trace 判断，本任务不扩展公共来源类型。

未知属性静默丢弃等其他已知问题留给 G1，不在本任务偷偷改变接口。

## Sol 独立验收

为减少舍入歧义，优先使用 `NormalizedValue::from_raw` 固定输入。

| 场景 | 固定期望 |
| --- | --- |
| default=40000，无贡献 | 40000，空 trace |
| default=40000，唯一贡献 value=10000、满权重 | 10000 |
| default=40000，同优先级满权重贡献 10000／20000 | 20000 |
| default=40000，value=0、满权重 | 0，trace 有贡献 |
| 高优先级 10000，低优先级 50000，均满权重 | 10000，trace 仅高层 |
| 高优先级贡献 weight=0，低优先级满权重 30000 | 30000 |
| 所有贡献 weight=0 | default，空 trace |
| 同层存在非满权重 | 按已有 scale 舍入后比较；不改变缩放实现 |

先证明低于非零默认值的场景在基线上失败。既有优先级和 LTP 测试不得回归。
针对性命令：`cargo test -p stagemaster-engine --locked --offline`；集成验证按根 AGENTS.md。

## 交付

记录实际测试、版本、模型请求和接管情况。清楚标记修复范围仅为 R02，不能据此宣称完成专业 HTP／LTP 合成体系。
