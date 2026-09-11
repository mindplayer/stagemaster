# 系统设置、维护、诊断与升级

可观测性是专业系统的一部分：灯不亮时应能判断是编排、总控、授权、配适、协议、接口还是设备本身的问题。升级和恢复则要分别处理软件、灯具库、控制面固件与节目文件。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M20-01 | 运行负载 | System Info：每帧计算、界面渲染、CPU、内存、磁盘等 | DMX Overview：处理节点、Line、槽位及负载 | [MA Info](https://help.malighting.com/grandMA3/2.5/HTML/si_system_info.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M20-02 | 硬件健康 | 部分型号显示温度／风扇，onPC 等不提供同样读数 | USB Expert 查看面板、连接和硬件事件 | [MA Info](https://help.malighting.com/grandMA3/2.5/HTML/si_system_info.html)、[Titan USB Expert](https://manual.avolites.com/docs/system-settings/usb-expert) |
| M20-03 | 系统消息 | Message／Status Center、System Monitor | 系统提示、诊断工具及版本资料 | [MA Message](https://help.malighting.com/grandMA3/2.5/HTML/system_message_center.html)、[Titan System](https://manual.avolites.com/docs/system-settings/the-system-menu) |
| M20-04 | 输出检查 | DMX Sheet、Tester、参数和来源优先级 | DMX View、输入列、Patch View | [MA DMX](https://help.malighting.com/grandMA3/2.5/HTML/patch_dmx_sheet.html)、[Titan Change](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M20-05 | 网络诊断 | Network Tests、站点状态、接口、协议监视 | IP／适配器、连接状态、端口与 DMX 节点设置 | [MA Tests](https://help.malighting.com/grandMA3/2.5/HTML/network_tests.html)、[Titan Ports](https://manual.avolites.com/docs/networking/ports-used-by-titan) |
| M20-06 | 用户默认行为 | User Settings、Preference and Timings | User Settings 分类设置录制、Clear、Release、时间、效果、轮控等 | [MA User Settings](https://help.malighting.com/grandMA3/2.5/HTML/user_settings.html)、[Titan User Settings](https://manual.avolites.com/docs/system-settings/user-settings) |
| M20-07 | 显示和控制面 | 显示配置、Desk Lights、本地设置 | 外部显示、触屏、面板亮度、Virtual Hardware | [MA Local](https://help.malighting.com/grandMA3/2.5/HTML/local_settings.html)、[Titan Displays](https://manual.avolites.com/docs/system-settings/external-displays) |
| M20-08 | 软件升级 | 按设备类型选择包，可经网络更新多设备 | Upgrade Installer／Recovery Installer，支持保留多个版本的工作流 | [MA Update](https://help.malighting.com/grandMA3/2.5/HTML/update.html)、[Titan Upgrade](https://manual.avolites.com/docs/system-settings/upgrading-the-software) |
| M20-09 | 延后激活 | 网络更新文件复制与设备重启激活分开 | 安装、重启及所需恢复镜像按发布说明执行 | [MA Network Update](https://help.malighting.com/grandMA3/2.5/HTML/update_network.html)、[Titan Upgrade](https://manual.avolites.com/docs/system-settings/upgrading-the-software) |
| M20-10 | 控制面固件 | onPC 硬件／节点等有专门更新流程 | USB Expert 自动／手动面板固件更新及按键、推杆检查 | [MA Hardware](https://help.malighting.com/grandMA3/2.5/HTML/update_windows_hardware.html)、[Titan USB Expert](https://manual.avolites.com/docs/system-settings/usb-expert) |
| M20-11 | 灯具库维护 | Fixture Share／GDTF／内置灯型编辑与导入 | 官方 Personality 库、自定义目录、已配适更新 | [MA Types](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[Titan Personalities](https://manual.avolites.com/docs/fixture-personalities) |
| M20-12 | 恢复与重置 | Clean Start、Panic 等有不同影响范围 | Standard Recovery／Factory Restore／Full Erase | [MA Clean Start](https://help.malighting.com/grandMA3/2.5/HTML/ts_clean_start.html)、[Titan Recovery](https://manual.avolites.com/docs/system-settings/recovering-reinstalling-the-console) |
| M20-13 | 远程支持数据 | World Server 提供灯型文件和 Crash Log 上传 | 版本、USB Expert、库问题及官方支持流程 | [MA World Server](https://help.malighting.com/grandMA3/2.5/HTML/system_world.html)、[Titan USB Expert](https://manual.avolites.com/docs/system-settings/usb-expert) |
| M20-14 | 工程文档输出 | 对象导出、截图及信息窗口等 | Reports 支持灯具、Cue、Chase、Cue List、Palette、Group，输出多种文档格式 | [MA Screenshot](https://help.malighting.com/grandMA3/2.5/HTML/screenshot.html)、[Titan Reports](https://manual.avolites.com/docs/titan-basics/creating-reports) |
| M20-15 | 已知问题与版本要求 | Release Notes、Known Limitations、系统要求 | Release Notes、机型下载和恢复要求 | [MA 2.5 Notes](https://help.malighting.com/grandMA3/2.5/HTML/release-notes-2-5.html)、[Titan Notes](https://www.avolites.com/support/release-notes/) |

## 维护边界

MA World Server 官方说明的主要能力是灯具档案获取和崩溃日志上传；不能把它写成完整云端项目协作／分发平台。[MA World Server](https://help.malighting.com/grandMA3/2.5/HTML/system_world.html)。

Titan 的 Standard Recovery、Factory Restore 和 Full Erase 对节目、个人档案和许可有不同影响。新版本软件、控制面固件和恢复镜像也不总是同一次升级完成；应查对应型号说明。[Titan Recovery](https://manual.avolites.com/docs/system-settings/recovering-reinstalling-the-console)、[Titan Upgrading](https://manual.avolites.com/docs/system-settings/upgrading-the-software)。

## 工作流程与 StageMaster 建议

自拟诊断流程：先查逻辑灯具值和输出来源，再查 Universe 帧、路由／设备状态、发送统计及物理链路。对“灯没响应”的界面，应该能指出卡在哪一层，而不是只显示连接成功。

以下属于设计建议。单机基础诊断包含执行周期耗时、超时次数、命令积压、输出帧状态、连接重试、媒体缓存及当前运行包版本。日志与监控在有界缓冲上异步处理，磁盘或网络慢不能拖住输出。

升级流程分下载、校验、兼容检查、激活和回退。云端记录期望版本／已下载版本／正在运行版本，并能导出脱敏诊断包。性能数字和故障恢复时间都留待实现后测量，不因采用 Rust 就宣称达到实时或永不崩溃。
