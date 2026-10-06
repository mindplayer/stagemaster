# PREVIS-014：统一预演运行环境的目录归属

状态：实施中，2026-10-06。基线 main `0eb833e`，主工作区单写者、tracked干净，output/未读未动。范围先提交，按[ADR-181](../decisions/PRODUCT-ADR-181-renderer-runtime-environment.md)实施；属于首版H5进程／资料归属及H1三维稳定性，不扩展平台或另写播放器。

## 已核对问题与范围

PREVIS-013已完成工具助手收尾。桌面`renderer_local.rs`的共同路径仅指定UE UserDir／缓存／TMPDIR；Foundation用户目录只有component分支覆盖，editor分支可能继承外部CFFIXED_USER_HOME／默认全局路径。现有component动态加载去污染只遍历父环境，两个固定键之外的显式Command加载覆盖仍可残留。此处按当前代码确定性复现，不把它推断成历史忙循环／媒体／GATT根因。

在原共同配置中统一设置已验证RuntimePaths.user/platform-user，并清理所有继承与显式DYLD_*覆盖；不改HOME、不修改用户全局配置、不搬迁／删除旧缓存或报告。复用原路径预检与mkdir，错误在任何命令配置／启动前返回。编辑器的预编译-game SDK跳过只保留在原角色，Game继续去掉该快捷环境；既有包资格、网络令牌、沙盒／签名、播放器、桥协议和工程不改。

仅修改桌面预演环境适配器及小型独立测试文件；不实现自动杀桌面报告助手，也不在本项启动新UE／Game／真实声音、签名／目录授权、输出或硬件。实际独立Foundation子进程验证用户／ApplicationSupport目录解析；如宿主实际行为不符合目录约束就保留失败，不改全局系统路径或把env字符串当原生证明。

## 验收与交付

先红后绿覆盖editor／component的外部home与额外加载项、SDK角色保持、两个owner隔离、失败保持原Command、不搬旧home资料；测试不修改父环境。实际OS子进程留证仅使用项目内独立临时目录和既有系统工具，Foundation路径与UE参数分别核对，不冒称GPU／沙盒资格。

按风险实际执行相关桌面／工作区Rust测试、默认与内部release桌面严格Clippy、fmt；UI／依赖／UE源码未改不重跑无关界面验收，既有验收不削弱。冻结源码和实际结果，日志`logs/previs-014-*`、证据`data/PREVIS-014/`、临时／缓存项目tmp/；更新STATE、逐项提交。失败和历史原因保持，当前音乐／唯一三维组装授权及H1–H5实物／长期／客户出口仍开放，完整Goal active。
