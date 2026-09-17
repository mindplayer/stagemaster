# CORE-002 交付

> 已完成任务的历史记录：执行者、工作区和验证结果保留作追溯；不作为当前模型分工或重做任务的指令。当前规则见 [开发方法](../README.md)。

- 状态：已集成
- 执行者、模型／运行时版本：Qwen3.8-27B MLX 8-bit + MTP；mlx-vlm 0.7.0 / MLX 0.32.2；Sol 负责保护验收、候选审查和额度耗尽后的收尾
- 契约版本、基线提交、工作区／分支：工单固定契约；`8ef3024d7a5fde3a03312a6927f202b6b12e84b9`；`/Users/sunqi/projects/stagemaster-worktrees/core-002`／`core-002-htp-fallback`
- 验收提交：`0dda451c7428bec970de87395cc3da6fe03977be`
- 结果提交／补丁哈希：`9c87cbe559245a8f033101c70da4fe39532a97ce`；最终提案 `2e1f19e78dff09f4d1bd1177717922f921e5ca81f1ec12870acb485ad5cf95b0`
- 集成提交：`81ed816fff5d8a358d5e1ecf933057a148d02e8f`

## 结果

Mixer 的 HTP 分支现在仅在过滤后没有有效贡献时返回 descriptor default 和空
trace。有贡献时只选择最高 active priority 层，对该层逐项执行既有 `scale` 后取
最大值；低于 default 的值和显式零都保留。LTP、公共接口、缩放实现、未知属性与
Playback 控制行为均未改变。

Qwen job `CORE-002-QWEN-001-62c5ebba6722` 只生成
`crates/stagemaster-engine/src/lib.rs` 的候选。两次 repair 后模型额度用尽；最终候选
测试通过但因内部 `expect` 触发 Clippy 的 panic 文档规则，Sol 将这一行改为无 panic
的 `unwrap_or(descriptor.default)`。保护验收未授权写入且未发生变化。

## 验证

| 命令／设备场景 | 退出码／结果 | 被测版本 | 证据位置 |
| --- | --- | --- | --- |
| `cargo test -p stagemaster-engine --test htp_default_fallback --locked --offline` | 101；2 passed、6 failed，复现 default 下限缺陷 | `0dda451` | 交付会话记录；保护验收文件 |
| attempt 1 `cargo test -p stagemaster-engine --locked --offline` | 101；E0308，非发散 `let-else` | Qwen attempt 1 | 原始诊断文件已于 DEV-005 清理；历史结果保留于本记录 |
| attempt 2 同命令 | 101；E0282，`Option` 类型无法推断 | Qwen attempt 2 | 原始诊断文件已于 DEV-005 清理；历史结果保留于本记录 |
| attempt 3 引擎测试／工作区测试 | 0；11 / 24 passed | Qwen attempt 3 + rustfmt | 交付会话记录 |
| attempt 3 Clippy | 101；`expect` 触发 `missing_panics_doc` | Qwen attempt 3 + rustfmt | 交付会话记录 |
| `cargo test -p stagemaster-engine --locked --offline` | 0；3 项已有单测 + 8 项独立验收通过 | `9c87cbe` | CORE-002 worktree |
| `cargo test --workspace --locked --offline` | 0；24 passed | `9c87cbe` | CORE-002 worktree |
| `cargo fmt --all -- --check` | 0 | `9c87cbe` | CORE-002 worktree |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 0 | `9c87cbe` | CORE-002 worktree |
| `git diff 0dda451 -- crates/stagemaster-engine/tests/htp_default_fallback.rs` | 空；独立验收未改动 | `9c87cbe` | CORE-002 worktree |
| 合并后格式、工作区测试与 Clippy | 全部通过 | `81ed816` | 主 checkout |

独立验收未为实现修改。所有固定行为均有测试覆盖，无未执行验证。

## 投入与问题

- 请求次数、修复次数及原因：3 次真实模型请求；2 次显式 repair，依次修复非发散 `let-else` 和缺失 `Option` 类型标注
- 本地输入／输出 token、生成时间：累计 input 18,998、output 997、total 19,995；模型请求累计 100.804 秒
- 云端用量：unknown
- 主控做过的手工逻辑修改：模型额度耗尽后，将最终候选的一处 `expect` 改为 `unwrap_or(descriptor.default)` 以消除公共函数 panic 并通过 Clippy；其余实现逻辑来自 Qwen 候选
- 实现限制、接口偏离与未解决事项：本次仅解决 R02 的 A0 兼容规则，不宣称专业 HTP／LTP 合成体系已经完成；无接口偏离
- 是否需要 Astra 决定：否；未发现契约外或重大架构问题
