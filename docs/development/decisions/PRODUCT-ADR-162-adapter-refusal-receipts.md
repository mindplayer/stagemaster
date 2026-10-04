# PRODUCT-ADR-162：适配层拒绝与原回执

状态：2026-10-05 当前实施会话审定，先记录再修复；对应 [EXEC-016](../tasks/EXEC-016-observation-receipt-boundaries.md)。无工程／包格式、依赖、时钟或控制权变化。

## 证据与问题

既有 `commands::execute` 把 `Application::action` 的全部 Failure 归并为 `Problem::Invalid`。媒体适配读取权威 observer 使用非阻塞槽，可返回 `Failure::busy()`；生命周期关闭可返回 closed。它们并不等于用户输入非法。确定性适配故障注入 `logs/exec-016-adapter-red.log` 证明 busy／closed 的 code 被改成 invalid，连合法 invalid 的原说明也被丢失：1 通过、3 失败，退出 101。

FIXTURE-012 的历史循环 rejected 没有具体 code，基线单次和 12 次有界诊断全部通过，不能反推其原因。下面修复只处理已证明的分类缺陷，不将它写成所有历史拒绝或音频失败的根因。

## 决定与失败行为

- 复用原 `Failure`、完整回执和单次任务。适配失败发生在向 Runtime 提交之前，回执仍为 complete／rejected，保留适配原 code／message，不虚构 state／媒体 request，不更换输入权限 Binding，不提交 Runtime 意图。
- HTTP 任务序号已接纳则仍消耗原会话序号；Runtime 序号尚未提交则不消耗。无重试、回滚、重发、强行接纳或无限等待；下一次用户明确操作沿既有新网络序号和原 Runtime 序号。
- Runtime 的 Revision／Busy／Deadline、unknown 与 binding 清理沿原逻辑；无新的成功种类。非法参数仍 rejected／invalid；短暂繁忙不会变成假成功，关闭也不当 unknown 重放。已有客户端将 code 当作诊断字符串，界面已有中文 message 展示，错误对象及字段不新增。
- Client POST 失败仍保留原待确认序号；若宿主没有原回执，不可自行判定未执行／释放 pending。POST 已接纳但最终 GET state 繁忙时，只读查询原回执；accepted 与同一媒体 request 的 Applied／Failed／TimedOut 分开。

## 复用与验收

沿用本项目成熟的回执丢失代理 `client_receipt_loss.rs`、有界只读等待与 trusted adapter 分层，不新建客户端路由、播放器或计时器。提取测试代理以覆盖提交前 503、提交后观测 503、旧修订拒绝和真实媒体失败，分别核对序号／转发次数／权威状态；原正常回执、循环及保护验收不放宽。适配分类先红后绿；执行完整 Rust、严格检查、正式包及对应可见原生回执检查。软件声音输出不是听音、声卡或实灯验收。
