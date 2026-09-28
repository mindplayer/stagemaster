# DEVICE-002：安全记录分片验收

日期：2026-09-29；基线 `949facd`；主工作区、当前会话单一写入负责人。结果为本次 `feat(device): frame secure records with bounded ordered fragments` 提交。完整目标仍进行中。

## 实现与审查

按 [ADR-048](../decisions/PRODUCT-ADR-048-bounded-secure-record-framing.md)，在现有设备连接模块增加无堆 `secure` 子模块，分开记录、时间、接收、发送；复用现有 Serial，未重复实现序号算法。新增生产文件最大 104 行。采用固定大小缓冲、单条背压、固定期限和终止式错误处理；无安全凭据或业务授权语义。[接口调用](../../module-api/secure-record-channel.md)。

参考 SMP BLE 的长度首部／顺序分片机制，但加密消息不是 SMP 明文，不沿用业务 Assembler 或官方特征 UUID；保持协议边界真实。手动审查确认上层认证期限不可由分片续期、发送结果不确定须断开、完成记录仍不可信、序号跨记录连续。全部关键边界由测试保护，没有放开旧 LESC 安装权限。

## 验证与限制

- 新增 9 项用例：8 项纯分片、1 项 Noise 组合。包括固定黄金字节、20／21／244 B 预算下所有 1～1297 B 长度、首部逐字节重组、持续序号、重复读取待发、缺片／乱序／重复、超容量／非法头／两记录合并、固定期限／时钟回退与溢出、取消／错误调用后不可恢复。Noise 组合在 20／244 B 各完成双向握手、确认、256 轮最大消息与加密保活回复；结构正常而标签损坏的记录仍被 Noise 拒绝并清空明文。
- 首轮长度遍历测试用例本身在第二条消息把时间从 1 倒回 0，正确触发 Clock；已修正测试时钟为不倒退，未放宽产品检查。失败日志 `logs/device-002-secure-framing-tests-first.log` 保留。
- 相关测试／严格 Clippy 通过，日志 `logs/device-002-secure-framing-{tests,clippy}.log`。最终全工作区与 Xtensa 结果在下文记载。
- Xtensa 的 `session-readiness` 严格 Clippy／完整 release 构建通过，日志 `logs/device-002-secure-framing-xtensa-{check,build}.log`；保留既有 RWX 链接告警。构建证明目标兼容，不代表未接入的分片已实板运行或测过栈峰值。

本增量没有再刷机、改节目／工程、操作系统绑定或发送 DMX；板卡保持上一验收恢复的只读固件，A 第 9 代有效。实际 GATT 密文通道、可信开发凭据／业务授权、无线节目安装和桌面完整流程仍待接通。云端、生产认领、文件 24 小时许可没有被本模块代替；完整 DEVICE-002 goal 保持 active。


## 最终复核

394 项全工作区 Rust 测试、严格 Clippy、工作区 fmt 和差异检查通过，日志 `logs/device-002-secure-framing-workspace-{tests,clippy}.log`。原生应用复查仍已连接，387 次保活、最近往返 156 ms，原工程没有编辑。上述连接运行的是已恢复的只读镜像，不把它当作新分片无线验收。原有板内安全核心资源结果继续见 [B 验收](DEVICE-002B-session-acceptance.md)。
