# ADR-067：音乐上的独立灯光片段

状态：采纳；2026-10-01；AUDIO-006；当前会话评估。基线 `5933e5d`。

问题：旧模式把已绑定卡点视为持续到下一绑定点的场景，删除／移动一个点会改变相邻段落；成组卡点操作不能代替独立片段。增加视频等轨道前先拆开节奏标记与灯光持续区间。

参考 [Premiere 非破坏性时间线编辑](https://helpx.adobe.com/ca/premiere/desktop/edit-projects/intro-to-editing/edit-video-in-premiere.html)、[移动片段](https://helpx.adobe.com/au/premiere/desktop/edit-projects/change-clip-sequence/different-ways-to-move-clips.html)、[删除与波纹删除](https://helpx.adobe.com/ca/premiere/desktop/edit-projects/change-clip-sequence/remove-clips-from-a-sequence.html)：片段独立开始／结束、普通编辑保留空隙，波纹属于显式不同操作。此项目保留单一宿主音频和独立 Rust 求值；首步只实现单灯光轨、拒绝重叠，不复制视频混合／波纹与多轨行为。

## 数据与兼容

`AudioTimeline.lightingClips?: AudioLightingClip[]`，存在（包括空数组）表示独立片段模式；省略继续逐字保持既有卡点绑定语义。新能力 `media.audio-clips@1` 必须与字段一起出现。片段包含 UUID `id`、`name`、`sceneId`、`startMs`、`endMs`、`fadeMs`、`locked`，开始／结束是裁切后整数毫秒，区间左闭右开，严格正长、排序、互不重叠，最多 512 个，渐变不大于长度。引用独立场景，未内嵌重复场景数据。

显式 `convertLightingClips` 一次事务把已绑定卡点的既有持续区间转为新 UUID 片段，同时保留原节奏点身份／名称／时间、清除其场景／渐变。可撤销；不在打开旧文件时自动转换。转换后卡点只能作节奏标记，不允许两条调度路径同时存在。两种模式在音乐有效范围内转换前后输出等价；新模式在音乐结束处按半开区间回到默认值，旧模式终点行为保持兼容。

`AudioEdit` 新增 putLightingClip／copyLightingClip（源 ID、目标开始）／removeLightingClip／setLightingClipLock。创建／复制 ID 由核心生成（创建通过专用 addLightingClip），复制保留时长和进入渐变，副本解除锁定。任何越界、重叠、失效引用、超容量或锁定修改原子拒绝；复制锁定来源允许。移动和修改结束不会暗改邻段、卡点或音乐。音频裁切不得使片段越界，即使片段未锁；移除整个音乐仍显式确认整个时间线删除。

## 执行

`AudioTimeline.lighting_at` 返回统一只读引用（ID／场景／起点），由文档编译选择旧卡点或新片段，宿主缓存按工程版本、片段身份和起点失效。空隙采用灯具档案默认值；默认值不等于全黑，不伪称应急熄灯。新片段效果从片段开始重启，改变左边界也重设计时；首版没有源偏移或保持相位的无损裁切，不提供会错误重启的“分割”。

渐变沿用 ADR-061：只有前片段恰好在此处结束，才采样其末端状态作为进入起点；有空隙则从默认值进入。旧效果在边界冻结，新效果持续运行；功能属性直接切换。任意定位／回退／局部循环与顺序求值一致，最多编译两个场景，不扫描累计帧。

## 界面与状态

波形／宿主音频／唯一三维共用原有会话；片段专用选择和属性表单、精确开始／结束／渐变、添加／复制／删除／锁定和横向操作使用独立模块。沿用统一未应用草稿拦截、撤销／重做／保存、错误定位和取消；节奏点与片段选择互斥，搜索不移动播放头。时间线显示真实空隙。转换按钮说明会保留节奏点，转换可撤销；片段可独立选择、移动和长度调整。

边界：单轨、无重叠混合、无波纹、多轨音视频、正式设备下发、跨设备时钟和真实 DMX。本能力只改变主机编排／预演，不自动操作硬件。
