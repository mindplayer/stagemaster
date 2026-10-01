# 位置求解模块

2026-09-28；POSITION-001；依据 [ADR-016](../development/decisions/PRODUCT-ADR-016-previsualization-and-positioning.md) 与 [ADR-025](../development/decisions/PRODUCT-ADR-025-moving-head-workflow.md)。Rust 静态求解已接入工程档案、场景轴角、既有编译器和内嵌 UE 姿态。没有真实设备输出或实测自动校准；共同空间直线轨迹已由 EFFECT-007 实现。

## 已实现接口

模块：`stagemaster-spatial::positioning`。不依赖工程 JSON、UI、时钟、文件、网络、DMX 或 UE。复用 `glam 0.33.10` 的双精度向量／四元数，避免另造通用矩阵库；依赖已锁定，当前启用 std／f64，不要求把它放入 ESP32 执行固件。

```rust
// 输入均来自调用方已经选定的工程／档案修订。
let model = IntersectingHead {
    pan: AxisRange { min_degrees: -270.0, max_degrees: 270.0 },
    tilt: AxisRange { min_degrees: -135.0, max_degrees: 135.0 },
    zero_correction: ZeroCorrection::default(),
};
let solution = model.solve(installation, target_meters, previous_angles, None)?;
let emitted_ray = model.ray(installation, solution.angles)?;
```

数值仅是 API 用法中的测试模型范围，不是内置真实型号或所有摇头灯的默认档案。

| 输入／输出 | 含义 |
| --- | --- |
| `Installation` | 世界坐标米、固定 X／Y／Z 轴旋转角；内部组合 Rz×Ry×Rx，不允许缩放灯体来改变物理行程 |
| `AxisRange` | 分别校验的水平／垂直物理角范围；保留展开角，可表达超过一周但有限的行程 |
| `ZeroCorrection` | 实例零偏，仅在物理正逆变换应用一次；不是编码器反向或 DMX 字节反向 |
| `JointAngles` | 档案解码出的物理角，尚未加实例零偏；不是 0–100% |
| `solve(..., previous, branch)` | 显式上次设定角、可选解分支；选轴角欧氏行程最小的静态可达解，不持有隐藏历史 |
| `Solution` | 展开角、前／后解分支、奇点标记和垂直于射线的米制残差；不代表实灯反馈或无碰撞轨迹 |
| `pose(...)` | 安装底座、水平支架和灯头的世界 +X／+Z 基向量，以及同一光束方向；不包含 UE 类型 |
| `ray(...)` | 同一份正向计算，给渲染器出光方向；底座／灯体显示也须使用一致姿态 |

模型**明确限于相交、正交两轴，轴心兼出光原点**：水平绕局部 +Z、垂直绕随水平转动的 +X，修正后双轴为零时光束沿局部 -Z。与 GDTF 几何基准不同的档案必须由适配器明确转换；有轴距、出光口偏心、多头／多轴或连续旋转功能的档案不能直接声称适用。

参数先检查有限性与范围。精确轴向目标保持调用方给出的水平角并标记奇点；目标在轴心、非法输入、行程不可达分别返回错误。机械边界只修正不超过 `1e-10°` 的数值舍入，并再次正向验证；不把不能到达的目标夹到边界。

## 工程和编辑接口（已实现）

`Profile.positioning` 是可选的 `{kind:"intersectingOrthogonal",pan:PositionAxis,tilt:PositionAxis}`；`PositionAxis` 包含十进制字符串 `minDegrees/maxDegrees` 和独立 `reversed`。两轴分别限于 ±3600°、最小值严格小于最大值，必须有 normalized／LTP 属性和实际通道映射。增加此字段必须声明 `lighting.positioning@1`，旧应用拒绝未知能力；旧固定灯工程无需新增字段。

`Fixture.zeroCorrection` 是可选的 `{panDegrees,tiltDegrees}`，均为 ±360° 内十进制字符串，作用于单灯全部场景。它只是手工零偏，没有测量、拟合和实灯校准精度承诺。安装位置和旋转仍在 `stage.placements`，与实例零偏、档案物理角映射反向分别归属。档案 `reversed` 不等于实例最终 DMX 输出反向；后者和编码器操作反向尚未提供。

```ts
type PositionCommand =
  | {op:"axes"; sceneId:string; fixtureIds:string[];
      panDegrees:string|null; tiltDegrees:string|null}
  | {op:"aim"; sceneId:string; fixtureIds:string[];
      targetMeters:{x:string;y:string;z:string}; branch:"front"|"back"|null}
  | {op:"offsetAxes"; sceneId:string; fixtureIds:string[];
      panDegrees:string|null; tiltDegrees:string|null}
  | {op:"flip"|"home"; sceneId:string; fixtureIds:string[]}
  | {op:"calibrate"; fixtureId:string;
      correction:{panDegrees:string;tiltDegrees:string}|null};
// project_request：{kind:"edit",generation,command:{op:"position",command}}
```

- Rust `Document::edit` 管理校验和写入，`Document::position_model` 提供只读档案。选择 1–128 台唯一灯具；UI 不求逆解。批次先逐灯求解，任一缺灯位、不可达或范围错误均不写入工程。错误携带灯名，Session 统一历史、保存和预览失效。
- `axes` 至少填写一轴，未填写的轴保持；`home` 将档案两轴默认值记录进当前场景，不动光色、不发送设备复位。
- `aim` 输入世界坐标米，逐灯使用安装、零偏、行程和当前场景值／预设／默认值选解。结果写回场景两轴整数；不持久保存目标点，后续移动灯位不会自动追踪。分支约束不可达时拒绝，不强行夹到边界。
- 原有归一化整数仍为唯一播放值。16 位按 65535、8 位按编码器高字节／255 解码；角度输入就近量化。反向只作用于映射一次，零偏只在运动变换应用一次。预演读实际量化值，不假装八位灯有十六位精度。
- “释放位置／清除位置”复用原子批次内的两轴 `setSceneValue`，分别结束列表跟踪／删除本场景记录。清除在继承列表中可能沿用前一场景值；不是回默认值。场景间仍按轴角渐变，不保证光点沿世界直线移动。
- 换档案除属性语义一致外，还必须有相同运动定义；不同物理范围／反向的换灯暂拒绝，避免已有场景值被静默解释成另一个位置。
- UE 通过中立姿态显示独立底座、支架和灯头，直接使用 Rust 出光方向；关节为通用形状，不是型号尺寸、碰撞或光学仿真。见[预演协议](previsualization.md)。

## 桌面共同目标编辑

[UX-040](../development/tasks/UX-040-common-target-plane.md) 使用已有场地投影提供平面点选／拖动、视图导航及方向键微调。XY 与标高独立，手势抬手只改草稿，应用才调用上述 `aim`；取消、失焦、尺寸改变不留下半次手势。输入失败保持草稿并定位，取消恢复上次已接受目标。目标是本地编辑草稿，重开不恢复目标引用；保存的是 Rust 求解后的两轴值。此图不承担光束渲染、现场发送或连续目标跟随。

## 后续产品接口边界（尚未实现）

EFFECT-007 已有独立 `trajectory::LineTrajectory`：显式灯具模型／安装、世界起止点、previous 和可选分支，构造时解析检查整线行程与转轴奇点；`sample(0..1)` 无状态返回连续展开轴角。`derivative_bounds(from,to)` 给出区间内轴角一阶／二阶导数绝对上界（对归一化进度，非电机速度）；线性近似误差不超过二阶上界乘区间长度平方除以 8。由工程 `worldLine` 效果按余弦进度往返，独立编译器结合导数、相位与通道量化误差认证固定 32 帧；已接桌面编辑、草稿预演和既有执行包，空间层不直接发送设备。参见 [ADR-084](../development/decisions/PRODUCT-ADR-084-world-line-effects.md)。

`PositionIntent` 将在工程契约中分为轴角和空间目标，后者记录坐标系及目标身份，不能同时保存两套互相抢控制的权威值。目标绑定和灯组成员顺序归工程；求解上下文归编译／独立执行；姿态显示归渲染适配。

`CalibrationService` 负责采样、拟合、误差、修订与失效；不让界面直接改物理矩阵。校准记录包含档案修订、设备模式、安装姿态和参考点，拟合参数数量、样本分布及留出验证必须匹配，不能把“四个点”当作万能公式。

`PositionTrajectoryCompiler` 负责轴锁定、分支连续性、角行程、已知速度／加速度和明确的黑场移动策略。静态解靠近上一帧不保证穿越奇点时路径平滑；不得直接每帧调用当前 `solve` 就称为专业跟随。实时硬件输出需要经过该层与控制权检查。

`FixtureEncoder` 的线性粗细编码和工程物理角映射已接通；功能段和完整型号转换仍待扩展。直接轴角操作、定位状态、机械复位、停放策略分别发出明确命令。翻转操作和退出空间指向都要显示影响并单次撤销；仅改变观察视角不能写工程。

## 验证

首轮 10 项保护测试覆盖右手坐标基准、挂装／侧装、同目标逐灯求解、双解、540° 行程不回绕、轴向奇点、零偏、不可达、非有限输入，以及 216 组安装／校准／轴端点正逆往返。首次端点测试发现浮点边界误判，已修复边界舍入并保留原测试。模块测试与严格 Clippy 通过；尚无光学精度或实灯校准结论。

## 相对轴编辑与翻转（POSITION-002）

依据 [ADR-085](../development/decisions/PRODUCT-ADR-085-relative-axis-and-flip.md)。`offsetAxes` 的值是各灯本场景基值上的角增量，空或零轴保持且至少一轴非零；修改轴由预设引用转当前场景常值，预设资源不变。基值使用本场景常值／预设／档案默认，明确不取运行帧／列表跟踪结果。若所请求轴被启用效果控制、越界或增量小于该灯可分辨精度，整批拒绝并报灯名／效果名。

`IntersectingHead::flip(previous)` 复用射线与静态求解，取另一支架分支的最近可达展开角；零偏只算一次。无须舞台安装，因为共同安装刚体变换不改变两分支的射线等价性；轴向奇点拒绝。工程 `flip` 仍按实际 8／16 位量化写双轴，终点的近似相等不等于中途光点保持或碰撞／速度保证。两命令是工程编辑，不发送设备。

UI 将相对微调草稿、1／0.1／0.01 度步幅和精确输入单独封装；每轴按钮不会改另一轴，步幅设置不产生草稿。应用、取消、错误、模式切换沿用位置事务。手工零偏仍是实例物理模型修正，未增加最终输出补偿或实测拟合；不得称作 MA 的四点校准已完成。

POSITION-003 增加独立[参考点记录与检查](position-reference.md)：固定场景轴设定和档案版本，用同一正向模型检查世界目标；不是实灯反馈或自动拟合。
