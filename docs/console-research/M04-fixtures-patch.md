# 灯具档案、配适与设备维护

配适（Patch）把节目中的逻辑灯具关联到真实设备、运行模式和 DMX 地址。档案定义如何解释属性，配适定义当前灯具在哪里、用哪种模式以及输出到哪里。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M04-01 | 灯具档案 | Fixture Type 含模式、通道与物理信息；内置编辑器及 GDTF 来源 | Personality 定义通道、属性和设备行为；独立 Builder 与官方库 | [MA Types](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[Titan Personalities](https://manual.avolites.com/docs/fixture-personalities) |
| M04-02 | 批量配适 | 灯型／模式、数量、ID、地址等向导 | 厂商／型号／模式、数量、Line、地址、Offset 与 Handle | [MA Add](https://help.malighting.com/grandMA3/2.5/HTML/patch_add_fixtures.html)、[Titan Add](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-03 | 多种灯具身份 | Fixture ID、Channel ID／ID Type、名称等 | User Number、Handle、Legend 等 | 同上 |
| M04-04 | 地址及冲突 | 配适表、Universe 和 DMX Sheet | Patch View 显示占用；冲突可取消或 Park 相关灯具 | [MA Universes](https://help.malighting.com/grandMA3/2.5/HTML/patch_dmx_universe.html)、[Titan Change](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-05 | 多 cell 灯具 | 子灯与几何层级、属性定义 | Super Fixture／Sub-fixture，整体移动并保留 cell 控制 | [MA Fixtures](https://help.malighting.com/grandMA3/2.5/HTML/patch_what_are_fixtures.html)、[Titan Add](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-06 | 同步地址副本 | Multipatch 跟随主灯，独立地址／3D 位置；不参加 Selection Grid | 多 Dimmer 可配到同一 Handle；不是同一种完整灯具镜像 | [MA Multipatch](https://help.malighting.com/grandMA3/2.5/HTML/patch_add_multipatch.html)、[Titan Add](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-07 | 独立调光器 | 灯型和属性模型表达控制关系 | Pending Dimmer 可把独立调光通道与灯具合并操作 | [MA Types](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[Titan Add](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-08 | 虚拟属性 | 参数可包含虚拟控制及 XYZ 等计算 | Generic RGB 可使用 Virtual Dimmer | [MA Parameters](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[Titan Personalities](https://manual.avolites.com/docs/fixture-personalities) |
| M04-09 | 地址重排 | Patch／Live Patch 修改可编辑地址项 | Repatch 保留编程；批量地址、交换地址与布局策略 | [MA Live Patch](https://help.malighting.com/grandMA3/2.5/HTML/patch_live.html)、[Titan Change](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-10 | 暂停物理配适 | 根据配适及输出功能分别处理 | Park 移出 DMX Map 但保留编排与原地址；不是 Freeze 输出 | [Titan Change](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-11 | 灯型交换 | Fixture Type／PSR 等支持替换和迁移工作流 | Fixture Exchange、函数映射、范围映射 | [MA PSR](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[Titan Change](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-12 | 属性修正 | DMX Invert、Encoder Invert、3D Invert、Pan／Tilt Offset 等 | Attribute Behaviour 的反向、冻结、曲线和限制 | [MA Live Patch](https://help.malighting.com/grandMA3/2.5/HTML/patch_live.html)、[Titan Change](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-13 | 空间／分类信息 | Stage、Layer、Class、位置、旋转、标签与外观 | 可视化位置、灯具标签、备注及颜色 | [MA Patch](https://help.malighting.com/grandMA3/2.5/HTML/patch.html)、[Titan Change](https://manual.avolites.com/docs/patching/changing-the-patch) |
| M04-14 | 库与已用版本 | 档案导入／导出、Show 内灯型数据分别处理 | 更新官方库不改变已配适档案；Update Personality 才升级已用版本 | [MA Types](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[Titan Personalities](https://manual.avolites.com/docs/fixture-personalities) |
| M04-15 | 通用档案救急 | 可创建灯型或采用相应通用灯型 | Generic Multi-DMX、RGB 等提供基本控制 | 同上 |
| M04-16 | RDM 发现与维护 | 支持发现、Get／Set、设备信息和地址等，受设备能力限制 | 配适 RDM 工作流及设备信息 | [MA RDM](https://help.malighting.com/grandMA3/2.5/HTML/rdm.html)、[Titan Add](https://manual.avolites.com/docs/patching/patching-new-fixtures-or-dimmers) |
| M04-17 | 灯具维护动作 | 通过灯型属性／功能和相应命令操作 | Fixture Macro 执行复位、灯泡等动作 | [MA Types](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[Titan Advanced](https://manual.avolites.com/docs/controlling-fixtures/advanced-options) |
| M04-18 | 复制灯具与节目 | 配适中复制／粘贴可克隆相关 Show 数据，离开配适时执行 | Copy 同时复制 Cue／Palette 数据，新灯具先处于 Park，需重新分配地址 | [MA Clone Patch](https://help.malighting.com/grandMA3/2.5/HTML/operate_clone_patch.html)、[Titan Copy](https://manual.avolites.com/docs/patching/copying-moving-and-deleting-fixtures) |
| M04-19 | 删除灯具的影响 | 按对象与依赖范围分别检查，本行不承诺删除可恢复 | 删除灯具会移除相关编程，手册说明不能撤销；在原 Handle 重新配适不恢复旧编程 | [Titan Delete](https://manual.avolites.com/docs/patching/copying-moving-and-deleting-fixtures) |

## 实时修改边界

MA Live Patch 只允许不需要重新上传 Show 的字段，不能在其中增删灯具，允许的修改立即应用。Titan Park 保留编排但移除输出映射，与冻结某属性的当前值不同。[MA Live Patch](https://help.malighting.com/grandMA3/2.5/HTML/patch_live.html)、[Titan Changing the Patch](https://manual.avolites.com/docs/patching/changing-the-patch)。

GDTF／MVR 的使用已经在 MA 官方手册中确认。当前这组 Titan 手册未建立与之等价的直接导入承诺；不能因为支持 Capture 交换就认定能导入任意 GDTF／MVR。

## 工作流程与 StageMaster 建议

自拟流程：配适普通 Dimmer、16-bit 摇头和多 cell 灯条；制造地址冲突、跨 Universe 边界，改地址、换模式、换灯型并更新档案，逐步核对组／素材／Cue 是否保持预期。特别检查某灯缺少颜色或快门功能时如何提示。

以下属于设计建议。Rust 核心拥有通用属性、物理范围、离散功能、粗细通道、几何层级及校准模型。档案解析器、配适服务和 DMX 编码器分别模块化；UI 只编辑和展示。

灯具库以独立版本发布，工程固定使用的版本。未来云端库更新先生成兼容性报告，避免自动替换正在运行的档案。基础单机要把配适冲突、模式错误、批量改址和通道观察做完整，RDM 与完整档案编辑器可分阶段实现。
