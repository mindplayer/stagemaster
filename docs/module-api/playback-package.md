# 独立播放包接口

PLAYER-003A／FIXTURE-003A；ADR-028／[ADR-059](../development/decisions/PRODUCT-ADR-059-discrete-playback-attributes.md)；容器 1，执行语义 1 或 2。`stagemaster-package` 是 `no_std + alloc`；不依赖工程 JSON、桌面、文件系统、蓝牙、授权或物理输出。

## 调用边界

```rust
// 主机：真实工程快照 → 选择 → 编译与完整验证。失败返回带编辑位置的问题，不返回部分包。
let built = document.build_package(&[PackageSelection::Sequence { id }])?;
// built.bytes / built.report；不会建立新工程修订，也不改变保存／撤销状态。

// 独立端：ReadAt 由只读文件或已提交的 Flash 分区实现，读取版本须固定。
let archive = Archive::open(&reader)?; // 全包散列、目录、所有节目语义及预算扫描
let program = archive.load(&reader, index)?; // 再核对块散列，只分配这个节目的 Plan
let mut player = Player::new(program.plan, monotonic_ms);
player.execute(0, monotonic_ms)?;
player.advance(next_ms)?;
program.output.render(player.values(), &mut slots)?; // 无分配，512 字节，粗细通道统一 u16 求值
```

`Archive::open` 不创建播放器、不自动启动，不把“首个节目可读”当整包有效。`Archive` 只保留来源、目录、各节目资源报告；装载需要调用者继续提供固定版本读源。`load` 读入单块并重查散列再解码；返回后块缓冲释放。未来安装层需先验证未提交分区，再切换活动指针，并保证活动读源不被后台传包改写。

`Output` 与 `Plan` 分开拥有，移动 Plan 给 Player 时不必复制计划。输出空槽为零；8 位属性使用 u16 高字节，16 位粗／细地址不要求相邻。映射错误或 values 维度不符时原输出数组不变；停止恢复计划默认值。

主机 `Builder` 只积累编码块，不同时持有所有节目计划。它按类型、16 字节标识排序；同一快照／选择／编译版本的字节确定，选择顺序无关，重复标识拒绝。来源摘要使用完整 Value 的紧凑有序 JSON UTF-8 字节，不含末尾换行，流式送入 SHA-256；与原始文件空白排版无关。包包含全部被编译的线路映射、默认值、步骤标签／目标、时间、循环及效果。当前工程编译器要求整个配适在一个输出域、一条线路；不静默删除未选节目以外的已配适灯具默认值。

## 资源与错误

参考档位 `reference-single-line-v1`：64 节目、2 MiB 总文件、16 KiB 目录、32 KiB 单块、128 步、每步 128 个效果通道、512 个属性／DMX 槽、每条 2–32 帧、每字符串 1–512 UTF-8 字节。内核时间和累计效果／关键帧限额继续生效。扫描使用定长容器、显式计数和范围；不按不可信长度直接建立 Vec。

参考装载峰值上限 64 KiB，报告包括目录常驻、单块编码缓冲、u16 目标／默认／当前／起点数组、64 位保守结构大小、字符串、每次堆分配 32 字节余量和 8 KiB 扫描／散列工作区余量。所有数据从内容重算，不信任文件自报资源。这个值不是堆分配器或板卡运行测量：无线栈、真实栈深、分区、双计划切换另行实测。

`Error` 区分不支持版本、完整性不符、字段无效、超限、读取、分配和内核计划错误；块扫描错误保留规范排序中的节目索引。工程适配把可定位错误指向灯具、场景或场景列表。未知曲线／过渡／类型、错误时间、重复标识／槽位／效果属性、非递增关键帧、零时长自动循环都拒绝。SHA-256 是完整性校验，攻击者可重新计算；这不是签名、加密或授权。

## 桌面／文件系统

- `package_build(generation, selection)`：复制当前已应用快照，退出会话锁后编译；只缓存最后一个成功结果，返回 generation、随机缓存标识、报告或问题。
- `package_export(generation, token)`：严格核对版本／缓存标识，在工程操作队列内打开系统另存对话框；整个导出不持有播放轮询的会话锁。取消不写文件。
- UI：工程工作区的“播放包”；搜索、类型筛选、跨页选择、选择筛选结果、清空、生成／取消、问题定位、过期保护、另存。草稿先由既有编辑队列应用；取消只放弃结果，已开始的有界计算可完成。
- `PackageFile::select(path, source)`：仅 `.smpkg`，不接受当前工程路径／符号链接；覆盖目标必须已经是完整有效包，其他文件或损坏包用新文件名。
- `PackageFile::save(bytes)`：独立校验全包，复用工程存储的稳定锁、前后基线比较、目录内临时文件、fsync、原子替换／不覆盖创建。成功后可返回目录同步提醒；不修改工程修订。

软件参考包的生成、保存、检查不表示设备安装就绪；当前没有包签名或播放许可。DEVICE-002 已完成专用开发身份下的真实 GATT 安装；物理输出另验收。商业身份、内容保护与许可独立于参考包；按 [ADR-101](../development/decisions/PRODUCT-ADR-101-commercial-security-boundaries.md)，暂不考虑限时，具体加密授权协议后议。

## 字节格式 v1

文件头 64 字节；以下整数均为小端，CBOR 内则遵循 RFC 8949。

| 偏移 | 长度 | 含义 |
| --- | --- | --- |
| 0 | 8 | `STMPLAY\0` |
| 8 | 2 | 容器版本 1 |
| 10 | 2 | 头长度 64 |
| 12 | 2 | 执行语义 1；有离散属性时为 2 |
| 14 | 2 | 参考档位 1 |
| 16 | 4 | 目录长度 |
| 20 | 4 | 完整文件长度 |
| 24 | 2 | 节目数量 |
| 26 | 6 | 保留，必须全零 |
| 32 | 32 | SHA-256(header[0..32] + file[64..]) |

后接定长 CBOR 目录数组：`[compiler, projectId16, revisionId16, snapshotDigest32, projectName, entries]`。

- compiler 与头部一致：语义 1 为 `stagemaster-lighting-1`，语义 2 为 `stagemaster-lighting-2`。
- entry 为 `[kind, id16, name, offset, length, blockDigest32]`；kind 为 0 场景、1 列表。
- offset 相对于所有节目块的起点；排序后必须从零连续，无空洞／重叠；块总长度必须正好覆盖文件尾。
- id 和摘要为确切长度的 CBOR 字节串；字符串为有界 UTF-8 文本；无压缩。

每块为 `[universe, mappings, defaults, repeat, steps]`：

- mapping：`[coarseSlot, fineSlotOrZero]`，槽位 1–512，全节目唯一占用，0 仅表示无细通道。
- step：`[id16, name, number, target, delayMs, fadeMs, waitMsOrNull, effects]`；target/defaults 同长度；null 表示手动等待。
- effect：`[attributeIndex, low, high, periodMs, phaseU16, dutyPercent, curve, frames]`。
- curve：0 平滑、1 三角、2 脉冲、3 关键帧。前三种 frames 必须为空。
- frame：`[phaseU16, valueU16, transition]`；transition 0 保持、1 线性、2 平滑。
- 标为单场景的节目必须一步、无循环、零延时／渐变、手动等待；仍可有持续效果。

FIXTURE-003A 增补语义 2：带直接切换属性的块是 `[universe, mappings, defaults, repeat, steps, snapAttributes]`，末项为 1–512 个严格递增、在属性范围内的 u16 索引，不允许重复／空数组或与任何一步的动态效果属性冲突。旧五字段块仍可与新块共存于语义 2 包；语义 1 头部不允许新块。每属性延时后直接切换，其他属性照常渐变；暂停／跳转／释放详见[运行接口](sequence-preview.md)。

`Archive::semantics()` 返回包所需执行语义。目标支持性由安装／发布上层核对；旧固件读取语义 2 头部会拒绝，不会当旧渐变计划执行。本次未升级实板，当前工程编辑仍生成语义 1；功能区间接入时再贯通目标兼容提示。新解码器支持旧包，旧参考包重建逐字节相同。新增 `Usage.snap_attributes`，预算增加 `2 × 索引数` 和一次分配余量 32 字节；空列表无新增堆分配。计划有效负载由 `snap_buffer_bytes()` 单列，不能漏算成仅 value／effect 字节。

整数曲线、时间／跟踪解释或输出量化若产生行为变化，必须提升执行语义／编译器版本，旧解码器显式拒绝。

全部容器确切长度，无无限长数组、标签、浮点、未知字段或块尾附加对象。确定性编码器使用最短整数；解码器允许 CBOR 合法的较宽整数表示，只要数值范围和所有契约不变，不将字节规范化视为身份校验。

## 可复现独立读包

```sh
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run --locked --offline \
  -p stagemaster-package --example inspect -- data/PLAYER-003A/final.smpkg
```

示例使用文件随机读取，无工程编译依赖；逐个装载节目，以合成单调时钟重放 10 秒／400 帧并打印输出摘要。它不会连接设备或发送 DMX，也不能替代现场时序／长稳测试。
