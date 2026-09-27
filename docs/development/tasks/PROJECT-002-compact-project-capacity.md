# PROJECT-002：紧凑工程容量一致性

状态：done。2026-09-28；基线 `21f3306`；main；工作区 `/Users/sunqi/projects/stagemaster`。当前会话直接实施，无委派。

依据 [ADR-029](../decisions/PRODUCT-ADR-029-project-capacity.md)。范围：Document 私有编码与容量验证、相关核心／存储／宿主保护测试、文档；原子保存、恢复封装、工程语义与界面操作保持原契约。

验收：紧凑可接受但缩进超限的工程能编辑、保存重开与恢复；真实超限在提交前拒绝，不损伤当前工程和文件；必要修订增长已预留；小工程保留可读排版；实际 UTF-8 字节、转义和极限边界；相关回归、fmt／严格 Clippy、桌面构建及独立原生验收。原用户 14610／UE 14928 保持运行，不刷机／输出 DMX。

后续：PLAYER-003B 分离并验证设备安装事务，再接 GATT 与实际板级运行；当前容量修复不等同于设备发布完成。

## 实现与验证

新增私有 `encoding` 模块，复用 serde_json 的流式编码与标准 Write；Document 打开／新建／编辑统一使用有界计数器，保存／恢复复用带紧凑回退的完整编码。没有改存储提交算法、恢复封装、工程 Schema、UI 或播放包摘要。必要修订空间随当前父列表计算，空列表预留 38 字节；尾换行不使极限文件越界。

- 全工作区 `cargo test --workspace --locked --offline`：212 项通过（新增 7）。包含大型缩进溢出、字段完整保留、UTF-8／转义字节、精确 8 MiB 连续保存、父修订余量、整批回滚、磁盘重开、恢复副本及原文件保护、宿主超限不破坏代数／历史／检查点。
- `cargo fmt --all -- --check`、workspace/all-targets 严格 Clippy、桌面构建通过。日志 `logs/PROJECT-002-workspace-tests.log`、`PROJECT-002-clippy.log`、`PROJECT-002-desktop-build.log`。未改 UI 逻辑，不重复已有 70 项 UI 测试。
- 独立原生 QA 应用打开上一轮发现问题的 7,002 场景／5,414,064 字节工程，修改工程名称并成功建立恢复点；另存为 `data/PROJECT-002/saved.project.json`（5,414,102 字节）后真实重开，场景数和非工程信息字段完全一致。
- 重开后修改首场景为“中断后恢复场景”，确认恢复点后，仅对 QA 进程 90581 执行 SIGKILL；重启从恢复中心恢复为副本，7,002 场景／修改保留、无自动播放，保存要求选新位置。另存 `recovered.project.json`（5,414,111 字节）后来源文件与此前已保存文件摘要均不变，恢复记录清理完成。
- 原生辅助树与实际文件对照证据 `logs/PROJECT-002-native-recovered.txt`、`PROJECT-002-native-saved.txt`、`PROJECT-002-native-file-verification.json`。QA 已退出并删除临时 .app；原用户 14610／UE 14928 保持运行，无设备／固件变化。

## 出口审查

通过。结果为本次 `fix(project): keep compact projects editable and recoverable` 提交，运行契约见[工程容量](../../module-api/project-capacity.md)。8 MiB 输入边界未扩大，不能用本次修复推断支持无限工程或实时大规模性能；磁盘不足与写入冲突仍明确报告。极限外部输入若没有必要修订空间会在打开前拒绝，不改写原文件。
