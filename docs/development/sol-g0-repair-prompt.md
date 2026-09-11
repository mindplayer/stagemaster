你继续担任 StageMaster 的 Sol 开发负责人。Astra 已审查 G0：两个核心修复通过，但本地工作器存在四项已复现的契约缺口，G0 暂不放行到 G1。

项目目录：/Users/sunqi/projects/stagemaster。

先读取 AGENTS.md、docs/development/STATE.md，然后读取：
- docs/development/reviews/G0-review-20260911.md
- docs/development/tasks/DEV-004-worker-correctness.md
- docs/development/reviews/g0_review_regressions.py

本轮执行 DEV-004。授权你在工单范围内自主修复、测试、建立任务 worktree、提交并集成。先核实当前版本和未提交改动，将 Astra 本轮审查文件独立纳入版本，再开展实现。由你更新 STATE.md 的实际队列。

这些问题由你直接处理，优先使用确定性并发测试、合成响应与回环 HTTP 测试，不必交给 Qwen，也不需要重做全部资格样本。修复取消与候选发布的原子性、准备失败的状态收尾、完整 HTTP 请求截止时间，以及错误响应的原始证据与耗时记录。详细验收以 DEV-004 为准。

保留 CORE-001／002 已通过的产品修改，不扩大到新平台、模型运行时或 G1 核心重构。不要只让四个复现脚本表面变绿；把契约行为纳入正式测试，允许合理重构测试接入点，但不得削弱断言。

完成实际验证和集成后，保存 DEV-004 交付记录，更新阶段状态，返回精确提交和简短摘要供 Astra 复审。工单内实现自主解决，不逐步询问确认，不停在提出计划。
