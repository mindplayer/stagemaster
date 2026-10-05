# PRODUCT-ADR-169：内部包与客户发行前检查分开

状态：2026-10-05有限只读决定已实施并验证，结果 `64e2a6a`。关联[PREVIS-010](../tasks/PREVIS-010-macos-distribution-preflight.md)、[ADR-167](PRODUCT-ADR-167-development-desktop-assembly.md)、[PREVIS-009](../tasks/PREVIS-009-background-frame-busy.md)。不改变运行时公开API、工程或控制语义，不选定最终发行渠道／权限方案。

## 事实与成熟机制

009两轮实际独立Game／Node／音乐GPU通过，只证明内部本机资格。只读codesign显示桌面／Game为ad-hoc且无Team；Game保留App Sandbox、get-task-allow和五个开发机专用目录。Node是官方Developer ID Application: Node.js Foundation签名，Team为HX7739G8FX，runtime标志及安全时间戳真实存在；本地Apple锚＋Developer ID叶证书要求实际验证退出0。保留原Node身份，不凭外层ad-hoc推断所有嵌套程序无签名。

参考Apple[签名要求TN3127](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements)、[发行签名](https://developer.apple.com/documentation/xcode/creating-distribution-signed-code-for-the-mac)、[公证先决条件](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)和[临时文件资格](https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/AppSandboxTemporaryExceptionEntitlements.html)。复用本机codesign／plutil及既有Mach-O检查；独立证书校验使用Apple锚、Developer ID中间证书OID与Application叶证书OID，而非仅匹配Authority文字。不同合法第三方Team不等于坏包，跨进程权限／库校验仍需独立决定与验证。

## 有限决定

1. 先增加可用的只读Mac ARM64桌面静态诊断。输入仅项目内单包，检查四个现有入口及Game静态闭包，不创建第二组装／签名／启动器。
2. 结构完整、签名封套与真正Developer ID先决条件分别记录。签名失败、ad-hoc、缺Hardened Runtime／安全时间戳／Team、调试真值、绝对路径临时资格、内部验收身份／最低系统不足都报告中文定位并阻止候选预检；畸形／未知数据不默认合格。
3. 静态先决条件通过仍不授予发行资格，报告显式保留Shipping来源、完整嵌套代码／动态加载、最终权限、团队／签名／公证、许可证与真实客户移位／GPU验收。候选检查与完整商业出口分开；不增加占位产品服务或假的成功字段。
4. 工具只读现有包，CLI输出报告，拒绝退出非零。实际009资格包应被阻止，Node独立证书通过应被保留；这两点是正／负观察，不是静态客户包通过。
5. 不扩大签名资格、改沙盒、操作私钥／证书、重签原包或提交公证，不新建Xcode／UAT构建。后继真实可移动权限方案另记证据／决定，不在此诊断中偷偷实施。

本轮不是“已做客户打包”，也不把旧未解决事实转成通过。用户output/及原产物／失败证据保持；相关测试／实际只读报告保存后自审集成，完整goal active。

## 实际验证与决定边界

167 Node（32新增）、六文件语法／格式及原始采集严格JSON通过；只读原资格包四入口严格签名均通过，但CLI实际退出1、定位20项候选阻止。Node的独立Developer ID校验／runtime／安全时间戳有效，**实际调试资格get-task-allow=true也存在**；证书有效不等于符合公证先决条件，不凭外层ad-hoc误报Node无签名。桌面／宿主／Game仍为ad-hoc，Game五个固定目录及内部身份保持；原包／受保护记录前后清单不变，无新原生启动或签名操作。

全部绝对路径临时资格被此候选规则保守阻止，目的是阻止本机固定目录冒充可移动客户包；Apple本身提供这种临时资格机制，**不把项目规则写成Apple普遍禁令**。候选最低系统14.0只是四入口／静态闭包一致，不证明客户旧系统、动态加载或GPU运行。

证据 `data/PREVIS-010/verification.json`／`native-preflight-complete.json`及工单交付记录；初始红灯、自审失败、路径与证据记录错误均保留并解释，没有调整产品验收。最终权限／团队／Shipping／Node调试资格处置仍需下一项明确决定，不能因此自动重签第三方、移除沙盒、操作证书／私钥或提交公证。这个ADR只关闭诊断实现，不关闭客户发行与H1–H5。
