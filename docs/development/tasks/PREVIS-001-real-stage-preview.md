# PREVIS-001：真实三维预演闭环

- 状态：in progress；用户明确要求 goal 模式，目标保持 active。
- 基线：`9d49939`；工作区 `/Users/sunqi/projects/stagemaster`，开始时干净。
- 当前会话直接规划、实现、验证；无子代理、硬件输出或部署。
- 决策：[ADR-016](../decisions/PRODUCT-ADR-016-previsualization-and-positioning.md)，空间沿用 ADR-010，灯具沿用 ADR-009。

## 增量与验收

1. 安装环境核验与真实 UE 运行环境准备；官方来源、具体版本和阻塞如实记录。
2. 独立空间模块与工程持久化，覆盖多个房间、凹多边形、独立标高／净高、舞台及灯位；拒绝非法几何和悬空引用，撤销与保存重开不丢信息。
3. 真实桌面布置组件，精确输入、选择／搜索、复制、拖动取消、一次撤销与上下文保持；UI 不持有第二份权威工程。
4. 独立预演桥与 UE 适配，编辑／播放来源明确、同版本快照、背压和断线隔离；仅本机预演，无输出租约。
5. 原生联动验收：真实 UE 画面、固定调光／RGB 灯响应、执行同步、镜头导航、拾取／拖动以及失败恢复。未运行 UE 不通过本项。
6. 摇头灯高返工边界预先验证：展开轴角、安装变换、单灯校准、正逆求解、多解连续选择、奇点／不可达、编码与输入方向分开。完整摇头灯产品能力另立增量，不伪报本轮已支持实灯指向。

## 当前记录

2026-09-27：核对现有产品仅有 DMX 数值离线预览，无真实三维。机器 M5 Pro／64 GiB 可开展开发，UE 和完整 Xcode 缺失。已查 Epic、MA3、Titan、GDTF 官方文档，形成 ADR-016；官方 Epic 启动器下载至项目 `tmp/previs/`，签名为 Epic Games International、Apple 信任链，尚未安装引擎。

## 验证与未完成

已新增 `stagemaster-spatial`（glam 0.33.10／geo 0.33.1，精确锁定），独立双轴静态求解／安装变换、凹多边形验证与三角化。10 项指向测试覆盖 216 组安装／零位／限位往返、反向分支、奇点保持和不可达；测试发现角度边界浮点误差后用严格有界 1e-10 度数值修复，不夹紧实际不可达目标。2 项几何测试覆盖正反绕序、凹轮廓面积与自交等反例。

按 ADR-017 新增工程 stage 模块、Schema 扩展、空间／构件／灯位命令和只读投影；8 项工程测试与 1 项 Session 历史测试覆盖真实保存重开、撤销重做、引用保护、原子批次、复制和移除关系。UI 拆分 StageWorkspace／StageInspector／StageCanvas 及类型／输入转换。无默认样例场地；用户操作创建的对象直接进入真实工程。接口见 [空间 API](../../module-api/stage-spaces.md)。

当前验证：107 项 Rust、31 项 UI、59 项格式、workspace fmt／严格 Clippy 和桌面构建通过。日志位于项目 logs/previs-*。原生创建空间、标高 1.2／净高 4.5、拖动 X +1 米、一次撤销原位、围护和 4×2 米／0.6 米舞台、真实保存至 data/PREVIS-001/ 已验证。尺寸校验出现浏览器英文提示，已加入 noValidate 复用中文业务校验，原生复查已确认中文提示、焦点落在净高字段和取消恢复。再次重开实际文件通过；两台 RGB 调光灯保存 X=2／6、Z=5.2，第一台底座 X 旋转 25 度，第二台 0 度，文件独立复核通过。空间复制只新增范围／围护，删除原空间保留舞台／灯具，连续两次撤销回到已保存状态。布置↔灯具切换保持选择。工作区补显式 aria-hidden 后原生辅助功能可正确恢复子树；删除弹窗在原生辅助树中仍有读取缺失（画面与鼠标操作正常），需进一步核对。拖动中的 Esc 尚未通过原生自动化实测；代码支持但不能以已验收描述。

用户已登录 Epic 并启动 UE 5.8.3 安装，安装路径为启动器默认 `/Users/Shared/Epic Games/UE_5.8`；这是用户操作的软件安装位置，项目源码／工程仍全部留在仓库。启动器已显示“启动”，实读 Build.version 为 5.8.3／58210709。首次启动经过 macOS 动态库／签名加载后退出；Unreal.log 明确报告缺少完整 Xcode，无法为 Metal 编译着色器。尚未进入编辑器，保留进程采样诊断。用户要求下载 Xcode，已核实 26.1.1 Apple silicon 官方文件链接要求 Apple 登录，未认证请求重定向 unauthorized；已打开并保留登录页。未代用户接受许可，尚未下载 Xcode、编译 C++ 适配器或通过三维验收。

继续增量（基线 `ba0cc2f`，结果为本次 `feat(previs): add neutral projection and guarded local bridge` 提交）：按 ADR-018 新增独立 Rust 预演投影，生成凹多边形构件、通用固定灯方向和真实场景／播放灯值，限制网格预算并拒绝未支持档案。新增本机 HTTP 桥、随机会话、有限接口、版本缓存、三维安装编辑接统一历史、来源／播放过期状态及故障隔离。默认不监听，不向前端返回凭据。源码及接口见 [预演 API](../../module-api/previsualization.md)。6 项投影、6 项真实回环／并发测试和 1 项独立播放时钟保护测试新增通过；工作区共 120 项 Rust、fmt、严格 Clippy、桌面构建通过，日志 `logs/previs-bridge-*`。原生重开 `data/PREVIS-001/空间与灯位.project.json` 及房间／舞台／两灯参数恢复通过。没有把未接 UE 的端口当作产品三维入口。

环境判断补充：核对本机 UE 5.8.3 的 MetalRHI.cpp，Xcode／Metal 编译器检查位于 `PLATFORM_MAC && WITH_EDITOR`；当前只有命令行工具不足以启动 Mac 编辑器。Xcode 是本机开发构建依赖，成品应打包成独立预演组件，最终客户不必安装 Xcode、启动器或 UE 编辑器；也可由另一台已配置 Mac 构建本平台成品后分发，但目前没有该构建环境。参考 [Mac 要求](https://dev.epicgames.com/documentation/en-us/unreal-engine/macos-development-requirements-for-unreal-engine)与[打包流程](https://dev.epicgames.com/documentation/en-us/unreal-engine/packaging-your-project)。

目标保持 active。待完成：布置操作完整原生验收；UE 消费端／实际项目、进程启动管理和灯光与构件、桌面来源选择与跨窗口草稿保护、同源播放实际画面同步、导航拾取和三维拖动、原生故障隔离与性能核验。完整摇头灯档案／动态路径／校准工作流及实灯验证另作后续增量。
