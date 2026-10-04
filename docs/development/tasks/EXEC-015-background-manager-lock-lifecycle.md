# EXEC-015：后台管理锁生命周期

状态：限定修复、自审、实际验收完成，完整回归出口恢复；2026-10-04 基线 main `266aa76fcfe6ec1fa6b95758f178bf337d84b68f`，计划先提交 `c34062e`，结果为本次 `fix(execution): release manager locks at transaction exit` 提交。当前 6.1 Sol 单写者，主工作区只保留用户 output/。持续 goal active，不创建会话／子代理，不启用旧工作器。

## 问题与证据

FIXTURE-009 原全量 `logs/fixture-009-workspace.log` 退出 101，旧 `execution::manager::media_tests::missing_configuration_and_damaged_music_do_not_publish_background_records` 在 `media_tests.rs:183` 顺序预留音频失败，报“另一个应用正在管理此后台”；独立目录为项目 tmp/.tmp6byohb/execution。单独运行原断言通过（`management-lock-diagnostic.log`），仅作为诊断，不能覆盖全量失败。

`execution/files.rs::lock` 当前返回裸 File，事务退出只关闭局部描述符，并把所有锁错误映射为忙。复用 PROJECT-003 已验证的私有 guard 思路；复制／受控子进程保留描述符是待确定性证明的候选机制。本次原失败具体持有 PID／唯一根因未知，不写成已证明。

## 限定范围与失败行为

- 先保留原生命周期，补确定性复制／受控子进程回归，真实管理路径下一操作必须在旧描述符仍存活时尝试；失败先复现、记录原始锁错误。测试子进程限时并只清理自身。
- 证实后，私有管理锁仅成功取得时构造 guard；正常、早退及 unwind 时明确解除自己持有的锁。失败取得不得解除其他持有者，旧描述符迟到关闭不得解除新写者；实际忙仍立即拒绝，不等待、删除锁文件或盲目重试。
- 系统错误与 WouldBlock 分开定位，保持私有目录权限与文件打开边界。是否需要普通文件校验由证据评估，不能跟随锁修复扩大权限。
- 不更改音频 OutputScope 所有权、长期宿主 lifetime.lock、进程死亡判断、管理事务范围、控制租约、后台协议、持久化格式、播放器／时钟、UI 或依赖。若证据指向另一个边界，单列事实后只修受影响部分。

## 验收与交付

新增确定性生命周期／竞争测试、原失败测试及实际桌面管理／音乐保护专项；原独立断言不削弱。运行原全工作区、严格 Clippy／Rust 格式／差异与文档检查；有生产 Rust 变化则重建正式 .app，并在项目内隔离实例经真实 UI 载入／关闭／再次载入软件后台，确认只读重开与设备未连接，不播放音乐、UE 或实灯。没有 UI 源码变更不重复 UI 测试冒充本轮结果。

源码在 apps/desktop/src/execution/，新锁／测试分文件；证据 data/EXEC-015/，日志 logs/exec-015-*，缓存／临时目录明确项目 tmp/。自审、更新 STATE 和本单、逐项提交及核对版本。恢复完整出口后继续 H3；历史音频预留所有原因、H1 听音、GPU 首帧、完整实灯／物理／长期和客户 UE 包仍独立未关闭。

## 确定性复现与最小实现

限定计划先提交 `c34062e`。先将原裸 File 生命周期等价提取到私有 ManagerLock，未加 Drop；新增回归在修复前 1 通过／4 失败／1 子进程入口 ignored，退出 101（`logs/exec-015-lock-red.log`）。父测试实际以安全 `Stdio::from(File::try_clone())` 传递描述符给限时子进程，原锁仍持有时得到实际 WouldBlock；父事务退出后下一真实 `Manager::reserve_editor_audio` 仍忙，且子进程明确仍活着。由此证明一个生产生命周期缺陷，而非推定 FIXTURE-009 原失败唯一原因或全部历史音频问题。

修复仅在成功取得后构造 guard，退出 Drop 尽力明确 File::unlock；系统 unlock 错误无法由析构返回，仍关闭自己的 File，不虚称系统失败绝不发生。所有原管理调用／编辑音频操作持有时段保持，取得失败不调用 unlock，迟到旧副本关闭不能解除新写者。实际 WouldBlock 仍为原忙提示，系统错误另附原因。原打开选项／0700 目录检查、后台记录、音频 OutputScope／长期宿主锁／死亡判断与协议均未改，没有新依赖或 unsafe，属于私有资源生命周期修复，无公共 ADR 变化。

修复后新 5 项通过（`logs/exec-015-lock-green.log`）：受控真实子进程仍存活时下一管理操作成功；旧副本迟到关闭保护新写者；早退与受控 unwind；取得拒绝保持真实写者且不发布后台记录；忙／系统／打开错误分类。ignored 仅是由父测试真正调用并检查退出的子进程入口，不是跳过保护测试。第一次测试临时目录权限不满足原私有保护，已在测试 fixture 显式设置 0700，保留 `lock-setup-failure.log`，未放宽生产目录规则。

锁／领域测试／子进程 helper 分别 41／137／78 行；入口只组装类型，原 media／manager 事务规则与旧测试断言不改。

## 正式桌面与工程保护

正式重建 `tmp/framework-001-light-target/debug/bundle/macos/舞台大师.app`，使用 `STAGEMASTER_ACCEPTANCE_INSTANCE=exec-015-uEemPR`；独立 `data/EXEC-015/manager-lock.project.json` 来自 EXEC-014，不改来源或用户默认目录。

1. 真实界面打开副本，只选择“01 · 金色序幕”场景载入软件后台，得到独立运行 `0496c15c-6c94-4030-bcf6-b343931489d3`，默认只读、待执行、设备未连；明确确认关闭后 current 消失。
2. 再选择“02 · 蓝色起势”载入，得到不同运行 `a2859ea9-e26e-44c5-a84d-1ad18c49d915`，依然只读。两个真实 sources.json 均 version 1、一个场景＋手动层、没有音频字段；没有打开音乐输出或 UE，没有实际物理发送。
3. 正常退出编辑窗口，原后台保留。按同一隔离实例重启，经最近工程打开后进入后台：仍为第二场景及同一运行、只读不抢权，不调用再次载入。截图 `native-reconnected.png`、完整 AX `native-reconnected-ax.txt` 和实际身份断言 `logs/exec-015-native-identity.log`。
4. 确认关闭本轮第二后台，界面回到尚未载入、工程已保存；current 不存在，最后 AX `native-final-ax.txt`，自有界面退出。不删除 manager.lock 或运行证据。

来源与副本 SHA-256 均保持 `a27abb8b90de6bf80884d6bb61d6b3e0cff243144660c8f54f29d515e123b097`；默认最近工程保持 `69d620946c1bc483de120ab9570ede44ca50e663e4e93a3fd09071f5405dc637`，用户 output/ 不动。正式 GUI 日志 `logs/exec-015-native*.log`，证据／日志／缓存保持项目内忽略。

本轮仅 Rust 私有生命周期与测试变化，没有 UI 源码／播放器／UE 修改；不把 FIXTURE-009 的 418 UI 与组件验收写成本轮重复执行。首次构建漏传专用 CARGO_TARGET_DIR，发现后取消自己的构建（`logs/exec-015-build.log`，130），未触碰项目外目录或删除已有缓存；按明确项目 tmp/ 目标重新构建通过（`build-final.log`）。

## 最终实际验证与审查

| 实际命令／检查 | 结果／日志 |
| --- | --- |
| 新确定性管理锁测试（修复前／后） | 1 通过／4 失败→5 通过；父测试真正调用子进程，`lock-red.log`／`lock-green.log` |
| `cargo test --workspace --locked --offline` | **1230 Rust＋2 文档，0 失败**，出口恢复；`workspace.log`，原失败与所有音乐／后台管理保护断言不改 |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 通过；`clippy.log` |
| `cargo fmt --all -- --check`、`git diff --check` 与交付文档／证据检查 | 通过；`fmt.log`／`delivery-check.log` |
| `node tools/desktop/run.mjs build`（明确项目 TMPDIR／CARGO_TARGET_DIR） | 正式 `.app` 通过；`build-final.log`，原 UI 生产构建与大包提示保持 |
| 正式 GUI 管理与重开 | 两次不同场景载入、明确关闭、再次载入及同一后台只读重开通过；无设备／音频／UE，见上节 |

全量 3 个 ignored 分别为原 abrupt_exit_worker、PROJECT-003 inherited_file_holder 和本轮管理锁 inherited_file_holder，均由各自父测试实际启动并验证，不忽略原失败。日志头部并发 Cargo 等待缓存锁属于构建互斥，不是测试保护禁用或产品竞争错误。

自审核对成功取得／释放所有权、失败不能解锁其他写者、原管理操作范围、目录／记录与音频长期保护、私有类型接线、旧断言未改及正式原生证据；无需公共 ADR 或格式修改。完整原命令通过证明本轮出口恢复，不证明 FIXTURE-009 当时具体持有 PID，亦不覆盖所有历史音频偶发原因。其旧失败日志和交付记录保持原事实。

提交后将实际版本写入 `data/EXEC-015/verification.json` 并核对 HEAD／工作区。接续 H3 FIXTURE-006 尚未完成的独立语义；不重复已经验证的速度、色盘／图案／镜头增量，不伪造未知档位／物理测量。H1 听音、历史音频预留所有原因、GPU 首帧、H3 完整实灯、H4 物理／长期和 H5 客户无 UE 编辑器包仍未关闭，goal active。
