# 证据差异、未核实项与后续验证

核查日期：2026-09-10。这里记录会改变功能判断的来源差异和实测缺口；“未核实”不等于“不支持”。研究中的流程和验收场景是自拟建议，没有连接真实控台、触发灯具、测量延迟或验证冗余接管。

## 版本与来源差异

| 编号 | 差异或限制 | 本轮处理 | 后续核查方式 |
| --- | --- | --- | --- |
| E01 | MA 2.5 帮助包内仍有专题页显示较早页面版本 | 分别保留目录版本和 `displayed_version`；不把旧标签改写成 2.5 | 对影响实现的细节继续对照发行说明和目标软件 Help |
| E02 | MA Lua 专题／Plugin 页保留 5.4.x 信息，2.5 新版说明明确升级至 5.5.0 | **文档层面已解决**：M15／M21 采用新发行说明，记录旧字节码重编译要求 | 部署插件前核对目标版本的 API 和执行行为 |
| E03 | MA 时间码专用 Slot 页描述 16 个槽，而部分 Timecode 设置描述仍使用 1–8 | M13 采用专用 Slot 页的 16，但不据此宣布旧设置描述全部已更新 | 目标版本 UI 核对可选槽、外部源映射及是否随 Show 迁移 |
| E04 | MA PSR 对 Recipe／Phaser Shape 引用的说明存在范围差异，不能笼统归纳“引用全部保留” | 按对象类别列迁移限制，不承诺 Recipe 经 PSR 后等价 | 用包含自定义 Shape、预制 Shape、Preset、Group 的小工程验证依赖 |
| E05 | Titan 当前软件为 19.2，稳定手册仍标 19.0；部分硬件／媒体页面已经更新 | 以 19.0 手册讲基础语义，以 19.1／19.2 发行说明覆盖变更 | 软件更新时重新核对 RTSP、时间码范围、备份与 API 变化 |
| E06 | Titan API 19.2 简介仍有旧版本返回示例，且 KillPlayback 的说明文字有误导 | **文档层面部分解决**：以方法页确认 Kill 是停止播放；示例中的版本号不作为当前版本证据 | 按实际目标版本读取信息，执行接口契约测试；不依赖简介旧样例 |
| E07 | Titan GPIO 接口表覆盖新机型，后文仍保留较早机型／引脚限制的叙述 | 仅记录“支持范围依机型”，不将一项引脚限制推广到所有控台 | 使用目标机型接线文档及硬件批次说明核查 |
| E08 | Titan PC 支持页和发行说明对 Windows 要求的表述不同 | 不据此承诺旧 Windows 的长期兼容性；Tauri 项目的跨平台能力另行验证 | 实际选购／部署前再查该版本安装要求及支持状态 |
| E09 | D9 的 RTSP 功能除 Titan 19.2 外还与系统镜像／硬件批次有关 | M12／M21 列出镜像条件，不把软件升级包当作充分条件 | 根据具体 D9 序列／批次与官方恢复镜像说明核对 |
| E10 | MA Programmer 与 Preview 专题中关于预览实例／用户状态的表述应区分对象层次 | 已确认同 User Profile 共享 Preview 状态，且 Store／Update 可影响实际节目；不推断每个用户都有完全独立工程 | 在多用户、不同 Profile／Data Pool 下验证实例和数据修改范围 |

E01–E04、E10 来源：[MA Lua](https://help.malighting.com/grandMA3/2.5/HTML/lua.html)、[Plugins](https://help.malighting.com/grandMA3/2.5/HTML/plugins.html)、[2.5 Other Enhancements](https://help.malighting.com/grandMA3/2.5/HTML/rn_otherenhancements-2-5.html)、[Timecode Slots](https://help.malighting.com/grandMA3/2.5/HTML/timecode_slots.html)、[Timecode Settings](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[PSR](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[Recipe Editor](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[Programmer](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[Preview](https://help.malighting.com/grandMA3/2.5/HTML/preview.html)。

E05–E09 来源：[Titan PC Suite](https://www.avolites.com/support/all-titan-pc-suite-downloads/)、[稳定手册](https://manual.avolites.com/docs/)、[19.0 发行说明](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf)、[19.2 发行说明](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf)、[API 19.2 简介](https://api.avolites.com/19.2/)、[KillPlayback 方法](https://api.avolites.com/19.2/api/Playbacks.KillPlayback.html)、[外部触发接口表](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering)、[D9 支持说明](https://www.avolites.com/support/D9/)。

## 不能通过手册对照直接得出的结论

| 编号 | 未完成的验证 | 当前结论边界 | 对后续开发的作用 |
| --- | --- | --- | --- |
| V01 | 稳态抖动、重负载、长期播放和设备驱动测试 | 官方有功能说明，不等于达到 StageMaster 的性能目标 | 建立代表性灯具／Universe／效果负载和输出测量方案 |
| V02 | 双机故障接管与多用户冲突实验 | 可描述官方同步和接管过程，不能承诺无缝或零丢帧 | 区分 Show 备份、运行状态同步、控制权和输出恢复 |
| V03 | 每个 Keyword／API 方法的实际调用 | 完整目录索引仅便于查找，不是数千个接口已测试 | 只对选用 API 建小范围契约测试，检查版本、权限和上下文 |
| V04 | 所有硬件型号、固件和输出授权组合 | 现有型号表为文档基线，不能替代具体设备能力查询 | 设计独立能力声明，按设备实际能力接受／拒绝部署 |
| V05 | 灯具色彩、位置、切割片和光学效果实测 | 档案、校准和真实机构影响结果，3D 不保证物理一致 | 灯型校验和物理校准独立于编辑界面 |
| V06 | 未找到的跨系统等价能力 | 没有确认 Titan 原生同构 Recipe／PSN／XYZ 等，不表示绝对不存在 | 保留“待查”，不能在产品选型表里打确定的否定标记 |
| V07 | 私有 Show 文件互转 | 没有逆向格式，也没有建立无损互转承诺 | 优先自己的稳定工程模型和官方公开交换格式 |
| V08 | 两家完整云端项目存储／分发体系 | 局域网 Session、备份、远程界面和在线下载服务不足以证明完整云存储 | StageMaster 自行定义离线修订、不可变发布包和设备激活契约 |

## 覆盖口径

本轮完成的是全模块功能地图、重点行为对照、当前版本差异和完整官方主题入口。没有逐字翻译每一页手册，也没有把每个按钮、命令参数、维护零件、旧版修复或多语言重复页面都列为独立功能。完整目录和逐模块整理的数量分别在[来源覆盖表](source-coverage.md)说明。

对于未来选择进入开发范围的功能，应继续沿功能编号查原文，把本系统默认行为、边界及验收明确下来；不能把目前的“概念相近”直接改为“兼容实现”。
