# 工程检查服务

实现：CHECK-001，依据 ADR-026。工程核心、桌面宿主与 UI 导航分别独立；不改变工程 JSON 版本或执行语义。

## Rust 调用

```rust
let document = stagemaster_project::Document::decode(&bytes)?;
let report = document.check();
// report.desktop_ready 只表示现有电脑播放编译通过。
// 调用不会保存文件、装载播放器或访问硬件。
```

Document 已完成 Schema、引用、档案和配适重叠校验；不合法文件在打开时被拒绝，不伪装成可修复编辑快照。检查在现有有效 Document 上报告缺失配适、跨域／线路限制、空工程、节目计划容量与三维缺失灯位。所有灯具都纳入当前单域单线路输出，不因场景暂未引用就忽略配适。

`CheckReport` 序列化字段：

| 字段 | 含义 |
| --- | --- |
| projectId / revisionId | 工程与保存修订身份；revisionId 不能识别未保存编辑 |
| desktopReady | 有节目、配适条件成立、全部节目编译成功；警告不阻断 |
| issues | code、severity(error/warning)、中文 message、location |
| programs | 名称、location、status(passed/failed/blocked)、usage 或 null |
| limits | attributes、steps、targetValues、effectChannels、keyframes，直接使用执行器常量 |

位置是带 kind 的封闭联合：`fixtures`、`scenes`，或携带 `id` 的 `fixture`、`placement`、`scene`、`sequence`。它不是路径字符串、命令或可执行脚本。稳定问题码：`patch.empty`、`patch.missing`、`patch.multipleLines`、`plan.attributes`、`program.empty`、`program.compile`、`stage.unplaced`。

配适公共错误阻断所有节目并保留完整节目清单，不复制数千条相同编译错误。配适通过后逐个调用既有场景／列表编译管线，失败不终止后续节目；每个 Plan 统计后释放。一次检查复用本 Document 的只读 ProjectView，避免每节目重建全部视图；内部辅助接口不接受外部视图。步骤／目标总量在分配前检查并输出实际数量与上限，效果与关键帧累计超限明确注明“已累计”。

`usage` 包含上述五种实际数量，另含：

- `valueBufferBytes`：目标／默认／当前／渐变起点的 u16 数据，公式 `(steps + 3) × attributes × 2`，不含元数据与分配器。
- `effectBufferBytes`：本机架构下效果通道结构及关键帧负载，不含外层 Vec 和分配开销。

两者均不是序列化设备包体积，也不是整体 RAM 上界。编译失败时 usage 为 null，不能以零占用代表成功。当前无设备包预算计算。

## 桌面适配

`check_request(generation: u32) -> { generation, report, deviceRelease: "unavailable" }`。

单独服务门拒绝重叠检查。锁定 Session 时验证 generation 并克隆 Document，随后释放会话锁，再计算报告；不会更新 generation、contentVersion、文件、历史或播放 epoch。UI 调用前通过编辑队列提交有效草稿，无效输入中断并聚焦原字段。

报告仅在 projectId、generation 都相同且没有待应用草稿时有效。编辑、历史、保存、重开都会使旧 generation 失效。异步结果允许返回，但按当前状态标注历史报告；定位操作再次进入队列，在草稿处理后重新核对 generation，拒绝过期对象导航。新开不同工程的组件卸载后丢弃旧请求结果。

取消表示放弃这次结果；已开始的只读后台编译自然结束，期间不再开启第二份检查。界面明确显示这一点，不承诺中断当前步骤。当前每个节目检查仍不提供进度流或硬截止时限。

## UI 与复现

“工程 → 工程检查”：错误／提醒搜索筛选，问题和节目每页 20 条，容量按需展开。灯具定位清除配适筛选并选中目标；场景定位清除场景搜索；列表定位清除列表／步骤搜索并保留可用的步骤选择；缺灯位定位到舞台待布置灯具。导航不触发播放。报告、搜索和翻页在工作区切换后保留。

开发期只读 CLI：

```sh
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run -p stagemaster-project --example check_project --locked --offline -- path/to/project.json
```

标准输出为报告 JSON，标准错误为编译检查耗时；输入上限仍为 8 MiB。命令正常生成诊断即正常退出，调用者必须读取 desktopReady 才能判断工程是否通过；它不是设备部署命令。当前设备发布由宿主明确声明不可用，未来应由独立目标适配器提供资源、安装及实际执行证据。

## UX-028：宿主音频资源完整性（ADR-057）

响应附加 `audioResource: null | { fileName, resources: { local, companion, localSource } }`。每份文件状态为 `{state: "valid" | "missing" | "notSaved"}` 或 `{state: "invalid", message}`；本机来源为 `cache`／`companion`／null。核心 Document 报告与 JSON 不变。捕获路径与文档在同一锁内，实际文件检查在锁外；取消仍是放弃报告，已开始的校验自然完成。

`Resources.inspect()` 采用现有播放缓存优先规则，独立验证工程旁同名 `.assets` 副本；SHA-256 使用现有 16 KiB 固定缓冲，每份上限 512 MiB，同路径不重复读取。检查不会写缓存／创建文件夹、解码音频、占用声卡或调用播放器。无音乐为 null，不作为故障；未保存工程的随附状态为 notSaved。文件系统可独立变化，报告只对检查时文件有效。

UI 分别展示灯光编译、本机文件及随附文件；音乐入口沿同一 generation 和草稿保护导航到既有音乐页，该页仍沿用现有音频准备／重新定位流程。检查通过不等于音频设备或声光同步验收，设备包仍不含音频。检查中的“保存并补齐”入口即使工程未改也可调用既有归档流程；提交前再次核对 generation，之后旧报告过期，需要重查。
