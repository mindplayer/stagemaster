# DEV-002 交付

- 状态：已集成
- 执行者、模型／运行时版本：Sol 实现工作器；真实资格任务使用 Qwen3.8-27B MLX 8-bit + MTP，mlx-vlm 0.7.0 / MLX 0.32.2
- 契约版本、基线提交、工作区／分支：工作器 v1（2026-09-11）；`b466bc6786eb7d25c3b9a1cfb69b41f7be602061`；`/Users/sunqi/projects/stagemaster-worktrees/dev-002`／`dev-002-local-worker`
- 结果提交／补丁哈希：初版 `f2189ba8534aa70ce561927137e6848179ec0bab`；加固结果 `f07493cebc73f1bf60847677385c2630c71b6ad5`
- 集成提交：`55098271a6c9e1cebb995b8a8dad354297a35157`

## 结果

在 `tools/local-worker/` 实现 Python 3.12 标准库候选改动 CLI，提供 `run`、`status`、
`cancel`、`repair`、`verify-source` 和显式 `recover`。v1 前台运行、无守护进程和队列，
用跨进程锁拒绝并发生成。任务要求干净的精确 Git acceptance commit；路径、UTF-8、
大小、符号链接、保护文件和身份在请求前校验。

模型只收到列出的文件与契约，只能返回 `create` 或精确 `replace` 文本操作。
全部操作先在内存验证，随后将候选文件、diff、来源／提案／候选哈希、原始响应、
usage、时间与分类错误保存到 `AI_ROOT/outputs/local-worker/<job_id>/`；不写任务
worktree、不执行模型命令或生成代码。配置示例和 Sol 接纳／修复流程见
`tools/local-worker/README.md`，当前主机配置在
`/Users/sunqi/ai/outputs/local-worker/config.json`。

真实资格任务在独立本地仓库运行：

- acceptance commit：`6b68bacb29b9f7171662c1a1d24c68eac703ada7`，基线测试因占位实现按预期失败
- job：`DEV-002-DMX-SPAN-REAL-001-759a8e050957`
- attempt 1：84.064 秒，input 2163 / output 1401 / total 3564 token；实现行为正确，但模型自写单测把合法的 511+2 误判为越界
- Sol 自动格式化后保存失败 diff 和诊断，恢复自己接纳的单文件改动；未修改保护验收
- attempt 2：74.914 秒，input 3812 / output 1401 / total 5213 token；修复后通过
- 资格结果提交：`672af4cf8c546df90cc8a14aa7edcd83c67d7e7e`

## 验证

| 命令／设备场景 | 退出码／结果 | 被测版本 | 证据位置 |
| --- | --- | --- | --- |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/local-worker/tests -v` | 0；18 passed | `f07493c` | 本交付会话记录；`tools/local-worker/tests/` |
| 路径、符号链接、保护优先、原子提案、过期来源、身份冲突、busy、截断、超时阻断、取消迟到结果、恢复、repair 上限 | 全部合成测试通过 | `f07493c` | 同上 |
| 资格基线 `cargo test --locked --offline` | 101；占位实现按预期失败 | `6b68bac` | `/Users/sunqi/ai/outputs/local-worker/qualification/dev-002-dmx-span` |
| 资格结果 `cargo fmt --all -- --check` | 0 | `672af4c` | 同上 |
| 资格结果 `cargo test --locked --offline` | 0；7 模型单测 + 4 独立验收通过 | `672af4c` | 同上 |
| 资格结果 `cargo clippy --all-targets --locked --offline -- -D warnings` | 0 | `672af4c` | 同上 |
| `cargo fmt --all -- --check` | 0 | `f07493c` | DEV-002 worktree |
| `cargo test --workspace --locked --offline` | 0；10 passed | `f07493c` | DEV-002 worktree |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 0 | `f07493c` | DEV-002 worktree |
| 合并后上述 Python 测试与三项 Rust 检查 | 全部通过 | `5509827` | 主 checkout |

独立验收未为实现修改。资格任务的模型自写单测错误通过一次显式 repair 修复；
格式化没有消耗模型请求。

## 投入与问题

- 请求次数、修复次数及原因：真实模型 2 次；1 次显式修复，原因是模型自写边界测试期望错误
- 本地输入／输出 token、生成时间、任务总时间：累计 input 5975、output 2802、total 8777 token；模型请求累计 158.978 秒；宿主审查和测试时间未统一计时
- 云端用量：unknown
- 主控做过的手工逻辑修改：工作器由 Sol 实现；资格模块未做手工逻辑修改，只做候选接纳、rustfmt、诊断与精确撤回
- 实现限制：当前只验证 macOS；候选路径限制不是 OS 沙箱；取消为协作式，当前非流式请求返回前保持 `cancel_requested`，迟到结果保存后丢弃，不杀共享 MLX 服务；API 超时／断开或进程丢失时后端标记 unknown 并阻止新生成，须人工核实后显式 recover
- 范围限制：不支持二进制、删除、重命名、权限变更、依赖安装、自动测试／接纳／合并、MCP、后台服务或持久队列
- 是否需要 Astra 决定：否；实现符合已收敛的工作器 v1 契约
