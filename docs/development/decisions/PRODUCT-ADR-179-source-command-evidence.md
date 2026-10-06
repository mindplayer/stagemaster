# PRODUCT-ADR-179：最后明确节目控制的原请求诊断

状态：2026-10-06当前实施会话审定，先记录再实现；关联[EXEC-020](../tasks/EXEC-020-source-command-evidence.md)，沿用[ADR-174](PRODUCT-ADR-174-media-request-evidence.md)已验证的客户端就地有限诊断机制。

DESKTOP-014新原生实例一次暂停后仍Running，原提交未留存，历史宿主回执已notRetained；不能根据按钮点击或后继旧State推断接纳／拒绝。Client已有最后operation_record，但界面不呈现节目目标／原HTTP，现有媒体诊断只覆盖音乐。扩大宿主回执窗口或新采集平台不在范围内。

Client新增最多一个可选`View.sourceOperation`，仅非音乐普通节目start／pause／resume／next／stop。合法限定目标包含准确host／source、十进制revision和类型化动作；无效输入不复制任意字符串，原业务校验／发送行为不因诊断改变。level／patch／manual／batch／output／media不替换，下一明确普通节目操作替换；pending时不得擦除原项。打开／重连为空，不持久入工程。

原发送路径开始POST尝试前绑定原serial，`attempted`不证明网络送达或服务接纳；未提交本地分类、原HTTP状态／正文完整性／固定错误枚举、原serial GET分别留存，复用已有8KiB请求／8MiB响应和固定code白名单，不复制正文、自由message、凭据或工程。校验通过的同serial回执才关联pending／complete及普通节目applied／rejected／unknown；只有完整applied可附该准确source的有界状态／步骤／revision，旧State、拒绝或未知不借来宣称本次执行完成。单项序列化小于4KiB以测试验证。首轮真实宿主1绿／2红暴露误套媒体accepted类别，按既有group receipt的applied纠正；不改变协议、不合并两种成功语义。诊断检查不改变原操作预检：若原路径仍尝试一个诊断目标无法保留的无效revision／step，照实记录尝试及结果，不能标为本地未提交。

节目控制区增加默认收起“最近节目控制详情”，只读中文呈现目标、原网络序号、尝试与HTTP、原回执，状态明确属于当次回执，不当当前运行状态。展开／刷新／跨页不控制，不新增计时器／复制／导出历史或重试。公开应用视图仅可选增量，原消费者可忽略；HTTP v2、工程格式、时钟／控制权与宿主回执窗口不变，无新依赖。

真实Client／宿主先红后绿、响应损坏和陈旧revision、pending／维护／脱敏／容量、真实React以及新来源原生闭环验收。旧暂停根因仍未知；软件诊断不代替物理、客户发行或完整首版资格。
