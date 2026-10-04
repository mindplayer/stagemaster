# PROJECT-003 最近工程写锁生命周期

状态：本轮修复、自审与验收完成，完整回归出口恢复；当前 6.1 Sol 单写者。2026-10-04 基线 main `5ca37121b3d698f946623c3b3900eed4ba0ac707`，限定计划先提交 `8395957cf8b127a0b294cdbe45960586ec0ad543`，结果为本次 `fix(project): release catalog locks at transaction exit`。主工作区开始干净，仅用户 `output/` 未跟踪。范围仅桌面最近工程锁、相关测试／诊断和状态；保护用户工程、output/、目录格式和非阻塞竞争拒绝。

## 证据与问题

- EXEC-013 最终原完整回归退出 101，旧 `multiple_instances_merge_latest_data_and_busy_writer_preserves_it` 在 `apps/desktop/src/recent/store_tests.rs:90`，即顺序第二次 remember，返回“其他窗口正在更新最近工程”。原日志 `logs/exec-013-workspace-delivery.log`，此前 EXEC-009 已记录偶发但根因未确认。
- `Store::update` 获取非阻塞 File 锁，只依靠局部句柄关闭释放；它将所有锁错误映射为同一提示，缺少实际错误分类。近期宿主测试会并发创建子进程；是否继承锁句柄须受控证明，不把推测当根因。
- 原最近工程六项隔离通过，`logs/exec-013-recent-isolated.log`；没有重写断言或把此结果当完整命令通过。
- [Rust File::try_lock 官方说明](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock)：最后复制／继承句柄关闭或明确 unlock 才解除。项目 tmp/ 实际复制句柄实验返回 `WouldBlock`，明确 unlock 后下一写者取得，日志 `logs/exec-013-lock-probe.log`。只证明候选机制，不等同生产重现。

## 实施要求与失败行为

1. 先保留实际锁错误分类与受控竞争／继承证据，形成可稳定失败的生产生命周期回归；隔离检查只能记专项，不能覆盖已有全量失败。
2. 根据证据最小修正锁生命周期，成功、错误、早退均释放自己取得的锁；仍有真正写者时立即拒绝且不改目录。不得阻塞等待、删除锁文件、放宽容量或禁用保护。
3. 不改最近工程 JSON、持久化格式、原子写入、限额／内容校验或 UI 交互，不新增通用代理／后台平台或第三方依赖。音频保留锁独立跟踪，不未经证据批量修改。
4. 原六项存储测试、新确定性回归、全工作区原测试命令／严格检查／格式均实际执行；若生产桌面改变则重建正式包并作相关打开／重开检查。
5. 自审、提交、更新 STATE；恢复出口后接续交接路线 H2 剩余批量／保护。不能把一次孤立复测通过写成所有历史锁问题已根治。

## 本轮限定与验收记录

先将目录独占写锁提取为私有生命周期对象，保持旧行为，加入受控复制／子进程继承描述符的确定性失败回归；使用安全标准库和正式测试子进程，不增加 unsafe、第三方或全局模型工具。证实问题后，仅在自己已取得的锁退出时明确解除；成功／早退／异常均保护，失败取得不解除其他写者。

目录事务、非阻塞拒绝、原子提交与 JSON 不变，属于局部资源生命周期修复，不改变公共模块／格式／时间或控制权，无需新公共 ADR。系统锁错误与真正 WouldBlock 分开定位；其他模块特别是音频保留锁不跟随修改。实际证据 `data/PROJECT-003/`，日志 `logs/project-003-*`，临时／缓存在根目录 `tmp/`。

## 实现与自审

- 提取私有 `recent/store/write_lock.rs`，只有成功取得才构造 guard；事务退出执行明确 `File::unlock`，返回错误、容量拒绝及 unwind 都走 Drop。unlock 为析构中的尽力解除，失败仍关闭自己的 File；不承诺无法观测的系统错误绝不发生。取得失败的 File 只关闭，不调用 unlock，不解除真正持有者。
- 保留原普通文件检查、打开选项、非阻塞拒绝、JSON 校验、12 条／64 KiB 限额、临时原子提交及目录同步。只将 WouldBlock 留作“其他窗口正在更新”，系统锁错误单独附原因；不引入等待／重试、删除锁文件、新依赖或 unsafe。
- 原六项 `store_tests.rs` 只调整 OpenOptions 导入，断言完全未改。新增生命周期测试独立文件：复制／真实子进程继承、成功／早退／异常、旧副本关闭不得解除新写者、失败取得不得解除真正写者、忙状态目录字节保护、非文件与系统错误分类。
- 受控子进程通过安全标准库 `Stdio::from(File::try_clone())` 持有原锁描述符；父进程在子进程仍活着时执行下一次真实 `Store::remember`。它证明原生命周期存在“事务已结束但描述符仍在”的确定性问题；旧 EXEC-013 全量现场没有具体持有 PID，不能声称已证明那次偶发的唯一根因。测试子进程有 6 秒边界，正常退出检查结果，失败只清理自己的子进程。
- 存储／锁／测试分别 151／42／121／187 行；没有超限大文件或入口堆积。自审核对取得与释放所有权、退出路径、迟到副本、旧断言、原子写入及证据完整性；没有修改音频、执行控制、UE 或 UI。

## 实际验证

所有项目命令使用根目录 `tmp/`；Rust 使用项目 `tmp/cargo-home`、`tmp/framework-001-light-target`，npm 缓存在项目 tmp/。先失败再修复，不禁用保护或降低独立验收。

| 实际检查 | 结果／日志 |
| --- | --- |
| 原仅关闭 File 生命周期＋新增确定性回归 | 2 通过／3 失败，真实下一事务及早退／副本用例失败；`logs/project-003-lock-red.log`，退出 101 |
| 最终 `cargo test -p stagemaster-desktop --locked --offline recent::store -- --nocapture` | 原 6＋新 5＝11 通过，子进程入口由父测试调用；`logs/project-003-lock-green.log` |
| 最终原命令 `cargo test --workspace --locked --offline` | 1202 Rust＋2 文档，0 失败；`logs/project-003-workspace-final.log`。此前同命令也通过，保留 `project-003-workspace.log`，不替代最终结果 |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 通过，`logs/project-003-clippy.log` |
| `cargo fmt --all -- --check`、`git diff --check` | 通过，格式日志 `logs/project-003-format.log`；交付前再次检查差异 |
| `node tools/desktop/run.mjs build` | 正式 `.app` 通过，`logs/project-003-build.log`，当前修复源构建，非 Vite 页面 |

完整回归中的两项 ignored 分别是本次 `inherited_file_holder` 与原 `abrupt_exit_worker`，仅为子进程独立入口，由父测试真正启动与检查；没有忽略原失败测试。新模块首次相对测试路径错误已修正（`project-003-lock-path-failure.log`）；严格检查发现测试手动 panic 写法，改为等价 assert 后重新通过（`project-003-clippy-first-failure.log`）。原失败日志保留，不写成全过程零失败。

没有 TypeScript／UI 源码或 UE 修改，本轮未重复 UI 测试、类型／UE 自动化；正式打包执行其原 UI 生产构建，不能把上轮 409 UI 记作本轮运行。

## 正式桌面验收与工程保护

使用正式 `tmp/framework-001-light-target/debug/bundle/macos/舞台大师.app`。复用已有 debug `STAGEMASTER_ACCEPTANCE_INSTANCE=project-003-aVuSb6`，最近目录位于 `tmp/desktop-project-003-aVuSb6/navigation/`，不改产品代码或用户默认目录；两份独立工程在 `data/PROJECT-003/`。原生操作走真实界面及文件选择器。

1. 从空隔离目录打开第一工程，实际新增一条。受控外部写者 PID 3093 持有该真实 recent.lock 后，正式界面打开第二工程：工程成功、已保存，显示“最近工程记录未更新：其他窗口正在更新最近工程”；持锁前／期间 recent.json 字节完全相同。
2. 写者按其 60 秒边界明确 unlock 并正常退出，没有删除锁文件。日志确认退出后再次正式打开第二工程，警告清除、记录变为两条，第一条身份不变。迟后的释放标记不是解除锁的证据，不把已结束进程记作仍运行或人工释放。
3. 最近列表按第一份确切文件名搜索，只命中该条；取消保留第二工程且不改内容。再次打开最近第一工程，前移但不新增身份；退出正式应用后按同一隔离实例重启，两条记录保留，实际再次打开第一工程且“已保存”。不宣称最近弹窗搜索草稿跨关闭保持（原行为未改）。
4. 证据为 `native-before-conflict.json`／`native-during-conflict.json`／`native-after-lock.json`／`native-after-recent-open.json`、相应 AX 与 `native-recent-reopen.png`、最终 `native-final-ax.txt`；进程日志 `logs/project-003-native-*`。本轮没有音乐、UE、设备或后台运行操作，也没有硬件／刷机／实灯。

来源 `data/EXEC-013/output-master.project.json` 与两份 `data/PROJECT-003/recent-lock*.project.json` 前后 SHA-256 均为 `a27abb8b90de6bf80884d6bb61d6b3e0cff243144660c8f54f29d515e123b097`，无工程编辑或历史。用户默认 `data/navigation/recent.json` SHA-256 仍为 `69d620946c1bc483de120ab9570ede44ca50e663e4e93a3fd09071f5405dc637`。最后保留已保存的隔离工程；受控持锁／单测子进程已退出，用户 `output/` 保持。

## 结果与下一步

相关源码／测试／工单／STATE 逐项提交，本轮版本为 `fix(project): release catalog locks at transaction exit`；提交后核对 HEAD／工作区并将实际完整版本保存到 `data/PROJECT-003/verification.json`。数据、截图、原始日志、临时试验和缓存保持项目内忽略，不加入主仓库。

EXEC-013 当时最后全量失败仍是历史事实，本轮确定性修复与原全量恢复单列，不覆盖旧记录。接续 H2 执行工作面的代表演出批量操作／保护，先核对已有目录、来源身份／版本和控制权，再限定增量，不前置完整专业控台页。AUDIO-020 听音、历史音频预留原因、GPU 首帧、指定真实灯型、物理差分／实灯／长期及客户无编辑器 UE 打包仍未关闭，完整 goal active，不据本次锁修复关闭整个交付目标。
