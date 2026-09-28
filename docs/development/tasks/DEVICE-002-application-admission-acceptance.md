# DEVICE-002：应用权限与安装工作器组合验收

日期：2026-09-29；基线 `9fd381a`；主工作区 `main`，当前 Astra 单一负责人。
结果：本次 `feat(device): gate encrypted installation on bounded application permission` 提交。此增量完成软件核心及目标库检查；DEVICE-002 全目标仍 active。

## 实施与审查

依 [ADR-050](../decisions/PRODUCT-ADR-050-application-installation-admission.md)，在原模块内增加可选的应用权限和 Gateway，不改变旧 LESC／工程／播放包／双槽格式。参见[接口与时序](../../module-api/application-installation-session.md)。新手写产品源文件最大 156 行，按许可、活跃会话、安装状态、发包、线格式分开。

开发许可核对受控配置中的设备、公钥、主体及修订，消费真实 Noise 相互确认后的通道；固定期限最多 10 分钟，认证保活不能延期，过期／撤销不可复活。它不是正式云端签发，也不改变当前文件临时 24 小时及到期收尾要求。

Gateway 仅在准入成功后返回 Open。工作器确认维护和打开完成后才发 SMAP 加密就绪回执，发完前拒绝业务；一条完整 SMP 消息／一个待发送密文／一个待回复心跳的有界状态，任何错误撤销。原子 LIVE_EPOCH 仍由物理连接所有者负责，不能把析构内部对象当作跨核自动撤销。

审查结论：可集成核心增量，不开放当前设备安装能力。旧系统绑定状态和新应用权限完全分开；稳定主体不从本机蓝牙定位符推导。云端可新增独立签名凭据验证适配，不能直接把开发许可改一个来源标签充当生产授权。

## 实际验证

- 工作区启用 `stagemaster-install-worker/application` 的完整测试 **413 项通过**（原 394＋新增 19）；`logs/device-002-admission-workspace-tests-final.log`。
- 默认命令 `cargo test --workspace --locked --offline` **397 项通过**，默认严格 Clippy 也通过；日志 `device-002-admission-default-{tests,clippy}.log`。默认与特性检查有重叠，不把两组计数相加；可选应用权限没有成为普通构建的强制依赖。
- 全工作区／全部目标严格 Clippy、fmt／差异空白检查通过。新增特性是显式附加覆盖，没有屏蔽原失败用例；日志 `logs/device-002-admission-clippy-final.log`。
- 新 Gateway／应用权限依赖链在 `xtensa-esp32s3-none-elf` 上 release 严格库编译检查通过；日志 `logs/device-002-admission-xtensa-final.log`。这是目标库检查，不是固件链接、刷机或栈高水位实测。
- 既有 `worker-readiness` 固件严格检查通过；日志 `logs/device-002-admission-firmware-check.log`。普通配置保持原有权限和只读行为。
- 软件组合使用实际 Noise 密码状态、已确认后连续的 20／244 B 记录分片、真实工程导出包、现有 Upload／Endpoint／ManagedWorker、文件双槽存储。安装后关闭并重开存储，摘要一致；维护未确认时拒绝且无写入，完成不会启动播放。
- 中断实际块写回执、丢弃提交成功回执后，重新握手得不同加密会话，再继续对账；已提交文件不重复写入。认证取消保留之前的已安装版本。
- 未确认、错误设备／启动／连接／控制端密钥、零主体／修订／无界时长、过期、时钟倒退、篡改、密文重放、多个待回心跳、未发送就绪、半条 SMP、旧完成、撤销队列工作均有拒绝验证。周期加密心跳能维持 6 秒安全租约，但在固定许可截止时仍拒绝。
- SMAP 与旧 SMAS 严格分离；新回执的版本、长度、权限、认证方法、预算、保留字节、全身份关联和期限不增加均检查。

## 失败与修正记录

最初严格检查发现两处文档调用未加反引号，已修正，日志 `device-002-admission-clippy-first.log` 保留。恢复测试首次编译误写 Cancelled 为带字段枚举，修正为实际无字段变体；随后取消测试直接载入格式规范样例，而该样例含当前产品明确拒绝的非空 entryPoints，测试因此在打开工程时失败。改为与既有维护测试一致的已支持输入（entryPoints 为空），继续真实导出／取消验证；未修改产品校验或削弱旧节目保留断言。

失败日志 `device-002-admission-integration-tests.log`、`device-002-admission-integration-tests-final.log`、首轮 `device-002-admission-workspace-tests.log` 均保留。修正后定向恢复两项通过、完整 413 项通过；没有用仅筛选成功测试代替完整重跑。

## 尚未完成／下一接线

本轮没有刷机、建立额外蓝牙连接、修改用户工程、重开 UE 或输出 DMX。板卡最后验收状态仍是上一增量的只读镜像 A 第 9 代／B Empty；此处未重新实测无线状态。

还须：设备专用开发密钥与可信控制端配置（项目内私密文件，拒绝公共默认值）、固件握手后准入／队列／独立撤销、共享记录发送与通知背压、桌面同连接保活与安装回执接入、真实 28 场景下发／取消／恢复／持久核验。结合真实 NOR 擦写验证 32 B 堆占用变化来源、栈及无线期限；不将上一轮纯回送吞吐视为安装吞吐。

云端认领／转移／撤销、防回退及 24 小时文件授权仍未实现。下一轮直接从此 Gateway 接入继续，不重做已经通过的纯回送实验，不以此核心增量关闭父目标。
