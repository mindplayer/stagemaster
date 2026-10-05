# DESKTOP-005：正式优化桌面的隔离验收

状态：ready，2026-10-05。基线 main `7e7f31b3e2e266d51b133c9ba5c40d64ca6e5e33`，主工作区单写者；计划／[ADR-171](../decisions/PRODUCT-ADR-171-isolated-release-validation.md)先纳入版本再实现。H5独立增量，保护用户output/、原包／工程／默认最近目录和其他会话改动。

## 一个问题与可用增量

已有桌面构建脚本强制--debug，后台也是debug；storage_paths仅debug支持STAGEMASTER_ACCEPTANCE_INSTANCE。普通release使用正式用户数据目录，不符合开发验收文件归属。已验证的Development／内部GPU包不证明优化桌面的功能、非debug编辑器回退禁用或隔离行为。先让真实release代码能安全验收，不能以模仿release或修改客户默认目录代替。

- Rust仅增加默认关闭的internal-acceptance能力及私有隔离选择／测试，客户普通release仍使用原Tauri数据目录，debug原行为不变。隔离release必须同时满足编译能力、准确内部应用身份和有效实例名称；内部包遗漏实例或身份不符拒绝，不能静默写正式数据。环境变量本身不授予客户版此能力，不增加可执行原始灯值／新控制权入口。
- 复用现有tools/desktop/run.mjs与prepareHost，不另建组装平台。新增明确build-internal-release模式／小型纯构建计划和保护测试：桌面与带audio的原后台都用release，目标缓存／sidecar／配置／证据全在项目内且不覆盖原debug包与tmp/desktop-bin。生成唯一内部身份／中文内部标题，锁定离线，无新增依赖或锁升级。
- 内部构建不操作用户证书／私钥、公证、UE／UAT、Game权限或真实设备。构建明确不执行正式签名，报告不可作为客户发行资格；不绕过既有分发检查。现行Game沙盒／Node调试资格与客户签名／Shipping门槛仍保留。系统临时例外缺失只继续暂停UE／Xcode受影响部分。
- 目录选择、构建计划、执行入口和测试按职责分文件，小步接线；不改工程格式、公开模块接口、时钟、执行／输出语义、恢复点格式、最近目录格式或既有UI交互。

## 实际验收

先以旧仅debug行为复现隔离release缺口，原断言不弱化；覆盖普通release忽略环境覆盖、debug默认／实例保持、默认关闭能力、准确身份／实例、缺／非法实例、错身份与链接／越界构建输入、release后台／目标／sidecar一致及非零构建结果不登记成功。相关Rust和构建工具测试、fmt／严格Clippy、JSON／类型／差异实际执行；按目录选择与构建边界风险跑当前全工作区，不把旧1303结果叫本轮通过。

实际在独立项目内目标构建release .app及audio后台，确认不是旧debug二进制／sidecar，保存源码与产物哈希／构建命令。正式原生用独立工程执行无音乐的代表编排：场景／人工等待／执行、暂停继续、手动接管归还和编辑／输出分离、取消／搜索／撤销与保存重开；缺三维组件明确拒绝，不启动编辑器／Node／Game，不因拒绝破坏工程／执行。实际后台／应用关闭及默认目录／原包／来源清单保持。没有真实听音／客户Game／物理DMX，不宣布对应门槛通过。

证据data/DESKTOP-005/、日志logs/desktop-005-*与logs/DESKTOP-005/，临时目录显式项目tmp。完成后自审、更新工单／STATE并提交，继续H1–H5；真正release核心可用不等于客户签名／Shipping／公证／全产品许可或实物出口。

## 本轮真实排错边界

优化版严格Clippy实际拒绝renderer.rs的仅debug Path导入与renderer_paths.rs的仅debug／测试editor构造。只按原条件编译，不禁用告警、不恢复release编辑器回退。工具自审实际发现共用runCommand再次合并父环境：补默认保持的显式不继承选项，真实子进程红灯与失败记录保护；旧UE执行默认不变。

本轮原全量退出101，audio_client.rs:65在重新取得控制权后收到503；单独诊断通过不作已修复证据。现有Client先保存并发送原序号、随后refresh，观察繁忙可能发生在已发送之后；client_observation已有六秒只读wait，client_audio.apply已遵守同一规则，取得／释放测试仍直接unwrap。限定追加真实后台＋既有回环故障代理，确定性复现“原控制已送、随后观察503”；测试辅助仅查询原回执、严格同序号／acquired或released结果，不重发控制、不扩大期限、不忽略拒绝／错误、不削弱音频／帧断言。运行时、公开Client、HTTP／503／控制权和时钟语义不改。首次503未保存具体HTTP正文，不追认所有历史偶发根因；全量需修复后原命令实际重新执行。
