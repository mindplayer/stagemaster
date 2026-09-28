# DEVICE-002C：运行维护与存储工作器集成验收

日期：2026-09-28；基线 `a35eab8`，结果为本次 `feat(device): enforce runtime maintenance around installation worker` 提交。主工作区，当前会话单一写入负责人。沿用受控固件测试授权，禁止真实输出／eFuse；用户工程和 UE 不参与测试。

## 实现与边界

按 [ADR-041](../decisions/PRODUCT-ADR-041-maintained-install-worker.md) 接入 [ManagedWorker](../../module-api/maintained-install-worker.md)：组合已有 Runtime／Worker，每条同步安装命令进入有效维护窗口；运行／暂停／结束／等待静默确认时均不能写存储。精确静默确认释放旧计划及读源；未完成、失败、待确认事务不能退出维护；结束时校验最新持久目录，不自动选择／载入／执行，并撤销旧安装连接。

ESP32 存储核已改用组合层，启动参数要求主任务持有 OutputDisabled。当前硬件适配 GPIO21 始终低电平、无真实输出队列；实际生产输出静默确认仍未实现。固定本地主体仅存在编译测试探针，公开能力依然只有诊断，认证为 0，没有新增无线写入口。

## 软件和目标检查

- `cargo test --workspace --locked --offline`：347 项通过；最终 `stagemaster-install-worker` 定向 19 项通过（其中新增维护用例 8 项）。全工作区 fmt、严格 Clippy、固件 fmt、Python 探针语法检查通过。
- 八项覆盖：未确认时零写；运行／暂停拒写而不停止；已结束节目须显式停止；静默确认释放读源；未完成／失败事务须取消；待确认提交须对账；坏快照恢复失败保留维护；旧连接／旧确认／时钟倒退及撤销后的副作用核对。若同一测试覆盖多条边界，上述数量不重复累计。
- worker-readiness、worker-write-test 的真实 Xtensa release 构建及严格 Clippy 通过。保留已有 RWX 段告警；没有以关闭检查掩盖失败。锁文件只补本项目内 runtime／package 依赖，没有升级第三方库。
- 初次维护测试错误地假设待确认提交必然令 Upload::accept 返回错误；现有协议明确允许接收 Uncertain 后自动对账。修正为严格检查远端 Uncertain、事务阶段和无终态，保持“维护不得结束”的断言；首轮失败日志保留。
- 首次更新固件锁文件的检查在根目录执行，未加载固件目录的 Xtensa target 配置，错误使用 aarch64-apple-darwin 并失败。随后使用既有 firmware.sh 从固件目录执行原 locked/offline 检查和构建通过；未改变依赖来掩盖此执行错误。

日志：`logs/device-002c-maintenance-{tests,tests-final,targeted-final,workspace-tests,workspace-clippy,final-clippy,lock-check,firmware-build,firmware-clippy,test-build,test-clippy}.log`。`tests.log` 与 `lock-check.log` 保留上述首次失败。

## 真实 ESP32

先只读搜索确认当前唯一 StageMaster 诊断设备，未发配对请求。随后刷入 693,184 B 本地测试镜像，沿用现行双槽分区。初始 A 第 7 代／B 第 8 代；106,825 B 的 27 场景包持久提交为 A 第 9 代，B 第 8 代保留。

- 27 个节目各 400 帧，共 10,800 帧，与此前电脑参考摘要完全一致。
- 实际退出维护绑定 27 个目录项；没有所选／载入／运行实例。Operation 下的安装 Open 被拒绝，然后通过显式本地控制请求再次进入维护并释放目录。探针断言与日志 `INSTALL MAINTENANCE CYCLE PASS` 通过。
- 75 秒观察中 204 次诊断保活通过，最大往返 649.612 ms、P99 139.709 ms；测试前后堆使用均为 41,080 B，剩余 89,992 B，分配器观测峰值 52,884 B。
- 组合工作器 2,792 B；命令／完成为 1,288／1,292 B，端点 2,632 B。32 KiB 存储核栈，地址采样深度 17,341 B；这不是 ROM 嵌套或整任务栈的高水位证明。
- 本地 20／244 字节两种分片路径通过；244 字节路径 109 条请求、526 个输入片段／109 个输出片段。108 次写、28 次擦除；单工作操作观测最大 501,123 微秒。仍不是已认证无线安装吞吐测试。

记录：`data/DEVICE-002/maintenance-cycle.json`、`logs/device-002c-maintenance-{test-flash,board}.log`，电脑参考 `logs/device-002c-venue-next-reference-frames.log`。

测试后已恢复新版 **552,656 B 只读** worker-readiness 镜像并实际重验：A 第 9 代／B 第 8 代摘要正确，零写／零擦除，165 次保活通过，最大往返 139.649 ms、P99 139.290 ms，堆回到 41,080 B；只读栈地址采样 15,709 B。记录 `data/DEVICE-002/maintenance-readonly.json`、`logs/device-002c-maintenance-{restore-flash,readonly}.log`。当前板卡不会自动安装测试包。

该轮成功不覆盖此前压力失联的失败记录，也不能证明长期无线稳定性、真实拔电或物理 DMX 时序。没有配对、eFuse、输出、部署操作。原生程序 PID 44624 仍显示圆弧剧场已保存、三项播放包选择保留；没有编辑／保存用户工程。

## 文件规模与后续

新增产品文件分别为 128、52、12、60 行。测试数据／存储适配独立为 140／94 行；维护集成测试 322 行，八项紧密相关边界集中保留，未把夹具或产品实现塞入测试主体。既有 Runtime 大文件未新增实现，控制规则复用原代码。

本轮维护集成可交付；完整 DEVICE-002 未完成。B 的新系统配对确认仍待答复，候选正向认证、绑定持久化／撤销和受限业务会话未完成；C 正式已认证 GATT、D 实际桌面下发／取消／恢复及整链故障／资源验收依赖这些结果。当前不再扩充旁支替代真实链路，阻塞核对见[完整目标审查](DEVICE-002-blocking-audit.md)。
