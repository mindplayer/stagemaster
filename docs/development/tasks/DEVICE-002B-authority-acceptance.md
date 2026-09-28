# DEVICE-002B：绑定准入与连接权限核心验收

日期：2026-09-29；基线 `0a713dd`，结果为本次 `feat(auth): gate live authority on bounded pairing and durable bonds` 提交。主工作区／当前会话单一写入。

按 [ADR-043](../decisions/PRODUCT-ADR-043-binding-admission-and-link-authority.md) 增加独立无堆 Authority：物理准入调用边界、固定 90 秒／三次尝试、设备分配的连接代次、受限密钥恢复证明、6 秒有效保活、降级／过期／取消撤回、提交前暂停权限和提交后重新连接。主体从档案查得；不把无线自报身份、普通加密或存储提案当授权。见[接口](../../module-api/device-link-authority.md)。新增产品模块按状态、连接、绑定与类型拆分，最大 143 行；不新增运行依赖。

## 验证

- auth 开启 persistence 后 28 项定向测试、严格 Clippy 通过；包含前一轮 11 项与本轮 17 项。全工作区默认特性 367 项及严格 Clippy／fmt 通过，其中 20 项与上述定向重叠，不能简单相加。
- 新旧事件交错、弱认证／缺少绑定／不同密钥／错误身份、查询不续期、期限边界、时钟倒退、代次耗尽、窗口额度、取消晚到成功、配对只返回提案、更换密钥主体稳定、撤销／恢复／陈旧档案与容量满拒绝通过。
- 组合真实 EKV 适配与 Authority：准入后提交、重新打开、再次证明密钥；127 个实际撤销 I/O 点 × 三种故障，写入不确定时原连接权限始终关闭，恢复后的权限严格服从实际完整旧／新存储结果。
- Xtensa `xtensa-esp32s3-none-elf`、persistence 开启／locked／offline 的库级 check 通过；不是完整固件链接或实板认证。
- 首次全工作区运行在原 `stagemaster-install-store` 的 `every_write_failure_reconciles_and_reopens_to_a_complete_version_then_retries` 失败：旧槽 snapshot 报“此槽正在接收新包”。该测试独立原断言复跑通过，继续用原全工作区命令复核；没有串行化、忽略测试或放松断言。此处保留失败，不宣称根因已修复。

日志：`logs/device-002b-authority-{first-tests,first-clippy,tests,clippy,xtensa-check,workspace-tests,storage-failure-repro,workspace-retry,workspace-clippy}.log`。全量原命令复核 367 项通过；首次存储租约偶发失败根因仍待定位，保留为后续集成审查项。后续原文件存储整组单独复核 12 项亦通过，见 `logs/device-002b-authority-storage-suite-recheck.log`；没有修改该模块或其测试。

## 未完成项

Authority 尚未接真实 ESP32 栈事件、凭据分区或桌面授权流程，不能据软件测试开放安装能力。本轮没有刷机、改变当前绑定、操作用户工程／UE 或物理输出。继续单一 Flash 所有者、真实绑定重启／取消／撤销和正式 GATT；现有测试配对确认持续有效。

用户新增要求设备状态灯：官方原理图已确认 RS485 绿灯直接接 GPIO17／TXD1，计划下一独立小增量在发送器始终禁用的诊断构建中显示连接状态；DMX 输出开始后此灯归 UART 波形，不能独立指定其闪烁频率。该硬件事实不会改变本模块的连接权限语义。
