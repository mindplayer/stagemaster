# PRODUCT-ADR-171：优化代码验收与客户运行目录分开

状态：2026-10-05有限决定待实施。关联[DESKTOP-005](../tasks/DESKTOP-005-isolated-release-validation.md)、[PREVIS-010](../tasks/PREVIS-010-macos-distribution-preflight.md)、[PREVIS-011](../tasks/PREVIS-011-signalling-notices.md)。不选客户沙盒／签名团队，不改变正式数据所有权、工程或运行语义。

## 新事实

tools/desktop/run.mjs实际强制--debug，prepareHost构建／复制debug后台。Rust storage_paths的隔离分支仅cfg!(debug_assertions)，普通release会走Tauri app_local_data_dir；此前内部包真实验收未覆盖优化代码与release禁用编辑器回退。不能直接带现有实例环境启动普通release来假定不写用户目录。

成熟机制：复用Cargo[显式可选能力与cfg](https://doc.rust-lang.org/cargo/reference/features.html)和Tauri[build的配置／features／release](https://v2.tauri.app/reference/cli/)，本机已安装CLI的build --help确认默认release、--features、--config、--no-sign。只增加内部构建口，不写第二工程／播放器或签名平台；优化代码的内部验证与客户权限／发行分开。

## 有限决定

1. 默认关闭internal-acceptance，仅允许准确内部应用身份与实例匹配时为release选择原项目验收目录。内部身份缺实例／身份错误拒绝，不回退正式数据；普通release无此能力仍用原用户目录，debug行为不变。该可选能力只补隔离验证，不代表生产许可、输出或设备访问资格。
2. 现有构建入口增加明确内部release模式，原dev／build保持；桌面和原audio后台都以release构建，独立目标与sidecar避免覆盖已经资格通过的debug包或产物。配置保留现有窗口／安全策略，只标内部身份／标题并指向此后台。
3. 构建不选择用户证书、提交公证或修改Game沙盒；明确不执行正式签名的内部源包仍不能通过客户分发检查。这不是把失败改成成功，也不豁免客户Developer ID、Hardened Runtime、Shipping、许可和真实客户环境。
4. 真实优化原生包先完成无音乐编排／后台隔离与缺三维组件拒绝。无debug editor fallback意味着缺件应失败，基础编排和已准备运行不能依赖编辑器；不以mock／构建代替实际桌面验收。保持来源工程／默认目录／旧包，结束回收所属进程。

客户Game应使用何种沙盒容器／共享目录、最终身份、Node调试资格处置、正式签名与Shipping仍需原独立决定和真实资格。新UE／Xcode构建的OS临时例外未答复，不能自行设置授权开关；本轮独立Rust／UI工作可继续。完整goal active，不扩H6，不自动召回Astra。
