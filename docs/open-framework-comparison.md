# 开源框架比较：专业灯光单机系统与后续移动编排

核查日期：2026-09-10。状态：框架调查资料；未安装候选框架，未实现原型，未测量成品体积、性能或真实 DMX 输出。

> 用户于 2026-09-10 确认技术框架：[最终技术评估](final-technology-assessment.md)中的 E（Tauri 2＋Rust＋TypeScript，React＋Vite 界面），以及 Fastify＋PostgreSQL＋对象存储服务器端。下文保留比较过程，发生建议差异时以最终评估为准，不再作为待选清单。

用户要求：扩大候选范围、比较后由用户选择；尽量减少不同开发语言，降低依赖体积、集成错误和维护成本。现阶段在 macOS 开发，首先完成专业单机控台，未来覆盖 Windows、Linux，以及手机、平板的简单本地编排。

用户在讨论 Web 控制后再次明确：尽可能复用代码、减少开发语言种类。后续验证优先比较整个产品的共享代码和维护成本；同一语言内仍充分模块化，各端直接依赖共享模块，平台差异集中在适配层。若采用 E，默认收敛在 TypeScript 与 Rust 两种主要业务语言内，只有具体 SDK 或目标能力确有要求时再引入局部适配代码，不因新增终端而增加一套业务实现。

本轮调查 18 条框架路线，另记录 3 个基础库／输出框架。事实来自官方文档、维护者仓库及许可文本；“适合本项目”“优先考虑”等属于工程判断。本文不推断 MA3／老虎的内部技术，也不把支持某平台等同于已通过本项目的商业验收。

**如何理解“尽量少语言、少依赖”**

建议优先采用一种主要业务语言，确有收益时才增加第二种。QML、XAML、Slint、CSS 等界面描述单独列出；它们有维护成本，但不能简单按另一套通用语言运行时计算。构建脚本、平台模板和第三方库实现语言，也不等同于我们需要维护的业务语言。

多语言会增加绑定、类型转换、构建和调试边界，但成品大小更直接取决于附带的渲染引擎、语言运行时、编解码器、字体、CPU 架构和调试信息。只用一种业务语言，仍可能包含多种语言编写的第三方库。反过来，Rust 与 C 的小型静态库也不必带入两套庞大虚拟机。

要分别计量开发机上的 SDK／构建缓存、安装包下载量、安装后占用和运行内存。不能用开发目录里的依赖体积代替成品大小。例如 Electron 包含 Chromium 与 Node.js，Tauri 使用系统 WebView；Flutter 自带图形引擎与 Dart 运行时；Compose Desktop 的独立分发包含所需 Java 运行时模块。这些结构差异比手写语言的数量更能解释包内包含什么。[Electron 架构](https://www.electronjs.org/docs/latest/)、[Tauri 进程模型](https://v2.tauri.app/concept/process-model/)、[Flutter 架构](https://docs.flutter.dev/resources/architectural-overview)、[Compose 分发](https://kotlinlang.org/docs/multiplatform/compose-native-distribution.html)。

**八条值得用户重点选择的路线**

下表“五端”指 macOS、Windows、Linux、Android、iOS／iPadOS。表示有官方实现或文档，不表示系统版本、插件和功能支持完全一致。字母只用于选择，不是排名。

| 选项 | 主要业务语言与界面描述 | 平台覆盖 | 包内主要组成 | 对本项目的价值 | 主要代价 |
| --- | --- | --- | --- | --- | --- |
| A．Qt 6 | C++；移动共用界面优先考虑 QML | 五端 | 所选 Qt 模块、平台插件、渲染与媒体依赖 | 同一语言可负责核心和原生接入；适合专业桌面界面路线 | 需要管理 Qt 模块与许可，触摸布局仍需设计 |
| B．JUCE | C++，界面也可直接用 C++ | 五端 | 所选 JUCE 模块和平台依赖，可编译进应用 | 音频、设备、图形和界面同栈，适合把音频时间线纳入核心工作流 | 专业控台页面需要较多定制；闭源产品需评估商业许可 |
| C．Slint | Rust＋`.slint` 界面描述 | Rust 路线覆盖五端；iOS 当前只支持 Rust | 所选 Slint 后端、渲染器及字体等依赖 | Rust 核心与界面集成直接，可不引入 WebView | 需验证专业控件与移动操作；嵌入式产品许可需另看 |
| D．Dioxus | Rust 为主，RSX 宏＋CSS；特殊 Web API 可能补 JS | 五端及 Web | 原生 Rust 程序、WebView 集成、界面资源与内部脚本 | 界面与领域规则主要用一种语言，减少手写 TypeScript／Rust 边界 | 仍受 WebView 差异影响；复杂绘图与移动插件需验证；原生 WGPU 渲染器不能按成熟替代品假定 |
| E．Tauri 2 | Rust＋TypeScript；HTML／CSS | 五端 | Rust 程序、Web 资源、系统 WebView 集成 | 能使用 Web 界面生态，同时保留独立原生核心 | 明确需要两种主要语言，IPC 和状态同步需设计 |
| F．Avalonia | C#＋XAML，也可用 C# 组织界面 | 五端及 WebAssembly，支持等级分平台版本 | Avalonia、图形依赖、.NET 相关运行组件／AOT 产物 | UI、工程编辑和应用逻辑可统一 C#；主框架 MIT | 移动交互、原生 SDK 和时序需实测；自包含分发有运行组件成本 |
| G．Flutter | Dart 为主 | 五端及 Web | Flutter 引擎、Dart 运行时、原生插件 | 手机和平板与桌面共享 UI 的完整路线 | 成品含引擎；专业桌面多窗口与关键播放时序需单独验证 |
| H．Compose Multiplatform | Kotlin 为主 | 五端；桌面基于 JVM | 桌面包含 Java 运行时模块和图形依赖；移动端按对应目标编译 | 单一主要语言，共享业务和界面；移动端值得考虑 | 桌面运行时负担、硬件绑定和复杂桌面交互需评估 |

A 的平台与接口依据：[Qt 平台支持](https://doc.qt.io/qt-6/supported-platforms.html)、[Qt Quick](https://doc.qt.io/qt-6/qtquick-index.html)。B 的平台与模块化依据：[JUCE 维护者说明](https://github.com/juce-framework/JUCE)。C 的当前移动支持依据：[Slint iOS](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/ios/)、[Slint Android](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/android/)。D 的现有 WebView 路线和实验性渲染器依据：[Dioxus 移动端](https://dioxuslabs.com/learn/0.7/guides/platforms/mobile/)、[Dioxus 仓库](https://github.com/DioxusLabs/dioxus)。

E 的平台与宿主方式依据：[Tauri](https://tauri.app/)。F 的平台支持依据：[Avalonia](https://docs.avaloniaui.net/docs/supported-platforms)。G 的架构依据：[Flutter](https://docs.flutter.dev/resources/architectural-overview)。H 的官方稳定性矩阵将 Android、iOS、Desktop JVM 列为 Stable，但这不是我们应用的可靠性认证。[Compose 平台状态](https://kotlinlang.org/docs/multiplatform/supported-platforms.html)。

**八条路线的具体取舍**

**A．C++／Qt：专业桌面与原生核心统一。** 界面、工程和执行器可以全部使用 C++ 模块划分，QML 负责可复用的显示与交互。若只做桌面，Qt Widgets 可进一步减少界面语言；考虑后续触摸编排，应把 Qt Quick／QML 作为更值得验证的 UI 路线，而不是承诺 Widgets 原样搬到手机。引擎不必依赖 QObject 或 Qt 的 UI 类型。选择 C++ 也不等于必须使用全部 C++26 特性，应按目标平台工具链决定标准基线。

**B．C++／JUCE：尤其值得补充考虑。** 它是跨桌面与移动端的 C++ 应用框架，不只用于音频插件。对于音频播放、波形、时间线、MIDI 等需求，值得验证其现有模块能节省多少工作。核心与 UI 可以同为 C++，也不必引入 QML 或 Web 前端。代价是配适表、灯具选择、属性面板、执行器布局等专业界面仍要自行组织，不能把音频框架当作现成灯控系统。[JUCE](https://juce.com/)。

**C．Rust／Slint：少语言、原生界面的候选。** 采用 Rust 核心和 Slint 描述界面，不再加 C++ 的 Qt 适配层。后端选择会影响实际依赖；本项目应评估不引入 Qt 后端的配置。最新 iOS 文档明确支持 Rust 应用，不能继续引用旧的“iOS 尚在计划”搜索结果。iOS 路线使用 Winit 与 Skia，所以不能拿 Slint 在极小嵌入式设备上的运行时宣传数字来估算我们的桌面或移动安装包。[Slint iOS 实现](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/ios/)。

**D．Rust／Dioxus：重点比较的单一主要语言路线。** UI 与业务代码主要写 Rust，能减少我们手工维护两端数据类型和业务规则的机会。它的桌面／移动常规路线仍使用系统 WebView；Rust 编写界面不等于没有浏览器渲染或内部 JS。波形、Canvas／GPU、自定义输入和成熟 Web 组件的接入，可能使项目仍出现局部 JS 代码。应先用复杂控台交互验证现有 WebView 路线，不把实验性渲染器当成已经解决的方案。[Dioxus 桌面说明](https://dioxuslabs.com/learn/0.7/guides/platforms/desktop/)。

**E．Rust／Tauri：两种主要语言换取 Web 界面生态。** 它没有因用户要求少语言而被排除，但需要证明使用 TypeScript 的界面生产效率足以抵消接口和构建复杂度。核心保持独立，命令和显示状态流分开；通用事件系统不能承担输出主时钟。WebView 的平台差异需测试。[Tauri 通信说明](https://v2.tauri.app/develop/calling-frontend/)。用 Rust 的 WebAssembly 前端替换 TypeScript 也可以调查，但这会引入 WASM 与宿主边界，不能据此宣称整体系统更简单。

**F．C#／Avalonia：不默认再加一个 Rust 引擎。** 工程、编排与播放调度都可以先评估 C# 实现，只有真实时序指标或 SDK 要求证明有必要时，再引入原生模块。GC 的存在需要纳入最坏延迟和资源压力测试，但不构成“不能做灯控软件”的结论。图形、音频、USB SDK 等第三方原生依赖仍需审查。[.NET GC 延迟模式](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/latency)。

**G．Dart／Flutter：优先多端编排体验时有吸引力。** 业务和界面都可用 Dart，原生插件仅处理必须的平台功能。编译成本地机器码并不意味着没有 Dart 运行时，应用仍包含 Flutter 引擎。若本机需要严格的连续播放指标，应验证 Dart 调度、音频接口与设备缓冲的完整链路，再决定是否确需第二种语言；不预先强塞 Rust 核心。[Flutter 架构](https://docs.flutter.dev/resources/architectural-overview)。

**H．Kotlin／Compose：不应遗漏的多端路线。** 可以主要用 Kotlin 编写业务和 UI；桌面与移动端共用源代码，但不共用完全相同的运行环境。桌面分发会包含所需 Java 模块，可通过打包工具裁剪；这与“纯原生、没有运行时”的路线不同。项目若把桌面体积和低层设备接口放在最高优先级，应与 A／B／C／D 同场比较。[Compose 分发与裁剪](https://kotlinlang.org/docs/multiplatform/compose-native-distribution.html)。

**其他十条已调查路线**

| 框架 | 主要语言 | 平台与当前边界 | 本项目暂不优先的原因 |
| --- | --- | --- | --- |
| Uno Platform | C#＋XAML／C# Markup | 桌面、移动、Web；Apache-2.0 | 可以选，但同属 .NET 路线，先与 Avalonia 比较控件、图形与原生接入，不同时引入两套 UI。依据：[平台文档](https://platform.uno/docs/articles/getting-started/requirements.html)、[开源范围](https://platform.uno/platform/) |
| .NET MAUI | C#＋XAML | Windows、macOS（Mac Catalyst）、Android、iOS；官方目标未列 Linux 桌面 | 对未来 Linux 的要求不如 Avalonia／Uno 直接。依据：[官方平台列表](https://learn.microsoft.com/en-us/dotnet/maui/supported-platforms?view=net-maui-10.0) |
| Electron | TypeScript／JavaScript | 三大桌面；随包包含 Chromium 和 Node.js | 不直接覆盖手机 App，且与减少分发依赖的目标较难兼顾。依据：[官方介绍](https://www.electronjs.org/docs/latest/) |
| React Native | TypeScript／JavaScript，原生模块按平台 | Android／iOS 主线，Windows／macOS 由其他维护项目提供 | 移动端有价值；我们当前先做专业桌面，额外桌面实现增加验证面。依据：[桌面等扩展平台](https://reactnative.dev/docs/out-of-tree-platforms) |
| Fyne | Go | 覆盖桌面与移动，构建依赖 C 编译器接入图形；BSD-3-Clause | 一种主要语言值得关注，但专用控台控件、时序与图形需求缺乏本项目验证收益。依据：[开始文档](https://docs.fyne.io/started/quick/)、[许可](https://github.com/fyne-io/fyne/blob/develop/LICENSE) |
| Wails | Go＋TypeScript／JavaScript | v2 面向桌面；v3 已有 iOS／Android 文档，主页仍标 v3 为 Beta | 两种主要语言，移动路线还要承担 Beta 阶段验证；不能简单说它永远不支持移动。依据：[Wails 状态](https://wails.io/)、[v3 移动端](https://v3.wails.io/guides/mobile/) |
| egui／eframe | Rust | 官方 eframe 列出 Web、三大桌面、Android，未把 iOS 列入同一支持清单 | 适合调试工具、编辑器探索；未来 iPad 路径需要额外证据。依据：[维护者说明](https://github.com/emilk/egui) |
| Iced | Rust | 当前 README 列桌面和 Web，并自称实验性软件 | 在手机、平板与商业交付要求下，承担的框架验证工作较多。依据：[维护者 README](https://github.com/iced-rs/iced) |
| Kivy | Python＋KV 描述 | 有桌面与移动路线，MIT | 编排 UI 可以探索，但若为关键执行另加原生核心，会回到用户希望减少的多语言边界。依据：[官方介绍](https://kivy.org/doc/stable/gettingstarted/intro.html)、[仓库](https://github.com/kivy/kivy) |
| SDL 3＋自研界面 | C／C++ | 官方支持五端，zlib 许可 | 它主要提供窗口、输入、图形与音频底层能力；需要自行建设大量 UI，不应与完整应用框架视为同等工作量。依据：[SDL 官方介绍](https://www.libsdl.org/) |

**商业销售时的许可差异**

开源与可销售并不冲突；是否能闭源、如何分发和是否需要商业许可取决于具体条款。以下是框架主项目层面的概览，不覆盖所有插件、字体和媒体依赖，也未计算商业价格。

| 路线 | 本次核查到的主许可 | 对选择的影响 |
| --- | --- | --- |
| Qt | LGPL／GPL／商业许可，按模块区分 | 不能把所有 Qt 模块当作 LGPL；必须按实际模块和发布形态选择。[Qt 许可](https://doc.qt.io/qt-6/licensing.html) |
| JUCE | 当前模块为 AGPLv3／商业许可双许可 | 需要与商业闭源方式一起评估；不能沿用旧版本个别模块许可的记忆。[当前 LICENSE](https://github.com/juce-framework/JUCE/blob/master/LICENSE.md) |
| Slint | GPLv3、Royalty-Free 专有应用许可、商业许可 | 桌面／移动与商业嵌入式不是同一条件；未来带界面的盒子需另查。[Slint 许可条件](https://slint.dev/pricing) |
| Dioxus | MIT／Apache-2.0 | 主框架许可较宽松；第三方依赖另列。[仓库许可](https://github.com/DioxusLabs/dioxus#license) |
| Tauri | MIT／Apache-2.0 | 主框架许可较宽松；WebView、插件和前端依赖需单独归档。[MIT 文本](https://github.com/tauri-apps/tauri/blob/dev/LICENSE-MIT) |
| Avalonia | MIT | 专业工具和扩展产品不应自动视为同一许可。[主框架许可](https://github.com/AvaloniaUI/Avalonia/blob/main/licence.md) |
| Flutter | BSD-3-Clause | 插件和媒体依赖需另查。[主框架许可](https://github.com/flutter/flutter/blob/master/LICENSE) |
| Compose Multiplatform | Apache-2.0 | 分发中的 Java 运行时和第三方图形组件需另列。[主框架许可](https://github.com/JetBrains/compose-multiplatform/blob/master/LICENSE.txt) |

**可复用的基础库与输出框架**

这些是可选模块，不需要把它们全部带进产品，也不决定主界面技术。

| 项目 | 可以节省的工作 | 采用边界 |
| --- | --- | --- |
| miniaudio | 音频播放、采集、解码与基础混音；单文件 C／C++ 库，官方列五端 | C++ 路线可直接评估；其他语言可选维护良好的绑定，但要核对版本和所有权。许可为公有领域或 MIT-0。[官方说明](https://miniaud.io/) |
| Open Lighting Architecture（OLA） | 多种灯控网络协议和 USB 设备接入，可作为服务或后端 API | 官方矩阵显示各平台并非功能对等；不能直接作为所有桌面／手机的统一前置服务。版本、模块许可与每个输出驱动另审。[官方矩阵](https://www.openlighting.org/ola/) |
| SQLite | 本地结构化数据存储候选 | 是否用作工程容器、索引或缓存需后续设计，不把数据库方案绑定到引擎。主项目代码与文档为公有领域。[许可说明](https://www.sqlite.org/copyright.html) |

**怎样比较真实体积与出错风险**

本轮没有相同工作负载下的实测数据，因此不列“成品 X MB”或凭空打分。第一轮资料比较能确定包含哪些运行组件，以及哪里需要额外语言、绑定或插件；无法精确预测我们的成品大小。

用户选出一个或两个方案后，应使用相同的代表性功能验证：中文与输入法、灯具表格、颜色／属性操作、两个窗口、波形缩放、Cue 编辑、模拟输出和本地保存。它们能同时暴露控件、图形、数据流与本地依赖问题。

对比时固定目标系统、CPU 架构、优化级别、字体、素材与音频样本，使用 Release 构建并剥离调试符号。分别报告压缩包、安装后大小、需要额外下载／预装的运行时、冷启动和运行内存；Mac 的 universal 包与单架构包分开记录。应用商店上传包也不等同于用户最终下载量。[Flutter 体积测量说明](https://docs.flutter.dev/perf/app-size)。

维护成本记录：我们实际编写的主要语言数、额外描述语言、必须维护的跨语言接口数、直接依赖与原生动态库清单、平台专用模块数、构建工具链、关键依赖升级后的验证范围。少语言是其中一项，不能遮住很大或很复杂的依赖树。

稳定性记录：播放计算耗时与漏期、输入到引擎的延迟、界面重绘对播放的影响、后台保存、内存压力、睡眠恢复和设备重连。模拟输出通过不能代替真实 DMX 时序验证。C++／Rust 的无 GC 路线和 C#／Dart／Kotlin／Go 的运行时路线都应按相同节目目标检验，不能仅凭语言宣布合格或不合格。

**供用户选择的取向**

- 希望主要用 C++，专业桌面与硬件优先：重点看 A（Qt）；音频工作流占比高、接受其许可条件时加入 B（JUCE）。
- 希望主要用 Rust，减少手写语言与绑定：重点比较 C（Slint 原生界面）和 D（Dioxus WebView 界面）。
- 接受两种主要语言，重视 Web 界面生态：看 E（Tauri）。
- 希望主要用一种较高层语言完成产品，优先多端编排体验：比较 F（Avalonia）、G（Flutter）、H（Compose）。执行层是否要另用原生语言由实测决定。

共同约束保持不变：工程、领域、编辑流程、执行器和设备适配分模块；同一种语言也必须解耦。不自动叠加 Rust＋C++＋TypeScript 多套业务实现，不因框架宣传声称全平台免适配。本比较阶段未初始化任何一条路线，后续收敛判断见最终技术评估。

**Web 远程控制对 Tauri 路线的价值**

用户进一步提出未来通过 Web 页面远程控制灯具。这使 E 的两种主要语言分工具有更明确的产品收益：TypeScript 界面可以面向桌面 WebView 与普通浏览器复用，Rust 承担共享业务、控制服务与现场执行。该需求支持优先验证 E，并已纳入后续最终技术评估。

Tauri 本身不是浏览器中的 Rust 后端。官方说明其前端由 HTML／CSS／JavaScript 等静态资源组成、由 WebView 显示；Tauri 专用调用需要封装在宿主适配层。Web 构建加载普通网页资源，并通过网络控制接口连接运行引擎的电脑或盒子。这样既能复用控件和交互，也不会把 Tauri API 散布到所有界面组件中。[Tauri 前端配置](https://v2.tauri.app/start/frontend/)、[Tauri 通信](https://v2.tauri.app/concept/inter-process-communication/)。

建议调用链为：桌面／移动 App 或浏览器 → 控制接口 → 现场引擎 → 输出适配器 → 灯具。远端提交 Cue 触发、参数修改和编排内容，执行器本地负责时钟、效果和连续输出。远程即时操作仍受网络延迟影响；需要精确同步时，发布节目、校准时钟与带时间的执行命令应单独设计，不能承诺跨互联网零延迟。

Web 控制不以云端为必要条件，可以先在同一局域网连接现场电脑／盒子。互联网控制后续再增加安全连接、身份与设备授权及多控制端的控制权处理。界面断开时的输出策略归现场引擎管理，不让持续播放依赖网页保活；具体失联行为按设备与节目定义。

若按 E 实现，本机接口与现场 HTTP／WebSocket 等网络接口可以共用 Rust 应用服务与命令模型。后续服务器端补充评估推荐 TypeScript＋Fastify＋PostgreSQL，承担云端管理和分发；需要工程处理时调用 Rust 模块。以最终技术评估为准，各种入口不能分别维护两份 Cue、效果或配适规则。
