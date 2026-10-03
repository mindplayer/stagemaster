# 开发方法

更新：2026-10-03。依据 [DEV-ADR-002](decisions/DEV-ADR-002-astra-direct.md)，由当前 Astra 会话直接负责规划到交付的完整开发过程。Qwen 与 Sol 委派流程结束。
入口：[AGENTS.md](../../AGENTS.md)、[STATE.md](STATE.md)、[当前执行计划](execution-plan.md)及当前工单。项目文件遵循[目录规则](project-files.md)。

## 工作循环

1. 核对现状、已完成项和用户改动；明确本任务行为、修改范围、依赖、契约、验收和完成条件。
2. 需要改变公共 API、模块依赖、持久格式、数据所有权、时间或控制权语义时，先填写[架构变更说明](templates/architecture-change.md)，记录判断后再实现。
3. 当前会话直接实现和排错。缺陷先复现；契约验收按行为定义，不依附具体实现细节。
4. 工具执行格式化、适用测试及静态检查，审查完整差异及文件职责／规模，按根开发规则拆分新增的大文件；集成后验证准确版本。
5. 按[交付模板](templates/delivery.md)记录结果并维护 STATE；重要边界和阶段出口单独形成审查结论。

任务状态：`planned → ready → running → verifying → review（需要时）→ done`。`needs-decision`、`failed`、`cancelled` 单独标记；路线取消不能写成实现通过。任务仅在验收满足并完成集成后标 done。

## 工作区与记录

- 每个任务绑定基线和写入范围；单会话可使用干净主工作区，需要隔离时使用 `.worktrees/<task-id>/`，同一模块只有一个写入负责人。
- [任务模板](templates/task.md)记录精确接口与失败场景。跨工作区交接带分支、基线和结果／集成提交。
- 当前会话同时承担架构与实现责任，仍保留契约先行、差异审查和行为验收，不以模型名称替代验证。
- 不自动创建额外会话、启动子代理、切换模型、部署或操作真实设备。普通实现细节在授权范围内自主解决。
- 只按任务加载上下文；耗时和用量有真实数据才记录，不建设模型路由或统计平台。

## 当前衔接

G0 的 Git 基础和 CORE-001／002 产品修复保留。当前已进入平台框架的可运行边界验证；具体接续只读 STATE 和当前执行计划，不依据历史阶段说明重新启动 G1 或旧委派流程。
旧工作器、资格试验、两版旧执行计划和 Sol 提示词已从当前文件树清理。原受版本管理的内容可在清理前提交 `121efa311855d977364f7ad8707ea729b5c0e367` 查阅；未纳入 Git 的旧试验原始文件已删除，没有另做副本。

## 可选能力验证

默认构建不能代替可选模块验证。涉及[共同节目包交付](../module-api/package-delivery.md)时，在项目内 CARGO_HOME／TMPDIR 环境下追加：

```sh
cargo test -p stagemaster-delivery --features http --locked --offline
cargo clippy --workspace --all-targets --features stagemaster-delivery/http --locked --offline -- -D warnings
```

HTTP 仅在需要网络交付的产品组装启用；默认桌面／设备上传不因此增加 TLS 生产依赖。新依赖先单独获取，后续验证保持锁定、离线；本机 HTTPS 测试生成短期证书，不修改系统信任，也不访问真实云服务。
