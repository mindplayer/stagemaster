# 项目文件归属与迁移记录

2026-09-11，按用户要求：StageMaster 的软件工程及开发产物统一放在当前项目目录 `/Users/sunqi/projects/stagemaster`。
之前把项目试验和任务证据放在共享 AI 目录不符合项目归属，现已迁回。

## 当前目录规则

| 内容 | 项目内位置 | Git |
| --- | --- | --- |
| 产品源码、测试、工程清单 | `crates/`、`apps/` 及根工程文件 | 纳入版本 |
| 项目脚本、工具和配置模板 | `tools/`；部署接线按需放 `infra/` | 可复现且无秘密的文件纳入版本 |
| 需求、架构、工单、交付与迁移索引 | `docs/` | 纳入版本 |
| 持久开发产物及原始试验证据 | `data/development/` | 本机保留，忽略原始大文件与状态 |
| 日志与临时文件 | `logs/`、`tmp/` | 忽略；项目命令需要临时根时显式指定 `tmp/` |
| 任务 worktree | `.worktrees/<task-id>/` | 子目录忽略，由 Git worktree 管理 |
| 当前 Cargo 编译产物 | `target/` | 忽略 |

不为尚未使用的功能建立空基础设施。外部安装的工具链与共享模型属于环境依赖，不能因此把项目源码、配置、测试或输出也放到共享目录。
日常仍由 Sol 直接开发，Astra 调度；迁移不恢复 Qwen／工作器路线。

## 本次迁移

| 原位置 | 当前项目内位置 | 普通文件 | 字节数 |
| --- | --- | --- | --- |
| `/Users/sunqi/ai/outputs/local-worker` | [data/development/legacy-qwen/local-worker](../../data/development/legacy-qwen/local-worker) | 460 | 17,294,541 |
| `/Users/sunqi/ai/tmp/stagemaster-qwen-trial-20260911` | [data/development/legacy-qwen/trial-20260911](../../data/development/legacy-qwen/trial-20260911) | 384 | 19,834,226 |

共 844 个普通文件、37,128,767 字节（约 35.4 MiB）。使用同一文件系统内的目录移动，原目录已不存在；没有在原位置保留副本或符号链接。
迁移前后逐文件 SHA-256、大小、权限及目录结构一致；两棵目录内没有符号链接。独立资格工程的 `.git` 与原构建产物一起保留。
迁移后独立资格仓库工作区干净，`git fsck --full` 通过；项目文档链接和配置示例 JSON 检查通过。旧的项目外 worktree 父目录已确认为空并移除。

完整清单：[migration-20260911.json](../../data/development/migration-20260911.json)。该文件 SHA-256：
`57537d075be085007834b178c7a60fe6d245e0ed32a54823d2b75efbb4897399`。

清单与原始产物位于项目 `data/`，不进入普通 Git 提交；需要完整归档本项目时须同时保留 `data/development/`。此文件是纳入版本的短索引。

## 历史路径与配置

迁移保持原始响应、任务 JSON、哈希、日志、试验源码和原配置字节不变，因此封存文件内仍可能包含原绝对路径或已清理的 worktree 路径。查证时按上表替换根路径，不修改历史内容冒充新的运行记录。
封存的 `config.json`、`run_trial.py` 和旧命令仅作历史证据，不是当前运行入口；不要运行它们或按其中的旧路径重建 AI 目录。
当前文档中的查阅链接已改为项目内位置；项目专属配置与新脚本必须使用本项目根目录解析路径。
