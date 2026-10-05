# 桌面告知材料收集

从项目根运行，使用本地已安装依赖和Cargo缓存；不联网、不安装、不编译、不签名、不改原包：

```sh
TMPDIR="$PWD/tmp" node tools/desktop/collect-dependency-notices.mjs data/DESKTOP-006/my-new-materials
```

需要现有Cargo、Node和Python 3.11以上标准库（tomllib／tarfile）；不增加第三方解析依赖。目标必须是data/DESKTOP-006/中的全新目录，已有目录或链接拒绝。生成licenses/notices.json与THIRD-PARTY-NOTICES.txt，供后续准确候选的发行组装引用，当前不会修改或自动附加到已签名.app。

范围是Mac ARM64桌面／音频后台的非dev源码闭包（包含构建依赖与工作区解析可能统一的特性）和界面manifest的非dev树。当前界面把构建工具放在dependencies，清单保守包含它们与其他平台可选包，不是最终二进制／前端打包清单。

Rust使用Cargo locked／offline元数据、标准TOML解析器和原始.crate：对照Cargo.lock摘要，再逐字节核对缓存Cargo.toml及已收集原文，不展开写入压缩包。界面核对锁版本／名称／许可及安装清单，原文逐份留摘要；npm归档完整性字段只保留锁的声明，**不声称已重新验证npm归档**。

collected只表示找到原文；missing表示已安装但缺独立原文；not-installed表示本机没有该声明依赖，optional另列。未知／多重许可原样保留，不自行选项，不猜README许可，不把材料完整等同许可批准。仍需单独核对最终产品的UE、素材、原生内嵌第三方及商业发行条件。

固定边界：1024包／节点、单包16份原文、单文本512KiB、收集及合并16MiB；原包64MiB、展开读取128MiB／16384成员，核对单进程60秒期限。违规、篡改、重复、未知来源、路径／链接、非UTF-8或旧目标明确拒绝；写入失败不登记成功，可能留下未完成新目录，旧目标不被覆盖。

相关验证：`TMPDIR="$PWD/tmp" node --test tools/desktop/*.test.mjs tools/previs/*.test.mjs`。实际版本、结果与未完成项见[DESKTOP-006](../../docs/development/tasks/DESKTOP-006-dependency-notice-materials.md)。

后继[DESKTOP-009固定补充](notices/README.md)另对已安装的45个缺项核对原包／固定提交／发布关系，离线生成新材料；本入口及旧索引的missing事实保持，不自动改写或附加到旧.app。补充不等于全产品许可、最终链接清单或发行批准。
