# 预览、盲编、3D 与舞台布局

二维布局解决选灯和操作组织，三维舞台解决空间关系与可视化，Blind／Preview 解决编辑是否进入现场。它们互有关联，但不能用一个“预览模式”包办全部职责。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M14-01 | 编程器盲编 | Blind 隔离编程器的输出 | Blind 编程；可用 Blind to Live 过渡 | [MA Programmer](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[Titan Palette](https://manual.avolites.com/docs/palettes/using-palettes) |
| M14-02 | 独立节目预览 | Preview 环境可载入序列，切 Cue 和播放 | 单 Playback 可设 Blind，仅输出至 Visualiser | [MA Preview](https://help.malighting.com/grandMA3/2.5/HTML/preview.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M14-03 | 多项预备后过渡 | Preview 与 Live 的环境切换和复制工具 | Scene Master 先组合多项变化，再用推杆送入现场 | [MA Preview](https://help.malighting.com/grandMA3/2.5/HTML/preview.html)、[Titan Scene Master](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M14-04 | 预览界面提示 | 相关窗口显示预览边框，3D／DMX 可设是否跟随 | Blind、Scene Master 的状态提示与可视化 | 同上 |
| M14-05 | 内置可视化 | 3D Viewer 显示灯具、场景、光束及雾效 | 内置 Capture Visualiser，与配适和节目结合 | [MA 3D](https://help.malighting.com/grandMA3/2.5/HTML/patch_3d_viewer.html)、[Titan Rig](https://manual.avolites.com/docs/capture-visualiser/setting-up-the-rig) |
| M14-06 | 舞台物体与坐标 | Stages、灯具位置／旋转、网格／Mesh 等 | 舞台地面、墙及附加物体、灯具位置／姿态 | 同上 |
| M14-07 | 摄像机与视角 | Camera Pool、移动／环绕／适配、多个 Viewer | 摄像机、单视图／四分屏、平移／旋转／环绕 | 同上 |
| M14-08 | 渲染性能设置 | Render Quality 和光束／表面等显示设置 | Capture 外观与质量；Throws Light 可减少渲染负担 | [MA Render](https://help.malighting.com/grandMA3/2.5/HTML/patch_render_quality.html)、[Titan Rig](https://manual.avolites.com/docs/capture-visualiser/setting-up-the-rig) |
| M14-09 | 在场景里选灯 | 3D 可选择灯具，视角投影可生成选择网格 | 在控台选择灯具并编辑可视化位置；具体选择入口依窗口 | [MA 3D](https://help.malighting.com/grandMA3/2.5/HTML/patch_3d_viewer.html)、[Titan Rig](https://manual.avolites.com/docs/capture-visualiser/setting-up-the-rig) |
| M14-10 | 世界坐标编程 | XYZ 可与 Pan／Tilt 结合，MArker 定义目标空间 | 本次未确认同语义世界坐标引擎，不把 3D 摆灯当作 XYZ 编程 | [MA XYZ](https://help.malighting.com/grandMA3/2.5/HTML/xyz.html) |
| M14-11 | 场景交换 | MVR／MVR-xchange 关联舞台与灯具资料 | Capture 场景随 Show 保存，也可单独导入导出 | [MA MVR](https://help.malighting.com/grandMA3/2.5/HTML/patch_mvr.html)、[Titan Capture Files](https://manual.avolites.com/docs/capture-visualiser/capture-show-files) |
| M14-12 | 外部可视化连接 | 节点／协议／viz-key 等需结合授权核对 | Stand-alone Capture 有专门连接流程 | [MA Parameters](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[Titan External Capture](https://manual.avolites.com/docs/capture-visualiser/linking-the-console-to-stand-alone-capture) |

## 预览仍可能修改现场节目数据

MA Preview 中 Store／Update 直接修改实际节目对象，更新正在播放或向后跟踪的 Cue 可能改变现场输出。Preview 的隔离范围不能被解释为一份独立工程分支。相同 User Profile 的用户还共享 Preview 开启状态。[MA Preview](https://help.malighting.com/grandMA3/2.5/HTML/preview.html)。

Titan 内置 Capture 的版本基线为 Capture 2024；外部较新场景需要导出为兼容版本，且外部场景中的配适不能在控台内任意编辑。可视化库里缺少灯型、坐标错误或简化渲染都会使预览与真实灯具不同。[Titan Capture Show Files](https://manual.avolites.com/docs/capture-visualiser/capture-show-files)。

## 工作流程与 StageMaster 建议

自拟流程：现场运行一个 Cue，在盲编／预览环境修改另一个 Cue；分别更新未运行和正在运行的对象，明确哪一次应影响现场。再用 Scene Master 式预备状态组合颜色、亮度和效果，通过一个推杆过渡。

以下属于设计建议。第一阶段先完成二维灯位图、输出／来源检查和明确的编辑隔离。3D 可独立消费只读场景和输出快照；渲染掉帧不能阻塞执行引擎。预览可以复用核心求值，但拥有独立运行实例；“提交编辑”与“把预览送到现场”是两个明确命令。

手机可以提供简单布局与预览，桌面提供精细编排。是否研发完整 3D 光学引擎应单独评估，不能因为对标控台而让它成为第一版可靠输出的前置条件。
