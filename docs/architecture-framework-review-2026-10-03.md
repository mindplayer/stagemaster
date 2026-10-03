# 平台框架整体验收：2026-10-03

基线 `8464d8a`，当前 Astra 会话直接审查；对应 [FRAMEWORK-001](development/tasks/FRAMEWORK-001-platform-exit-review.md)。状态：审查和修复进行中，尚未宣布本轮完成。依据 [ADR-097](development/decisions/PRODUCT-ADR-097-integrated-stage-platform.md)、[设备组装 ADR-100](development/decisions/PRODUCT-ADR-100-composable-device-family.md) 与 [商业安全 ADR-101](development/decisions/PRODUCT-ADR-101-commercial-security-boundaries.md)。

## 判断依据

本轮目标是验证未来扩展所依赖的职责边界，并保持现有软件可用。不能以拆出 crate、写出接口、某项测试通过，推断全部产品链已经接通；也不把手机／控台、实际云服务、专业光学和真实灯具同步全部前置。以下同时记录实现与未实现范围，原 [AUDIT-001](product-audit-2026-10-01.md) 和架构 A01–A15 的完整产品门槛仍有效。

## 六组职责核对

| 职责与状态所有者 | 当前可运行证据 | 本轮不能据此声称已完成的能力 |
| --- | --- | --- |
| 工程与资源：Document 编辑／编译；DiskFile 持久化；资源摘要固定内容 | project-store 的 persistence／recovery／capacity 用例覆盖过期保存、提交失败与原文件保护；后台从固定工程与资源准备，原文件移除后继续。包内容独立于工程路径 | 全部高级灯具、多单元、跟踪编辑、未知扩展迁移、全媒体发布依赖和跨用户协作 |
| 编排与现场控制：Player 执行；Live Session 合成；Host 独立调度；Authority 仲裁 | 实际独立控制进程被杀后播放继续；后台 Observer 锁占用时跳过发布，调度继续；多来源和手动层先合成属性后编码，拒绝旧控制者与来源 | 正式后台目前灯光仅为软件采样；真实 DMX 输出、完整专业执行器页面、硬件拾取和主备切换未接通 |
| 媒体时间域：提供方拥有音源；time 映射；媒体组拥有跟随状态 | 独立时钟的误差区间、有效期、组内暂停／定位、旧代次拒绝；真实 PCM 经原 Host、原许可和回执执行；循环／末尾／故障恢复与多进程输出占用已有软件及历史原生静音验收 | 消费位置不是声卡实际呈现时刻；跨盒子、蓝牙音箱延迟与音视频同步精度未测；视频、外部动作派发与去重未实现 |
| 仿真与渲染：核心固定场地／灯值；预演只读投影；UE 拥有画面 | Reader 仅用读取权限；原工程移除、观察器重建不改变控制者和执行；PREVIS-003 原生 UE 关闭／重开后同一后台继续 | 第三方控台输入、完整 GDTF 光学／棱镜／图案模拟、专业精度和客户独立 UE 安装包未完成 |
| 设备与控制面：能力／承载／应用会话／安装／运行／端口分别持有状态 | 相同安全应用会话通过 TCP 与 GATT 软件分片安装同包；端口库拒绝旧来源、旧帧及旧回执，需真实静默回执才能维护；media_output 用例验证定位不重置端口序号 | 桌面发现仍是 BLE；正式程序未接端口库与物理驱动，ESP32 当前 RS485 禁用。真实有线／无线节点、面板和手机／平板宿主需独立产品适配 |
| 云端与商业化：交付只取得内容；Installer 提交；Runtime 校验播放许可 | 真实本机 TLS／普通文件共同完整校验，同身份安装去重；中断与坏包不改已运行快照；来源移除后可用；允许／拒绝策略不因交付途径改变 | 软件 TLS 测试不是云服务上线；生产身份、授权签名、账户／目录、发布审计和大型媒体分发未完成，当前不实施商业限时 |

主要代码与代表性用例：

- [工程存储](../crates/stagemaster-project-store/src/lib.rs)、[恢复失败测试](../crates/stagemaster-project-store/tests/recovery.rs)、[后台准备](../apps/execution-host/src/group/prepare.rs)。
- [独立调度](../crates/stagemaster-runtime-host/src/worker.rs)、[慢观察／控制者生命周期](../crates/stagemaster-runtime-host/src/tests/lifecycle.rs)、[真实客户端死亡](../apps/execution-host/tests/process_lifecycle.rs)、[多来源合成](../crates/stagemaster-live/tests/boundaries.rs)。
- [独立时钟前向模型](../crates/stagemaster-time/tests/interval_model.rs)、[组间隔离](../crates/stagemaster-live/tests/media_groups.rs)、[提供方新代次恢复](../crates/stagemaster-live/tests/media_recovery.rs)、[输出权交接](../apps/desktop/src/execution/audio_ownership_tests.rs)。
- [只读观察进程](../apps/execution-host/tests/readonly_observer.rs)、[中立预演](../crates/stagemaster-previs/src/lib.rs)、[PREVIS-003 原生证据](development/tasks/PREVIS-003-background-observation.md)。不因本次审查重启用户 UE。
- [TRANSPORT-001](development/tasks/TRANSPORT-001-record-carriers.md)、[OUTPUT-001](development/tasks/OUTPUT-001-port-authority.md)、[媒体定位与端口组合](../crates/stagemaster-live/tests/media_output.rs)。
- [共同交付](development/tasks/DELIVERY-001-common-package-ingress.md)、[实际安装／运行组合](../crates/stagemaster-delivery/src/http_tests/lifecycle.rs)。

## 实际依赖与本轮发现

无界面播放／时间／端口核心不依赖 Tauri、UE、云账号或音频设备。工程模型和媒体时间线语义可进入编译主机，但解码、声卡、HTTP/TLS 和具体硬件应由产品组装选择。ESP32 清单仍使用受限 Runtime／Package／Install 等，不携带主机的 std 线程、桌面或 HTTPS 导入。

发现 R1（已修复）：基线实际执行应用固定依赖音频，禁用默认特性仍包含 Rodio／CPAL／解码库，见 `logs/framework-001-baseline-host-tree.log`。按 [ADR-126](development/decisions/PRODUCT-ADR-126-optional-host-audio.md) 增加可选后端、拆出音频组装，纯灯光 26 项和默认音频 42 项回归、两种严格检查通过；实际依赖树及二进制链接确认纯灯光已移除后端。桌面显式构建音频版本，并从实际构建目录复制，摘要核对一致。未以运行时不开声卡冒充构建隔离。

发现 R2（已修正）：TIME-001 顶部过期接续已更新，指向已完成的输出占用和共同交付；总体架构 A01–A15 保留完整门槛，并引用当前证据。TIME-001 的外部动作／真实同步仍未关闭，未将不适用的测试或接口草案列为已完成。

## TIME-001 原要求与证据对应

| 原要求 | 现有证据与判定边界 |
| --- | --- |
| 1. 时钟身份／代次／单位与素材位置分开，映射有误差和期限 | time::Clock／Mapping 和独立前向模型；运行控制仍用宿主单调时间。软件要求已有证据，不代表外部时钟已校准 |
| 2. 准备在边界外，组与代次控制，失败／失联／恢复可查 | live-host 固定准备／观测／控制槽，真实后台准备、回执丢失与恢复用例；音频故障恢复显式暂停，不承诺进程崩溃自动恢复 |
| 3. 媒体跟随与自主来源独立，复用正式编译／播放／合成 | media_groups／media_recovery／media_output、实际 PCM／Host 与独立应用测试；未实现外部离散动作派发，不能将未发动作视为完成去重 |
| 4. 窄适配口，不绑 UI／UE／BLE，真实同步单列 | time／live／live-host 正常依赖及 MediaPort／Provider 边界；R1 已补实际应用的可选音频组装，声卡和跨盒子精度后续实测 |

## 后续实施边界

本轮通过后，下一产品增量应优先打通已安装包到受控设备运行／真实输出的交付主线，并安排相应实测条件；物理输出不由本次软件审查自动授权。桌面设备必须持续区分安装、运行和物理发送状态。尚未准备硬件实测时可推进共享设备运行命令／状态的可用软件入口，不能显示假成功。

原 AUDIT-001 的 F02 专业灯库完整性、U03 效果可视编辑、U05 现场工作面、混合人工／音乐调度、专业渲染与生态导入等按原验收继续。已有增量不等于原条目全部关闭；不把本轮框架出口当作整个持续目标完成。

## 验证记录

311 项 UI 逻辑、UI 类型及模块伪 API 类型检查通过。R1 修复后的纯灯光 26 项、默认音频 42 项、两种组合的严格检查及实际桌面后台准备／摘要核对通过；fmt、脚本语法和当前文档链接通过。完整 Rust 工作区基线仍在运行，待其结果后形成整体验收结论，不预先标记完成。原生 UI／UE、实物 U 盘／云服务、物理声卡及板卡未重验。
