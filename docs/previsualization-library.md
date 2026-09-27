# 预演资源库与编排能力

2026-09-28；PREVIS-001／POSITION-001；UE 5.8.3。用户要求尽量吸收现有灯光效果库。资源按以下边界复用，不将素材存在等同于编排能力已经完成。

| 层 | 复用对象 | 当前状态 | 后续接入位置 |
| --- | --- | --- | --- |
| 灯具外形 | Epic DMXFixtures 固定灯、摇头灯、LED 染色灯、固定矩阵、摇头矩阵、频闪灯、反射镜扫描灯 | 7 个蓝图类实际加载测试通过；固定灯外壳／镜片已接渲染并验收；两轴使用引擎基础网格与固定灯网格组合，Rust 给定底座／支架／灯头姿态，未接官方摇头蓝图的 DMX 逻辑 | UE 可替换视效适配 |
| 光学视效 | MI_Beam、MI_Lens、MI_LightNoGobo、MI_MatrixBeam | 4 种材质实际加载通过，已核对参数；镜片接亮度与 RGB，图案／矩阵未接编排 | UE 材质与灯光表现 |
| 专项视效 | 激光、火焰、烟花、喷泉蓝图 | 4 个蓝图类加载通过；尚未生成到真实工程，不代表设备控制支持 | 预演专项适配器，真实动作另走设备网关 |
| 灯具档案 | GDTF／GDTF Share、OFL 与用户个人档案 | 已有研究与架构，尚未实现导入及复杂功能分段 | FixtureLibrary／导入适配，不由 UE 覆盖通道语义 |
| 动态编排 | 追逐、波形、颜色渐变、相位分布、指向轨迹 | Rust 已实现亮度呼吸／追逐、双色循环及最多 32 帧循环；位置支持两轴渐变与共同点静态对焦，连续轨迹后续 | 工程效果领域／编译器／播放器 |

已启用引擎自带 DMXFixtures 和 Niagara，资源目录约 116 MB。源码只引用引擎资产路径，未复制官方二进制素材到 Git、工程 JSON 或 ESP32 包。交付打包按真实引用包含资源；完整光学／外形库与微控制器执行包分开。所有 UE DMX 收发默认关闭，预演不持有实际输出权。

## 可复现资产清单

根：`/DMXFixtures/LightFixtures/`。

- `BP_StaticHead`、`BP_MovingHead`、`BP_WashLED`、`BP_StaticMatrix`、`BP_MovingMatrix`、`BP_StaticStrobe`、`BP_MovingMiror`；最后一个名称按官方实际拼写。
- `DMX_Materials/MI_Beam`、`MI_Lens`、`MI_LightNoGobo`、`MI_MatrixBeam`。
- `Meshes/SM_Static_Base`、`SM_Static_Lens`、`SM_Beam_RM` 三个网格实际加载通过；镜片局部平面位于 +Z 14.747 厘米，接入时显式转换到 Rust 提供的光轴原点和方向。
- `/DMXFixtures/Laser/BP_LaserModule`、`/DMXFixtures/Pyro/BP_PyroModule`、`/DMXFixtures/WaterFountains/BP_WaterSource`、`/DMXFixtures/Fireworks/BP_FireWorksLauncher`。

自动验收入口为 `StageMaster.Previs.OfficialVisualLibrary`，位于 [UE 测试](../apps/previs-unreal/Source/StageMasterPreview/PreviewLibraryTests.cpp)。测试加载真实资产并读取材质参数，不创建灯具演员或发送 DMX。加载通过不证明真实光度、遮挡质量、帧率或完整硬件行为。

## 取舍与参考

[DMX 官方快速开始](https://dev.epicgames.com/documentation/unreal-engine/dmx-quick-start-in-unreal-engine)提供现成模块；[DMX 预演示例](https://dev.epicgames.com/documentation/unreal-engine/dmx-previs-sample-project-for-unreal-engine)提供进一步光束质量参考。示例工程尚未下载，不宣称已安装。现有官方光束的遮挡／材质机制可复用，但编排时间、状态与真实灯具映射仍以 Rust 为准。

[GDTF Share](https://gdtf-share.com/help/users/gdtf_share/)按档案修订与模式使用；当前调光／RGB 与相交正交两轴增量不足以无损表达所有复杂灯具，导入仍需逐功能适配，不能批量下载后直接认定可控。真实光度和色盘、图案、棱镜、变焦、柔光等逐能力声明、逐灯档验证。

视窗呈现按 [ADR-019](development/decisions/PRODUCT-ADR-019-embedded-previsualization.md)集成在舞台大师内部。渲染失败不影响主工程或播放器。
