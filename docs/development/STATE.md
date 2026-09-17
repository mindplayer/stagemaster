# 当前开发状态

更新：2026-09-18。当前 Astra 会话直接负责架构、实现、测试、审查、集成和状态维护；不再委派 Sol／Qwen。
依据：[DEV-ADR-002](decisions/DEV-ADR-002-astra-direct.md)、[开发方法](README.md)、[当前执行计划](execution-plan.md)。文件统一留在本项目内，见[目录规则](project-files.md)。

## 产品基线

- 产品仍为 A0 静态原型，能力和限制见[实现状态](../implementation-status.md)。本次清理不改变核心语义。
- 清理前主线：`121efa311855d977364f7ad8707ea729b5c0e367`；仅一个 `main` 工作区，无待合并分支、标签或远程。
- CORE-001 已修复 Cue 编号碰撞，集成 `df64f98603ca28462cf76a515b65fb39dda9b26d`。
- CORE-002 已修复 HTP 默认值下限，集成 `81ed816fff5d8a358d5e1ecf933057a148d02e8f`。
- G0 按开发基础范围结项；已停用工作器的三项残留缺陷没有被认定为修复通过。

## 任务状态

| 任务 | 状态 | 说明 |
| --- | --- | --- |
| [DEV-001](tasks/DEV-001-delivery-foundation.md) | done | Git 与验证基线保留 |
| [CORE-001](tasks/CORE-001-cue-uniqueness.md) | done | 实现与保护验收保留 |
| [CORE-002](tasks/CORE-002-htp-fallback.md) | done | 实现与保护验收保留 |
| DEV-002／003／004 | retired / 历史结项 / cancelled | 旧工作器路线退出，专属文件转由 Git 历史追溯，不重启补修 |
| [DEV-005](tasks/DEV-005-current-version-cleanup.md) | done | 当前项目清理完成；源码重建、24 项测试、fmt 与严格 Clippy 通过 |
| G1 首批产品工单 | 尚未发布 | 由本会话整理契约并直接实施，不再等待另一会话 |

## 最新验证

DEV-005 清理结果已集成：`5a15e97a1ecedf0a4c821fe38936089c9f2dc2de`；后续状态记录提交不改变产品代码。

- 已删除 35 个受版本管理的旧工具／流程文件、846 个旧试验及迁移文件，清空旧构建缓存后只生成当前源码的验证产物。Git 历史保留。
- 产品源码、Cargo 清单、保护测试、模块 API 和控台研究与清理前基线一致。
- `cargo fmt --all -- --check`、`cargo test --workspace --locked --offline`（24 项）、严格 Clippy 及文档引用检查通过。
- 真实灯具、固件、云端及共享模型环境未操作。

## 下一步

DEV-005 当前项目范围已完成，交付见 [清理记录](deliveries/DEV-005-delivery.md)。下一步细化 G1 的领域边界和依赖顺序。R03–R09 等既有问题保持待处理，不因清理文件而标为完成。
旧独立项目 `yunwei-ma` 有未提交源码及未跟踪文件，删除范围尚待用户明确；当前不改动该仓库。
