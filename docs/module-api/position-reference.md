# 摇头灯参考点记录与检查

依据 [ADR-086](../development/decisions/PRODUCT-ADR-086-position-reference-checks.md)，POSITION-003。引用世界点和实际输出精度的场景设定，用于复查安装／零偏模型；不是实灯测量或自动定位。

`PositionEdit` 新增：

```ts
type ReferenceCommand =
  | { op: "captureReference"; sceneId: string; fixtureId: string; name: string; targetMeters: SpatialVector3 }
  | { op: "removeReference"; fixtureId: string; pointId: string }
  | { op: "clearReferences"; fixtureId: string };
```

捕获只读取指定场景的静态两轴值（包括已有静态预设引用，缺省／释放使用档案默认值）；有启用的轴效果则拒绝。不采样运行帧、不继承其他场景。前端不传轴值。8 位轴保存实际高字节乘 257，16 位保存实际 u16，均可无损重建输出设定；零偏不叠加到记录值。必须存在模型与安装位置，目标不能等于轴心。

Fixture 可选 `positionReference = {profileId,profileRevision,points}`，要求 `lighting.position-reference@1`。points 为 `{id,name,targetMeters,panValue,tiltValue,source:"sceneSetpoint"}`；每灯 1–16 点、全工程最多 1024 点，坐标 ±100000 米、轴值 0–65535、名称 1–256 字符且逐灯唯一、身份全工程唯一。无未知字段。删除最后一点删除容器；能力声明可保留。文件包含历史档案标识，不要求历史版本仍存在。

`FixtureView.positionReference` 省略无记录项；有记录时含历史档案标识、compatible、`points:[{point,check}]`。check 是 `checked`（沿光束有符号距离、正向射线最近点／误差米数、角误差）或 `unavailable`（明确原因）。没有自动的“校准合格”阈值。

换档案／档案修订不同会保留历史但标记不相容，禁止追加，清除后可新建。安装或单灯零偏修改会按当前模型重算旧点；场景修改／删除不会改点。不存在安装／模型仍可读取和清理旧记录。检查不改变亮度、两轴、灯位、场景、三维或播放包。

UI 单灯位置区提供“轴与指向／参考点检查”。录入复用平面选点及世界 XYZ，名称／坐标草稿由独立组件维护，错误定位、取消／Escape、上下文切换继续走共享草稿事务。删除在当前草稿完成或取消后可用，单次应用对应一次可撤销历史。原轴编辑移到独立组件，入口只组合工具。

核心所有者分别为 `stagemaster-spatial::positioning::check_reference` 的空间数学和 `stagemaster-project::position::reference` 的记录、校验、事务与投影；UI 不计算几何误差。实灯对齐、机械速度限制、反馈测量和拟合求解均另行实现。
