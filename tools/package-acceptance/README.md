# 最大长度包的软件验收语料

本目录仅含测试／离线生成器复用的确定性原始容器和读请求记录，不进入生产包 API，不发送 DMX，不连接或安装到设备。依 [MEMORY-004](../../docs/development/tasks/MEMORY-004-maximal-malformed-package.md) 补 H4 最大畸形输入的软件证据。

`bounded_corpus.rs` 用原节目编码器产生 320 属性／17 步的合法 64 节目大包对照，复用 MEMORY-003 已验证的代表性形状，但不读写历史原包；再独立编码目录／封包。四个恰好 2 MiB 的文件分别覆盖正确散列下单块超限、极大目录计数、目录未覆盖文件尾，以及包尾完整性损坏。另两份有效外层散列输入在第 64 项的语义／块摘要处失败，前 63 项仍须完整扫描。不是所有合法语义形状的理论最大值，也不是对历史裸包的安全背书。

`reader_metrics.rs` 只记录正式 `ReadAt` 的每次请求，验证完整散列的连续覆盖和请求长度，不能当作分配器峰值、CPU 占用或 ESP32 期限。NOR 测试使用原严格 NOR 模型，完整接收、拒绝提交、保持旧快照／A 槽、取消／换启动恢复并再次合法安装；不是真实断电或维护输出静默证据。

在项目根运行，缓存和产物留项目内：

```sh
TMPDIR="$PWD/tmp" CARGO_HOME="$PWD/tmp/cargo-home" CARGO_TARGET_DIR="$PWD/tmp/framework-001-light-target" \
  cargo test -p stagemaster-package -p stagemaster-nor-store --test maximal_malformed --locked --offline
TMPDIR="$PWD/tmp" CARGO_HOME="$PWD/tmp/cargo-home" CARGO_TARGET_DIR="$PWD/tmp/framework-001-light-target" \
  cargo run -p stagemaster-package --example malformed_capacity --locked --offline
```

文件与 TSV 报告输出 `data/MEMORY-004/`，记录主机耗时不预设为板卡通过。任何真实坏包安装必须另核对具体设备、授权、维护状态、原包恢复与期限证据；不能直接用此生成器绕过原发布保护。

## 源工程到安装后完整剧本

[EXEC-017](../../docs/development/tasks/EXEC-017-source-package-script-consistency.md) 新增 `stagemaster-runtime` 的 `script_replay` 文件验收范例。先以完整工程和原选择重编译核对包身份，再用源工程直接编译的 Player／输出，对照真正 Archive／Installer／FileStore 安装后的 Runtime；参考播放器不来自解码包。只覆盖单次三步“人工保持→定时延时／渐变／等待→人工保持”，其他形状明确拒绝，不冒称全类型脚本已验收。

```sh
TMPDIR="$PWD/tmp" CARGO_HOME="$PWD/tmp/cargo-home" CARGO_TARGET_DIR="$PWD/tmp/desktop-012-exit-target" \
  cargo test -p stagemaster-runtime --all-targets --locked --offline
TMPDIR="$PWD/tmp" CARGO_HOME="$PWD/tmp/cargo-home" CARGO_TARGET_DIR="$PWD/tmp/desktop-012-exit-target" \
  cargo run -p stagemaster-runtime --example script_replay --locked --offline -- \
  data/DESKTOP-010/native.project.json data/DESKTOP-010/native-saved-a.smpkg aab97a9f-ceeb-4147-8872-6a9f9be5e72d
```

第二条依赖本机已有的真实验收快照／包；其他机器可提供自己的项目内同形状工程和包，测试夹具则由 Git 源码复现。逐毫秒比较 512 槽、步骤／时间／状态／实例；验证暂停继续、重复下一步、旧修订与旧租约拒绝、无控制者自动推进、重连不重启和停止默认值。装载完成后测试读源主动拒绝任何读请求，报告运行时读数须为零。输入有界、拒绝链接／父目录跳转／越界与 output/，不修改原文件；不是公开的抗本地并发篡改服务。软件接受策略不等于正式安装许可，无声音／GPU／通信／真实 DMX，不把合成毫秒当墙钟或板卡时序。
