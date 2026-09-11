# 多用户、会话、备份与故障接管

多用户解决多人编辑和操作；备份解决运行设备失效；分布计算解决输出负载；文件同步解决数据保存。它们可组合，但不能用“联网”一个功能覆盖。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M18-01 | 会话 | 多站点加入 Session，共享节目，存在 Master | TitanNet 的 Multi-User、Backup、组合模式 | [MA Session](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[Titan Users](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-02 | 版本条件 | 会话要求相同 Streaming Version，即版本号前三段 | 联网控台要求相同 Titan 版本 | 同上 |
| M18-03 | 用户共享状态 | 同用户／Profile 可共享编程器与相应状态；Screen Configuration 可不同 | User 与 Handle World 决定布局和连接操作的共享方式 | [MA Users](https://help.malighting.com/grandMA3/2.5/HTML/user.html)、[Titan Users](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-04 | 多编程器 | 不同 User Profile 各有编程器，输出共同合成 | 用户及 Remote 有独立编程器，可清全部编程器 | 同上 |
| M18-05 | 操作范围 | User Rights 与 World 对灯具／属性范围限制 | Handle World 主要是操作柄布局，不等于 MA World 的灯具范围 | [MA Worlds](https://help.malighting.com/grandMA3/2.5/HTML/worldfilter.html)、[Titan Users](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-06 | 编辑冲突 | Object Ownership 对整个对象或部分内容加临时锁 | 多用户同灯编辑有接管规则；本次未确认同构细粒度对象锁 | [MA Ownership](https://help.malighting.com/grandMA3/2.5/HTML/user_ownership.html)、[Titan Users](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-07 | 锁交接 | Drop／List Ownership，可请求释放当前编辑权 | 不将 Handle 锁定或 Venue Mode 当作对象编辑锁 | [MA Ownership](https://help.malighting.com/grandMA3/2.5/HTML/user_ownership.html) |
| M18-08 | 会话加入的数据方向 | 加入站优先级影响谁上传／下载 Show | Slave 加入 Master，接收 Master 的 Show | [MA Master](https://help.malighting.com/grandMA3/2.5/HTML/network_session_master.html)、[Titan Users](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-09 | 主站选择 | 站优先级、在线时长、IP 等规则决定接管 | 手册明确 Backup Takeover 操作；不承诺同等自动选主 | [MA Master](https://help.malighting.com/grandMA3/2.5/HTML/network_session_master.html)、[Titan Backup](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-10 | 输出责任 | 会话 Master 发送网络 DMX，节点按配置输出 | Master 输出；Takeover 使原主机进入输出禁用状态 | [MA Session](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[Titan Backup](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-11 | 同步与恢复 | Full Tracking Backup 共享运行和编程状态，失联设备可配置重新邀请 | 备机在主机保存／自动保存时同步，也可 Sync Now | [MA Users](https://help.malighting.com/grandMA3/2.5/HTML/user.html)、[MA Session](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[Titan Backup](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-12 | 备机能力条件 | 主站与授权参数按系统规则决定 | 备机许可 Line 上限不能小于主机；控制柄数量也影响接管操作 | [MA Parameters](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)、[Titan Backup](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-13 | 连接状态查看 | Session／Station 状态、版本、网络负载及缺失站点 | Sessions View 和备份连接／同步状态 | [MA Session](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[Titan Backup](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup) |
| M18-14 | 离开会话 | Leave／Dismiss 是独立操作 | Slave 离开后恢复加入前本地 Show | [MA Session](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)、[Titan Users](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M18-15 | 分布计算设备 | Processing Units 与节点有不同职责 | TNP 处理输出；不把普通协作控台叠加为许可扩容 | [MA Expand](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |

## 高影响行为

MA 新站点若优先级高于现有 Master，可能成为主站并把自身 Show 上传到其他站点。加入会话不是简单“打开远程视图”。官方还明确了网络时延和超时要求；这些是该系统的运行条件，不是任意互联网连接都能满足的能力。[MA Session Master Selection](https://help.malighting.com/grandMA3/2.5/HTML/network_session_master.html)、[MA Session](https://help.malighting.com/grandMA3/2.5/HTML/network_session.html)。

Titan 手册将 Show 同步描述为保存时发生，可手动 Sync Now；Takeover 明确禁用原主控输出。不能据此承诺所有未保存状态都已瞬时同步，也不能把商业宣传中的平滑接管解释成已实测零丢帧。[Titan Backup](https://manual.avolites.com/docs/running-the-show/linking-consoles-for-multi-user-or-backup)。

## 工作流程与 StageMaster 建议

自拟流程：两用户各自编辑不同灯组，再同时改同一预设；断开一个站点、重新加入，核对内容版本、编程器和输出责任。备份测试应分别覆盖程序崩溃、网络分区、设备断电、主机恢复与许可／能力不匹配。

以下属于设计建议。当前仍先做单机。保留命令来源、用户／会话 ID、工程版本和播放实例标识，为后续协作准备，但不提前实现分布式复杂度。

未来按三个独立模块推进：云端项目版本同步、远程控制单机、局域网备份／多机协作。云端同步不参与逐帧输出；备份需明确谁拥有输出权、防止双主、如何取得一致快照和重放事件。所有接管性能都要通过故障注入测量。
