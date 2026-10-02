# TRANSPORT-001：可替换的应用记录承载

状态：软件承载增量完成；基线 `674031a`，结果为本次 `refactor(device): share application sessions across record carriers` 提交；main，当前 Astra 单写者。上轮 HOST-002 已提交独立进程与真实客户端隔离证据，属于进展；持续 goal active，接续框架主线。用户 `output/` 保留，未操作窗口、设备、固件或真实输出。

依据 [ADR-102](../decisions/PRODUCT-ADR-102-record-transport-boundary.md)，抽取既有 BLE 应用会话并实现第二种字节流承载，验证协议与安装语义复用。改动范围为新 `crates/stagemaster-device-channel/`、device-host 的 BLE 应用适配、工作区清单／锁文件及对应契约／状态。保持现有 Transport 服务、GATT 字节协议、工程格式、设备固件和 UI 公共入口。

按职责分文件：记录接口／错误、共同应用会话、握手、消息处理、字节流适配，以及独立的软件端点和故障测试。参考机制、预算和验收门槛见 ADR；实施后记录实际结果，不以仅有接口或假成功消息作为第二承载完成。

## 实现与审查

`stagemaster-device-channel` 从原 BLE 模块抽取控制端应用握手、既有可信开发配置、就绪回执、保活与加密消息；`RecordIo` 只负责完整不可信记录。BLE 生产路径已使用共同实现，保留原 UUID、片段和通知序号；新 `StreamRecords` 接受已连接的字节流，使用 2 字节长度前缀、固定缓冲／队列和独立读取任务，不依赖蓝牙扫描或诊断。调用见[契约](../../module-api/application-record-carriers.md)。

发送／保活在等待前禁止旧会话复用，完整成功才恢复，修复此前通道层在 Future 中途取消后仍可能看起来可复用的缺口；外层已有取消连接保护仍保留。字节流的半包期限不因后续字节延期，溢出使排队数据全部失效。No_std 安全协议、设备固件、工程格式、安装事务和桌面入口未改。

参考 Tokio 既有异步 I/O／有界队列和项目已有 Noise、权限及安装核心，未新增第三方版本或密码算法。源码按职责拆分，新模块手写文件最大 136 行；测试响应端组合真实 session／auth／transfer／FileStore，安装器实际打开并附加已确认权限后才发送就绪回执。该软件组合不冒称重跑了固件 Gateway／NOR／射频验收。

## 验证

- 新模块 14 项：真实本机 TCP 与 20 字节 GATT 软件分片用同一 `Channel` 安装同一工程包，安装期间插入保活，已提交包目录及落盘字节与来源一致。
- 错误可信公钥、不同启动上下文、加密旧就绪回执、篡改、重放、跨连接密文、固定活性／开发权限期限拒绝；查询及保活不延长权限。
- 取消或失败的应用发送、迟到保活回复、整个握手 10 秒期限、分段／合并流记录、超长／空长度、接收溢出、部分断开、半包 5 秒绝对期限、部分发送取消／超时与析构资源释放通过。
- 既有 device-host 的 45 项通知／连接／安装回归通过，合计 59 项专项；没有削弱原断言或改变其故障期待。
- 全工作区 779 项通过、0 失败、1 项原有忽略；25 个 crate 文档测试运行，1 个宿主调用示例通过。全工作区严格 Clippy 通过；最终仅改进软件响应端就绪时机的测试组织，14 项及相关严格检查再次通过。

日志：`logs/transport-001-tests.log`、`logs/transport-001-channel-final.log`、`logs/transport-001-workspace-tests.log`、`logs/transport-001-workspace-clippy.log`、`logs/transport-001-clippy.log`。依赖树与本地链接／差异／格式检查另存 `logs/transport-001-dependencies.log`、`logs/transport-001-document-check.json`。初次检查发现测试组合 Future 超过栈大小审查线，改为明确堆上固定，不抑制 lint。

## 出口与接续

本次软件出口通过，整体设备框架仍未完成：外层 `Transport` 和桌面还是 BLE 发现／诊断适配，不宣称端点目录已通用化；真实 USB／以太网产品、跨平台驱动、输出、生产身份和商业保护未验收。不设置商业使用限时；开发会话权限期限沿既有契约保持。

接续按 ADR-097／100 审查端口输出权与多执行器边界：本地执行、外部帧和人工／硬件控制的所有权及旧指令失效，先用软件端口验证，不能把 Observer 的软件采样当作物理发送确认。不回到局部 UI 细节丰满，持续 goal active。
