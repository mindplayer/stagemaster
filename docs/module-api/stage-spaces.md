# 场地编辑模块 API

PREVIS-001／ADR-017，Rust `stagemaster-project::StageEdit` 是真实命令，TS `stage-types.ts` 是对应宿主输入；当前 `stage.spaces@1` 为草案扩展。所有命令通过现有 `Document::edit`／桌面 `project_request`，共享版本校验、原子回滚、撤销重做及真实保存。

```ts
host.request({kind: "edit", generation, command: {
  op: "stage", command: {
    op: "putSpace", id: null, name: "演出厅",
    outlineMeters: [["0","0"],["8","0"],["8","6"],["0","6"]],
    floorElevationMeters: "0", clearHeightMeters: "5"
  }
}});
```

`putSpace`／`putConstruction` 中 `id: null` 创建新身份，指定身份只更新已存在对象，不隐式补建。`duplicateSpace` 复制范围与围护并生成新身份，不复制灯具或舞台；`duplicateConstruction` 复制独立平台或支撑体，不复制挂灯。`removeSpace` 的 `detachMembers` 明确选择是否解除归属并删除围护；有成员时 `false` 拒绝，无成员时可直接移除。空间归属不是坐标父级，解除归属后世界位置保持。

`putPlacement` 按 `fixtureId` 创建／替换唯一灯位；`removePlacement` 只移除灯位，不删除灯具。删除已布置灯具必须先移除灯位，可以通过一个原子批次组合，不能留下悬空引用。批次 1–256 项，所有命令通过后一次提交。

安装坐标使用右手 XY 平面、Z 向上、米制十进制字符串。`rotationDegreesXYZ` 是底座安装欧拉角，按 Rz × Ry × Rx 组合，单位度；它不代表摇头灯控制轴。未布置灯具不自动置于原点。主工程不保存相机、网格、选择、隐藏状态或三角网格缓存。

空间轮廓 3–128 顶点、不重复首尾；`stagemaster-spatial::polygon::floor_plan` 使用 geo OGC 验证及 Earcut 三角化，接受凹多边形，拒绝自交、退化、非有限、过小／过大几何。暂不支持洞、曲线、共享墙、门窗、坡顶。空间最多 64，构件 512，灯位 1024；围护需要有界净高且每空间一组。世界 XY／灯位在 ±100000 米内，标高 ±10000 米，净高 0.1–1000 米，台高 0.001–1000 米，厚度 0.001–10 米，安装角 ±3600 度。

桌面 `StageWorkspace` 持有选择／搜索／未提交草稿；`StageInspector` 精确编辑；`StageCanvas` 只处理镜头与平面手势。手势中不写核心，结束提交一次；取消不提交。字段草稿与原生保存／切换／关闭共享 collect／accept 流程，验证失败保留输入。渲染器不得直接改写 JSON。

开发期 JS 格式审计覆盖 Schema、引用与数值边界，几何有效性由 Rust／geo 权威校验；JS 工具不能代替产品加载器。PREVIS-001 已接内嵌 UE；Rust 把围护／平台／支撑体生成可重建三角网格，与同份已应用灯位一起送入 UE，修改后同步。二维与三维不各存一套场地。真实现场输出仍未接通，参考预演不表示完成专业光学验证。

## UX-013 前端尺寸适配

`StageCreateDialog` 将矩形／L 形尺寸转换为既有轮廓命令，确认后一次创建，取消不发命令。`OutlineDimensions` 保留未完成数字草稿，统一表单校验后才发命令；按包围框缩放原多边形，不转换成矩形。`StageSelectionOverlay` 与 `StageCanvas` 提交单次手势结果，原坐标／标高／灯位成员关系仍由工程对象持有。移动空间轮廓或改变尺寸不移动成员的世界坐标。

层级、折叠、搜索、相机与尺寸标注均为临时 UI 状态；不新增工程字段。高级顶点仍可编辑任意合法轮廓。门洞、共享墙及空间间碰撞暂未实现。

## UX-014 灯位排列与成组操作

`placement-tools.ts` 是界面编辑坐标提案器，复用原子 `batch + putPlacement`，不新增运行语义、工程格式或板卡 API。`ArrangementDialog` 管理搜索／灯组范围、有序选择与未提交参数，在自身俯视图显示结果；应用后才更新工程和三维。取消不进入历史。Rust 继续检查引用、坐标、角度及整批合法性。

- 直线：以中心为基准，按选择顺序等间距排列，可设置世界 XY 角度。
- 矩阵：每排灯数、灯间距和排间距；逐排填充，末排不足时保持列网格。整体按网格包围中心定位。
- 圆弧：中心、半径、起始角和 0.01–360° 范围；满圆按 N 等分避免首尾重合，非满圆包含两端；单灯位于起始角。
- 平移／旋转：对已有灯位按 XY 包围中心旋转，再加 XYZ 位移；保留各灯高度差、空间归属和底座朝向。
- 对齐／等距分布：沿 X／Y／Z 对齐最小／中点／最大值，或按原空间次序分布，保留两端和其他坐标。
- 底座安装朝向只在显式勾选时统一；这些角度不解释为摇头灯水平／纵向参数。阵列默认沿用已有灯具的底座方向，新灯位默认零度。

最多 256 台一次应用，超过批次上限明确拒绝，不偷偷拆成多次历史。排列处理已有或未布置灯具，后者仍必须是工程中已有的灯具身份，不自动创建／复制灯具。已选范围不会因搜索隐藏而缩小；提示隐藏数量。已选行按灯序显示，未选行保留候选顺序；选中筛选结果按灯组或工程原序追加。灯组仅提供有序范围，不创建持久挂接关系。

平面画布分为选择／移动／平移视图三种工具。选择模式在场地上拖框只选灯位，Shift／Command 点击可增减；移动模式拖动所选灯具时整组移动，结束一次提交，取消不提交；方向键按 0.1 米、Shift 按 1 米微调。聚焦选中组、灯具名称显隐、灯序编号、相机与选择均为 UI 状态，不写进工程。空间／舞台轮廓仍单对象编辑，不能把灯位框选扩称为任意场景层级组变换。

三维组平移接续 STAGE-005／ADR-094（下文）；旋转、缩放和混合构件组不属于此次平移增量。

## STAGE-001 支撑体与挂接

依据 [ADR-023](../development/decisions/PRODUCT-ADR-023-rigging-assembly.md)。`stage.rigging@1` 是新增必需能力；旧工程省略 `attachments` 时读取为空，含支撑体或挂接数据时必须声明能力。

```ts
// 包在上述 op: "stage" 的 command 中；十进制数均为字符串。
{ op: "putConstruction", id: null, name: "前桁架", shape: {
  kind: "rig", rigKind: "truss", spaceId: null,
  positionMeters: { x: "4", y: "3", z: "5" }, yawDegrees: "0",
  lengthMeters: "6", widthMeters: "0.3", heightMeters: "0.3"
}}
{ op: "attachFixtures", constructionId: "支撑体身份", fixtureIds: ["灯具身份"],
  layout: { startMarginMeters: "0.3", endMarginMeters: "0.3", dropMeters: "0.1" }
}
{ op: "attachFixtures", constructionId: null, fixtureIds: ["灯具身份"], layout: null }
{ op: "removeConstruction", id: "支撑体身份", detachFixtures: true }
```

`rigKind` 为 `truss|pipe`，当前均为水平直线实体，尺寸为外包络。支撑体中心是世界坐标，空间只是归属。一次挂接 1–256 个有序唯一灯具；沿长度均布，首末端余量从两端计算，下挂距离从底面计算。单灯置于可用跨度中心；已有灯具保留底座角度，新灯位零度。`layout:null` 保持既有位置，未布置灯具拒绝；指定另一支撑体即明确换挂。

`stage.attachments` 保存 `{fixtureId, constructionId}`，一灯最多一个支撑体，必须存在灯位且空间归属一致。独立 Rust `rigging` 模块维护关联，世界灯位是唯一坐标来源。支撑体更新时旋转／平移／升降与挂灯变换在同一事务提交；底座 X/Y 不变，Z 随水平角变化并归一化。独立微调灯位后仍可随支撑体移动；修改支撑体长度不自动重排。单独改变挂灯的空间归属须先解除。

删除默认拒绝有挂灯的支撑体；`detachFixtures:true` 保留全部灯具和世界灯位。移除灯位同时解除对应挂接；删除空间时的 `detachMembers:true` 清除空间归属，保留支撑体关联及世界位置。复制支撑体只复制实体，不复制灯具／地址。失败回滚和撤销恢复覆盖全部依赖更新。

`OrderedFixturePicker` 由排列与挂灯共用搜索、灯组、顺序和隐藏选择逻辑；`RigFields` 由新建和属性面板共用。TS 中跟随位置仅是未提交的二维视觉草稿，Rust 计算正式变换。UE 继续读取原网格协议，桁架／灯杆没有专属播放规则或直接写工程权限；本轮在应用内验证房间、平台、两种支撑体、灯具与光束，以及旋转后的同步。

新增测距工具只报告世界 XY 距离和 ΔX／ΔY，不能解释为三维净距；测量、清除、Esc 和测距时方向键均不改变工程。相机、测距、按支撑体选灯不写入历史。

## STAGE-002 场地编辑保护

依据 [ADR-065](../development/decisions/PRODUCT-ADR-065-stage-edit-locks.md)。可选 `stage.editLocks` 保存 `{kind: "space"|"construction"|"placement", targetId: UUID}`；非空必须声明 `stage.edit-locks@1`，上限 1600 个唯一有效引用。旧工程省略字段，清空后同时移除字段与能力声明。

```ts
{ op: "stage", command: { op: "setEditLocks", locked: true,
  targets: [{ kind: "placement", targetId: "灯具身份" }]
}}
```

一次接受 1–1600 个对象，整批一次历史。锁定属于工程编辑保护：可选择、检索、复制、撤销及保存，不改变运行编译、灯光值或设备包。空间只锁自身属性；成员位置独立。灯位还锁安装朝向、空间归属与挂接关系；桁架联动或解除空间归属若改变锁灯，则整个事务拒绝。围护保护依赖的房间几何。复制空间／构件使用新身份且不继承锁；不能通过“先改后解锁”绕过保护，显式“解锁→修改→重锁”可作为原子批次。

Rust 独立 `stage_locks` 在每个子操作前后比较受保护投影；三维提案、界面和 API 都经过同一检查，失败不增加历史。UI 的目录标记、右侧锁定区、混合选择移动禁用、三维移动禁用是提前反馈，不是权威校验。隐藏对象仍参与依赖保护。锁定前既有字段草稿先按原规则校验应用，错误保留并定位，草稿取消不会修改锁。

`useStageObjects` 管理创建／复制／移除，`StageObjectDialogs` 渲染相应对话框，`useStageRigging` 管理挂灯流程；工作区保留选择／草稿协调。位置和轮廓属性各自独立组件，不在工作区继续堆积创建表单。三维仍只有一个预演视窗，锁状态不新增 UE 网格或播放协议。

## STAGE-003 参数化观众座区

依据 [ADR-066](../development/decisions/PRODUCT-ADR-066-parametric-seating.md)。新构件 `seating` 需要 `stage.seating@1`；编辑入口复用 `putConstruction`，整区使用同一身份、锁定、复制、删除和历史。例：

```ts
{ op: "stage", command: { op: "putConstruction", id: null, name: "中央座区", shape: {
  kind: "seating", spaceId: null,
  positionMeters: { x: "5", y: "-2", z: "0" }, yawDegrees: "0",
  rows: 5, columns: 6,
  seatWidthMeters: "0.46", seatDepthMeters: "0.48",
  columnSpacingMeters: "0.56", rowSpacingMeters: "0.9",
  aisle: { afterColumn: 3, widthMeters: "1.2" }
}}}
```

XY 是整个座区外包矩形中心，Z 是椅脚平面；0° 面向 +Y，第一排在 +Y 端，列从局部 -X 到 +X，角度逆时针。排／列间距是中心距离；通道净宽替换指定两列间普通间隙，不增加或删减座位。所属空间只做组织归属。`aisle:null` 不留通道。

Rust `SeatingShape::seat_count` 先限量，`layout()` 校验尺寸／边界并输出座心、轮廓、朝向和椅宽深；TS `seatingLayout` 仅为有界草稿投影，共享测试向量在 `tools/test-data/seating-layout.json`。无效草稿保留原平面图，不向 UE 提交；应用成功后才更新三维。每区不超过 512 座、工程不超过 1024 座，排／列分别不超过 64；三维每椅 72 面，整体预演仍有 100000 面预算。

`SeatingFields` 共用于创建与属性，`SeatingPlanObject` 独立画座椅／方向；目录整区列一行，专属座区平面显隐。UE 复用网格协议，每区一个网格且不保存派生座椅。示意椅坐面高 0.45 米、靠背顶高 0.85 米，无逐座持久身份；不可将生成的排／列索引作为其他模块稳定引用。STAGE-003 不含任意轮廓裁切、阶梯、多通道、座位编号覆盖或场地碰撞／疏散合规判定；弧形后续扩展见下文；旧示意构件不会自动转换。


## STAGE-004 弧形座区

按 [ADR-075](../development/decisions/PRODUCT-ADR-075-curved-seating.md)，seating 可附 `arc:{radiusMeters:"5"}`，实际弧形需要 `stage.seating.arc@1`；缺省／null 仍直排。原 putConstruction／复制／锁／历史接口不变。radius 为前排座心半径 1–10000 米，每排同座数且面向同一焦点；列中心距在弧形中解释为前排弧长间距，排距为径向间距。保留半圆与座区预算，过密导致的椅脚印重叠明确拒绝。

内侧通道以两侧椅子内角之间的实际净宽求解，后排按同一角度延伸并加宽。positionMeters 仍为局部外包矩形的世界中心；弧半径调整不引入第二个持久焦点，派生焦点仅用于显示。SeatingLayout 增加 `seat_yaws_radians`（每座世界角度）与 `focus`（可选世界点）；原 outline 四点／全区 yaw 保持。UE 根据每座角度生成已有三角网格，无新增后台接口。

界面创建／属性共用排列方式、半径及可视排布，主平面显示逐座朝向与选中区的焦点连线。原矩形依独立旧向量保持；弧形共用 `tools/test-data/seating-arc-layout.json`，离线格式审计另外以二分法求通道角度／多边形分离轴检查。选区仍用外包矩形；完整轮廓裁切、跨座区碰撞、阶梯与自动增加后排座数不在本增量。


## STAGE-005 三维共享选择与组平移

依据 [ADR-094](../development/decisions/PRODUCT-ADR-094-shared-3d-fixture-movement.md)，当前验收状态见[工单](../development/tasks/STAGE-005-shared-3d-fixture-movement.md)。核心新增：

```ts
{ op: "stage", command: { op: "translatePlacements", fixtureIds: ["灯具身份"],
  deltaMeters: { x: "1.25", y: "0", z: "-0.625" }
}}
```

选择必须为 1–256 个唯一已布置灯具。位移最多六位小数、绝对值不超过 200000 米；由 Rust 从权威世界坐标计算，移动后每轴仍在 ±100000 米。校验所有成员后原子应用，保留每灯高差、安装角、空间和挂接；锁定成员的实际变更会拒绝整个事务。未移动轴保留原十进制文本，全零不增加历史。工程格式不变。

平面／目录／三维共享有序选择，最多 1024 台用于查看，超过 256 台禁用整组移动；末项为活动对象。查看模式点击替换、Shift／Command／Ctrl 增减，普通空白清空；移动时抓住所选成员保留整组，加选点击不启动拖动。水平和升降为显式模式，升降需透视镜头。渲染器只发位移提案，正式位置由上述核心命令决定。查看状态、临时手势、取消和相机不写工程；协议与过期保护见 [预演 API](previsualization.md)。


## STAGE-007 混合对象平移

依据 [ADR-146](../development/decisions/PRODUCT-ADR-146-mixed-stage-translation.md)，使用相同 Document 事务入口：

```ts
{ op: "stage", command: { op: "translateObjects",
  targets: [{ kind: "construction", targetId: "桁架身份" },
            { kind: "placement", targetId: "灯具身份" }],
  deltaMeters: { x: "0.25", y: "0", z: "0.1" }
}}
```

1–256 个唯一直接目标，支持已布置灯具、支撑体、地台和座区；空间／围护拒绝。三轴位移为最多六位小数的严格十进制字符串，绝对值不超过 200000 米，最终对象边界由既有格式和几何验证。桁架挂灯自动加入影响集合，显式选中同一挂灯不重复移动；间接成员可超过 256，但不突破既有工程灯位容量。地台平移轮廓和基准标高，其余对象平移世界坐标；保留尺寸、角度、空间、挂接及未移动轴的原字符串。任何直接／间接锁定成员的变更或任何非法结果整体拒绝；全零不增加历史。

useStageSelection 统一目录与平面的有序对象身份；纯灯具保留排列／挂接，混合组使用相对 XYZ 属性与展示预览。操作通过原 collect／accept 与保存／切页协作；提交后选择复用 useCommittedStageAction 的最新数据和生命周期隔离。usePlanGesture 负责指针源快照、取消、锁定和释放时一次提交，关闭吸附仍按六位精度提交。框选目前仍只选灯具；三维只接收纯灯具组，混合组不发送灯具子集，避免只移动一部分。已提交场地仍走原 UE 网格投影，没有第二份持久场地。
