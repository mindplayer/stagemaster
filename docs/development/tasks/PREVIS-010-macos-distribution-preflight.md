# PREVIS-010：Mac 桌面发行前的只读静态检查

状态：**有限只读诊断增量完成，自审通过**，2026-10-05。基线 main `cb224a8b0471ce54e20b00f9d6a1ca0c8f9042a1`，主工作区单写者；计划／[ADR-169](../decisions/PRODUCT-ADR-169-macos-distribution-preflight.md)先提交 `bdcfd4a8b8a6b2859107a74de71e6d1cfb924ca9`，实现结果 `64e2a6aaa61aa21b76450ae5bc73ffe5e9ee590b`。PREVIS-007／008／009有限内部资格已通过，完整客户权限／发行仍未通过；用户output/及既有包／工程／证据保持。

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

## 实际交付与自审

新增签名解析、纯候选规则、只读采集／CLI和三份独立测试，共六文件，44／125／254及119／209／69行。路径／角色／执行位／工具返回与资格畸形均先拒绝；原始采集含严格签名和独立原生证书要求校验，不信任Authority文字。CLI接收单一项目内`.app`，只输出报告，不创建第二播放器、组装器或签名器；静态通过始终带`releaseQualified:false`。

- 实际初始22项：7通过／15断言失败，原22项实现后全通过。首次相关29项因文件URL根路径尾分隔符出现5失败，规范化根路径后通过，未放宽越界／链接保护。自审新增入口执行位、畸形重复CodeDirectory和真实Node调试XML保护，实际30通过／2失败后修复。最终全部previs **167项／167通过，32项新增**，无跳过。
- 六文件`node --check`与已安装项目缓存Prettier 3.9.9实际通过；错误`.bin/prettier`路径退出127保留，没有安装／下载新格式工具。25份实际采集的独立镜像／原生命令记录通过严格JSON读取；大汇总证据不冒用产品文件预算。源码／已暂存差异检查通过；交付文档引用和四份既有JSON配置另存`data/PREVIS-010/delivery-checks.json`。
- 原资格包 `data/PREVIS-007/previs-desktop-Q7O6hY/舞台大师 内部验收.app` 实际CLI退出 **1，20项候选阻止**。四入口严格签名均退出0；独立Developer ID原生要求Node退出0，桌面／执行宿主／Game各退出3。Node Team `HX7739G8FX`、runtime与安全时间戳存在，**实际`com.apple.security.get-task-allow=true`仍在，不能据证书有效宣布公证可用**。Game保留沙盒／原调试权／五个限定绝对目录，桌面／Game内部身份被定位。第三方不同合法Team不被自动拒绝，不重签Node、选择证书或删除资格来凑通过。
- 桌面声明14.0满足四程序与Game静态闭包最高14.0，Game6镜像／66边保持；这不是旧系统或运行期加载验收。原正式debug `.app`实际缺`Contents/Resources/previs/node`，退出1且标准输出为空、中文错误定位，不回退系统Node／编辑器，不启动进程。

**绝对路径临时文件资格阻止是本项目可移动客户候选的保守策略，不是Apple对所有此类资格的普遍禁令**；最终发行目录／权限仍需独立决定。正向静态全绿仅在单元用例成立，本轮实际包被阻止，客户发行仍未通过。

最终证据 `data/PREVIS-010/verification.json`、`native-preflight-complete.json`；日志 `logs/previs-010-all-node-complete.log`／`logs/previs-010-format-complete.log`／`logs/previs-010-native-complete.log`／`logs/previs-010-verification.log`及全部首轮失败。`data/PREVIS-010/baseline.json`的redSourceHashes在绿灯补丁后采集，计数解析又误按TAP处理spec日志；原记录保持，`baseline-correction.json`明确采集阶段和实际22／7／15，不宣称有红灯源码哈希快照。

实际包前后清单／原Game32文件／Node2273文件／旧007资格包／受保护工程、默认最近目录和历史证据保持，009五Rust修复源码哈希保持。工具仅执行只读codesign／PlistBuddy／plutil与Mach-O检查，无签名／权限／私钥／公证／UE构建／原生启动操作。无Rust／UI／UE／固件源码改变，本轮不重复其编译／测试，不冒记新GPU、PCM、听音或设备通过。

下一步限定客户可移动目录／最终权限与签名／Shipping来源及完整资格；Node官方调试资格须作为真实问题保留。最终嵌套代码、动态加载、公证／Gatekeeper、许可、无编辑器客户GPU／代表任务仍未完成；Xcode临时例外／物理测量条件仍未答复，H1–H5完整goal保持active。
