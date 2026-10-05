# 桌面缺项的固定原文来源

DESKTOP-009，仅补充 DESKTOP-006 保守源码材料集的45个已安装原文缺项；49个未安装可选平台包不因此补造。此目录的原文、清单和注册元数据保持取得时字节与换行，不格式化；sources.json 是按版本／许可声明／锁摘要／安装清单固定的索引，不是法律意见、完整二进制清单或商业发行批准。

| 来源 | 数量 | 必要对应证据 |
| --- | --- | --- |
| Rust固定提交 | 40 | Cargo.lock原包摘要；Cargo.toml／原始清单／VCS记录与原.crate逐字节核对；固定提交清单与原发布清单相同；原文位于该包目录或其祖先目录 |
| Rust源码显式指针 | 1 | selectors的锁定原lib.rs与固定提交同字节、原告知头明确指向Mozilla MPL 2.0；保留该告知头与其指向的官方2.0原文，不从声明字符串猜模板 |
| npm固定提交 | 2 | Epic common／frontend官方精确版本元数据、锁完整性、实际归档与安装清单、gitHead及固定提交清单／LICENSE.md |
| npm同版本发布父包 | 2 | esbuild／rollup与对应Mac ARM64子包同gitHead；原发布父包声明确切optionalDependencies版本，父包原LICENSE.md与安装文件一致；属于发布关联材料，不声称覆盖未知内嵌组件 |

41个Rust原包和上述4目标npm／2父包共6个npm归档已实际核验。npm取得时的完整性证据保留，但离线补充不重新下载它们，原报告的archiveIntegrityVerified=false保持；当前安装清单和父包原文每次重新比对。不能把这6项推广为全部npm归档、产品内嵌第三方或客户发行资格。

dasp_sample原仓库地址发生官方301迁移，索引保留原地址／提交、仓库ID与规范地址，固定清单仍与原包相同。selectors的明确原指针见[固定源码](https://github.com/servo/stylo/blob/635e1a19d02960588a00e189bd4bd5bdb150ec3d/selectors/lib.rs)和[官方版本原文](https://www.mozilla.org/media/MPL/2.0/index.txt)。其发行义务、源码提供及全产品许可仍需单独审查；这里只收集原文与身份。

79份内容寻址原资料，共344,278字节，含原文、原清单、注册／迁移元数据和selectors源码依据；重用相同原字节，不重写版权行或选择多重许可。每项URL／路径／摘要和原包依据详见sources.json。取得与原失败保存在项目data/DESKTOP-009/、logs/desktop-009-*；原006材料和证据不回写。

.gitattributes禁止对原资料作换行转换。两份原文（c76f740d…／fa1bd040…）本就带行尾空白，仅其精确摘要路径保留该原文差异，其余空白规则及手写代码检查不变；全部79份暂存原字节再次与摘要核对。初次git diff --cached --check拒绝记录单列，不改原文来伪造逐字节一致。

## 离线使用与验证

从项目根运行，需原006基线材料、当前锁定安装与项目Cargo缓存：

```sh
TMPDIR="$PWD/tmp" node tools/desktop/supplement-dependency-notices.mjs data/DESKTOP-009/my-new-materials
```

只写全新任务内目标，拒绝已有目标、链接、外部路径、版本／身份／声明／摘要变化；不联网、不安装、不运行包脚本，不改原包或.app。若原006基线不存在，先按[原收集说明](../dependency-notices.md)从相同锁定来源生成，必须匹配索引绑定的原报告／原文摘要，不能覆盖旧材料或改索引迁就结果。

单原文512KiB／每包16份；原材料合并全文和补充后全文16MiB封顶，读合并全文不借此放宽单原文。复用原.crate只读／不展开核对与原期限；当前证明、目录／原文和离线合并按职责分文件。每次产出仍是516包保守源码资料，不是最终链接清单，不自动附加或重签历史候选。

相关全套为 `TMPDIR="$PWD/tmp" node --test tools/desktop/*.test.mjs tools/previs/*.test.mjs`。本机Node24.17的非ASCII输出／V8解析缺陷已以受控探针复现，原全量失败保留；后继用项目内隔离官方Node24.21复验相同287项，未改用例、并发或门限，未修改全局／产品运行时。详见[工单](../../../docs/development/tasks/DESKTOP-009-pinned-notice-supplements.md)，不以一次全量通过宣称旧运行器已修复。
