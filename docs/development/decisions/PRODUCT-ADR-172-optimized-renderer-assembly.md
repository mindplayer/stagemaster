# PRODUCT-ADR-172：优化内部来源保留已编译验收身份

状态：2026-10-05有限决定待实施，关联[PREVIS-012](../tasks/PREVIS-012-optimized-audio-renderer-acceptance.md)、[ADR-171](PRODUCT-ADR-171-isolated-release-validation.md)、[ADR-167](PRODUCT-ADR-167-development-desktop-assembly.md)。仅内部来源／组装工具，不变更公共工程格式、应用API、时间、控制权或客户运行目录。

## 事实与决定

DESKTOP-005原生代表包只有优化基础编排；提交后E0F5rq优化来源包尚未启动，现场核对runtime／新组装目标不存在。原组装器只接受cn.stagemaster.desktop，随后给debug副本换唯一身份；release内部隔离由Tauri编译配置的identifier与环境实例严格绑定，仅更改Info.plist不能更改编译配置。

1. 原debug来源与唯一新实例保持。优化来源必须显式传入DESKTOP-005提交后构建记录；在任何组装写入前核对记录、raw build-record哈希、原唯一计划／构建标志、完整来源清单、20份原源码／配置／锁哈希及实际Plist身份。仅匹配前缀或自行声称优化不构成来源资格，未完成／畸形／变更来源拒绝，不回退debug或系统目录。
2. 优化副本保留该编译身份和准确desktop-release实例，限定沿用原组装目录布局，runtime不存在才可预留；已有目录拒绝，不迁移／清空历史状态。原build暂存与组装暂存属于同一既定来源实例，新产物在独立组装归档，不覆写来源包或构建记录。
3. Game仍是已验证Development组件，只给新副本原四项＋准确五目录资格，副本桌面ad-hoc封套复用原实现。原组件、嵌套库、正式证书／私钥／公证不改，新UE／Xcode构建仍暂停。该内部组合没有获得客户权限、Shipping、Developer ID或全产品许可。
4. 真实优化音乐／GPU联动以原后台、音源、唯一公共视窗和既有只读桥验收；控制接纳／实际Applied、软件PCM／声卡、静态签名／实际画面分别报告，2秒失效不变。来源／保存／失败证据保持，结束明确关闭所属进程。

成熟机制沿用已经实际验证的Cargo显式能力、Tauri编译配置、原Mach-O／codesign／Plist只读检查及PREVIS-007组装；不是新代理／签名／渲染平台。工具领域之外客户沙盒与签名方法仍需具体条件和独立决定，现场听音、厂家、物理／长期和外部任务不因此关闭。
