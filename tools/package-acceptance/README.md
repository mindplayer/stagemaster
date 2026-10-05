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
