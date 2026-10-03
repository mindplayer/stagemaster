# STAGE-006：三维整组旋转与间距缩放

状态：本增量完成，2026-10-04；基线 `5dd48f8`，main，结果为本次 `feat(stage): rotate and scale fixture groups in 3D` 提交。当前会话单写者，用户 `output/` 保持。前一 goal 回合仅复核 ESP32 结论，对桌面目标为 no progress；本回合恢复实际 U08 增量。完整 AUDIT-001／goal 保持 active。

依 [ADR-145](../decisions/PRODUCT-ADR-145-group-fixture-transforms.md) 实施 Rust 原子变换、宿主版本门、UI 工具／精确输入和 UE 临时手势。持久化格式、灯光执行及硬件不变。

## 实现和实测修正

1. Rust 按原灯位共同中心原子计算旋转／等比间距，安装 Z 角随整组旋转，灯体尺寸／X/Y 安装角／挂接身份不变。受限小数、锁定／缺失／越界及 256 灯上限、单灯和等效零操作均有保护。宿主复用既有场地来源、版本、历史和撤销；UI 与 UE 只提交提案。
2. 三维交互版本 3，提供水平／升降／整组旋转／间距缩放及精确数值。未接纳或过期的提案还原，选择、来源、连接、取消和工具变化隔离排队提案；UE 按原位置计算临时草稿，连续帧保留安装朝向的预览。
3. 原生发现工具栏展开后鼠标命中偏移：官方 Pixel Streaming 组件只监听 window resize，工作区内部高度变动没有重配坐标。独立 ResizeObserver 适配调用 SDK 公开的坐标重配入口，实际尺寸变化取消当前手势；不修改依赖库或伪造窗口缩放。真实组件用 SDK InputCoordTranslator 验证 640×360→640×440 的同一视频四分之一点仍为 32768／16384，隐藏／卸载后无无效更新；最终原生真实拖动通过。
4. 原生还发现通用表单样式撑高工具栏，改为横向紧凑输入（实测 76 px）；撤销后旧旋转提示已清理。新增错误类型测试揭露数组被 String 强制转换误认工具名，改为严格字符串；失败记录 ui-release-tests.log 保留。

## 验证结果

- Rust：完整 project／desktop 相关 371 项通过，后补最大 256 灯及挂接身份用例后的专项 5 项通过，合计 372 个不同用例。见 logs/STAGE-006/rust-regression.log、transform-capacity-tests.log。全工作区全部目标严格 Clippy 通过（clippy-release2.log），fmt／差异检查通过。初次 Clippy 的字面量、函数长度／传参告警已通过局部抽取修正，未降低要求。
- UI：331 项通过（ui-final-regression.log），最终类型检查、格式和正式桌面打包通过（ui-geometry-check.log、desktop-geometry-build.log）。实际组件还验证非法角度／比例保留输入、Escape 清理、切换选择重置、提交保留输入、忙时禁用、真实布局变化及监听清理；验收入口 tests/previs-transform.html，截图 data/STAGE-006/02-component.png。无第三方新增／升级。
- UE：最终构建通过，全部 8 项自动化 Success、0 失败（ue-build-release-console.log、ue-final-tests.log 与 tmp/STAGE-006/ue-final-tests/index.json）；含独立世界坐标期望、左右手转换、逆变换、提案精度及非法参数。原首次构建名称遮蔽错误修正为 Angle 后通过，不改警告级别。
- 正式桌面：80 灯工程副本中两台共享选择、混合锁定组禁用、俯视／聚焦保持选组、旋转及缩放的真实鼠标拖动、一次撤销／重做、精确 90 度及 1.25／0.8 比例、无效零比例、Escape 输入清理、比例 1 零操作和拖出视窗取消通过。重开保存文件恢复位置／安装角及空历史；最终临时操作均撤回已保存状态。鼠标按住同时按 Escape 未用原生组合手势测试，不把输入框 Escape 冒充该证据。
- 独立文件对比 acceptance.json：来源 UX-050 哈希保持；副本仅 01／19 两灯变为 (5,−4.958333,6.4)／(5,5.541667,6.4)，安装 Z 为 90 度。其余 78 台灯、空间／构件／挂接／锁、场景和音频等字段保持。01 及 19 的原 X 安装角 0／180 保留。截图 data/STAGE-006/01-native-group-transform.png，最终辅助功能状态 native-final-ax.txt。

## 审查和接续

本增量可集成。新职责按核心数学、UE 草稿、协议和 UI 尺寸适配拆成小文件；stage.rs 439 行保留稳定命令枚举／分派并抽出既有 put placement，PrevisViewport 413 行保留单连接生命周期和组件组合，新增数学／表单／尺寸观测均独立；没有超过 500 行的新手写文件。

最终窗口保留已保存的独立副本与内嵌三维，两灯已选、无未提交草稿，音乐未播放／设备未连接。未操作原工程、用户 output/ 或真实灯具。旋转目前为世界竖轴、间距为三轴等比；任意轴旋转、混合构件编辑、三维框选及 U08 其他缺口继续推进，不将该增量视为完整建模或整个 AUDIT-001 完成。
