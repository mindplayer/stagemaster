# 内部候选与告知材料交接

只用于内部材料审查，不是客户安装器、最终二进制／许可清单或发行批准。复用准确组装引用、优化构建记录、产品来源哈希与现有告知材料，不联网、不编译、不重签、不改权限、不启动副本。

从项目根运行，目标必须为本任务全新目录：

```sh
TMPDIR="$PWD/tmp" node tools/desktop/notice-handoff.mjs \
  data/AUDIO-023/assembly-ref.json \
  data/DESKTOP-009/materials-delivery-a \
  data/DESKTOP-011/my-new-handoff
```

需要 Mac ARM64、Node、现有 `/usr/bin/codesign`；首次实际材料必须匹配来源记录和全部产品文件，不以路径／版本名称代替内容核验。工具沿用仓库的安全目录、完整文件清单与 Node 复制；两份原文、索引与操作来源文本逐字节保留。未安装可选依赖仍明列，已安装缺项、许可假批准、来源变化、旧目标及链接祖先拒绝。

输出包含原 .app 副本、licenses/全文与索引、docs/两份 `.source.txt` 操作来源原文、中文README与私有handoff.json。相对文档引用保留为仓库来源文本，不伪造目录内链接或复制全套历史研究。清单绑定原产品来源提交／实例、四份构建／组装证据摘要、四份工具源码哈希、材料和操作来源摘要、完整包字节／链接／模式及四入口原／副本签名结果。`internal-review-materials-ready`只表示这个有限目录核验完成；失败留未完成目录与failed回执，不清空旧内容；磁盘／回执路径无法写入时可能仍为assembling，不能当成功。

固定上限：输入JSON 8MiB、正文16MiB、包条目1024／每包16份原文／单原文512KiB；产品来源4096文件、包4096文件／8192树项、单文件1GiB／清单字节合计4GiB；单入口签名诊断64KiB、10秒。读取用一字节增长检测与严格UTF-8／JSON；签名只读，封套不修改。原文、诊断或容量超限拒绝，不借自动重试／截断取得成功。回执独占文件描述符，路径所有权变化不沿链接改写其指向内容。

最终实际材料在 `data/DESKTOP-011/candidate-notices-d/`，对应产品 `b0f3a7b`／`desktop-release-qooqnJ`，不可当作可移动客户包，也不自动启动这个相同身份副本。原内部绝对路径资格、ad-hoc及Node调试权等事实保持；原签名有效不等于客户签名、公证／权限或最低系统资格。桌面材料是保守源码集，原Node／信令告知还在包内；UE／素材／完整许可另审，不称告知已注入原签名封套。

相关全套：`TMPDIR="$PWD/tmp" node --test tools/desktop/*.test.mjs tools/previs/*.test.mjs`。本机实际使用项目内隔离官方Node24.21运行312项／25新增／无跳过；全局与产品Node未更换，旧24.17运行器失败保持。实际副本、保护哈希和首失败见[DESKTOP-011](../../docs/development/tasks/DESKTOP-011-candidate-notice-handoff.md)。Rust／UI／UE产品源码未变，不冒计它们的全量、原生、听音或物理验收。
