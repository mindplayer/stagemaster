# 独立应用的后台音频契约

对应 [ADR-119](../development/decisions/PRODUCT-ADR-119-background-audio-application.md) 与 [TIME-001](../development/tasks/TIME-001-independent-clock-boundaries.md)。本契约是现有本机执行应用的增量，复用 [多来源进程](multi-source-process.md)、[受控媒体提供方](controlled-media-provider.md) 和 [原生音源](resident-native-audio.md)，不新增平行播放器或控制服务。

## 准备与所有权

依 [ADR-126](../development/decisions/PRODUCT-ADR-126-optional-host-audio.md)，执行程序默认组装 `audio`，桌面构建显式启用；`--no-default-features` 生成共用灯光核心的纯灯光宿主，不链接解码／声卡后端。纯灯光程序拒绝本节音频清单、媒体控制与 `--audio-scope`，只发布实际能力；启用本后端仍需原内容、输出与控制权检查，不因编译进模块而自动播放。

现有 `stagemaster-execution-host <工程> group <来源清单> <新运行目录> --software-output` 入口支持清单版本 2。`--software-output` 表示灯光为软件输出；声音路由在清单中独立声明。按 [ADR-124](../development/decisions/PRODUCT-ADR-124-audio-output-ownership.md)，可追加 `--audio-scope <绝对私有目录>`；系统声音输出必须传入，桌面对所有后台均传入自己的共同范围。独立静音软件用例可省略，不把软件采样称为物理 DMX 或音箱反馈。

```json
{
  "version": 2,
  "audio": { "output": "software" },
  "sources": [
    {
      "id": "00000000-0000-4000-8000-000000000002",
      "priority": 0,
      "selection": { "kind": "audioTimeline" }
    }
  ]
}
```

此增量允许一条工程音乐编排，与已有场景／列表／手动来源共享最多 64 个来源的预算。必须同时有 `audioTimeline` 和明确的音频输出；不支持的版本、重复身份、额外音频来源和缺失配置拒绝启动。旧清单版本 1 行为保持。

- `software`：真实 WAV 解码和 Rodio Mixer 拉取 PCM，按墙钟推进；没有声卡输出。软件消费者超过 500 ms 未运行则报告故障，不快速补播伪装连续执行。
- `systemDefault`：使用现有原生输出适配；只在已授权播放准备时打开系统默认声音设备。必须由上层显式选择；本增量的自动验收仅使用 `software`。
- 启动先验证工程、随附资源身份和已有预算，把音乐复制到新私有运行目录 `store/media`。源工程和原随附目录随后移走不影响已准备的执行。
- 后台媒体线程独占 Transport、固定工程、准备器和媒体端口；灯光由原 Host 调度，操作仍通过原输入控制权。复制和解码不进入灯光调度或音频采样回调。
- 按 [ADR-163](../development/decisions/PRODUCT-ADR-163-software-audio-consumer.md)，静音软件输出的单个所属线程独立消费同一 Rodio Mixer，慢准备不阻塞真实回调；线程不读工程或掌握节目／输出控制权。48 kHz／双声道墙钟与 500 ms 保护保持，消费失败沿原 OutputBinding 终止；操作不重置墙钟补播，暂停回调只更新健康而不虚构 PCM 消费。输出销毁取消并等待所属线程，恢复先回收旧输出，不回落真实声卡。
- 程序目录提供 `backgroundLinearAudio` 能力和 `audio` 描述：输出、时长、组身份、`seekIncludesEnd`。新后台按 ADR-121 声明 true，旧后台的 false 仍被客户端识别并限制末尾定位。客户端不能按旧普通来源按钮发送音乐操作。

## 请求、接纳与完成

在现有会话 `submit` 中使用媒体操作；修订、组身份与播放代次均从当前观察读取：

```json
{
  "kind": "submit",
  "expectedRevision": "3",
  "action": {
    "kind": "media",
    "group": "00000000-0000-4000-8000-000000000002",
    "generation": "1",
    "action": { "kind": "seek", "positionMs": 3000, "playing": false }
  }
}
```

其他操作为 `play`、`pause`、`stop`。外围仍使用原请求序号、期限和回执查询。HTTP 解析只检查参数／能力，不播放、不定位；原 Host 完成控制权和修订校验后才产生提供方请求。

成功提交结果 `kind: accepted` 表示请求已接纳；读取 `state.media[].control` 的同一个 `request` 序号，只有 `status: applied` 才证明提供方确认且 Host 接纳完成。`pending`、`failed`、`timedOut` 分开显示，旧操作完成不能冒充新操作完成。相同会话序号重试返回原回执，不重新播放；跨会话接管仍需显式取得控制权。

按 [ADR-173](../development/decisions/PRODUCT-ADR-173-media-target-refusals.md)，已知音乐组的合法代次不匹配在原适配层返回 `mediaTargetChanged`；合法循环实例／区段／遍次不匹配当前权威目标返回 `loopTargetChanged`，中文说明要求核对当前目标后再明确操作。它们仍是 HTTP 200 的 `complete / rejected` 原回执，消耗网络序号但不创建音乐请求，控制绑定与节目保持；不是 HTTP 409 或 Applied。未知组、零实例／零遍次和区段越界保持 `invalid`，非法十进制格式仍在 HTTP 422 拒绝且未消耗序号。适配后才跨界的请求继续沿原提供方失败路径，不自动替换目标或重发。

音乐控制附近复用原请求关联规则：新拒绝、未知或缺少确认不能借用旧音乐 Applied 显示完成；新接纳与当前音乐请求一致才显示其实际完成／失败，旧观察等待，新请求替代明确提示。只读观察和已完成的非媒体操作可以保留历史音乐状态，输入草稿及原音源不因拒绝清除。

状态分工：

| 状态 | 含义 |
| --- | --- |
| `media[].generation` | 当前组播放代次；定位、结束与重新播放可使其改变 |
| `media[].status / positionMs` | 原灯光宿主已接纳的组状态与位置 |
| `media[].control` | 本组最新受控请求及实际完成／失败 |
| `media[].termination` | 最近终态请求的原播放代次、原因及是否成功释放；不一定属于当前新播放 |
| `audio.status / positionMs` | 原生媒体所有者的就绪、准备、播放、暂停、结束、停止或故障观察 |
| `audio.instance / frames` | 最近实际音源实例与 PCM 消费帧；不是声卡呈现确认 |

组快照与原生媒体视图来自各自所有者，可能存在正常的短暂观测差异。界面不能仅根据 `audio.status` 或 HTTP 请求成功就把最新控制请求显示为已完成。

## 生命周期和失效

开始／定位在媒体线程准备音源与现有工程灯光计划；先静音挂载并取得实际采样健康，再激活组，随后开始声音并等待原生请求确认。暂停不伪造消费，不改变另一自主灯光来源。观测保留原采样时间；短暂忙锁保留待发布样本，不刷新时间续命。

常驻音源不依赖操作者连接与租约。控制端进程被杀、释放控制或另一个客户端只读连接都不会隐式停止节目。自然结束和故障通过固定终态槽归还本组的灯光属性；新播放不受旧代次结束消息影响。终态请求内部保存完整组键，公开回执只保留组内代次及结果，避免在每份 64 组状态中重复身份。队列不增长，灯光线程只尝试取锁，不等待媒体线程。

损坏资源、输出故障、准备超时或同步组失效时停止受影响音源并报告失败；自主灯光来源继续。后台关闭先取消媒体工作、回收音源与线程，再关闭原 Host。启动失败不发布发现记录；正常关闭清除发现记录。运行资料的后续保留与数量策略沿用应用层任务，不在此静默删除。

## 共享客户端接入

[ADR-120](../development/decisions/PRODUCT-ADR-120-background-audio-client.md) 将能力解析与媒体状态接到原 `stagemaster-execution-client`。`Client::apply_media(host, revision, group, generation, MediaAction)` 复用原请求会话与回执；`Play`、`Pause`、`Stop`、`Seek` 不经过普通来源的步骤操作。普通 `apply` 对音乐只接受原有亮度电平调整，不能把音乐暂停翻译为独立灯光暂停。

`Catalog.audio` 和 `State.audio / media` 提供类型化输出、位置、播放代次与实际完成状态。`View.pending` 仅表示 HTTP 请求结果未确认；收到 `accepted` 后还应关联 `record.outcome.state.media[].control.request` 与当前状态的同一请求，检查 `MediaCompletion::Applied / Failed / TimedOut`。回执丢失只查原序号，不重发；新的只读连接不自动接管，也不开始播放。

原 `Reader` 可读取包含音乐的固定工程与真实完整采样，仍不创建控制会话。原桌面 `Background` 适配复用音乐段落输出定义，即使来源目录只有音乐，也能形成同一场地及灯光投影；移走原工程、重新连接观察端均不改变后台音源或操作者。

按[ADR-174](../development/decisions/PRODUCT-ADR-174-media-request-evidence.md)，客户端应用`View.mediaOperation`可选提供最后一条显式音乐操作的有界脱敏诊断，不改变宿主HTTP协议。合法目标、原网络serial、发送尝试、POST及同serial回执GET的HTTP状态／正文完整性、原已校验回执和准确媒体request分别表达；`attempted`不证明后台收到，`accepted`不证明Applied。无效输入不留原字符串，本地未提交不可借旧完成；维护不覆盖、重新连接不转移旧证据。最多一项且序列化不超过4KiB，不含任意正文／message、token、环境、工程或完整帧；只读详情不重发、不新建观察或时钟。

能力、音乐来源与组描述必须一致；错路由、时长／位置越界、缺失组、未知状态及非法计数拒绝。测试以真实进程和代理返回故障数据验证，不能把只读投影通过视为 UE 光学或物理输出验收。

## 桌面准备与操作

原 `execution_request` 的 `prepare` 请求接受 `selection: [{ kind: "audioTimeline" }]` 和 `audioOutput: "software" | "systemDefault"`。音乐必须恰有一个明确输出；不选音乐时不接受音频配置。既有场景／列表来源继续可组合，手动层由宿主补齐。桌面从当前工程随附目录或内容缓存找到资源，校验摘要并复制到私有运行工程目录，随后复用原执行进程，不在界面执行音频或灯光算法。

同一入口的 `media` 请求携带 `hostId / revision / group / generation / action`，直接进入原 Client 的受控媒体命令。界面初次载入与重开均只读；明确取得控制权后才允许音乐操作。滑动松手提交、精确秒数输入、取消和越界错误不改变工程；请求未确认或失败时保留输入。来源接受状态与音乐实际完成状态仍分开显示。

正式后台载入音乐前停止当前编辑器试听并使原慢准备失效；后台尚未确认结束时，新编辑试听播放／定位／循环请求在 Rust 入口拒绝，停止仍可用。慢准备任务持有进程内管理锁和相应文件锁至真实完成，界面未来被取消不释放正在执行的所有权。音源首次挂载时另持 OutputLease 至停止／清空／销毁，其他编辑窗口已在播放或暂停时拒绝后台载入，不自动停止对方。后台子进程自行取得同一范围的占用，贯穿就绪／暂停／停止及恢复，退出编辑器不释放，关闭后台或进程真正结束后才归还。占用检查与子进程启动之间仍持管理锁；HTTP 无响应或发现文件消失不允许抢占。此为同一应用数据实例内的合作进程协调，不是操作系统全局声音独占。

## 末尾定位

按 [ADR-121](../development/decisions/PRODUCT-ADR-121-background-audio-end-seek.md)，新后台支持 `0 <= positionMs <= durationMs`。等于时长时，实际音源准备并应用结束位置，playing 为 true 也不再发声；原 Host 确认当前请求、期限和原组代次后释放本组灯光，其他来源继续。原生状态为 ended／位置为时长，媒体组为 Stopped；回执仍区分接纳与实际完成，不伪造末尾渲染样本。提供方保留原组身份直到确认成功，错误继续收尾，防止已结束音源留下灯光占用。

再次 Play 从零位准备；向前 Seek 可继续暂停；Stop 回到零。旧后台仍按 seekIncludesEnd:false 拒绝末尾，超时／替换／旧代次和重复请求沿用原控制规则。自然结束用终态通知，显式末尾定位只用控制完成，避免双重释放。

## 正式循环与采样精度

按 [ADR-122](../development/decisions/PRODUCT-ADR-122-background-audio-loops.md)，后台准备读取同一工程的正式循环区段，复用原 LoopPlayback、PCM 缓存预算和预编译灯光采样。启用循环时目录同时声明 `backgroundAudioLoops` 与 `audio.performanceLoops:true`；旧后台缺少该字段按 false 处理，不能发送循环操作。

原生 `audio.loopState` 可空，非空时包含区段索引 `region`、名称 `name`、十进制字符串 `pass`、实际 `exitRequested` 和待应用 `pendingExit`。退出请求为 `{"kind":"exitLoop","instance":"1","region":0,"pass":"2","requested":true}`；false 取消本遍结束退出。请求绑定原音源实例和遍数，经过原控制权和实际完整帧确认，旧目标拒绝且不停止正常节目。实际循环结束后继续后续素材；循环回跳不更换播放代次或重编灯光，显式定位仍更换代次并重置遍数。

毫秒定位按采样帧量化。以 44.1 kHz 为例，2501 ms 对应 110294 帧，公开整数位置为 2500 ms。准备以真实静音音源位置求值，并在激活前刷新健康观测；原请求按整数帧换算精确核对，不放宽为任意近似位置。媒体消费连续性由实例、时基、累计消费和累计回跳共同验证；只有已声明循环的计划可回跳。

桌面的同一后台音乐组件展示循环名称和遍数，提供本遍结束退出／取消；只读或失效状态不允许控制。详细循环区段编辑仍沿原工程模块迭代。

## 尚未关闭的边界

- 提供方恢复按下述 ADR-123 扩展；循环增量的实际验证见 TIME-001，不据此声明全部声光同步已完成。

## 音源故障与重新准备（ADR-123）

后台声明 `backgroundAudioRecovery` 与 `audio.providerRecovery: true`；两者必须一致。缺失字段的旧后台按 false 处理，客户端不发送恢复操作。`MediaAction::Recover { position_ms }`／HTTP `{"kind":"recover","positionMs":0}` 复用原媒体命令和完整控制权检查，位置须小于时长，不隐式播放。

故障后媒体所有者记录失败并停止反复拉取旧音源，继续接收原受控恢复请求。恢复先回收旧解码器／循环缓存／输出，再核对固定工程的资源摘要，从私有资源和原输出种类重建 Transport；通过实际暂停健康样本和递增时钟代次激活同一组。准备失败保留失败状态，可显式重试；不自动切换输出种类、不重启其他灯光来源或重新取得控制权。恢复后只有新的播放命令才能发声。

桌面在失败且有能力时显示“重新准备音乐”，从头准备并保持暂停；旧播放／暂停／定位禁用。输入草稿只在对应恢复请求实际 applied 后清除，失败或待确认保留。既有控制回执和音源故障信息继续显示。此路径不承诺任意线程 panic、进程崩溃或物理声卡热插拔后的自动恢复。
- 标准桌面已接音乐载入、资源副本、受控操作与编辑试听互斥，并经原生软件验收。普通编辑试听仍由原编辑会话持有；同一应用数据实例内多进程音频占用按 ADR-124 实施验证；独立监听路由、临时／正式循环的统一协调继续后续。旧版本编辑进程未持新占用，升级时须先退出旧版；不宣称可排斥旧二进制或其他软件。
- 真实声卡、蓝牙音箱、板卡和跨设备同步未在本次验收，误差阈值不作物理精度承诺。生产授权、云端／U 盘交付继续后续。
