# PRODUCT-ADR-165：独立预演组件的文件权限边界

状态：2026-10-05 **边界决定已记录，具体发行权限组装尚未实施／验收**。关联 [PREVIS-004](../tasks/PREVIS-004-packaged-renderer.md)／[预演契约](../../module-api/previsualization.md)。本会话依据实际运行证据维护方案，不等待普通审批、不向其他会话自动发送消息；用户需要时可交 Astra 复审。

## 实际问题

已有桌面启动分支向独立渲染组件传入宿主的本地数据根，再指定 UserDir／缓存／日志。编辑器开发路径可用，不代表独立签名 Game 可写该根。

UE 5.8 UAT 默认生成的 Mac Game 签名带 `com.apple.security.app-sandbox=true`、网络 client／server 和调试资格。实际 Game 正常初始化、运行必需资源与 H264 用例，但无法写项目 `logs/` 和 `tmp/` 的报告。系统记录明确为所属 PID 94036 的 `deny(1) file-write-create ...runtime-assets-json/index.json`；预建报告目录仍拒绝。因此不是只缺目录或尚未运行测试，不能把退出 0／Success 文本当作完整自动化通过。原包还缺官方报告 HTML 模板，需单列处理。

证据：`logs/previs-004-sandbox-events.log`、`logs/PREVIS-004/previs-package-dGdEOa/runtime-assets-json-process.log`；实际签名只读检查保留在交付证据中。源码／资源／原生 GPU 语义不因报告权限而改变。

## 决定

1. **不关闭 App Sandbox、不传 noEntitlements、不静默切回编辑器，不用签名绕过来迁就验收。** 原工程桥的本机认证、连接身份、单观看者、命令禁用、Rust 时间／输出权威继续保持。
2. 渲染器不需要读完整工程或媒体，只消费原只读投影／查看输入。文件资格只覆盖它确实需要的运行缓存、日志和独立验收报告；不能因此获得工程、导航目录、商业密钥或真实设备的读写权。
3. 开发验收的项目内目录与客户安装的宿主数据目录分开组装。下一增量先验证官方支持的目录资格／子进程继承条件与实际签名，再定最小权限实现；不得把带本机绝对路径的开发资格当作客户通用配置。未验证前保持 PREVIS-004 的独立报告及 H5 客户包未完成。
4. 跨进程数据目录／权限变化在本 ADR 下明确范围、所有者、失败和退出行为后实施。权限失败应定位、回收所属组件并保持工程／播放器，不改变播放权威或降级成假成功。

## 已核对与未决定

- 官方 UAT 能生成烘焙 Game，已有模块足够；本轮无需新增视频编码库、插件依赖或改 UE 引擎。
- 新生成包运行实际检查，不缺 AVCodecsCore 二进制；冗余启动 CVar 过早读取反射枚举导致的断言已由独立 Game 正常启动反证收敛，属于原私有配置修正。
- 默认沙盒组装不能满足当前宿主目录；共享目录／权限是否采用继承、签名应用组或明确受限资格，需真实符合条件的组装验证，不猜用户拥有正式签名团队、App Store 发行需求或授权密钥。
- Apple Xcode／Foundation 系统自管理临时脚本不服从项目 TMPDIR。已向用户询问 OS 临时文件例外，未获答复前拒绝新的 Xcode 构建；不修改系统临时目录，不因该待定例外停掉独立软件检查。
- 2026-10-05 接续只读核对了 Apple 的[限定目录临时资格](https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/AppSandboxTemporaryExceptionEntitlements.html)及本机 UE 5.8 `XcodeProject.cs` 的 `PremadeMacEntitlements`／Shipping 分支。仅开发验收的实例日志、用户缓存与报告目录可以作为具体方案候选，不能给整个项目根或当成客户通用权限；目录写资格与 Xcode 系统临时文件例外是两件事。尚未生成／启用这些资格或验证签名，不以源码阅读宣布沙盒问题修复，也不提前选择客户签名团队或 App Store 路线。

成熟机制参考：Apple 的 [App Sandbox 数据保护与子程序说明](https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox)、[沙盒文件访问](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox)与[子进程继承资格](https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/EnablingAppSandbox.html)。这些是可复用条件，不代表本桌面组装已经满足；具体结论仍由实际签名与读写拒绝验证。

## 2026-10-05：已有 Development 包的受限副本验证

基线 `47e24da`。只读确认了当前四项资格（Sandbox、网络 client／server、get-task-allow）与本机 UE AutomationController 的导出路径：模板来自 `EngineContentDir()/Automation/Report-Template.html`，JSON独立尝试写指定 ReportExportPath。**新 Xcode 构建暂停不等于已生成包不可验证**；本次先实施现有 Game 副本的限定目录方案，不重建、不关闭沙盒，不猜客户签名条件。

1. 本次本地 ad-hoc 签名副本保留原四项资格，唯一增加 `com.apple.security.temporary-exception.files.absolute-path.read-write`，值恰为该实例的用户缓存／派生缓存／临时／日志／报告五个专用目录。所有目标先创建并验证真实路径，禁止父目录或项目外链接；不开放原工程、源文件、最近目录或完整项目。具体绝对路径记录在副本证据，不写成客户通用配置。
2. 原包先验证签名和静态依赖并保持字节；只复制到新实例，官方模板以来源哈希保留在副本。仅重签该副本主程序／资源封套，不重签或修改嵌套库，也不操作系统权限。日志、报告和进程退出归本工具所有，失败不会接入正式桌面或覆盖原包。
3. 真正资格须由副本的非编辑器 JSON／HTML成功和另一非授权 sibling 目录明确拒绝证明；负探针后回到授权目录验证恢复。保留原测试断言／report 校验，不能只靠签名字段或文字 Success。未知额外文件需要先定位必要性，不逐次扩大父路径来消除警告。
4. **这是 Development 自动化文件资格，不是客户发行决定**。App Group／父子继承／签名团队与宿主目录、临时文件例外、最低系统和 GPU 仍单列；先拿真实证据，再选择后续组装。现阶段不增加 inherit，因为当前父进程与具体条件未满足，也不做 App Store 声明。

Apple 官方目录资格说明规定绝对路径及目录尾斜线；本地 codesign 工具可对已生成副本组装相应资格，无需再次调用 Xcode。是否真正有效由实际签名／读写结果验证；即便开发探针通过，也不能把本机绝对路径发给客户或解除其他门槛。关联范围和验收见 [PREVIS-004 接续](../tasks/PREVIS-004-packaged-renderer.md#接续development-副本的限定文件资格)。
