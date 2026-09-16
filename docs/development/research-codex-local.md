# Codex 配合本地模型：路线研究

> 历史研究：2026-09-11 用户已取消本地 Qwen 开发路线。当前采用 [Sol 直接开发、Astra 高级调度](decisions/DEV-ADR-001-sol-astra.md)，不按本文继续接入或资格验证。

检索及核查日期：2026-09-11。结论依据官方文档、项目维护者文档和本机试验。
下列是公开资料里可观察到的做法，不是有统计依据的流行度排名。
未安装或执行候选插件，未验证其全部功能；上游自述与本机实测分开记录。

## 路线比较

| 路线 | 已有依据 | 对本项目的判断 |
| --- | --- | --- |
| Codex 直接使用本地 provider | Codex 支持自定义 provider；Ollama 提供 Codex 接入说明 | 适合独立的本地代理实验；本次不替换 Astra／Sol 主会话。MLX 基础 Responses 请求通过，不等于完整 Codex 工具循环兼容。 |
| Codex 云端主控 + 本地工作器（CLI／MCP） | 官方 MCP 扩展；多个社区项目实现有限委派 | **采用这种边界。** 保留云端判断，工作器和模型运行时可替换。先命令行可靠运行，再加 MCP 便利入口。 |
| Aider Architect / Editor | 正式文档明确两阶段模型分工，支持 OpenAI-compatible endpoint | 吸收规划与编辑分开的方法；可作为编辑适配器备选，暂不增加第二套主会话。 |
| 通过 MCP 包装 pi 等完整代理 | pi-delegate-mcp 提供后台任务与 steering，并隔离子任务上下文 | 能力更广，但新增代理和权限管理层；先不用于本项目写代码的默认路径。 |
| MCP 包装多个 codex exec | codex-specialized-subagents 保存独立运行产物并包装 Codex 子进程 | 更接近多个 Codex 实例的编排；本身不证明已经能稳定驱动本机 MLX Qwen。 |
| 自动换模型／代理转发整个会话 | 社区路由类方案 | 不作为长期基础。端点兼容、主会话登录、工具支持与升级耦合较多，不能仅根据“模型列表出现”判断可用。 |

## 官方边界

Codex 的当前 provider 配置中，`wire_api` 只支持 `responses`。
机器级 provider 配置不能通过项目 `.codex/config.toml` 覆盖；需采用适当的用户级配置。
因此旧教程里的 Chat Completions 配置不能直接照搬。
[OpenAI 配置参考](https://learn.chatgpt.com/docs/config-file/config-reference)

Codex 支持本机 STDIO 和 Streamable HTTP MCP；同一 host 的桌面、CLI、IDE 可共享相关配置。
MCP 是工具入口，不能自动保证背后的模型、文件操作或执行器正确。
[OpenAI MCP 文档](https://learn.chatgpt.com/docs/extend/mcp)

AGENTS.md 可保存项目工作规则，worktree 可分离多个任务的文件修改。
worktree 依赖 Git，并不自动同步各份文件，也不等于执行权限沙箱。
[OpenAI AGENTS.md 文档](https://learn.chatgpt.com/docs/agent-configuration/agents-md)、
[OpenAI worktree 文档](https://learn.chatgpt.com/docs/environments/git-worktrees)

Ollama 的官方 Codex 教程给出了本地接入方式，并建议较大的上下文窗口。
这是该路线的文档建议，不是我们已对 Qwen 完成 64k 上下文测试的证据。
本机已有 MLX，无需仅为跟随教程再装 Ollama。
[Ollama Codex 文档](https://docs.ollama.com/integrations/codex)

## 可借鉴的项目

### Aider

Architect 先提出解决方案，Editor 负责生成具体编辑。其 OpenAI-compatible 接口提供了本地运行时接入路径。
这支持“两种职责分开”的方法，但历史组合的榜单不能用来保证 Astra／Sol／本机 Qwen 的效果。
[模式文档](https://aider.chat/docs/usage/modes.html)、[端点文档](https://aider.chat/docs/llms/openai-compat.html)

### Local LLM Delegator

README 描述 `propose_patch`／`apply_patch`、补丁哈希、来源过期检查、限定路径和有限修复。
这些与我们的任务边界很接近。其提案状态仅存内存且会过期，不能直接替代几个月项目的持久工单记录。
GitHub API 核查：2026-07-29 创建，核查时 0 star，MIT；不能据此称为广泛采用或经过长期验证。
适合作为 DEV-002 的候选参考，需要审查源码及运行验收后才考虑采用。
[项目](https://github.com/tushrv/local-llm-delegator)、[项目元数据](https://api.github.com/repos/tushrv/local-llm-delegator)

### pi-delegate-mcp

文档提供可引导的后台委派，默认工具限制为只读，写入需显式开启。
它使用 pi 代理，不是简单调用任意 CLI。GitHub API 核查：2026-08-25 创建，16 star，MIT。
任务状态、上下文隔离可参考；目前未在本机验证其写入和模型配对。
[项目](https://github.com/howznguyen/pi-delegate-mcp)、[项目元数据](https://api.github.com/repos/howznguyen/pi-delegate-mcp)

### codex-specialized-subagents

文档通过 MCP 包装 `codex exec`，把提示、事件和结果保存在运行目录中。
GitHub API 核查：2025-12-29 创建，最近 push 为 2025-12-31，68 star，MIT。
其产物留存方式值得借鉴；必须检查当前 CLI 兼容性，不能把示例里的云端 Codex 调用等同于本地 Qwen。
[项目](https://github.com/leonardsellem/codex-specialized-subagents)、[项目元数据](https://api.github.com/repos/leonardsellem/codex-specialized-subagents)

### 社区 Third-Party Workers

OpenAI 仓库的 Show and tell 中也有“主代理保留，第三方模型负责有限任务”的公开案例。
作者明确它是非官方 beta，其 Qwen 证据涉及托管模型；不是本机 27B 的兼容性证明。
出现在 OpenAI 仓库讨论区也不代表 OpenAI 背书。
[原始讨论](https://github.com/openai/codex/discussions/38119)

## 本机证据与决定

- 已安装 Qwen3.8-27B MLX 8-bit + MTP；两组短生成实测约 16～19.6 token/s。
- 一个独立 Rust 小模块首次请求耗时 105.3 秒，输入 756、输出 1849 token；6 项模型测试和 4 项独立验收通过。
- 因格式差异触发过一次无必要重试，已取消并调整执行器：格式问题交给格式化工具。
- 未验证复杂多文件工作、完整 Codex 本地 provider 循环、长期记忆或自动合并。

**选择有限委派方法，先稳定交付契约，保留执行器替换能力。**
当前没有证据支持“装一个热门插件就能数月稳定省费”的承诺。
后续用真实任务评估：首次功能通过率、最终通过率、越界次数、重试原因、总耗时、Sol 审查投入及实际云端用量。
不通过修改测试或压缩必要的审查来追求漂亮数字。
