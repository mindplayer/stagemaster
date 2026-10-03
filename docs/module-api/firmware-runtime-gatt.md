# ESP32 软件运行入口

DEVICE-003 第七增量，依据 [ADR-133](../development/decisions/PRODUCT-ADR-133-firmware-runtime-gatt.md)。`runtime-gatt` 构建在原安装固件内接入[运行消息](device-runtime-wire.md)、[独立有效性队列](device-runtime-queue.md)和原 ManagedWorker。此镜像执行、采样软件节目，整个启动期间保留板级 `OutputDisabled`，不发送 DMX。

## 入口与权限

| 职责 | 所有者／约束 |
| --- | --- |
| 安装服务 | eda0／eda2／eda3，沿用原安装握手及维护约束 |
| 运行服务 | edb0／edb2／edb3，观察、控制及维护分别检查范围 |
| 共同通信 | `ble/application` 负责 Noise、记录分片、通知、保活和清理；不启用系统配对 |
| 运行状态 | 同一个第二核工作器，唯一 Runtime；无线回调不执行节目或慢存储 |
| 输出能力 | 就绪时声明软件播放、运行应用、1 路逻辑空间、25 ms 节拍和 64 KiB 装载预算；不声明物理 DMX 输出 |

服务 UUID 的共同后缀为 `-0100-4e83-968e-799ab99558fa`，前缀为 `f889`。一次物理连接在第一次应用写入时冻结入口和 MTU；之后跨入口写入拒绝并清理。切换安装／运行须明确断开重连。普通运行连接、观察和重连不进入维护、不取得控制权、不选择或开始节目。

可信本地 SMDV v2 明确赋予观察范围才可构建运行镜像；控制和安装仍分别要求对应位。旧 v1 配置只能安装，不能从网络声明提升权限。详见[开发配置](development-gatt-configuration.md#显式运行开发配置v2)。这里的开发连接许可和软件播放策略不是商业授权实现，也没有引入节目限时。

## 调度与失败

运行命令、运行完成各有一个固定队列槽；工作器另保留一个待交付结果。完整 `Live` 在临界区内发布／读取，通信侧在命令可见前发布，失败、关闭和 Drop 清空；工作前后重新核对实时权限。安装 epoch 与运行 Live 分开，互不借权。

第二核以 25 ms Ticker 独立调用原 Endpoint／Runtime，并采样 512 通道软件帧；没有控制请求时继续推进。完成队列满时使用 `try_send`，等待下个节拍继续推进，不阻塞在完成发送上。结果过期或属于旧连接就丢弃；已执行但失去回复不伪装为未执行。同步装载、Flash 和日志耗时仍会影响实际节拍，必须实板测量，不能把此结构称为已验证的硬实时输出。

启动只尝试绑定当前持久目录，不选择或开始节目。包绑定失败保留维护状态，仍可走原安装恢复路径。正常运行镜像进入安装前，应先经运行入口明确进入维护并收到静默状态，再切到安装入口；安装完成后回到运行入口明确结束维护、刷新目录。因为本镜像实际始终关闭发送且没有输出队列，板级调度可据此确认静默；未来物理发送器接入后必须改为真实停止和排空回执。

工作器关键推进／采样错误会撤销发布、标记不可用；观察者不能据旧描述继续宣称运行正常。无自动重播、自动抢权或自动刷写。

## 构建与证据

```sh
STAGEMASTER_DEVICE_CONFIGURATION="$PWD/data/DEVICE-003/runtime-build-validation/device.smddev" bash tools/hardware/firmware.sh runtime-application-check
STAGEMASTER_DEVICE_CONFIGURATION="$PWD/data/DEVICE-003/runtime-build-validation/device.smddev" bash tools/hardware/firmware.sh runtime-application-build
```

上例是本项目已生成的虚构编号 `01010101010101010101010101010101` 构建夹具，只验证编译，不能刷入实际板卡；设备启动会核对真实编号。实际设备已另用 `data/DEVICE-003/runtime-development-access` 对应真实编号的 v2 配置完成受控构建／刷写；夹具与实际配置不可混用。产物位于权限 0700 的 `target/esp32-runtime-application/`，嵌入开发密钥，保持本机且不提交；旧安装凭据未修改。

主机测试直接引用实际固件 `gate.rs`／`protocol.rs`，使用真实安全会话、包、安装器和运行工作器；覆盖旧安装、软件运行、观察权限、队列失败、保活、撤销和错误计时。Xtensa 严格检查及最终完整链接通过，镜像 808,704 字节，占 ota_0 的 25.71%。链接 `.bss` 192,468 字节已含 128 KiB 堆区与第二核栈，不能重复加总；不代表动态峰值。主核／工作器静态任务槽、堆和实际调用栈仍需实板采样。保留裸机链接 RWX 告警，原工具链记录见[硬件工具说明](../../tools/hardware/README.md)。

第七增量未进行实板验收；后续真实射频／28 节目启动恢复、原生运行控制与断线自主进度已在[第九增量](../development/tasks/DEVICE-003-runtime-board-acceptance.md)取得证据。600 秒 USB 观察不是完整峰值或长时稳定性证明，诊断 LIVE tick 不是节目实际帧时序。真实 UART DMX 和完整整机出口仍未完成。
