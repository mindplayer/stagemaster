# 本机最近工程

UX-024／ADR-056。Rust 桌面适配持有本机导航目录；不进入 Show JSON、撤销历史、恢复内容或云端。

- `recent_request({kind:"list"}) -> RecentProject[]`：最多 12 条，顺序为最近成功打开／保存；字段 `id,name,path,openedAtMs,available`。`available` 只表示普通文件存在，完整工程与媒体仍在打开／加载时校验。
- `recent_request({kind:"forget",id})`：只移除索引后返回最新目录，不删除工程、随附资源或当前文档。
- `project_request({kind:"openRecent",generation,id})`：仅解析目录内 ID，复用 DiskFile 校验及未保存提示；拒绝过期工程代次和失效 ID。前端没有任意路径读取能力。
- 成功打开／保存后尝试记录，元数据失败通过 `Snapshot.recentProblem` 单独报告，不推翻已成功的文件操作。文件打开失败或用户取消不记录目标。

调试位置 `data/navigation/recent.json`；发布为应用本机数据目录的 `navigation/`。格式版本 1，文件最多 64 KiB，12 条，绝对路径最多 4096 字节、名称最多 512 字节；目录的文件锁、临时文件、同步与原子替换防止协作写入互相覆盖。损坏／未知版本不自动清空。目录中只有路径与名称，不保存工程内容或声音。

启动页直接显示目录；编辑中按需打开。搜索匹配名称及路径；同名工程可按路径区分；文件不可用可重新选取。导航弹窗关闭后返回原上下文，Esc 可关闭，不阻止退出；工程编辑弹窗和未保存文档保护继续生效。完整资源健康聚合与云端最近列表未实现。
