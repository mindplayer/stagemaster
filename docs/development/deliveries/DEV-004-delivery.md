# DEV-004 交付

- 状态：已集成，供 Astra 复审
- 执行者：Sol 直接实现；Python 3.12.13 标准库；未调用 Qwen
- 契约、基线、工作区／分支：DEV-004 固定补修契约；`c296345a62c9c6af5c51a88c53b8bb136a7b4497`；`/Users/sunqi/projects/stagemaster-worktrees/dev-004`／`dev-004-worker-correctness`
- 验收提交：`c726818b35496c7b7ca21459b3918c83ad42f37b`
- 结果提交：实现 `83ecb1b173eba3d3f759239d0e95b0499ce18ac3`；清理／取消证据加固 `d87e7bd3df1565055419b39e3c39b300722cbed9`
- 集成提交：`b281532cee641549ab9bb66070ceb52c84aa34a1`

## 结果

| 审查项 | 修复与正式证据 |
| --- | --- |
| G0-R01 原子取消与发布 | 每个 job 的状态锁同时仲裁 cancel 和候选发布；取消在模型前、请求中、候选构建中均阻止候选资格，终态后 cancel 不改标记。repair 在生成锁内重新核对状态、attempt 和剩余预算；同进程线程锁与跨进程 `flock` 共同保护记录。事件屏障测试覆盖请求中、构建边界、终态后和竞争 repair。 |
| G0-R02 准备失败收尾 | job 初始 backend 为 finished；attempt 在构建上下文前登记。上下文超限、payload／产物准备失败写入 failed manifest、清除 owner 和当前候选资格，不谎报推理；repair 失败保留旧候选文件作为历史证据，下一合法任务不受阻。 |
| G0-R03 完整 HTTP deadline | `http.client` 请求由单调绝对 deadline 覆盖连接、响应头及成功／错误体；上限取单次配置和任务剩余预算的较小值。watchdog 到点关闭同一连接并被 join，不创建后台推理线程、不杀共享服务。回环测试覆盖滴流成功体、滴流错误体、慢响应头和剩余 0.25 秒预算。 |
| G0-R04 原始失败证据 | 收到的完整／部分字节先按上限落盘，再解析 UTF-8、JSON 和提案；非法 JSON、非对象 JSON、无效编码、HTTP 错误、超长和超时均记录真实耗时、hash、路径、incomplete 与可得 usage。超长只保存上限内字节并保持 backend unknown。 |

工作器版本升为 `0.1.1`。Astra 复现脚本只将已重构的传输 mock 接入点从
`urlopen` 改为 `_perform_http`；四项行为断言未削弱。未修改 Rust 产品代码、
CORE-001／002 验收、provider、模型或网关配置。

## 验证

| 命令／场景 | 退出码／结果 | 被测版本 |
| --- | --- | --- |
| 基线 `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/reviews/g0_review_regressions.py` | 1；4/4 按预期失败 | `c296345` |
| 正式回归首次加入后的完整 discovery | 1；28 项中 7 failed、2 errors，复现旧缺口 | `c726818` |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/local-worker/tests -v` | 0；33 passed | `d87e7bd` |
| Astra 四项复现脚本 | 0；4 passed；0.2 秒滴流场景观测 0.268 秒返回 | `83ecb1b`（实现与 `d87e7bd` 相同） |
| `python3 tools/local-worker/worker.py --version` 及 `--help` | 0；`0.1.1` | `83ecb1b` |
| `cargo fmt --all -- --check` | 0 | `83ecb1b` |
| `cargo test --workspace --locked --offline` | 0；24 passed | `83ecb1b` |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 0 | `83ecb1b` |
| 合并后正式 33 项、Astra 4 项、Rust fmt／24 项测试／严格 Clippy | 全部通过；0.2 秒滴流场景观测 0.256 秒返回 | `b281532` |

一次直接用文件路径选择局部 unittest 的命令因测试目录未进入模块搜索路径而报
`support` 导入错误，未执行测试、未改变源码；随即按项目规定的 discovery 入口
重跑 33 项并通过。最终集成版本已再次执行同组正式测试、Astra 复现和 Rust 检查。

## 未解决项

- 超时／断开／超长响应仍按契约保守标记 backend unknown，须确认后显式 recover；候选路径限制仍不是 OS 沙箱。
- 当前验证为 macOS 本机合成与回环 HTTP，没有重跑真实 Qwen 资格样本，也没有设备验证；DEV-004 不改变历史模型统计。
- 未发现契约偏离或需要 Astra 新决定的重大架构问题。G0 是否放行由 Astra 复审；本会话不进入 G1。
