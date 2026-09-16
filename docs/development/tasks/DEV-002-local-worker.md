# DEV-002：本地候选改动工作器 v1

> 当前状态：retired，2026-09-11 依据 [DEV-ADR-001](../decisions/DEV-ADR-001-sol-astra.md)停止使用与维护。下文保留原交付历史，不再执行。

状态：done。负责人：Sol。依赖：DEV-001（已完成）。

执行基线：`b466bc6786eb7d25c3b9a1cfb69b41f7be602061`。
工作区／分支：`/Users/sunqi/projects/stagemaster-worktrees/dev-002`／`dev-002-local-worker`。
修改范围：`tools/local-worker/**`、本工单、`docs/development/STATE.md`、
`docs/development/deliveries/DEV-002-delivery.md`与必要的工作器使用说明。

本工单按 Astra 的[执行计划 v1](../execution-plan-v1.md)和[工作器 v1 契约](../local-worker-contract-v1.md)实施。
2026-09-11 修订：将原先包含通用执行、持久队列和 MCP 的大范围目标缩小为候选文本改动 CLI。以 v1 契约为准。

## 目标与范围

实现 Sol 可调用的 run/status/cancel/repair 入口。工作器只提供经校验的候选文件与差异；Sol 负责接纳、运行测试和集成。

源码仅放 `tools/local-worker/`，开发文档放本目录；不成为 StageMaster 产品运行依赖。
历史产物已封存到本项目 `data/development/legacy-qwen/local-worker/`，不纳入主项目 Git；旧配置只是证据，不得按其路径写回共享 AI 目录。见[迁移记录](../project-files.md)。
使用现有 Python 3.12 标准库，不安装新的代理框架或模型。

## 实施顺序

1. 定义 JSON 工单、候选操作、错误和持久记录，写一个最小真实示例。
2. 先实现并测试纯校验模块：路径、保护文件、基线、精确文本替换、新文件、来源哈希和原子提案。
3. 实现本地模型适配、输出解析、独占生成、超时和可查询状态；不执行模型命令。
4. 接入取消和异常恢复。保持“取消候选”与“后端推理已结束”的区别，不终止其他使用者的共享模型服务。
5. 用合成响应覆盖失败，再沿用原 DMX span 行为做一次隔离真实模型任务。测试期望不改变，也不直接导入产品。
6. 写明 Sol 如何在任务 worktree 中验证来源、接纳候选、格式化、执行独立验收和反馈诊断。

## 完成门槛

v1 契约中的路径、提案、身份、busy、过期来源、输出截断、有限修复、取消迟到结果和异常恢复测试有实际证据。
真实模型生成、宿主独立验收至少一次通过。给出可运行命令与已知限制，不把模拟测试当成真实接入。
CLI 可靠即可继续 CORE-001／002；本批不增加 MCP、后台服务、持久队列或自动执行生成代码。

临时试验参考：`/Users/sunqi/projects/stagemaster/data/development/legacy-qwen/trial-20260911/run_trial.py`，仅复用已验证的接口经验。
不能把该脚本复制后改几个路径就宣称完成本工单。

实现结果：`f07493cebc73f1bf60847677385c2630c71b6ad5`；真实资格结果：
`672af4cf8c546df90cc8a14aa7edcd83c67d7e7e`。完整证据见
[DEV-002 交付](../deliveries/DEV-002-delivery.md)。
主分支集成：`55098271a6c9e1cebb995b8a8dad354297a35157`。
