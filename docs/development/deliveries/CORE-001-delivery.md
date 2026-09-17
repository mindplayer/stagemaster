# CORE-001 交付

> 已完成任务的历史记录：执行者、工作区和验证结果保留作追溯；不作为当前模型分工或重做任务的指令。当前规则见 [开发方法](../README.md)。

- 状态：已集成
- 执行者、模型／运行时版本：Qwen3.8-27B MLX 8-bit + MTP；mlx-vlm 0.7.0 / MLX 0.32.2；Sol 负责保护验收、候选审查与集成
- 契约版本、基线提交、工作区／分支：工单固定契约；`d24751c915cd9c3578a03ea964c9651b49cd778d`；`/Users/sunqi/projects/stagemaster-worktrees/core-001`／`core-001-cue-uniqueness`
- 验收提交：`49ddeadeb88eaf6448b1bf954298027157b915d3`
- 结果提交／补丁哈希：`22c24e24ab69bd26a7a9b2aadfb9602374c29abf`；提案 `a3720f0ee63410f3e6d17edfb7e5bfe4fac0a64da73ddda7034cd7d8484eeab2`
- 集成提交：`df64f98603ca28462cf76a515b65fb39dda9b26d`

## 结果

`Sequence::upsert_cue` 现在在任何修改前统一检查候选 Cue 编号是否被另一稳定 ID
占用。新增和替换碰撞都返回现有 `DuplicateCueNumber`，失败不改变序列；同 ID
沿用自身编号或移动到空闲编号仍成功并按既有规则排序。公共接口、持久化格式和
Tracking 语义均未改变。

Qwen job `CORE-001-QWEN-001-19aa51f9994f` 只生成
`crates/stagemaster-show/src/lib.rs` 的候选；保护验收未授权写入且未发生变化。

## 验证

| 命令／设备场景 | 退出码／结果 | 被测版本 | 证据位置 |
| --- | --- | --- | --- |
| `cargo test -p stagemaster-show --test cue_number_uniqueness --locked --offline` | 101；替换为另一 Cue 编号的用例按预期失败，其余 5 项通过 | `49ddead` | 交付会话记录；保护验收文件 |
| `cargo test -p stagemaster-show --locked --offline` | 0；4 项已有单测 + 6 项独立验收通过 | `22c24e2` | CORE-001 worktree |
| `cargo test --workspace --locked --offline` | 0；16 passed | `22c24e2` | CORE-001 worktree |
| `cargo fmt --all -- --check` | 0 | `22c24e2` | CORE-001 worktree |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 0 | `22c24e2` | CORE-001 worktree |
| `git diff 49ddead -- crates/stagemaster-show/tests/cue_number_uniqueness.rs` | 空；独立验收未改动 | `22c24e2` | CORE-001 worktree |
| 合并后格式、工作区测试与 Clippy | 全部通过 | `df64f98` | 主 checkout |

独立验收未为实现修改。所有固定行为均有测试覆盖，无未执行验证。

## 投入与问题

- 请求次数、修复次数及原因：1 次真实模型请求；0 次修复
- 本地输入／输出 token、生成时间：input 6811、output 478、total 7289；40.350 秒
- 云端用量：unknown
- 主控做过的手工逻辑修改：无；Sol 仅编写保护验收、审查并接纳候选、运行格式化和验证
- 实现限制、接口偏离与未解决事项：无；改动仅限既有方法内部和必要注释
- 是否需要 Astra 决定：否；未发现契约外或重大架构问题
