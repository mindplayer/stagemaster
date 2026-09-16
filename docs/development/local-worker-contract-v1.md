# 本地工作器 v1：首批实现契约

> 已停用：2026-09-11 根据 [DEV-ADR-001](decisions/DEV-ADR-001-sol-astra.md)退出开发路线。下文是历史契约，不是当前施工单；工作器残留问题未修复。

2026-09-11，Astra 选定。适用于 DEV-002；尚未实现。
它把旧工单中的完整执行器目标收敛为候选改动工具。本文与旧描述不一致时，以本文及执行计划 v1 为准。

## 模块与存放位置

源码：`tools/local-worker/`；用已有 Python 3.12 和标准库，至少分开任务／策略校验、模型端点适配、候选改动校验、运行记录、CLI。
具体文件名由 Sol 决定，不为每项职责建立复杂框架。
历史产物已迁到本项目 `data/development/legacy-qwen/local-worker/<job_id>/`，见[位置映射](project-files.md)。原配置和任务中的旧根路径仅作证据，不得重建外部输出目录。
仅当前 macOS 验收，不能声称已经测试 Windows／Linux。

核心函数的建议形式：

```text
validate_task(task, workspace_snapshot) -> ValidatedTask | TaskError
build_context(validated_task) -> ModelRequest
generate(request, cancellation) -> ModelResult | GenerationError
validate_proposal(model_result, snapshot) -> Candidate | ProposalError
write_report(job, candidate_or_error) -> ResultManifest
```

模型输出不进入 shell，不导入或执行模型生成的 Python／Rust。模型提供代码文本，Sol 负责接纳和执行验证。

## CLI 行为

建议命令（最终路径由交付文档给出）：

```text
python3 tools/local-worker/worker.py run --task <task.json>
python3 tools/local-worker/worker.py status --job <job_id>
python3 tools/local-worker/worker.py cancel --job <job_id>
python3 tools/local-worker/worker.py repair --job <job_id> --diagnostics <file>
```

`run` 可以保持前台运行，启动后立即打印 job_id；Codex 使用已有进程等待能力，不另写后台服务。
`status` 只读持久记录；`cancel` 设置取消请求；`repair` 使用原任务和明确诊断生成新 attempt。
v1 **没有自动队列**：用跨进程独占锁保护整个本地生成周期，另一任务返回 `busy`，由 Sol 排队。
同 task_id、同基线、同契约和输入摘要重复提交返回已有记录；内容变化但复用身份时明确报冲突。

## 工单数据

采用 JSON，字段名称固定；Sol 写一个真实示例和校验器。默认未知字段拒绝，以发现拼写和版本错误。

| 字段 | 含义 |
| --- | --- |
| schema_version | 固定整数 1 |
| task_id / contract_revision | 工单身份与契约版本 |
| workspace_root / base_commit | 已存在任务 worktree 的绝对根路径、完整基线提交 |
| read_paths / write_paths / protected_paths | 精确相对文件路径列表；write_paths 中已有文件必须可读取；protected 优先 |
| goal / contract / acceptance | 行为、错误、不变量和验收条件文本；不夹带 shell 指令 |
| model_profile | 宿主配置中的名称，不允许模型修改端点 |
| max_output_tokens / max_attempts | 初始 4096／3，宿主上限校验 |
| request_timeout_seconds / total_timeout_seconds | 初始 360／900 |

任务的验证命令由 Sol 单独保存和执行，不由工作器解释。宿主配置解析 `127.0.0.1:18100/v1` 与 `default_model`；初版只允许明确配置的本地端点。
路径列表初始上限各 6 个文件；整体上下文设置可配置的字节上限，超限返回错误而非静默省略。约 8k 输入 token 是规划目标，字节数不能冒充精确 token 数。

仅接收 UTF-8 普通文件；拒绝绝对候选路径、父目录穿越、符号链接及解析后越出根目录的路径。
拒绝目标文件及其父目录为符号链接，避免通过链接访问其他工作区。
首版不支持二进制、删除、重命名、权限变更、依赖安装和动态模型端点；需要这些操作交给 Sol。
任务工作区干净与否须显式检查。Sol 准备的验收应先提交并记录，以便受保护文件有可复现基线。
JSON 中的 `base_commit` 指包含独立验收的 acceptance commit；交付记录另记增加验收前的产品基线。开始生成时要求工作区与该提交一致。修复任务的独立验收在这一提交上预期失败，不能把这个已证明的目标缺陷误判为无关的环境失败。

## 模型输出与候选改动

使用已验证的 Chat Completions function calling。工具名称由 Sol 固定，但只收集提案，不实际授予模型文件写入工具。

提案可包含两种文本操作：

- `create`：允许新增路径与完整内容；文件已存在则拒绝。
- `replace`：允许的已有路径、精确匹配文本和替换文本。匹配文本不能为空，必须在当前候选版本中恰好出现一次。

可对同一文件顺序应用多个操作，在内存或私有候选目录计算最终内容；任一步无效就拒绝整份提案。
用这种方式减少每次重发整份已有源码的输出成本，不要求模型准确生成行号。
校验完毕生成最终文件、可读 diff 和哈希，保存在产物目录；v1 不写任务 worktree 或主 checkout。

记录每个来源文件的 SHA256、基线提交、契约版本、提案哈希及候选内容哈希。
Sol 接纳前重新检查来源与保护文件；基线过期则拒绝或重新生成，不能盲目应用。接纳后的真实 diff 也必须仍在工单范围内。

模型响应异常、JSON 解析失败、多余工具、非法操作、超范围、空改动及因长度上限截断应分类记录。
不得通过修补明显截断的代码尾部、忽略非法操作或自动扩大允许路径来“完成任务”。

## 状态、取消与恢复

状态最小集合：`running`、`candidate_ready`、`failed`、`cancel_requested`、`cancelled`、`interrupted`。
`candidate_ready` 只代表候选文本校验通过，不代表编译或产品验收通过。
通过临时文件加替换原子写入状态；产物以 attempt 编号分开保存，避免重试覆盖证据。

**取消承诺：取消请求被接受后，后续模型输出不得成为可接纳候选。**
后端停止推理另行记录：`finished`／`still_running`／`unknown`，不以 HTTP 客户端断开冒充 GPU 已停。
最小实现允许等待当前有界请求结束后丢弃输出并释放锁；此时才标 cancelled。
不因为取消一个任务就终止共享模型服务器。

超时或进程异常后，如果无法确认后端空闲，标记 `backend_state=unknown` 并阻止工作器再次生成。
Sol 根据已验证的运行时状态恢复；不得清除标记后假定安全。重启检测 owner PID 的身份及持久记录，PID 存在本身不代表仍是原工作器。
恢复是显式状态处理，不自动重放之前的请求或文件操作。

## 修复、报告和执行边界

Sol 自动格式化候选并运行受信任的编译／测试命令；把有限诊断交给 `repair`。
修复保持原契约与写入范围，最多两次；需要改变契约时创建新工单修订。
每次修复仍以原 `base_commit` 为来源，输出相对于该来源的新完整候选，不在未知的脏工作区上叠加替换。Sol 保存失败候选和格式化后的 diff，再仅撤回自己接纳的任务改动，核对来源及保护文件后调用 `repair`；不得使用清空整个工作区的方式。模型读取原始来源、上一候选差异和本次有限诊断，仍受上下文总量限制；必要时返回需要拆分，不能静默截断。
`repair` 必须显式触发，不自动用满三次机会。900 秒总时限累计本任务各次工作器执行耗时，不包括 Sol 阅读、接纳和测试之间的等待；单次请求始终受 360 秒限制。超过上限不开始新请求，运行中的请求受剩余预算约束。
没有执行权限沙箱的环境必须如实记录。候选路径限制是该工具的能力，不等于对任意生成代码的 OS 隔离。

结果包含：任务与基线、attempt、状态、提案／候选哈希、读取与变更路径、实际模型 ID／运行时版本、时间、上游返回的 usage、错误和未解决项。
没有取得的 usage 写 null。主控实际验证、接纳和集成结果写入独立交付记录，不由模型宣称。

## 验收重点

解析和路径规则、候选全有或全无、保护文件不变、重复提交、busy、截断响应、超时、取消后迟到结果、异常恢复、两次修复上限，应优先用合成响应验证。
完成后，沿用原独立 DMX span 试验的明确行为做一次真实生成与宿主验收，不将其直接加入产品。然后由 CORE-001／002 验证已有代码编辑。
CLI 实现可靠即满足本阶段工具入口要求；MCP、持久任务队列及自动执行生成代码留到需要时另立工单。
