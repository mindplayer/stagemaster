# 运行模块契约：灯具模式与配适

FIXTURE-002／FIXTURE-003B，依据 [ADR-024](../development/decisions/PRODUCT-ADR-024-fixture-authoring.md)。这是已运行接口；不替代 ADR-009 中完整灯具能力设计。

`stagemaster-project::fixture` 维护定义与实例绑定；`Document::edit(EditCommand::Fixture { command })` 原子校验；Session 提供版本检查／一次历史／预览失效。UI 模式草稿、地址建议和搜索均无真实输出权限。编排编译和 DMX 编码仍用既有 Rust 编译器与 stagemaster-dmx，UE 消费语义光值，不读物理通道顺序。

```ts
interface ProfileDefinition {
  name: string; manufacturer: string; model: string; mode: string;
  positioning?: PositionModel; // 见位置求解契约，缺省为固定灯
  footprint: number; // 1–512，包含空余通道
  channels: {
    attribute: "dimmer" | "red" | "green" | "blue" | "pan" | "tilt" | "zoom" | "focus" | "iris" | "color-wheel" | "gobo-wheel" | "shutter" | "prism";
    coarse: number; // 从 1 起
    fine: number | null; // 从 1 起，非相邻和细调在前均可
    defaultValue: number | { functionKey: string; position: number }; // 连续值或功能选择
    functions?: { key: string; name: string; mode: "slot" | "range"; dmxFrom: number; dmxTo: number; dmxDefault: number }[];
  }[];
}
type FixtureCommand =
  | { op: "saveProfile"; id: string | null; definition: ProfileDefinition }
  | { op: "removeProfile"; id: string }
  | { op: "repatch"; fixtureIds: string[]; layout: Repatch }
  | { op: "exchange"; fixtureIds: string[]; profileId: string; layout: Repatch | null };
interface Repatch { universe: number; address: number; gap: number }
// 宿主发送：{kind:"edit",generation,command:{op:"fixture",command}}
```

- 允许调光、完整 RGB、调光加 RGB 三种基础属性集合，可另加成对 pan／tilt 和 positioning；属性、粗细通道不可重复，通道不可超出 footprint。基础属性为全范围线性映射；另支持四类离散功能通道，详见下文。空余输出 0。
- 粗细不是地址先后顺序。`coarse:5,fine:1` 存为 `encoding:"u16-be",offsets:[4,0]`；例如 0x1234 在相对通道 5 输出 0x12、相对通道 1 输出 0x34。8 位沿用既有归一化转换。
- 界面百分比草稿保留六位小数，可无损往返 16 位默认值；核心继续接收整数，界面不参与逐帧求值。
- 新模式 id=null 生成独立 id/revision；原位修改仅允许未使用模式并生成新 revision。复制通过新建命令实现，不继承身份。删除被引用模式拒绝；可撤销。
- 换模式必须保持相同受支持属性集合、数据类型与混合方式。POSITION-001 还要求运动定义完全相同，范围或反向改变时拒绝换灯。保持灯具 id、域、名称及全部编排和空间引用。新默认值可能改变未记录／释放的结果，界面明确说明。
- layout=null 保留地址；给出 layout 才按 fixtureIds 顺序重新配适。1–256 个唯一灯具，线路 1–65535、地址 1–512、gap 0–511；不跨线路自动溢出。混合宽度按各灯实际 footprint 计算，间隔不归灯具占用。整批完成后检验冲突，允许在所选灯原占用内交换位置，失败不修改任何对象。
- `ProjectView.profiles` 新增 revision/manufacturer/model/mode/channels/authorable；`FixtureView.profileId` 表明准确引用。authorable=false 的历史线性档案保留，不由当前编辑器改写。
- 文件复用现有 Profile/Fixture/Patch；FIXTURE-002 本身不改格式；POSITION-001 为可选两轴模型增加 `lighting.positioning@1` 和协议 2，见[位置接口](positioning.md)。授权格式不变。三原色无物理调光的预演用强度 1 和实际 RGB，不合成虚拟调光。

入口为“灯具 → 灯具模式库／替换模式／批量配适”。通道占用图按输出域和线路显示整段 footprint，包含未映射但被灯具占用的通道。地址建议仅为草稿，最终以 Rust 事务为准。模式草稿纳入顶栏保存、导航与关闭检查；取消不写入历史。

[FIXTURE-004 模式文件](profile-files.md)增加独立模式导入／导出与检查草稿；保存生成新身份，不替换已配灯具。

边界：工程内定义和可携带单模式文件已接通，尚无个人库目录服务或 GDTF/OFL 文件导入；已支持命名区间的色盘／图案盘／快门／棱镜建档与直接切换，尚无复位、虚拟调光、多单元及跨定义功能映射。相交正交两轴运动已由 POSITION-001 接入，复杂关节仍未支持。离线播放器仍限单输出域单线路；配适编辑支持其他线路不等同多线路播放已实现。无实灯授权或设备输出。

## 功能区间（FIXTURE-003B）

依 [ADR-060](../development/decisions/PRODUCT-ADR-060-fixture-function-ranges.md)，Rust `fixture_function` 负责有界表校验与整数转换，`fixture_value` 负责工程适配，`fixture_view`／`function_output` 负责只读投影。每个支持的功能属性最多一个物理通道（可含粗细），1–64 个不重叠区间；定义单位是原生 8／16 位。保留空隙不能被选择；固定档位使用代表值且 position 必须为 0；区间调节按 0–65535 比例映射，全部经过最终 Rust 校验。

- 格式能力 `lighting.fixture-functions@1`；属性及默认／场景／预设值的 kind 为 `function`。旧工程和值类型保持原样。
- 编辑命令 `{op:"setSceneFunctionValue",sceneId,fixtureId,attribute,selection:{functionKey,position}}` 纳入原子批次和历史；释放／清除沿用 setSceneValue 的 mode。普通数值不能写入功能属性。
- `FixtureView.attributes[].function` 包含 functions、default、fine；`SceneView.values[].functionValue` 保留已解析预设的语义选择。旧数值字段是编码后的输出监看值，不用于重新记录预设。
- 实际播放 `AttributeOutput.function` 给出 key、name、dmxValue 和按编码结果回算的区间位置（档位为 null）；量化后位置可能不同于原始编辑位置。没有第二个 TS 播放求值器。
- 功能属性编译为直接切换索引，跟随列表延时而不穿越中间区间，连续亮度／双轴仍渐变。动态函数叠加拒绝。带此类映射的包要求执行语义 2；旧文件和普通节目包仍保持原兼容性。
- 安全换灯要求功能表完全一致（功能身份、名称、区间、代表值、顺序）；物理粗细通道可变。多灯界面只显示定义一致的共同功能。跨厂商语义转换仍需将来的映射确认，不能根据中文名称猜测。
- 模式编辑拆为元信息、轴行程、物理映射、功能区间和分布组件；更换基础组合保留已配置的轴与功能。场景连续参数与功能参数分别显示，共用父级草稿、应用、取消及历史。

三维暂仅为这些灯具展示灯体和朝向，隐藏尚无模型的光束并列明受影响灯具；输出明细仍显示完整的实际通道。此边界见[预演接口](previsualization.md)，不等同真实光学验证。

## 镜头与光圈（FIXTURE-005）

依 [ADR-080](../development/decisions/PRODUCT-ADR-080-continuous-optics-controls.md)，`zoom`（变焦）、`focus`（调焦）、`iris`（光圈）各可独立增加一组全范围线性粗细通道；默认值必须为 0–65535 整数，不接受 functions 表。既有基础属性与成对位置轴要求不变。Rust `is_continuous_optics_attribute` 维护支持集合；界面独立建档区复用连续字段与草稿校验，切换基础组合保持镜头定义。

百分比仅代表通道控制位置，不推断光束角、焦距、开度或正反方向。场景、释放、预设及列表／音频段落渐变使用原有 normalized／LTP 管线；总控／熄灯不改变这些属性。快速预设增加“仅镜头与光圈”范围。可携带模式文件保持准确粗细、默认值及独立导入身份。

现有 lighting.basic 和通用播放包已承载这类值，不新增格式能力或执行版本；动态效果白名单未扩展。带宏／保留区间／非线性物理映射尚未建模。三维保留灯位并明确光学未模拟，不能把百分比显示当物理预演。
