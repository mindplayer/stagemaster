# 现场属性贡献

MIX-001／[ADR-104](../development/decisions/PRODUCT-ADR-104-live-attribute-contributions.md)。`stagemaster_engine::live::LiveMixer` 是标准系统宿主的有界属性合成器；`Document::compile_live_scene` 使用既有 Player 准备动态保持场景，`LiveOutput` 使用既有包编码器。当前为软件模块和组合验收，尚未接桌面、HOST-002 或 ESP32；多步骤列表仍走原单列表路径。

## 对象和实际调用

| 对象／入口 | 所有权与结果 |
| --- | --- |
| `Layout` | 完整编译快照指纹、按现有输出顺序排列的默认值／HTP／LTP／亮度／离散策略；构造后不可变 |
| `LiveMixer::new(fresh_boot, layout, source_capacity)` | 准备固定来源槽和属性缓冲；每次重建使用新启动身份 |
| `open(source, priority, layout_id)` | 注册可信来源，返回进程内 Handle；ID 非零且活动范围内唯一，初始无属性贡献 |
| `publish(handle, Frame { layout, serial, values, assert })` | 原子替换该来源完整属性贡献；None 为不控制／释放，Some(0) 为明确零；逐属性 assert 表示明确接管 |
| `set_level(handle, serial, level)` | 仅缩放布局已识别亮度，不修改源值、位置、颜色、功能或 LTP 次序 |
| `close(handle, serial)` | 立即撤掉该来源所有贡献，剩余来源接续；槽可复用，旧 Handle 永久失效 |
| `render(values, winners)` | 无分配合成；返回每属性数值及胜出来源，形状错误不改缓冲 |
| `sources()`／`source(handle)` | 查看活动业务身份、类别、优先级、电平与最后序号，不复制整份属性数据 |
| `LiveScenePlayer` | 拥有一个真实 Player、编译场景身份／修订、稀疏所有权与发布缓冲；不另写时间引擎 |
| `LiveScenePlayer::prepare_output()` | 在调度外准备绑定同一快照的 LiveOutput 与诊断缓冲 |
| `LiveOutput::render(mixer, slots)` | 验证工程指纹、合成后编码，成功返回逻辑线路；不同快照拒绝且不改槽数据 |

可运行组合见[工程验收](../../crates/stagemaster-project/tests/live/composition.rs)和[软件输出端口](../../crates/stagemaster-project/tests/live/port.rs)。核心关系如下：

```rust,ignore
let mut background = document.compile_live_scene(background_id, now_ms)?;
let mut accent = document.compile_live_scene(accent_id, now_ms)?;
let mut output = background.prepare_output()?;
let mut mixer = LiveMixer::new(fresh_boot, background.layout().clone(), 3)?;
let background_handle = mixer.open(background_source, 0, background.layout().id())?;
let accent_handle = mixer.open(accent_source, 0, accent.layout().id())?;
background.start(now_ms)?;
accent.start(now_ms)?;
// 由唯一可信宿主分配每来源命令序号；周期采样与推子命令共用同一序列。
background.tick(now_ms)?;
background.publish(&mut mixer, background_handle, background_serial)?;
accent.tick(now_ms)?;
accent.publish(&mut mixer, accent_handle, accent_serial)?;
let universe = output.render(&mixer, &mut slots)?;
// 仅成功生成的完整帧可提交 OUTPUT-001 的 Composite 许可；这不是物理发送确认。
```

## 控制语义

普通模式明确选择如下规则，不等同两家控台的所有可配置模式：

- HTP 只在最高有效优先级内取高；LTP 在最高优先级内取最近明确接管的属性。相同值也可明确重新接管；顺序由核心分配，不依赖网络到达壁钟或页面绘制顺序。
- 首次出现某属性自动取得一次顺序；连续采样、改变推子、暂停／恢复不重新抢占。场景 `start` 会重新声明其拥有的属性，失败发布保留该意图至下一次成功；上层不能在每帧调用 start。
- 优先级是来源配置，`Kind` 只记录播放、编程器或外部角色，不自行授权。手动来源可只控制一个属性，释放该属性后恢复仍在运行的下层值，不恢复一张过时截图。
- 推子归零保持控制权，普通亮度贡献为零；显式 close 或发布 None 才归还。自动关闭、全属性交叉、释放渐变和释放遮罩是后续独立模式。
- 离散功能只择完整值；本模式拒绝离散 HTP／亮度缩放策略。场景值来自现有灯具／功能编译器；手动和外部输入仍须在可信宿主进行对象、功能和权限校验，u16 接口本身不能证明值属于某个合法功能。
- 无任何来源控制时才使用档案默认值。默认值不是 HTP 最低值，也不等于全零、黑场或机械安全状态。

当前场景所有权取决于实际赋值与启用的效果：显式 set／预设解析结果和效果属性参与，remove／release 不参与，停用效果不凭默认值取得属性。新建场景目前会写入默认赋值，这些仍是显式内容，不能凭“与默认值相等”推断未编排；需要局部层时通过已有删除属性操作去掉其不应控制的值。本项不改变现有新建／保存交互或持久格式。

场景 stop 后 Player 仍恢复档案默认值，但其下一次成功发布全部为 None；不把默认值当作该场景继续覆盖的贡献。pause 保留数值与所有权，resume 继续原效果时钟。发布失败不修改混合器已接纳状态，可信宿主须处理错误／来源生命周期；此模块不自动证明后台发布者健康，也不取代 OUTPUT-001 来源新鲜度与物理驱动看门狗。

## 身份、预算与故障

布局身份用现有 SHA-256 对完整 `Document::encode()` 快照计算一次，包含实际内容。它用于防错配，不是签名或商业授权。保存修订相同但内容已经编辑时也会拒绝；旧准备组继续使用旧快照，新的编辑版本须明确重新准备／替换，不隐式拼接。

准备范围为 1～512 属性和显式 1～64 来源槽，这是当前实现的有界宿主预算，不是产品最终上限或 MCU 适配承诺。固定来源槽和属性数组一次分配；周期操作无 Vec 增长、排序、I/O 或完整 trace 构建。`reserved_bytes()` 报告预留向量载荷，排除分配器元数据、Player／工程／输出编码器；不能拿这个值代表整机总内存。测试核对一万次采样下容量／缓冲地址稳定，周期无分配结论同时来自代码路径审查，未冒称已测量整机分配器。

来源命令序号非零严格递增，允许跳号，所有来源变更由单一宿主协调；最终 u64 序号保留给 close，发布／电平不能耗尽释放路径。属性接管顺序与句柄代际同样不回绕，耗尽时拒绝新的取得／断言，已有贡献仍可显式释放。错误来源／布局／长度、旧命令和对 None 的接管声明在写入前拒绝，不部分消费批次。

输出值和 winners 只在成功 render 后读取，是历史逻辑结果，不是灯具反馈；初次 render 前的初始化缓冲不构成结果。`LiveOutput` 返回错误时不得把旧槽缓冲换一个新时间戳继续发送。HTP 胜出来源只是所选值的诊断，不替代完整贡献链；来源到步骤／预设的完整诊断、多步骤跟踪／延时／释放包络、混合多时钟和专业现场模式仍待后续验证。

## 参考边界

复用 domain 固定点计算、已有 Player／物理运动效果和包输出编码；A0 Mixer 保留作参考与旧入口。新默认模式在满电平的连续值子集上，与参考混合器用 448 组优先级／零值组合对照；不把旧的“全属性权重、零即释放”照搬成普通推子语义。成熟参考与取舍记录在 ADR，不宣称完整兼容 MA 或 Titan。
