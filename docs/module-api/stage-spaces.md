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

`putSpace`／`putConstruction` 中 `id: null` 创建新身份，指定身份只更新已存在对象，不隐式补建。`duplicateSpace` 复制范围与围护并生成新身份，不复制灯具或舞台；`duplicateConstruction` 仅复制独立平台。`removeSpace` 的 `detachMembers` 明确选择是否解除归属并删除围护；有成员时 `false` 拒绝，无成员时可直接移除。空间归属不是坐标父级，解除归属后世界位置保持。

`putPlacement` 按 `fixtureId` 创建／替换唯一灯位；`removePlacement` 只移除灯位，不删除灯具。删除已布置灯具必须先移除灯位，可以通过一个原子批次组合，不能留下悬空引用。批次 1–256 项，所有命令通过后一次提交。

安装坐标使用右手 XY 平面、Z 向上、米制十进制字符串。`rotationDegreesXYZ` 是底座安装欧拉角，按 Rz × Ry × Rx 组合，单位度；它不代表摇头灯控制轴。未布置灯具不自动置于原点。主工程不保存相机、网格、选择、隐藏状态或三角网格缓存。

空间轮廓 3–128 顶点、不重复首尾；`stagemaster-spatial::polygon::floor_plan` 使用 geo OGC 验证及 Earcut 三角化，接受凹多边形，拒绝自交、退化、非有限、过小／过大几何。暂不支持洞、曲线、共享墙、门窗、坡顶。空间最多 64，构件 512，灯位 1024；围护需要有界净高且每空间一组。世界 XY／灯位在 ±100000 米内，标高 ±10000 米，净高 0.1–1000 米，台高 0.001–1000 米，厚度 0.001–10 米，安装角 ±3600 度。

桌面 `StageWorkspace` 持有选择／搜索／未提交草稿；`StageInspector` 精确编辑；`StageCanvas` 只处理镜头与平面手势。手势中不写核心，结束提交一次；取消不提交。字段草稿与原生保存／切换／关闭共享 collect／accept 流程，验证失败保留输入。渲染器不得直接改写 JSON。

开发期 JS 格式审计覆盖 Schema、引用与数值边界，几何有效性由 Rust／geo 权威校验；JS 工具不能代替产品加载器。三维预演桥、UE 构件生成和现场联动仍在父任务中实施；本文不把平面编辑或静态指向求解视为已完成 UE 预演。

## UX-013 前端尺寸适配

`StageCreateDialog` 将矩形／L 形尺寸转换为既有轮廓命令，确认后一次创建，取消不发命令。`OutlineDimensions` 保留未完成数字草稿，统一表单校验后才发命令；按包围框缩放原多边形，不转换成矩形。`StageSelectionOverlay` 与 `StageCanvas` 提交单次手势结果，原坐标／标高／灯位成员关系仍由工程对象持有。移动空间轮廓或改变尺寸不移动成员的世界坐标。

层级、折叠、搜索、相机与尺寸标注均为临时 UI 状态；不新增工程字段。高级顶点仍可编辑任意合法轮廓。门洞、共享墙及空间间碰撞暂未实现。
