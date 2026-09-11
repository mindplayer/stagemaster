# 像素映射、视频与媒体协作

像素映射把二维内容采样成灯具属性；媒体服务器则负责视频素材和屏幕输出。两者可以协作，但不应把控台可预览视频理解为它具备完整视频服务器能力。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M12-01 | 映射坐标 | Bitmap Canvas 映射 Selection Grid | Pixel Mapper 使用 Group Layout | [MA Bitmap](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[Titan Pixel](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M12-02 | 内容来源 | 图像、Gobo、Symbol、视频；支持 NDI 输入相关工作流 | 几何元素、文字、手绘、位图和 Synergy 内容 | 同上 |
| M12-03 | 内容变换 | 平移、缩放、宽高比、旋转、Clip／Wrap、透明度相关选项 | 元素／层／效果的变换、颜色和不透明度 | 同上 |
| M12-04 | 参数映射 | Bitmap Channel 将采样来源映射到属性上下界，不限于 RGB | 以灯具为像素输出二维效果，具体行为取决于灯具档案 | 同上 |
| M12-05 | 动画构建 | 通过视频、控制灯具及 Master 改变 Bitmap 行为 | Rotate、Slide、Zoom、Random、渐变、拖尾等动画 | 同上 |
| M12-06 | 分层及生成 | 不把多条 Bitmap Configuration 当作同时混合的视频层 | 效果由层、元素、动画组合；有生成数量和结束动作 | 同上 |
| M12-07 | 现场控制 | Bitmap Control Fixture 可将参数存为 Cue／Preset | Layer Masters、效果速度和不透明度等可现场控制 | 同上 |
| M12-08 | 预览与抑制 | 布局／选择网格辅助定位与预览 | Pixel Mapper Preview、Mask FX、Pre-Spool | 同上 |
| M12-09 | 媒体控制 | 灯具属性模型与媒体相关资源参与节目 | Synergy 控制 Ai／Prism，媒体属性可存 Palette／Cue | [MA Videos](https://help.malighting.com/grandMA3/2.5/HTML/videos.html)、[Titan Synergy](https://manual.avolites.com/docs/synergy) |
| M12-10 | 媒体管理 | 本次未确认与 Synergy 上传／转码同等的服务器管理功能 | Media Browser 上传素材，按目标处理转码及进度 | [Titan Operating Synergy](https://manual.avolites.com/docs/synergy/operating-synergy) |
| M12-11 | 灯光与屏幕内容统一 | Bitmap 采样外部媒体输入的工作流 | Lightmap 从 Ai／Prism 层或合成输出映射到灯具 | [MA Bitmap](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)、[Titan Synergy](https://manual.avolites.com/docs/synergy/operating-synergy) |
| M12-12 | 外部视频监看 | 依据输入及视频功能分别配置 | Multi View／Overlay 的 RTSP 能力按 19.2 和硬件条件确认 | [Titan 19.2](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf) |

## 边界与工作流程

MA Bitmap 文档提示媒体分辨率和帧率限制，多计算站使用 NDI 时每个计算站都需要获得输入。这里是该产品文档给出的约束，不能据此得出所有 DMX 或我们引擎都固定采用同一帧率。[MA Bitmap](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html)。

自拟流程：用不规则灯阵映射一条移动色带，核查旋转、缺灯、cell 顺序、画布缩放和不同灯型颜色转换；再加入一段外部视频，模拟输入中断、跳播和媒体缺失。对 Titan 同时核查媒体内容版本和 19.2 的视频预览恢复条件，不能仅看标为 19.0 的手册正文。[Titan Operating Synergy](https://manual.avolites.com/docs/synergy/operating-synergy)、[Titan 19.2](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf)。

## 媒体格式与素材交付

MA 官方支持格式页明确列出 WAV／MP3／OGG 音频、JPG／PNG 图片、VP8／VP9 WebM 视频以及 3DS／glTF／GLB 网格。媒体能导入、能预览、能在全部输出站点参与实时计算是不同条件；Titan 的 Pixel Mapper、Capture 和 Synergy 也需要分别遵守各自素材条件。这里不把两家的所有媒体格式合并成一个无条件支持列表。[MA Supported File Formats](https://help.malighting.com/grandMA3/2.5/HTML/file_formats.html)、[Titan Pixel Mapper](https://manual.avolites.com/docs/effects/pixel-mapper)、[Titan Capture Files](https://manual.avolites.com/docs/capture-visualiser/capture-show-files)。

## 对 StageMaster 的吸收建议

以下属于设计建议。先做可复用的二维布局、简单图形／渐变和像素采样；视频解码及网络视频放入独立媒体服务，通过有界缓冲传递采样结果，避免让解码阻塞灯光执行循环。

云端保存媒体元数据和对象存储地址，发布包锁定媒体内容哈希。桌面可预览高质量视频，低算力盒子使用离线转换结果或明确降低能力。视频传输带宽与灯光输出应分别计量和限流。
