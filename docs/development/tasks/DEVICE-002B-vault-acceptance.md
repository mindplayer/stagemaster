# DEVICE-002B：独立绑定档案与持久提交

日期：2026-09-29；基线 `d8692f3`，结果为本次 `feat(auth): add bounded persistent device binding vault` 提交。主工作区，当前会话单一写入。

## 本轮实现

按 [ADR-042](../decisions/PRODUCT-ADR-042-device-binding-vault.md) 实现独立 `stagemaster-device-auth`：固定容量绑定档案、稳定本机身份记录、严格编码、秘密字段 Debug 脱敏及自有缓冲清零。新增／更新／撤销只生成提案；持久化适配采用锁定的 EKV 1.0.0、CRC、显式空白初始化、配置检查、事务提交／回读及出错撤回缓存。接口见[模块说明](../../module-api/device-binding-vault.md)。

上游源码核对发现挂载候选元数据时可能忽略底层读错误，已经在本项目适配器增加 I/O 错误锁存与拒绝使用规则。错误后只允许显式恢复，不能用缓存旧绑定继续授权。当前默认模块不依赖存储；EKV／NOR 位于可选 persistence 特性，不将平台蓝牙、UI 或商业许可混进来。新增产品文件最大 143 行。

## 实际验证

- 352 项 Rust 工作区测试、工作区 fmt 和严格 Clippy 通过。
- 额外显式启用 persistence：11 项测试及严格 Clippy 通过。其中 5 项记录测试也包含在 352 项里，不能重复加总为 363。
- 127 个撤销存储操作点 × 写前／部分写／写后错误；652 个初始化操作点 × 三种错误；不确定结果只能恢复为完整旧／新状态或明确不可用，不报告假成功。
- 40 轮新增／撤销后，逐已用页前 32 字节单比特损坏均未恢复已撤销主体；182 个恢复读取点分别故障均拒绝授权快照，恢复可读后重新核验通过。这是列明故障模型的证据，不是任意恶意 Flash 回放的证明。
- 空白恢复零写、已有／部分格式禁止重建、过期提案不污染缓存、重复身份／主体、密钥更新、容量和修订耗尽、未知版本／保留字节／截断与调试脱敏通过。
- 用项目 Xtensa 工具链对真实 `xtensa-esp32s3-none-elf` 目标执行库级 check，persistence 开启且 locked／offline，通过；没有将主机编译冒充目标编译。没有完整新固件链接或板级资源实测。

记录：`logs/device-002b-vault-{first-check,first-tests,compaction-test,first-clippy,clippy,tests,workspace-tests,workspace-clippy,xtensa-check,final-tests,final-clippy}.log`。初次严格检查因缺失 panic 文档与测试冗余返回失败；已消除不必要 unwrap 和冗余返回，通过原严格命令，没有关闭 lint。依赖从官方 crates.io 获取 EKV，见 `logs/device-002b-ekv-info.log`；锁文件新增六个依赖，没有升级现有依赖。

## 当前状态与下一步

本轮尚未把档案接到真实 ESP32 Flash、蓝牙栈事件或正式会话；没有修改分区表、刷机、断开用户连接、操作工程／UE 或输出灯光。现有本机安全配对确认继续有效，不重复索要同一确认。

原 DEVICE-002 全目标仍未完成。接下来完成绑定准入／取消／撤销与连接级认证状态，再接同一存储执行器、真实绑定持久化与重启核验，最后把已认证会话接入已有 GATT 分片、安装维护与桌面任务。芯片生产保护、加密静态存储和商业离线授权仍按原独立范围推进。
