# EXEC-020：明确节目控制的有界原请求证据

状态：实施中，2026-10-06。基线main `41206f920f9ccdcdb8c5b98b7699c89e0b365f20`，主工作区单写者、tracked干净、用户output/未读未动；先提交范围和[ADR-179](../decisions/PRODUCT-ADR-179-source-command-evidence.md)。依据首版H2／H5及DESKTOP-014真实未确认暂停，不恢复工作器、不展开通用观测平台。

## 内聚范围

原Client最后明确非音乐普通节目start／pause／resume／next／stop，最多一项脱敏目标、原serial、原POST／同serial GET结果及校验后的回执；维护和其他控制不擦除。复用已有HTTP诊断字段和原单串行客户端，只读默认收起中文详情。没有已发送证据不声称已发送／接纳，普通节目applied不等于实灯输出完成；不靠读取旧State推断本次完成。

限定`crates/stagemaster-execution-client/`、`apps/ui-prototype/`对应类型／小组件／测试和`apps/execution-host/tests/`真实宿主接缝。改已有control职责时抽取source_control模块；入口仅组装。新增文件300–400行评估，超过500须拆分或说明。HTTP v2、工程格式、租约、请求序号、截止期、运行时、音乐／手动层／总控规则和保护不改，不增加重发、历史窗口、后台服务或新播放器。

最终检查发现新增视图诊断使既有`audio_client.rs`的`verify_operations`异步对象达到17,864字节，严格Clippy拒绝；限定补充该测试调用的`Box::pin`堆分配，业务步骤、断言、期限及生产逻辑保持。保留两次严格失败，新冻结版本重新全量及严格检查，来源指纹纳入该既有测试文件。

## 验收

先补真实宿主保护测试，原实现缺少sourceOperation应红；随后验证暂停／继续原目标与回执、陈旧revision拒绝、未取得控制／无效目标不借旧成功、已接纳但响应损坏只查原serial、HTTP未接纳、维护及非节目操作不覆盖、pending不换诊断、敏感文本／容量上限与重连清空。回执serial不匹配不得关联；仅原完整applied回执可给出对应来源状态，诊断不作为运行权威。

相关Rust／UI／真实组件类型及格式、当前全量／严格检查实际执行；新来源内部release桌面运行无音乐三步，默认详情不提交命令、明确操作各一次、原结果跨维护保留，暂停、固定版本隔离、零值持有／归还、人工→定时→人工、停止／保存最近重开与收尾实际核对。具体失败保存不盲重试、不放宽期限；构建／mock不能替代正式桌面。DESKTOP-014旧暂停根因未知保持，本项诊断成功不自动修复历史。

证据`data/EXEC-020/`，日志`logs/exec-020-*`，临时及缓存项目tmp/。不改旧包／Game／Node、不组装／重签或映射权限、不操作UE／声音／真实设备。完成后审查、逐项提交及更新STATE，完整首版仍须真实厂家／差分／完整资源／8小时、客户发行／许可和独立三任务。

## 实施中检查点（不是完成交付）

### 最新源码阶段验证

实现结果为本次`feat(execution): retain bounded original source command evidence`提交，完整工单仍待正式桌面。冻结版监督80918已退出0：全工作区1,355 Rust＋2文档、3既有ignored，包含最终9个新增Rust用例；workspace及internal-acceptance全目标严格Clippy、fmt全部0，前后指纹`dfde90fe3620c78682de9fe840a64c00db2fbb93154a022c7e60dff8e1eb982b`保持。`data/EXEC-020/source-verification.json`实际核对UI476＋4／类型／格式、真实组件8分支零控制与一次明确暂停一项、5严格JSON／本地引用／diff；既有596来源6改590保持，另一个既有audio_client仅Box::pin调用改变、断言保持，依赖锁不改。新增最大207行，无入口堆叠／新依赖环。

监督27463全量本身通过，但与中间22263均在严格检查拒绝17,864字节异步对象；按检查建议堆分配后新冻结版重新全量通过，不关闭lint或弱化断言。原红灯、GATT过期未知原因和夹具CLI错误仍保留。以下检查点为此前事实，不是最新进行中的监督。下一步只从干净实现提交的新内部副本进行原生无音乐闭环，不借旧包音乐／GPU／物理／客户资格。

范围提交`341368b`。客户端／类型与真实小组件已实现，产品代码尚未提交、最终源码资格／正式桌面待完成；最多一项普通节目原目标／原POST和同serial GET，复用原传输。原apply职责抽到65行source_control，新增最大207行；6个既有来源改变、590个保持，锁不变。状态来源需核对原后台、配适布局、准确节目及已知步骤；pending／拒绝／未知不能借旧状态成功，既有保护不动。

原宿主三项0绿／3红，首实现1绿／2红暴露误用媒体accepted而非普通applied，查现有group receipt后纠正。专项初绿5已通过，但早于第三传输用例及最后布局检查，不能标成最终9项通过。新增3个单元及6个真实宿主用例待当前全量确认。UI全476＋新增4、应用及已纳入配置的真实夹具类型、格式实际通过；真实React八个诊断分支零控制，一次明确暂停只产生一项，默认收起，图像／AX及component-verification单列。单独CLI夹具类型首次TS5112保留，改为项目tsconfig显式纳入而非删检查。

首全工作区101：既有GATT runtime_queue在outbound返回Permission(Secure(Expired))后测试辅助unwrap失败；隔离同两项通过，不改模块／期限／断言，也不能追认原根因。原日志保留。中间全量在布局补强前开始（监督22263），即使后继绿色也非最终源码全量；最新冻结版本另由监督27463执行全量／严格Clippy／internal-acceptance严格Clippy／fmt，`rust-checks-reviewed.json`检查前后源码指纹。所有原监督保留，不因暂时无输出重启。`source-review-checkpoint.json`及static-checkpoint记录进行中，不冒称源码／原生资格已完成。

下一步：等待原监督实际结束、修复有证据的当前失败、核对最后输入稳定与当前全量／严格检查；随后逐项提交源码，再运行已备`tmp/exec-020-native.mjs`的新内部无音乐正式桌面和收尾验收。不得现在运行尚未提交代码的构建，不重复已关闭32fMDY原后台，不移植旧资格。原暂停／GATT过期原因、预设重复应用错误通知及全部外部门槛仍开放，完整Goal active。
