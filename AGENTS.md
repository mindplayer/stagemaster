# StageMaster 开发协作规则

## 最小上下文

先读 `docs/development/STATE.md` 和当前工单，再按需读模块契约。首次了解产品时读 `README.md`。不要每次重新加载全套控台研究或重复技术选型。

当前首批执行依据为 `docs/development/execution-plan-v1.md`；DEV-002 以 `docs/development/local-worker-contract-v1.md` 的候选改动 CLI 为准，不扩展为完整代理平台。CORE-001／002 的修复契约已由 Astra 确定。先核对已完成项，从下一项继续。

## 会话分工

- 用户指定的 Astra 会话负责架构、契约、开发顺序和阶段审查，以规划和审查为主。
- 用户另开的 GPT-5.6 Sol 会话负责日常开发、测试、集成及本地 Qwen 的任务分配。
- 按已授权工单，可以调用本地 Qwen 完成边界明确的实现。模型输出是候选变更，须由工具验证、Sol 审查。
- 此分工不授权自动创建新 Codex 会话、修改用户的模型设置或启动更多云端子代理。

## 实施边界

- Rust 维护核心语义；TypeScript 维护界面和云端业务。模块显式依赖，核心不绑定 UI、具体硬件或云端。
- 实现遵循工单限定的文件和接口；公共契约、持久化格式、时间／控制权语义变更先形成架构变更说明，由 Astra 会话评估。Sol 可自行决定契约内的实现细节。
- 每个任务绑定基线版本和独立工作区；同一模块同时只有一个写入负责人。worktree 的文件不会自动同步到另一会话。
- 格式化、编译和测试由工具执行；格式差异自动修正，不作为模型重试理由。独立验收不得为迁就实现而削弱。
- 本地推理初期只运行一个任务；默认最多初次实现加两次修复，超限由 Sol 接管判断。不要让模型互相无限转交。
- 小改动可由 Sol 直接完成；确定性操作交给工具，不为委派而委派。
- 不自动操作真实灯具、刷机或部署。按用户已有授权处理相应任务，不因内部流程重复索要确认。

## 验证与交付

Rust 常用命令：`cargo fmt --all`、`cargo fmt --all -- --check`、`cargo test --workspace --locked --offline`、`cargo clippy --workspace --all-targets --locked --offline -- -D warnings`。
缺失依赖单独处理并记录，不修改命令来掩盖已有失败。非代码变更只做相关检查。

交付必须包含工单 ID、基线／结果版本、变更摘要、实际验证结果和未解决项。Sol 维护 `docs/development/STATE.md`；重大决定写入独立决策文档。方法详见 `docs/development/README.md`。
