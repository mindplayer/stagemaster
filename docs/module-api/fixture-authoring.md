# 运行模块契约：灯具模式与配适

FIXTURE-002，依据 [ADR-024](../development/decisions/PRODUCT-ADR-024-fixture-authoring.md)。这是已运行接口；不替代 ADR-009 中完整灯具能力设计。

`stagemaster-project::fixture` 维护定义与实例绑定；`Document::edit(EditCommand::Fixture { command })` 原子校验；Session 提供版本检查／一次历史／预览失效。UI 模式草稿、地址建议和搜索均无真实输出权限。编排编译和 DMX 编码仍用既有 Rust 编译器与 stagemaster-dmx，UE 消费语义光值，不读物理通道顺序。

```ts
interface ProfileDefinition {
  name: string; manufacturer: string; model: string; mode: string;
  footprint: number; // 1–512，包含空余通道
  channels: {
    attribute: "dimmer" | "red" | "green" | "blue";
    coarse: number; // 从 1 起
    fine: number | null; // 从 1 起，非相邻和细调在前均可
    defaultValue: number; // 0–65535
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

- 允许调光、完整 RGB、调光加 RGB 三种属性集合；属性、粗细通道不可重复，通道不可超出 footprint。仅全范围线性映射；空余输出 0。
- 粗细不是地址先后顺序。`coarse:5,fine:1` 存为 `encoding:"u16-be",offsets:[4,0]`；例如 0x1234 在相对通道 5 输出 0x12、相对通道 1 输出 0x34。8 位沿用既有归一化转换。
- 界面百分比草稿保留六位小数，可无损往返 16 位默认值；核心继续接收整数，界面不参与逐帧求值。
- 新模式 id=null 生成独立 id/revision；原位修改仅允许未使用模式并生成新 revision。复制通过新建命令实现，不继承身份。删除被引用模式拒绝；可撤销。
- 换模式必须保持相同受支持属性集合、数据类型与混合方式。保持灯具 id、域、名称及全部编排和空间引用。新默认值可能改变未记录／释放的结果，界面明确说明。
- layout=null 保留地址；给出 layout 才按 fixtureIds 顺序重新配适。1–256 个唯一灯具，线路 1–65535、地址 1–512、gap 0–511；不跨线路自动溢出。混合宽度按各灯实际 footprint 计算，间隔不归灯具占用。整批完成后检验冲突，允许在所选灯原占用内交换位置，失败不修改任何对象。
- `ProjectView.profiles` 新增 revision/manufacturer/model/mode/channels/authorable；`FixtureView.profileId` 表明准确引用。authorable=false 的历史线性档案保留，不由当前编辑器改写。
- 文件复用现有 Profile/Fixture/Patch；无 Schema、capability、帧协议或授权格式变更。三原色无物理调光的预演用强度 1 和实际 RGB，不合成虚拟调光。

入口为“灯具 → 灯具模式库／替换模式／批量配适”。通道占用图按输出域和线路显示整段 footprint，包含未映射但被灯具占用的通道。地址建议仅为草稿，最终以 Rust 事务为准。模式草稿纳入顶栏保存、导航与关闭检查；取消不写入历史。

边界：仅工程内灯库，尚无跨工程个人库或 GDTF/OFL 文件导入；没有范围功能、快门、复位、轮盘、虚拟调光、多单元或关节运动。离线播放器仍限单输出域单线路；配适编辑支持其他线路不等同多线路播放已实现。无实灯授权或设备输出。
