# PREVIS-010：Mac 桌面发行前的只读静态检查

状态：ready，2026-10-05。基线 main `cb224a8b0471ce54e20b00f9d6a1ca0c8f9042a1`，主工作区单写者；计划／[ADR-169](../decisions/PRODUCT-ADR-169-macos-distribution-preflight.md)先纳入版本再实现。PREVIS-007／008／009有限内部资格已通过，完整客户权限／发行仍未通过；保护用户output/及既有包／工程／证据。

## 一个问题与可用增量

现有内部包即使`codesign --verify --deep --strict`通过，也没有客户发行资格；桌面／Game为ad-hoc，Game带调试权和五个开发机绝对目录。包内Node实际上保留官方Developer ID签名／Hardened Runtime／时间戳，不能把它误写成未签名或重新签名来凑一致Team。需要可复现、只读、明确失败的发行前诊断，不继续凭文件存在／内机运行或外层签名通过判断客户可用。

- 只在`tools/previs/`新增按签名解析、纯规则、实际采集／CLI、对应测试职责分开的少量小文件。复用现有路径保护、Mach-O／版本解析、Game静态闭包、plutil和codesign；不改Rust、UI、UE、固件、运行时格式／API／时钟／控制权、既有组装或签名。
- 输入唯一项目内`.app`，按实际Info.plist确定桌面入口；精确检查桌面、执行宿主、包内Node和Game四个入口。链接／越界／缺件／错误角色、非Mac ARM64、元数据畸形或工具故障拒绝，不用系统Node／编辑器补齐。
- 采集严格签名与独立Apple锚＋Developer ID Application证书要求的实际校验，不能仅凭Authority文本／Team相同／进程退出0。记录ad-hoc、Team、Hardened Runtime、安全时间戳、调试权和绝对路径临时文件资格；第三方合法独立Team不自动拒绝，不选择用户证书。
- 核对桌面声明最低系统不低于四入口与Game静态依赖最高要求。内部验收身份、调试资格、固定绝对路径或缺少Developer ID先决条件均明确阻止此候选预检。未知／重复／缺失元数据不获得合格状态，中文逐项问题可定位到角色与路径。
- CLI仅标准输出JSON，阻止／工具失败退出非零；通过也只称`static-prerequisites-passed`，**始终不是客户发行通过**。报告明确仍需正式Shipping来源、最终权限方案／签名团队、所有嵌套代码／运行期加载、公证／Gatekeeper、许可、移位与无编辑器客户环境／GPU和代表任务。这个增量不选发行渠道或更改权限路线。

## 实际验收

先以旧“严格签名就够”的接缝执行红灯，原断言不变再实现。涵盖ad-hoc与Developer ID文本假象、实际证书要求失败、缺Team／Hardened Runtime／时间戳、get-task-allow真值与畸形值、绝对目录（含只读）、缺／重复角色、最低系统不足、相同与不同合法第三方Team、不由静态通过推导发行资格、缺件／链接／故障拒绝。当前全部previs Node保护、相关语法／Prettier、引用／严格JSON／diff实际执行；没有运行时源码变更不重复Rust／UI／UE全量。

对009原资格包实际只读执行CLI，保存四角色原始工具输出／结构化阻止报告、退出码及包前后清单。复核Node原签名和Game限定资格的真实差异；旧包／证据、原Game／Node／受保护工程保持。不得重签、重建、关闭沙盒、安装／读取用户私钥、提交公证、联网发布、启动Game／音乐／设备或改变全局配置。数据／日志留`data/PREVIS-010/`／`logs/previs-010-*`，临时测试仅项目tmp。

完成后自审、更新STATE／工单并提交。客户包仍是未完成门槛，不将本报告作为最终发行清单或许可结论；继续H1–H5，Xcode系统临时例外和物理测量条件未答复时仅暂停相应部分，不扩H6。
