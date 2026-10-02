# 后台节目的只读三维观察

依据 [ADR-110](../development/decisions/PRODUCT-ADR-110-background-previsualization.md)，实施与验证见 [PREVIS-003](../development/tasks/PREVIS-003-background-observation.md)。沿用唯一内嵌三维视窗；后台节目与编辑预演使用显式来源选择，生命周期分别管理。

## 调用与归属

```text
Reader::open(discovery_path) -> Reader     // 仅持读取凭据，不创建控制会话
reader.project() -> canonical_json        // SHA-256 必须与固定目录 layout 一致
reader.sample() -> Sample                 // 原子状态与完整 512 槽软件采样
compiled_output.observer() -> OutputObserver
observer.observe(universe, slots) -> PreviewOutput
LightRig::playback(output) -> lights
ApplicationHost.previs({kind: "background", generation}) -> PrevisStatus
```

网络调用为异步；通道还原及投影是纯内存计算，不调用 Player，不驱动节目时钟，也不触发输出。独立 Rust Reader 不依赖桌面／UE，且不保留可用于提交操作的凭据。桌面适配从后台固定工程准备场地、通道映射和 LightRig，工程准备在有界数据上执行。TypeScript 不持后台连接秘密。

v2 执行进程增加 `GET /project`，使用现有读取鉴权，返回准备时保存的规范工程；来源目录声明 `preparedProject` 能力。文件移除或后续编辑不改变这个版本，v1 不开放此出口。工程读取仍受客户端 8 MiB 响应上限保护。

## 一致性与失效

- 校验启动身份、固定工程摘要、状态／帧修订、非零线路、512 槽、无故障运行状态及规范十进制计数。输出观察器再核对编译线路；8 位映射恢复实际量化值，16 位按档案粗细通道还原，包括非相邻通道。
- 原始观察不与控制回执合并。采样不能晚于状态时刻或落后达到 2 秒；请求本身耗时达到 2 秒也拒绝。周期、采样时刻及合成序号不能倒退，相同序号不能换采样时刻；重复读取不延长该帧的本地有效期。
- 桥协议 2 的 `background { hostId }` 必须只读，`canEdit=false`。后台场地代次为 0，内容版本由每次绑定单调增加；编辑场地有效代次从 1 起。回包前再核对当前绑定，避免旧来源覆盖新选择。
- 后台来源禁用选灯与灯位修改；Rust 入口同时拒绝旧界面或延迟到达的移动操作。已有编辑总控不会再次衰减后台采样。
- 观察故障停止显示有效灯光并报告问题；不会停止、接管或重启后台。重新选择后台来源重新绑定，关闭／重开三维只更换预演桥与渲染进程。

## 验证范围

本增量覆盖单域后台软件输出到既有 RGB／亮度／轴投影。无法模拟的光学功能保留既有提示，灯具列表取自后台固定工程。软件采样不是物理灯具反馈；外部控台输入、多线路、完整光学、跨设备同步及 UE 客户安装包继续单独验收。
