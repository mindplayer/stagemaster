# PRODUCT-ADR-166：预演组件选择与运行目录归属

状态：2026-10-05 **决定，限定实施／验收中**。关联 [PREVIS-006](../tasks/PREVIS-006-component-launch-isolation.md)、[ADR-165](PRODUCT-ADR-165-packaged-renderer-file-access.md)。不改变公共工程、渲染协议、节目时钟、输出权威或模块依赖。

## 证据

renderer.rs 当前仅在 Game／node／signalling.mjs 全部存在时选独立组件，否则 debug Mac 可使用 UnrealEditor；安装但缺 node_modules 的组件仍被选择。完整组件根直接采用 Tauri app_local_data_dir，与 recovery.rs 已有项目内 data／STAGEMASTER_ACCEPTANCE_INSTANCE 的 tmp/desktop-<实例> 不一致。tools/desktop/run.mjs 正式本机打包使用 --debug，隐式回退确实影响验收可信度。

## 决定

1. 显式已安装组件具有选择优先级，也承担完整性责任。previs 目录确实缺席时保留原开发回退；目录或必需上下文存在但不完整／路径逃逸时，明确报告缺项或无效路径，**不隐式调用编辑器或系统 Node**。入口检查不是发行签名／全动态加载证明，实际信令启动及运行故障仍由原所属进程／界面错误机制处理。
2. 用小型宿主路径模块集中原数据所有者选择，恢复／执行／音频／导航不迁移、不改格式。预演持久 user／cache 位于该数据根的 previs 子空间；开发临时／日志仍位于项目 tmp／logs，验收实例位于其项目内隔离根。release 采用应用数据根的预演子空间，不要求客户有本项目或 UE 编辑器。
3. 预演只获得自己的目录，创建之前检查现有路径祖先／目标，不跟随链接到项目或实例外。宿主选择目录不是 OS 沙盒授权；资格／签名按 ADR-165 单独组装和真实验证，不扩大到完整工程、媒体、导航或商业密钥。
4. 部分安装或目录失败在组件进程启动前拒绝；错误不改变编辑文档或播放器。原桥服务、Rust时间／执行／控制租约、所属子进程回收与开发编辑器参数保持。若未通过当前正式桌面拒绝验证，本增量不得记完成。

这解决实际宿主启动边界，不提前建立组件商店、自动下载更新或新代理平台。来源核对使用本机实际 Tauri PathResolver、PREVIS-005发布内容及已有恢复入口；完整签名、客户目录可移动性、组合最低系统、发行许可和实际 GPU仍为后继门槛。
