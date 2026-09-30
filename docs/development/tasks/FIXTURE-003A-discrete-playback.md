# FIXTURE-003A：灯具离散属性执行基础

状态：完成。基线 `1162f74`；主工作区／当前会话唯一写入。对应 F02；契约见 [ADR-059](../decisions/PRODUCT-ADR-059-discrete-playback-attributes.md)。

范围：`stagemaster-playback` 的 Plan／Player 与测试；`stagemaster-package` 编解码、全包兼容／资源统计与测试；模块文档和 STATE。先拆 Plan 与独立效果编解码职责，避免入口继续增大。UI／工程文件尚不新增占位功能，后续 B 增量接入功能区间建档。

出口：延时后直接切换、连续渐变不变；暂停／跳转／循环／默认释放一致；无动态效果冲突；旧包字节不变，新包往返逐帧相同；旧执行语义头不能携带新块；不可信长度先校验再分配、预算包含新增存储。按当前平台执行 fmt、工作区测试／严格 Clippy，必要时目标构建；不刷机、不连接真实输出。


实现进展：Plan／Player 分离为 198／225 行；包扫描、效果编解码及主机 Builder 按职责拆分。新增索引有效载荷按 2 字节／项计入预算，空列表不分配；全包效果占用汇总使用 64 字节位集，避免再占 512 字节扫描栈。最大 512 属性的播放、最高索引和效果冲突有回归。

过程记录：首次拆分漏导入 Keyframe／Step 内部可见性，编译失败后修复；严格 Clippy 两次发现扫描函数超过 100 行，已抽取离散索引和效果扫描；测试辅助函数移出后遗留导入已移除，没有豁免检查。一次旧全量复跑与最终复跑重叠，已明确中断旧进程，最终结果以 `verified.log` 为准。Xtensa release 链接通过，工具链仍报告既有 RWX LOAD segment 警告；未刷机或实测新内核栈高水位。


最终验收：455 项工作区 Rust 测试、fmt、工作区全目标严格 Clippy、Xtensa runtime-readiness 严格检查通过；新增 6 项纯执行与 6 项包回归。614 字节真实旧包逐字节重建相同，新计划 2001 个采样点的主机／解码值和 DMX 数组完全一致；降级头／最高索引／错误效果／恶意长度与预算拒绝通过。最终日志 `logs/FIXTURE-003A/{verified,clippy-verified,xtensa-verified,fmt,links}.log`；8 个本地文档链接和 diff 检查通过。

审查结论：核心与包边界可集成，无真实输出／刷机；功能区间编辑、中文选项、工程到新计划以及目标版本提示尚待 B 增量。UI 未改变，不复跑无关界面验收。结果为本次 `feat(playback): preserve discrete attributes across fades and packages` 提交；持续目标继续。
