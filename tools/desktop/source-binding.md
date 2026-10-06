# 内部桌面构建来源

`node tools/desktop/run.mjs build-internal-release` 沿原内部构建路径执行，不加新命令或额外参数。运行所需 Node／Rust 继续使用当前安装版本，缓存／记录在项目内。

新 build-record 的 `sourceEvidence` 自动记录 Git HEAD、源码差异、相对输入路径／字节／SHA-256／执行位、输入指纹及实际 Node 平台信息；后台记录另带实际 `rustc -vV`。含未提交源码时明确不是纯提交版本，不能仅用 HEAD 代表真实编译输入。

输入范围为 Rust crates、桌面／执行后台／前端、桌面与预演构建工具，以及编译器实际嵌入的 project／common Schema。测试／示例、生成目录、安装依赖树和无关文档排除；output/、data/、logs/、tmp/ 不遍历。上限 4,096 文件、单份 64 MiB、总计 256 MiB；扫描／读取有界，超限不省略。

在源码捕获、编译后与归档后比较输入和 Git 基线。差异或异常保存 failed 记录，不重试、不登记成功。明确计划的后台编译使用其工作区，不再固定借用模块主目录。新包随附 `stagemaster-source.json`，路径记录为 `sourceResourcePath`；原 Tauri 数组／映射资源配置保持，来源文件必须唯一且与本轮字节一致。

归档后实际再次核对包身份、后台、来源和完整复制清单。旧目标拒绝覆盖，失败归档保留供诊断。旧包／旧记录不回填新字段或晋级；新 `isolated-release-built` 仍只表示构建／封存，原生资格须单列，客户资格始终 false。

这不是源码不可变 checkout／完整依赖供应链／二进制可重现保证，不捕获中途改写再恢复，也不增加共享目标事务锁或防止任何并行恶意替换。环境与安装依赖树仍属后续真实发行资格；源码／资源摘要不是授权签名。依据 [ADR-177](../../docs/development/decisions/PRODUCT-ADR-177-internal-build-source-binding.md)，交付 [DESKTOP-013](../../docs/development/tasks/DESKTOP-013-source-bound-internal-build.md)。
