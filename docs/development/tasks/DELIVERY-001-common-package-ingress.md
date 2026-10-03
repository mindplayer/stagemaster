# DELIVERY-001：云端／U 盘共同节目包导入

状态：本增量实现与软件验收完成，结果为本次 `feat(delivery): unify file and HTTPS package ingress` 提交。基线 `7ef847e`；main；当前 Astra 会话单写者。依据 [ADR-125](../decisions/PRODUCT-ADR-125-package-delivery-ingress.md)、ADR-100／101／097。完整框架轮保持 active。

## 本次范围

复用当前 Archive／Upload／Installer／Runtime，增加主机文件与 HTTPS 的统一有界导入和不可变读源，并接现有设备上传准备。写入范围：新增 `crates/stagemaster-delivery/`、工作区依赖、`stagemaster-device-upload` 包准备与专项测试、对应契约和状态。用户 `output/` 不改；无真实输出、刷机、云部署或用户工程操作。

## 验收出口

1. 真实文件与 HTTP/TLS 两来源完整校验得到相同内容身份；原文件移除、网络关闭后内容仍可独立使用。
2. 错误摘要、截断、额外字节、声明／实际超限、状态码／重定向／非原始编码、取消及超时拒绝，且不更改当前安装／播放。
3. 同一 Installer 安装两入口同版得到同一回执；新版本安装不替换已绑定运行快照，正常重开可读取提交。
4. 两入口均复用播放许可，拒绝许可不启动；目标能力不足时原上传入口拒绝，未向设备写入。
5. 相关自动回归、严格 Clippy／fmt、依赖范围及文档检查；新增依赖单独获取，后续锁定离线检查。无界面更改不重复桌面原生操作。

本项先验证受限灯光包交付；完整媒体发布、云目录／账号、生产签名授权和正式桌面导入入口仍须后续按范围集成，不以软件测试冒充已上线云服务或实物 U 盘测试。

## 实施记录

新增 Package／Incoming、文件与可选 HTTPS 适配，设备上传 Prepared 复用不可变内容；保持原 API／目标检查和安装／运行核心不变。标准 reqwest 0.13.5 使用 Rustls，默认构建不启用 HTTP；测试用 tokio-rustls 和 rcgen 生成本机短期证书，生产客户端不加入测试 CA。依赖单独获取记录在 `logs/delivery-001-{dependencies,test-dependencies-final}.log`；已有锁定包版本保持，无第三方升级。

初轮测试夹具使用长期证书，被 macOS 标准校验拒绝；改为测试运行时用 rcgen 生成短期 CA／服务证书，未关闭证书或主机名校验、未修改系统信任。另将普通成功路径的测试下载期限恢复为生产默认 30 秒，短超时仅留给明确挂起的超时用例；原 400 ms 在首次平台验证时会误超时。跨边界用例发现既有维护流程保留操作者控制权，测试改为使用原有效控制权显式载入新版本，不修改 Runtime 规则或自动抢占。此前失败日志保留。

## 最终验证与审查

- 交付模块启用 HTTP 的 8 项测试通过，见 `logs/delivery-001-verified-ingress.log`；包／上传／安装／存储／Runtime 的 61 项相关测试通过，原有子进程辅助用例忽略 1 项，见 `logs/delivery-001-related.log`。共 69 项通过，未重跑全工作区测试。
- `cargo check -p stagemaster-delivery --features http --locked --offline`、全工作区／全部目标严格 Clippy（`--features stagemaster-delivery/http`）及 fmt 检查通过，见 `logs/delivery-001-{check,clippy-final,fmt}.log`。首轮 Clippy 的 `manual_let_else` 已修复；未放宽 lint。
- 生产依赖树确认默认桌面／设备上传未引入 TLS／测试证书库，Runtime 没有交付／网络依赖，见 `logs/delivery-001-{upload,desktop,runtime}-tree.log`。新增生产文件最大 102 行；原上传任务测试 321 行仍围绕同一安装权限／能力流程，无超过 500 行的本次手写代码。
- 文件／HTTPS 两种顺序导入同内容，使用同一 Installer 得到同一提交；来源删除／关闭后运行 400 个完整 DMX 软件帧与原 Player 一致。新安装不更改原播放实例或控制权；需原操作者明确停止、维护绑定、载入与开始。拒绝播放许可和目标能力不足仍在原边界阻断。
- 无 UI 变更，不重复桌面打包／原生操作；未验证实际云服务、实物 U 盘、物理声卡／灯具、固件或 UE。用户 `output/` 与工程保持。
- 7 份变更文档的 412 个本地链接目标、锁文件基线版本保留及 `git diff --check` 通过；状态与执行计划指向整体框架审查。

审查结论：共同入口复用原格式、身份、安装及运行语义，可集成。整体框架轮仍需审查；正式桌面交付入口、云账号／目录／已签发布描述、大型媒体依赖与生产授权按后续任务接入，不把本增量当成已上线云商业系统。
