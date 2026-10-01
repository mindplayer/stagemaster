# ADR-072：灯光片段停用与恢复

状态：采纳；2026-10-01；AUDIO-008；基线 `2a5a9f4`，当前会话独立评估。

参考 [Premiere Enable / Disable](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/enable-or-disable-a-clip.html) 的保留片段、选择多个后统一停用／启用机制。本项目没有多层视频覆盖，明确停用区间为灯具默认值空隙，不延长前段、不把默认值称为全黑；状态不是播放器临时熄灯或音频静音。

`AudioLightingClip.enabled?: boolean`：省略／true 为原行为，序列化 true 省略，false 要求新能力 `media.audio-clip-state@1`。首次停用由编辑事务添加能力，恢复全部仍保留声明，清除音乐移除；能力必须有独立片段轨道，新旧版本／非法类型由原框架检查。单轨容量、占位／非重叠、引用完整性对停用片段仍有效，保证恢复时不会隐藏冲突。

`LightingClipGroupAction::Enabled {enabled}` 在原 EditLightingClips 上拓展；单片段也用一元素组，语义与校验唯一。含锁定项整组拒绝，不能通过普通 PutLightingClip 改启停，复制保持源状态并解除锁定。纯改状态保留全部字段及身份，一次历史；相同值无历史。范围与命令严格校验，无旁路。

`lighting_at(t)` 跳过 disabled 片段，空隙返回 None；直接编译指定 disabled ID 拒绝。编译进入渐变仅使用紧邻且启用的前段，停用前段等同默认值空隙。原生音乐游标／版本缓存机制不变，暂停或播放中编辑后重新求值；没有自动延续旧场景、第二播放器或 TS 输出求值。

界面目录、时间线和属性明确“已停用”，使用纹理／文字与状态筛选，仍可选中／移动／复制。单片段有停用／恢复；整组明确两按钮而非混合状态 toggle，显示停用数，锁定说明同步。源参数草稿先校验，保存不隐式切换启停。停用片段的预演入口以“从此位置预演”表述，播放实际默认值结果，关联场景仍可编辑且影响其他引用。
