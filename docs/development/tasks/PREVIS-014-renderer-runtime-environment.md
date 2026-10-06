# PREVIS-014：统一预演运行环境的目录归属

状态：完成、自审通过，2026-10-06。基线 main `0eb833e`、范围`8d4bd1a`，结果为本次`fix(previs): isolate renderer runtime foundation and loader paths`提交。主工作区单写者，output/未读未动；按[ADR-181](../decisions/PRODUCT-ADR-181-renderer-runtime-environment.md)实施。属于首版H5进程／资料归属及H1三维稳定性，不扩展平台或另写播放器，完整Goal active。

## 已核对问题与范围

PREVIS-013已完成工具助手收尾。桌面`renderer_local.rs`的共同路径仅指定UE UserDir／缓存／TMPDIR；Foundation用户目录只有component分支覆盖，editor分支可能继承外部CFFIXED_USER_HOME／默认全局路径。现有component动态加载去污染只遍历父环境，两个固定键之外的显式Command加载覆盖仍可残留。此处按当前代码确定性复现，不把它推断成历史忙循环／媒体／GATT根因。

在原共同配置中统一设置已验证RuntimePaths.user/platform-user，并清理所有继承与显式DYLD_*覆盖；不改HOME、不修改用户全局配置、不搬迁／删除旧缓存或报告。复用原路径预检与mkdir，错误在任何命令配置／启动前返回。编辑器的预编译-game SDK跳过只保留在原角色，Game继续去掉该快捷环境；既有包资格、网络令牌、沙盒／签名、播放器、桥协议和工程不改。

仅修改桌面预演环境适配器及小型独立测试文件；不实现自动杀桌面报告助手，也不在本项启动新UE／Game／真实声音、签名／目录授权、输出或硬件。实际独立Foundation子进程验证用户／ApplicationSupport目录解析；如宿主实际行为不符合目录约束就保留失败，不改全局系统路径或把env字符串当原生证明。

## 验收与交付

先红后绿覆盖editor／component的外部home与额外加载项、SDK角色保持、两个owner隔离、失败保持原Command、不搬旧home资料；测试不修改父环境。实际OS子进程留证仅使用项目内独立临时目录和既有系统工具，Foundation路径与UE参数分别核对，不冒称GPU／沙盒资格。

按风险实际执行相关桌面／工作区Rust测试、默认与内部release桌面严格Clippy、fmt；UI／依赖／UE源码未改不重跑无关界面验收，既有验收不削弱。冻结源码和实际结果，日志`logs/previs-014-*`、证据`data/PREVIS-014/`、临时／缓存项目tmp/；更新STATE、逐项提交。失败和历史原因保持，当前音乐／唯一三维组装授权及H1–H5实物／长期／客户出口仍开放，完整Goal active。

## 最终实际交付

共同路径在原目录预检成功后覆盖CFFIXED_USER_HOME，按原始字节前缀清理继承和该Command全部DYLD项，不复制环境值／凭据、不修改父环境。组件原路径保持，editor不再借外部／全局home；SDK预编译角色和Game禁用快捷标志保持，HOME／PATH／业务环境、DDC与原三UE参数不变。原适配器237行，新独立测试224行，无新增依赖、API、工程字段或权限／签名。

原确定性1通过／4失败后修复；首集成11通过／1失败因旧测试误将env_remove条目计成正向环境，现严格三正向路径＋全部加载项None。初全量**1,362 Rust＋2文档通过**，后继工作区严格Clippy在两处新测试字符串风格退出101，已直接修正，无allow或命令弱化。自审将component用例改用真实component目录工厂，Foundation原生测试分别执行editor／component，保护比最初只用editor布局更完整；新源码冻结后重跑原验收命令，不把旧全量当最终版本。

最终 **1,362 Rust＋2文档、专项12、默认工作区及内部release桌面全目标严格Clippy、fmt全部实际退出0**。已有3个ignored子进程入口保持原父测试调用，没有新增忽略；专项包含7新测试。两份实际系统Foundation回包逐项证明home／ApplicationSupport／Caches为owner内platform-user、准确三UE参数；组件的user／cache为真正data/previs布局，editor为原data/previs-user布局。两个owner隔离、失败原Command／目录保持、旧home哨兵不迁移删除、非Unicode加载键与其他业务环境保护均通过。没有新UE、Game、声音、设备或正式桌面／GPU运行，系统目录验证不是沙盒或当前组合资格。

原监督38152实际终止1，原cargo99823退出0后进入Clippy失败；资格监督43707实际终止0，全部五阶段完成，源码前后指纹保持，**2,091**其他源码／工具／锁保持。原多个测试程序曾在macOS `_dyld_start`前等待，原进程后来自行完成；采样112K footprint、系统记录及未取得Rustc采样的辅助失败保留，启动等待原因仍未证明，不改系统属性／签名／权限、测试期限或重启安静进程。本项Clippy修正后的新资格是新源码复核，不是重复执行原未终止任务。

`checks-qualified.json`／`verification.json`记录最终实际资格、两角色Foundation结果及证据摘要；旧checks.json和source-checkpoint.json按历史保存，不追认为最终通过。相关引用／JSON／diff和交付版本另核对，日志`logs/previs-014-*`、产物项目data／tmp保持忽略，output/未读未动未暂存。

本项限定环境归属出口已完成；桌面自动报告助手收尾、UE内部忙循环、历史媒体／GATT原因及当前来源音乐／唯一三维组装授权、真实灯型／差分／完整资源长期、客户发行与外部操作者仍未解决。完整首版Goal保持active，不扩大本项资格。

## 2026-10-06历史源码检查点（不是当前状态）

范围先提交`8d4bd1a`。首轮1通过／4失败证实editor外部home及component显式额外加载路径残留；原生测试在home预检失败处停止，未启动外部目录的Foundation子进程。共同配置成功准备目录后设置原platform-user，并按原始字节前缀清继承和显式DYLD项；HOME／PATH／业务环境、editor SDK角色与component去快捷标志保持。新独立测试文件205行，原适配器237行，无新增依赖／公开接口／持久字段。

首集成11通过／1失败，是旧测试将新增的DYLD清除条目误计为正向环境；现在严格核对三项正向所属路径，额外全部DYLD必须None，不削弱原路径／DDC／SDK断言。最终专项12通过，真实既有系统Foundation子进程实际home／ApplicationSupport／Caches均在owner的platform-user，原三UE参数准确传递；这是OS目录行为，不是UE／GPU／沙盒或新桌面组合资格。两个owner隔离、失败原Command不变、旧资料哨兵不迁移／删除、非Unicode加载键清理及其他业务环境保持通过。

`baseline.json`／`source-checkpoint.json`保存准确源码和2,091保护文件保持、原红绿及真实Foundation结果。全量监督句柄38152、原cargo PID99823现场仍活跃，依次工作区测试／默认严格Clippy／内部release严格Clippy／fmt；尚不能据计划或专项计为全量通过。runtime_access启动阶段的只读采样只有`_dyld_start`、112K footprint，无测试正文；原进程随后自行结束，原全量继续到后继程序，没有取消重跑／修改系统权限或保护。启动等待原因未证明；Rustc采样时已结束、未来日志未创建时读取等辅助工具返回不伪装成产品断言。

以原句柄／checks.json／日志终态继续验收，不能仅凭checkpoint字段当作当前仍活跃。源码尚未最终提交；验收及交付资料齐全后再完成本工单，完整Goal保持active。
