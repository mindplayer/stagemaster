# 桌面后台执行

依据 [ADR-109](../development/decisions/PRODUCT-ADR-109-desktop-background-execution.md)，实施记录 [HOST-005](../development/tasks/HOST-005-desktop-background-execution.md)。此入口接实际 [v2 多来源进程](multi-source-process.md)，仍为单域软件输出，不会自动接物理设备；[后台预演](background-previsualization.md) 通过显式只读来源接 UE。

## 职责与接口

`stagemaster-execution-client::Client` 不依赖 Tauri，负责私有本机发现、能力和身份检查、命令序号、回执与观察。`apps/desktop/src/execution` 管理不可变工程快照、后台进程和运行记录；界面只调用 `ApplicationHost.execution(request)`。工程、编辑预演、后台节目各自拥有生命周期。

```text
Client::open(discovery_path) -> Client          // 仅观察，不创建控制会话
client.refresh() -> View                       // 核对原回执，再读取状态
client.acquire(takeover) -> View               // 首次创建会话；接管须明确
client.apply(host_id, revision, source, action) // 执行、暂停、继续、下一步、停止、电平
client.release() -> View                       // 归还输入权，不停演
client.maintain() -> View                      // 观察、必要时续约；不驱动播放器
client.shutdown()                             // 明确结束整个后台
```

这些方法均为异步调用；`View.pending` 表示结果仍待确认，成功返回 HTTP 不等于节目动作已生效。界面请求：`snapshot`、`prepare { generation, selection }`、`reconnect`、`acquire { takeover }`、`release`、`apply { hostId, revision, source, action }`、`shutdown { hostId }`。

准备选择 1–63 个真实场景／列表，另加一个较高优先级的手动层，仍受来源组累计预算限制。生成固定快照再启动同一后台二进制；载入不自动播放。当前手动层界面只提供电平和释放，完整语义手动编辑保留 v2 服务能力，后续再接原编程器。

## 当前步骤与阶段进度

[EXEC-005](../development/tasks/EXEC-005-source-execution-progress.md)／[ADR-149](../development/decisions/PRODUCT-ADR-149-source-execution-progress.md)：原 `Player::progress()` 只读给出延时／渐变／自动等待／人工保持、步内时间和后续步骤，来源组与后台快照／回执沿原链投影。暂停冻结，结束保留末步；媒体来源继续使用原音频同步状态。

节目卡片就地显示当前／下一步、阶段和剩余秒数，有确定时长才有进度条。起始步骤和电平草稿不随观察变化；末步无后续时禁用下一步，循环末步标出回到首步。无控制权仍可观察；读取失败明确标出最后已知步骤、停止显示有效进度。旧主机缺少进度时明确不可用，TS 不自行计时或触发命令。

## 节目浏览与常用顺序

[EXEC-006](../development/tasks/EXEC-006-execution-board-navigation.md)／[ADR-150](../development/decisions/PRODUCT-ADR-150-execution-board-navigation.md)：搜索、类型／状态／常用筛选和显示顺序仅作用于 UI，不提交运行命令或改变混合优先级。筛选保留原卡片与输入，隐藏时禁用执行并取消旧重新执行确认；筛选外运行／暂停及未应用输入有显式入口。切到编辑预演／步骤编排后后台组件保留、停止观察轮询，重新显示先刷新状态；新 hostId／layout 丢弃旧草稿。

常用偏好为本机 `stagemaster.executionPins.v1`，最多 20 工程、各 16 个语义来源键；按后台 projectId 与原场景／列表身份保存，不按随机 sourceId。尚未载入的固定项保留并可显式清理；写入失败仅影响本机持久化，不影响节目运行。状态错误显示最后已知概览，不伪造全部已停止。

## 身份、回执与失败

- 发现文件及父目录只允许本机用户访问，不接受符号链接、远程地址、代理或重定向；只连接精确的 `127.0.0.1` v2 启动地址。连接信息不返回 TypeScript。请求 8 KiB、响应 8 MiB、连接／请求期限有界。
- 请求发出前保留待确认序号，取消或响应丢失不能释放它；只 GET 原回执，不自动重发。没有可确认回执时阻止后续修改；重新连接建立新的观察客户端，原操作不重放，控制权重新明确取得。
- 确认回执的运行状态优先于较旧的已发布状态；循环计数仍是观察采样的真实计数，不冒充新调度周期。新启动／布局不匹配则拒绝，旧界面的来源操作携带启动身份和预期修订。
- 控制租约 60 秒，桌面服务每 10 秒检查，剩余不足 30 秒时按相同回执路径续约。其含义是输入权限，与商业限时无关；退出应用或到期不会停演。
- 管理服务使用跨进程文件锁保护当前运行记录。后台从创建运行目录开始持有 `lifetime.lock`，启动及播放期间均保持。只接受 Child 终态或实际取得原锁作为结束证据；HTTP 失联不能触发清空和重启。
- 明确关闭先发送原 shutdown，再等同一进程终态；未结束时保留关闭状态及原记录。后台断线、应用重启、编辑切页和工程修改均不自动停演。

## 本机存储与构建

开发数据在项目 `data/execution/`；正式应用使用已有应用数据根目录。`current` 仅保存运行 UUID，快照、来源清单、日志和后台私有文件在该 UUID 子目录。当前保留已结束运行资料用于诊断，自动保留数量策略后续另设；不是用户工程备份策略。

桌面入口 `tools/desktop/run.mjs` 串行构建后台，并通过 Tauri `externalBin` 打包。直接 Cargo 编译桌面前需执行 `node tools/desktop/prepare-host.mjs`；不在 Rust 构建脚本内递归启动 Cargo。运行只启动随应用交付的固定文件，不向前端开放 shell 权限。

仅调试构建支持 `STAGEMASTER_ACCEPTANCE_INSTANCE` 的字母数字／连字符实例名，将所有桌面数据定向到项目 `tmp/desktop-<name>/`，供独立原生验收；生产路径不使用此选项。

## 本次边界

入口位于“执行步骤 → 执行视图 → 后台执行”，可检索／多选载入，读取固定来源和步骤，明确控制、独立操作并关闭后台。编辑预演保留旧音乐／草稿语义；两者切换不让旧键盘操作控制后台。后台卡片显示权威步骤和阶段时间；编辑预演总控仍不作为后台总控。

后台只读预演已按 PREVIS-003 接入；远程设备、物理输出、多时钟、完整光学、云端分发和生产安全仍按框架主线实施。若应用在创建启动记录与操作系统启动之间异常退出，且后台尚未留下生命周期证据，当前保留记录、禁止自动重启；专门的异常启动记录恢复工具后续补充，不能把连接超时视为进程死亡。
