# NOR Flash 播放包存储

PLAYER-003D，基线 `9ae315a`；实现 `crates/stagemaster-nor-store/`。契约依据 [ADR-032](../development/decisions/PRODUCT-ADR-032-nor-package-store.md)，与[安装事务](package-installation.md)、[传输](package-transfer.md)分离。

## 调用关系

```rust,ignore
// partition_nor 来自经过分区校验的专用区域，不传整个 Flash 驱动。
let device = NorDevice::new(partition_nor, Layout::new(2 * 1024 * 1024)?)?;
let (reader, recovery) = Installer::open(device.open_read_only()?, boot_id)?;
let installed = reader.snapshot()?; // 租约；只读源可跨存储会话存活
let program = installed.load(program_index)?;
drop(reader);

// 只有可信运行层在现场输出已停止、所需内存已释放后才能进入安装维护状态。
let (installer, _) = Installer::open(device.open_for_installation()?, next_boot_id)?;
let mut service = Service::new(installer)?;
// 按传输契约验证权限再 attach；诊断握手不能自动获得安装权限。
```

`NorDevice` 独占底层驱动，克隆只共享同一所有者／租约表。`open_read_only` 和 `open_for_installation` 同时只允许一个会话；默认构造和只读打开没有擦写副作用。`NorStore` 实现原有 `Storage`，`Snapshot` 实现原有 `ReadAt`。不向调用者暴露分区内原始可写驱动。单执行器 `Rc` 使这些对象不能直接跨线程；跨任务安排由宿主串行队列负责。

读取另一个槽的数据租约只保护不可覆盖，并不授予主 Flash 边播放边擦写的实时能力。存储库不认识 DMX、UI、GATT、许可或时间；可信运行层决定何时能进入维护状态。取消仅释放暂存状态，不删除已提交记录；对象释放不写 Flash。

## 布局与状态

分区相对地址：两个各 4096 字节元数据区，再接等长载荷 A、B。2 MiB 槽对应总长 `0x402000`。所有地址经过长度／溢出和底层粒度检查；每次擦除只有一个物理擦除单位。小于或等于 1024 字节的连续输入经 ≤256 字节尾部缓冲拼接；重复块由安装器核对，不重复编程同一物理字。

描述区与封印区各 256 字节，写入和读回分开完成；包完整校验先于元数据发布。撕裂记录无效，安装器只在两份完整包之间选代数更新的一份。读失败不能当空白；有效不兼容版本／布局拒绝打开，损坏的版本字节须先经过公共摘要验证才能被判定为升级。

失败的载荷写入使暂存不可再写，必须取消／重建。提交错误进入原有“待确认”，禁止直接再编程描述／封印，重读持久状态后决定成功或失败。每次准备先检查目标租约，再擦目标元数据、载荷；另一个槽的全部字节保持不变。

## ESP32-S3 承接

`apps/esp32-player/src/package_storage.rs` 用 esp-bootloader-esp-idf 0.6.0 校验分区表／MD5并取得受限 `NorFlashRegion`，再用本模块和同一个 Installer／Service。`package_layout.rs` 只负责板级策略校验，可在主机测试；不重写官方二进制分区解析。

`storage-readiness` 是独立构建特性：仅只读检查、保持 RS485 禁用、不创建授权连接，也不调用写方法。它保留真实驱动的 Service 处理函数，使完整读写路径必须编译和链接通过。运行前尚需受控刷写；本轮只生成本地镜像，没有装到板卡上。

当前准备的 16 MiB 配置如下；已有诊断固件的默认分区不改变，不能直接在旧分区上启用包安装。

| 区域 | 偏移 | 长度 |
|---|---:|---:|
| 分区表 | `0x8000` | 4 KiB 保留 |
| NVS | `0x9000` | 16 KiB |
| 应用选择数据 | `0xd000` | 8 KiB |
| PHY | `0xf000` | 4 KiB |
| 应用 0 | `0x10000` | 3 MiB |
| 应用 1 | `0x310000` | 3 MiB |
| `stmpkgs`，自定义类型 `0x40` | `0x610000` | `0x402000` |

所有条目均校验范围、对齐、重叠；节目分区还要精确匹配标签、类型、子类型、偏移、容量和零标志。加密开发板当前拒绝这一检查路径；未来生产加密独立设计。预留两个应用区不等于已经实现应用升级。

## 资源核对与限制

本次 Xtensa release ELF 实际布局：服务（含存储）1880 B，存储 284 B，读源 12 B，单帧 1284 B，重组器 1296 B，驱动引用 4 B。另有 `NorDevice` 的小型共享分配；不会将 2 MiB 包整体放入 RAM。服务与存储不能重复相加；帧／重组／调用栈放在哪里取决于后续宿主。

本地应用镜像 535,232 B，占单个 3 MiB 应用区 17.01%。ELF `.bss` 137,780 B（含 128 KiB 堆预留）、`.data` 12,728 B、`.data.wifi` 284 B、链接主栈 152,484 B；链接主栈大小不是已测高水位或可分配堆。

历史诊断 BLE 已连接时使用堆 41,020 B、剩 90,052 B；包校验的保守准入峰值 65,536 B，两者相减还剩 24,516 B，尚须考虑新运行对象、分配器碎片、无线瞬时峰值。**这不是实测安装余量**。保留旧运行计划时不能再假设有一个完整的 64 KiB 预算：仅两个 64 KiB 预算加历史无线堆就超过总堆。首个安装宿主应在维护状态释放旧播放器／装载目录，实测后再开放并存策略；快照的合法租约不等于资源准入。

反汇编函数入口栈帧：Service 处理 5824 B，NOR 擦除 3200 B，读适配 1088 B，元数据读取 1328 B，官方分区读取 4272 B。它们有嵌套调用，不能以最大单函数当总峰值；官方 `write_nor` 本次仍预留 4176 B 栈帧，对齐源缓冲不会自动消除编译后的整个栈帧。因此不能把该服务直接放进未经测量的 8 KiB 任务栈。当前只读准备在主栈、无线启动前执行；无线并发安装的栈高水位、看门狗、分配失败、擦写中断时长均待实板验证。未启用或借用未经验证的 PSRAM。

## 复现

从项目根目录运行，产物均留在项目内：

```sh
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo test -p stagemaster-nor-store --locked --offline
bash tools/hardware/firmware.sh storage-build
bash tools/hardware/firmware.sh storage-check
bash tools/hardware/firmware.sh storage-size
bash tools/hardware/firmware.sh storage-report
```

独立产物在 `target/esp32-storage-check/`；默认诊断固件仍在 `target/esp32-player/`。没有新增存储刷机命令。分区 CSV 可由本地 espflash `partition-table --to-binary` 与 `--to-csv` 往返校验，`save-image` 只生成本地镜像；详见[工单](../development/tasks/PLAYER-003D-flash-store.md)。

保护测试覆盖 NOR 1→0、每字每次擦除后仅编程一次、对齐／范围、每个读写点前／部分／完成但报错／静默损坏、撕裂版本及封印、提交对账、读源占用、跨扇区尾部、取消、只读拒绝、分区冲突与 20 字节片段传输后的全部节目逐帧等价。软件模型通过不等于实板断电通过；正式 GATT 安装、运行调度、设备认证／24 小时授权、真实 DMX 和耐久性仍分项验收。
