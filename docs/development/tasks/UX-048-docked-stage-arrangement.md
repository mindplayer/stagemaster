# UX-048：在场地中编辑灯位排列

2026-10-04；本增量完成。基线 `da62202`，main，当前会话单写者。上一回合核对既有 ESP32 证据，没有推进桌面目标，分类为 no progress；本回合重新核对干净源码后接续 AUDIT-001／U08，完整 goal 保持 active。

## 依据与范围

原生窗口截图 `data/UX-048/01-before-stage.png`、`02-before-arrangement.png` 已保存并检查：排列弹窗遮挡真实场地，孤立小预览缺少地台／桁架参照，1440×940 窗口中右侧部分参数还被裁切。独立工程副本位于同目录，来源 STAGE-005，不改用户原工程、output/ 或硬件。

借鉴 [Vectorworks Object Info palette](https://app-help.vectorworks.net/2023/eng/VW2023_Guide/Objects_edit1/The_Object_Info_palette.htm) 的固定上下文属性与多对象参数编辑，复用本项目 DockPane、StageCanvas、有序灯具选择、placement-tools 与工作台 collect／accept 事务；不增加另一个场地编辑器或播放引擎。

- 直线、矩阵、圆弧、精确平移／旋转、对齐分布、底座方向移到右侧排列属性。
- 中央真实平面显示草稿；进入排列自动显示平面与属性栏。三维仍采用已应用工程，不宣称 UE 草稿预览。
- 打开不修改工程；明确应用或工作台保存／切换沿原事务收集，整组一次历史。取消完整撤销草稿；相对操作不得重复叠加。
- 数字错误、空选、锁定、对象消失及源场地变更明确拒绝；保留草稿并定位错误。搜索、顺序与筛选外选择保持；视图平移／缩放仍可用。
- 草稿与正式工程分开，纯模型、控制器、属性组件和样式分文件；不改 Rust 语义、持久格式、协议或依赖。

## 验收

纯模型验证不同排列、原子命令、无变化、过期与锁定、新布置、错误／取消及预览不改来源。实际组件验证 collect／accept、保存／切换、收起再开、输入错误焦点、窄栏布局和视图操作；最终原生独立副本验证取消、单次撤销／重做及保存重开。记录实际命令与结果，不以组件验证代替核心持久化和正式桌面验收。


## 实现与验证结果

旧 ArrangementDialog 已删除，字段、侧栏、草稿控制器和纯投影各自独立；复用原 putPlacement 事务。草稿绑定原场地与灯具身份，外部变化拒绝重新套用相对量。打开不进入待修改态，输入或选择改变才进入；明确应用可接受初始新布置，零移动不新增历史。成功后收集的灯位投影用于显隐恢复，覆盖提交响应早于 React 工程刷新及目标空间原先隐藏的情况。无持久化、Rust、协议或依赖变更。

- `npm --prefix apps/ui-prototype run test`：324 项通过，含新增 6 项状态／投影保护测试，0 失败／跳过。最终日志 `logs/UX-048/ui-tests-final.log`。
- `npm --prefix apps/ui-prototype run check`：通过；新模型测试及工作区验收页纳入 tsconfig。最终日志 `typecheck-final.log`。
- 项目已有 Prettier 执行相关文件格式检查通过；`git diff --check` 通过。日志 `format-check.log`。
- `CARGO_TARGET_DIR="$PWD/tmp/framework-001-light-target" npm --prefix apps/ui-prototype run desktop:build`：最终正式桌面打包通过，日志 `desktop-build-final.log`。Rust 语义未改，不重复全工作区旧测试冒充本轮证据。
- 真实 StageWorkspace／PerformanceLayout 隔离验收：打开未修改、平面预览不写工程、空值保存／切页拒绝与字段焦点、取消、两台灯一次应用／重复保存不叠加、一次撤销、新布置、搜索回车隔离、筛选外成员保留、空选拒绝、外部锁定过期拒绝均通过；隐藏观众区内新增灯位应用后可见。1100×800／300 像素属性栏实测字段右缘不超过 1086，截图 `05-narrow-grid.png` 已检查。
- 原生 80 灯副本：空数字拒绝保存并聚焦；取消后 X 回到 1.127713 且无撤销记录；属性栏收起后由排列入口自动恢复；筛选到第 19 台仍保留第一台。两台 X 各加 0.25 后一次撤销恢复完整 stage，重做并保存后只有两处坐标及修订信息变化。文件独立对照见 `data/UX-048/applied-diff.json`、`persistence-report.json`。
- 最终构建正常退出／重启、最近工程重新载入通过，X=1.377713；打开排列仍已保存，零移动应用不产生新历史，拖动平移视图通过。原生截图 `04-native-draft.png`、`06-native-final.png`、`07-native-pan.png` 均已保存并查看。当前副本已保存、无待应用草稿，音乐未播放、UE 未启动、设备未连接；原来源工程及用户 output/ 未改。

## 审查与接续

当前新增手写模块最大 166 行，StageWorkspace 从 484 行降为 474 行。StageCanvas 为 502 行，审查后保留：本次仅接入独立投影和排列期间的查看手势门，原指针生命周期／绘制未迁移，避免为两行阈值机械拆散手势；后续专门重构该生命周期时再抽取。Workbench 的现有大文件只增加 4 行视图及布局装配，业务不进入入口。共享旧样式只删除失去调用者的小预览规则，桁架挂接仍使用的选择器保留。

本增量可以集成，结果为本次 `feat(stage): dock arrangement editing in the venue plan` 提交。U08 的非模态排列缺口关闭；三维草稿／混合对象变换、其他挂接与批量操作细节和完整 AUDIT-001 仍开放，不能据此宣布专业场地工具或整轮 goal 完成。接续检查挂接／换挂的批量交互与场地上下文，复用成熟对象属性机制，先核对既有实现与真实界面。
