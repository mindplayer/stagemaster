# DEVICE-003：运行入口实板验收

状态：本增量验收完成，完整设备运行链路保留物理出口。基线 `3af3e98`，main，当前会话单写者；上一回合桌面软件入口已验证提交，分类 progress。开工时工作区仅用户未跟踪 output/，不操作该目录。用户进度答复回合仅核对记录，分类 no progress；现接续实板可见流程。

## 范围与依据

沿 [ADR-133](../decisions/PRODUCT-ADR-133-firmware-runtime-gatt.md) 的运行固件与 [ADR-134](../decisions/PRODUCT-ADR-134-desktop-device-runtime.md) 的正式原生 JSON 入口，使用现有真实板卡测试授权（用户曾明确允许测试、替换原固件、不保留旧固件）。不操作真实灯具；GPIO21 由 OutputDisabled 全程保持禁用，不修改 eFuse，不引入商业限时。既有节目槽与分区布局保持。

本次 USB 枚举为 Espressif USB JTAG_serial debug unit，/dev/cu.usbmodem2101，序列 28:84:85:56:97:74；串口无占用，未发现正式舞台大师进程。原本地 v1 配置的公开设备标识是 534d4553503332533300288485569774，仅安装范围。新运行配置用现有生成工具在新的项目 data 私有目录产生，不改旧配置或输出密钥。构建、刷写、观察与无线验收均显式指定设备和文件，构建夹具不能刷入。

## 实施与出口

1. 捕获当前状态，生成实际设备对应 v2 配置；构建严格检查、固定分区刷写与启动日志，确认当前目录恢复、自检和禁止发送。
2. 原生验收工具只复用 Ble／Service／runtime_request，不重写协议；观察与操作验收显式分开，修改操作需要 exercise 模式，先确认稳定设备身份、自检及禁止输出。多个候选须明确选择。
3. 实际 GATT 目录、选择、载入、步骤与执行状态；暂停／继续、同实例断线自主推进、重新连接只观察不抢权，恢复控制后明确停止。真实设备出现错误必须保留原始日志与状态，不能削弱断言以通过。
4. 串口观察运行堆、栈采样、Flash 操作及故障。短期观察不作为完整峰值、独立供电或长时间稳定性证明。结束后断开测试连接，设备不自动开始节目。
5. 相关主机回归、全工作区严格检查、固件检查／构建与必要产品修复分别记录；不以编译或软件夹具取代实板证据。真实 UART DMX、波形测量和灯具验收仍是后续出口。

## 实板发现与修复范围

实际原生窗口已搜索连接、恢复 28 场景、取得控制、搜索／选择／载入并开始对称扇形；暂停保持成立。发现周期观察占用 `busy`，每次蓝牙读取期间禁用按钮，导致点击落在刷新窗口会被忽略。限定修复 useDeviceRuntime 的前端串行协调：后台观察不再禁用控制，最多一个用户操作等待当前读取，期间不给新观察插队；重复点击不堆积、读取失败不派发、连接变化废弃旧等待操作。实际控制权、修订、超时和执行仍由原生／设备判断，不新增播放服务或变更公共契约。既有音频队列允许中断旁路，不符合设备单请求约束，因此采用独立短小协调器，复用原 request 入口。新增有意义的延迟／失败／断线保护测试，再重建并实板复验。

## 已取得的实板证据

- 同一台微雪 ESP32-S3：使用真实编号生成新 v2 配置，旧 DEVICE-002 文件不改；两份新文件 0600、目录 0700，产物仅保存在忽略目录。实际配置对应 Xtensa 严格检查与完整构建退出码 0，`logs/device-003-runtime-board-{check,build}.log`。原 RWX 链接告警保留。
- 固定原分区表／ota_0 刷写退出码 0，`logs/device-003-runtime-board-flash.log`；808,704 字节，占 3 MiB 应用分区 25.71%。恢复原 B 槽第 18 代，110,772 字节、28 场景，摘要 `c0eeb129f0f343ea90d1447e0186906f06bd6d70361652203868a5fc7ec81f34`；没有擦除节目区、写 eFuse 或启用物理发送。
- `runtime_device observe` 与 `exercise` 都退出码 0，`logs/device-003-runtime-{observe,exercise}.log`。真实 Noise／GATT／原生 JSON／原 Runtime；全部目录读取、显式取得／载入／开始／暂停／继续成立。暂停进度为 3,510 ms，2 秒后不变；断线等待 5 秒后同一启动、同一实例继续，未持有控制；再次显式取得后停止／归还，最终 idle／无实例／无控制者。不是软件分片模拟或真实 DMX 波形验收。
- 600 秒串口观察正常退出，`logs/device-003-runtime-board.log`。打开串口触发 `USB_UART_CHIP_RESET`，不能称无复位或独立断电恢复。实际 PSRAM 8 MiB，既有缓存 2 MiB；采样堆最高记录 53,140 字节，载入后常见 50,936／剩余 80,136 字节；采样栈 20,525／32,768 字节并非完整高水位。已记录操作最大 8,336 μs 不等于帧间隔。诊断 `LIVE ticks` 属于原负载播放器，不拿来证明本次节目实际帧生成或 UART 刷新。短期未见故障不能代替长时间峰值与持续压力验收。

## 软件及原生验收

- 103 项 device-host／device-channel／device-upload 回归通过，0 失败／忽略：`logs/device-003-runtime-board-host-tests.log`。覆盖既有严格请求、可信配置、原生应用、传输和安装，未称全工作区全部测试。新增验收工具在同一全工作区全部目标严格 Clippy（application 与 development-device-access）通过，`logs/device-003-runtime-board-clippy.log`。
- 修复观察占用后，318 项 UI 测试通过，含新增 4 项调度延迟／失败／断线测试；类型通过，`logs/device-003-runtime-board-ui-{tests,check}.log`。新协调器 35 行，原 hook 321 行，保持状态／页面分离；hook 保留同一连接生命周期，不做机械拆碎。Rust 例程按连接、流程分三文件，最大 113 行；无第三方或锁文件变化。
- 正式 Tauri 带显式开发访问特性打包成功，`logs/device-003-runtime-desktop-build-fixed.log`。首次错误从项目根调用 CLI 导致 beforeBuildCommand 路径错误，保留 `runtime-desktop-build.log`，改为从 apps/desktop 执行；不是修改构建命令掩盖源码失败。既有前端大块产物告警保留。
- 独立验收环境 `STAGEMASTER_ACCEPTANCE_INSTANCE=device003-runtime`，空白工程，无媒体和 UE；读取实际 28 场景，搜索／选择／载入对称扇形；初次原生验证发现刷新禁用按钮并完成修复。重建后实板实例 3 开始、暂停 0:12 保持、单次点击继续；收起／重开保留“对称”筛选和起始步骤，设备进度继续。断开重连后仍是实例 3，运行 2:26，控制权为未取得，起始步骤清空但筛选保留。原生重连截图／AX 在 `data/DEVICE-003/ui-validation/native-runtime-reconnected.{png,txt}`。这证明可见流程与回执，不替代物理输出。

最终实际停止回执为“已载入，待执行”、当前步骤无、无运行实例；归还后控制权未取得，物理发送仍禁止。证据 `native-runtime-stopped.{png,txt}`。用户在验收中主动查看设备身份，保留该隔离窗口和只读连接供其查看；不关闭用户正在查看的窗口，不自动重新播放。串口观察与旧测试应用已正常结束。

审查结论：本增量按 `fix(device): validate board runtime and preserve controls during polling` 集成；fmt／差异检查通过；473 个本地文档文件链接无缺失，记录 `logs/device-003-runtime-board-doc-links.json`。基线 `3af3e98`，结果为该提交；无新依赖或协议／工程包语义变化，用户 output/ 不操作。完整 DEVICE-003 仍待实际节目帧的独立时序采样、内存长期压力、UART DMX 及灯具验收；本次逻辑进度证明不可替代这些出口。FRAMEWORK-001 不重开，AUDIT-001／goal 保持 active。
