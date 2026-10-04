# 运行模块契约：灯具模式与配适

FIXTURE-002／FIXTURE-003B，依据 [ADR-024](../development/decisions/PRODUCT-ADR-024-fixture-authoring.md)。这是已运行接口；不替代 ADR-009 中完整灯具能力设计。

`stagemaster-project::fixture` 维护定义与实例绑定；`Document::edit(EditCommand::Fixture { command })` 原子校验；Session 提供版本检查／一次历史／预览失效。UI 模式草稿、地址建议和搜索均无真实输出权限。编排编译和 DMX 编码仍用既有 Rust 编译器与 stagemaster-dmx，UE 消费语义光值，不读物理通道顺序。

```ts
interface ProfileDefinition {
  name: string; manufacturer: string; model: string; mode: string;
  positioning?: PositionModel; // 见位置求解契约；有轴但缺省时表示未定义物理模型
  emitters?: { key: string; name: string }[]; // FIXTURE-011，稳定单层光源身份
  footprint: number; // 1–512，包含空余通道
  channels: {
    attribute: string; // 既有属性，或 emitter.<稳定标识>.<连续属性>，严格集合见下文
    coarse: number; // 从 1 起
    fine: number | null; // 从 1 起，非相邻和细调在前均可
    defaultValue: number | { functionKey: string; position: number }; // 连续值或功能选择
    functions?: { key: string; name: string; mode: "slot" | "range"; dmxFrom: number; dmxTo: number; dmxDefault: number; appearance?: { kind: "open" } | { kind: "color"; colors: string[] } }[];
  }[];
}
type FixtureCommand =
  | { op: "saveProfile"; id: string | null; definition: ProfileDefinition }
  | { op: "removeProfile"; id: string }
  | { op: "repatch"; fixtureIds: string[]; layout: Repatch }
  | { op: "exchange"; fixtureIds: string[]; profileId: string; layout: Repatch | null; allowColorSlotRemap?: boolean };
interface Repatch { universe: number; address: number; gap: number }
// 宿主发送：{kind:"edit",generation,command:{op:"fixture",command}}
```

- 允许调光、完整 RGB、调光加 RGB 三种基础属性集合，可另加成对 pan／tilt，物理模型 positioning 可暂不定义；属性、粗细通道不可重复，通道不可超出 footprint。基础属性为全范围线性映射；另支持色盘／图案盘／快门／棱镜及独立内置程序功能通道，详见下文。空余输出 0。
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
- 显式换灯要求功能控制映射一致（功能身份、控制方式、区间、代表值）；FIXTURE-007 允许名称、外观及表行顺序不同，保留控制值而不保证实际颜色。物理粗细通道可变，按目标精度重新编码。多灯界面只显示定义一致的共同功能。跨厂商语义转换仍需将来的映射确认，不能根据中文名称猜测。
- 模式编辑拆为元信息、轴行程、物理映射、功能区间和分布组件；更换基础组合保留已配置的轴与功能。场景连续参数与功能参数分别显示，共用父级草稿、应用、取消及历史。

三维暂仅为这些灯具展示灯体和朝向，隐藏尚无模型的光束并列明受影响灯具；输出明细仍显示完整的实际通道。此边界见[预演接口](previsualization.md)，不等同真实光学验证。

## 镜头与光圈（FIXTURE-005）

依 [ADR-080](../development/decisions/PRODUCT-ADR-080-continuous-optics-controls.md)，`zoom`（变焦）、`focus`（调焦）、`iris`（光圈）各可独立增加一组全范围线性粗细通道；默认值必须为 0–65535 整数，不接受 functions 表。既有基础属性与成对位置轴要求不变。Rust `is_continuous_optics_attribute` 维护支持集合；界面独立建档区复用连续字段与草稿校验，切换基础组合保持镜头定义。

百分比仅代表通道控制位置，不推断光束角、焦距、开度或正反方向。场景、释放、预设及列表／音频段落渐变使用原有 normalized／LTP 管线；总控／熄灯不改变这些属性。快速预设增加“仅镜头与光圈”范围。可携带模式文件保持准确粗细、默认值及独立导入身份。

现有 lighting.basic 和通用播放包已承载这类值，不新增格式能力或执行版本；动态效果白名单未扩展。带宏／保留区间／非线性物理映射尚未建模。三维保留灯位并明确光学未模拟，不能把百分比显示当物理预演。

## 通道与物理模型分开确认（FIXTURE-006）

依 [ADR-087](../development/decisions/PRODUCT-ADR-087-fixture-mapping-without-geometry.md)，有两轴通道但没有物理模型的模式可建档、编排和无损导入导出，角度／指向／翻转／零偏操作拒绝。模型存在时仍必须具备完整两轴；旧工程和包语义不变。界面“定义轴行程与方向”独立于“两轴摇头灯”，新角度字段留空；取消保留原档案。已布置且无物理模型的灯具在三维预演时给出具名错误，不猜测几何。

来自[实际 18／11 通道资料](../fixtures/user-supplied/README.md)的已知行程、未知零位、颜色档位及控制宏分别记录。本增量没有把未支持的多个光源或持时命令发布为可执行模式。


## 自定义色盘外观（FIXTURE-007）

依 [ADR-088](../development/decisions/PRODUCT-ADR-088-custom-wheel-appearances.md)，`fixture_appearance` 管外观严格校验和能力声明；`appearance` 仅用于 `color-wheel` 的固定档位，缺省=未标记，`open`=通光，`color.colors` 为 1–2 个 `#RRGGBB` 屏幕近似色（单色／半色）。禁止把自动换色区间标成固定色块；图案资源和物理色度不在此字段里混用。

带外观工程需要 `lighting.fixture-wheel-appearance@1`，建档命令自动声明；独立模式文件保留全部外观，导入生成独立模式，旧读取器严格拒绝新字段。执行包保持原有语义，色块不进入 DMX 映射，不增加 ESP32 显示数据负担。

UI `planSlotBatch` 预检原生起点／宽度／数量、64 功能上限和重叠；`addSlotBatch` 仅显式加入草稿，可替换唯一未编辑的初始空白行并保持默认身份，其余功能与默认选择保留。复制模式保持功能键，保存产生独立模式身份，再经 `exchange` 指定目标灯具。外观差异使多灯共同功能不再合并显示；复制控制值或模式替换不是跨色盘颜色匹配。

## 固定色盘值的显式迁移（FIXTURE-008）

依 [ADR-089](../development/decisions/PRODUCT-ADR-089-color-slot-remap-review.md)，`allowColorSlotRemap` 缺省 false。true 仅放宽相同功能键、同为固定档位的色盘起止值／代表值差异，允许编译原场景与预设引用时使用新值。自动换色等连续区间、功能键集合／类型、其他属性和物理模型仍须相容；整批只改变目标灯具的模式引用，失败无部分提交。

桌面按源模式列出所选灯及色盘前后差异，需重新编码时必须显式勾选；更换选择／目标／源目标修订会撤销该选择。取消不改工程，一次应用对应一次历史；不隐式重写功能键，不推测实际颜色或自动反转通道。

## 两轴速度控制（FIXTURE-009）

依 [ADR-158](../development/decisions/PRODUCT-ADR-158-pan-tilt-speed-control.md)，建档新增 `pan-tilt-speed`，中文“两轴速度控制”，须同时具备完整 pan／tilt 位置通道，每模式最多一组共同速度通道。仅接受全范围线性 8／16 位粗细映射、normalized 默认值及 LTP；不接受功能表。通道控制位置并非真实速度、速度方向、角速度或软件渐变时长；不自动推断或反转未知厂家方向。缺物理模型仍可编排通道，原物理位置操作拒绝规则保持。

界面独立速度组件复用线性字段，新增默认值草稿留空，必须明确填写；切换基础组合／几何保持速度，移除两轴一并移除依赖速度草稿并有说明，取消恢复原定义。场景归入“控制”，预设“仅位置”仍只选 pan／tilt，“仅两轴速度控制”只选本属性；原场景、释放、稀疏预设、列表渐变、模式文件／工程保存及批量替换复用，不增加 UI 播放求值器。总亮度／熄灯不衰减此控制值，动态效果白名单不扩展。

既有 `lighting.basic` 和通用 normalized 包完整承载该映射，不改工程结构、能力／执行版本、时钟或输出权限。旧执行读取与旧建档支持不同，旧编辑器可能拒绝新组合；同属性换灯只保证控制值保留／按精度重新编码，不保证实灯真实速度等效。UE 不由此模拟机械延迟。FIXTURE-010 将自动／声控区间作为禁用资料接入，禁止用于演出；混合宏、复位持时、分轴速度和多发光单元仍未接入，软件测试档案不是完整实际 11／18CH 模式。

## 内置程序（FIXTURE-010）

依 [ADR-159 顶部用户安全政策](../development/decisions/PRODUCT-ADR-159-discrete-fixture-programs.md)，`fixture-program`（内置程序）是独立 LTP 功能属性，不是软件动态效果、普通百分比或定时命令。每模式最多一组 8／16 位程序通道，复用原通道表／场景／稀疏预设／直接切换编译。必须有 `external` 外部通道控制档位；`auto.<标识>`、`sound.<标识>` 仅保留说明书禁用区间资料，全部为 `slot`。**唯一可执行选择是 external/0；声控和内置自走均屏蔽**，普通／整批编辑、预设解析、工程读入、编译、手动输出及手动记录场景统一拒绝，不静默删除已有禁止内容。手动输出复用已准备的灯具映射，不允许普通数值绕过；external/0 仍持有属性，None 仍释放。区间与代表值须由说明书明确填写，不猜测默认 DMX、轨迹或覆盖范围；未知控制宏拒绝。

工程需同时声明 `lighting.fixture-functions@1` 与 `lighting.fixture-programs@1`，建档自动加入，新旧读取器按能力边界拒绝不支持内容；无新结构字段或封装版本。独立模式文件保留程序键和原生值，导入仍生成新身份／修订。已有执行语义 2 包表达延时后直接切换，旧语义 1 目标拒绝；ESP32 仍仅消费有界编译节目，不加载完整工程。未知未来程序能力版本拒绝。

界面独立程序建档区禁止删除外部控制档位、禁止切换成动态区间；未知原生区间留空。登记自走／声控禁用区间只是档案资料，场景选择器不显示这些档位，TS 命令预检也拒绝。场景归入“控制”，“仅内置程序”预设只可记录外部控制且独立于亮度／位置／速度；复制与基础组合保持定义。多灯只提供共同定义，换模式需控制表相容，整批错误无部分写入；不按中文名称猜测跨厂商语义。

总亮度／熄灯不改合法外部程序控制值，也不是机械急停。释放／清除遵循既有下层与默认规则，不能当作复位。三维按既有未建模功能边界保留灯体、隐藏不能准确模拟的光，不生成内置自动／声控轨迹。30W 第 10 通道已知九档仅作软件资料／拒绝验收；第 11 通道复位持时、18CH 混合控制／锁存、多发光单元及物理测量仍未实施，复位不列为普通场景功能。当前保护依赖明确分类的工程／档案，不能从无完整语义的历史裸播放包逆推厂家控制模式；不声称已检查或改写它们，也不把软件保护当作实灯验证。

## 独立光源连续控制（FIXTURE-011）

依 [ADR-160](../development/decisions/PRODUCT-ADR-160-independent-emitter-controls.md)，Profile／建档定义的可选 emitters 声明 1–32 个单层光源，各 key 唯一且长度 1–32，符合 `[a-z][a-z0-9]*(-[a-z0-9]+)*`；name 为 1–64 个字符。每个单元必须有准确 `emitter.<key>.<属性>` 通道；支持 dimmer、完整 RGB 或 RGBW，后两者可另带单元 dimmer。白光只允许完整 RGB 的单元额外声明，不是根级或虚拟混色值。本增量所有单元属性为 normalized 全范围 0–65535；dimmer 用 HTP、RGBW 用 LTP，不支持单元功能通道、轴、自动程序或控制宏。未声明／未知单元、空单元和属性／通道冲突原子拒绝。

带单元模式根级可有真实 dimmer 和既有位置／控制／功能／镜头属性，不允许根级 RGB 混用；没有实际总调光可省略根级 dimmer。工程必须声明 `lighting.fixture-emitters@1`，自动建档和严格读入共用规则。旧无字段工程不变；旧读取器拒绝新字段／能力。ProfileView 保留可选 emitters，FixtureView.attributes[].label 包含中文光源名，所有原目标／编辑／预设／手动仍携带准确完整属性键。

OutputMaster 有根级调光只缩放该通道一次；没有根级调光时分别缩放单元 dimmer，否则该单元 RGBW。没有白光模拟或虚构总调光。数值播放包、既有强度掩码、执行器预算与版本保持；不将完整工程交给 ESP32。准确属性集合／类型／混合和原功能条件不一致时禁止批量换模式，光源显示改名不改归属，稳定 key 改动不能迁移旧编排。模式文件及复制保留定义／生成新模式身份。

界面独立建档区自动生成只读稳定 key，名称可改，增删和组合只改草稿，新属性默认必填；粗细／默认精确值、取消、错误定位、历史和保存复用原入口。参数按基础属性分类，稀疏预设保留所选准确单元键，不扩大为全部属性。三维保留灯体／姿态，`unmodeled-emitters` 且编辑／播放光束强度为 0，提示未模拟，不影响实际 DMX。**有限连续单元已接入，独立频闪／轮盘、联动和物理光学未接入**；上文历史无多单元边界以此段为当前接续，不代表完整 18CH 实灯模式。
