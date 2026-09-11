# DEV-003 / G0 阶段交付包

日期：2026-09-11。状态：已集成，供 Astra 审查。执行基线：
`c8c834a416b83923df40ea2dd39079d05e5676b2`；状态更新结果：
`4c401d16d9d95406ebeca71fdc9bf67f7b0c23d7`；交付包提交：
`271d2538bb02bc00bd36b20dd8cafbfa4cabc9bc`；集成提交：
`e5ea78f2667ada991427c63b8d9fc74c3c5f9e39`。

## 结果与版本

G0 五项工单均已实施：DEV-001 建立本地 Git、独立 worktree 与 Rust 验证基线；
DEV-002 在 `tools/local-worker/` 交付 Python 3.12 标准库候选工作器；CORE-001
修复 R01 Cue 替换编号碰撞；CORE-002 修复 R02 HTP default 下限；DEV-003
更新实现状态并形成本文。产品仍是 A0 静态语义原型，不扩大为真实控台能力声明。

| 样本 | 版本与结果 | Qwen 实测 | 失败、接管与验收 |
| --- | --- | --- | --- |
| DEV-002 DMX span 新模块 | acceptance `6b68bac`；result `672af4c`；工作器集成 `5509827` | 2 请求；in 5,975 / out 2,802 / total 8,777；158.978 秒 | 首次实现逻辑正确但模型单测边界错误；repair 后 7 模型单测 + 4 独立验收、fmt、Clippy 通过；Sol 未改模块逻辑 |
| CORE-001 已有 show 代码 | acceptance `49ddead`；result `22c24e2`；集成 `df64f98` | 1 请求；in 6,811 / out 478 / total 7,289；40.350 秒 | 首次候选通过；4 已有 + 6 独立验收通过；Sol 未改实现逻辑 |
| CORE-002 已有 engine 代码 | acceptance `0dda451`；result `9c87cbe`；集成 `81ed816` | 3 请求；in 18,998 / out 997 / total 19,995；100.804 秒 | 两次 repair 处理 Rust 编译错误；最终候选测试通过但 Clippy 拒绝 `expect`，额度耗尽后 Sol 改一行为无 panic fallback；3 已有 + 8 独立验收通过 |

新工作器三项真实任务合计 6 次请求（3 次 repair），input 31,784、output 4,277、
total 36,061 token，模型请求累计 300.132 秒。云端用量和统一端到端人工时间不可得，
不推算节省比例。真实任务没有取消；取消迟到结果、busy、超时 unknown 与显式恢复
只在 DEV-002 的 18 项合成故障测试中验证。完整失败证据见各工单
[DEV-002](DEV-002-delivery.md)、[CORE-001](CORE-001-delivery.md)和
[CORE-002](CORE-002-delivery.md)。

历史临时试验单列：Qwen 首次请求 105.3 秒，input 756 / output 1,849；6 项模型
单测和 4 项独立验收首次通过。格式差异曾错误触发重试，随后取消并由 rustfmt 处理。
它未使用 v1 工作器，不计入上述汇总。

## 验证与边界

- 工作器支持 `run/status/cancel/repair/verify-source/recover`；候选不写 worktree、
  不执行模型命令、要求干净的精确 acceptance commit，并保存来源、提案、候选哈希、
  diff、响应、usage、时间和错误。无 MCP、守护进程、队列、自动测试或自动合并。
- `tools/local-worker` 18 项合成故障测试通过；两项产品保护回归均先红后绿且未为实现
  修改。CORE-002 集成版本的 `cargo fmt --all -- --check`、离线 workspace 24 项测试
  和严格 Clippy 全部通过；G0 最终集成后再重复同组检查。
- 所有验证均为本机软件测试。未连接灯具、DMX、控台或其他真实设备；未验证持续
  输出、实时性、跨平台和完整产品能力。R01、R02 已解决；R03–R09 仍明确保留。
- 最终集成版本 `e5ea78f` 再次通过工作器 18 项测试、Rust fmt、离线 workspace
  24 项测试和严格 Clippy。

## 路由结论与 Astra 审查点

- Qwen 适合契约固定、上下文小、写路径少、可由独立测试判定的纯逻辑模块或局部编辑；
  候选必须继续由 Sol 审查和宿主验证。
- Sol 应直接处理确定性工具步骤、零碎一行修正、验收与集成，以及模型 repair 成本
  已接近直接实现成本的任务；CORE-002 不能计为 Qwen 独立完成。
- 公共契约、持久格式、时间／控制权、专业推杆语义和跨模块所有权仍交 Astra 决定。
  三项样本不足以证明长期可靠性，后续只随真实任务累计约 10 个样本。

本批未发现需要额外架构裁决的契约偏离。请 Astra 审查 G0 是否通过，并据 R03–R09
的依赖关系细化 G1 工单；本会话不自行展开下一阶段重构。
