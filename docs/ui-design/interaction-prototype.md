# UX-003：舞台画布交互原型

日期：2026-09-21。用户选择第一张“舞台画布”，并补充剪映式编辑体验。保留深色舞台画布，将常用编排组织为“资源 → 舞台预览 → 属性 → 时间线”。[工单与验证](../development/tasks/UX-003-stage-canvas-prototype.md)，[视觉验收](../../design-qa.md)。

## 打开与体验

本次本地预览：[http://127.0.0.1:4173/](http://127.0.0.1:4173/)。源码在 `apps/ui-prototype/`，只监听本机回环地址。

1. 初始编辑 Cue 14“桥段”，模拟现场正在播放 Cue 12，下一条为 Cue 13。三个状态始终分别显示。
2. 选灯组或时间线片段，修改亮度、颜色、开始时间、时长、淡入／淡出。拖动片段移动，拖动两端裁切；一次拖动对应一次撤销。方向键微调，Delete 删除。
3. 从左侧拖入灯组／预设，或点击轨道旁的“＋”添加片段。吸附步长 0.5 秒，关闭后为 0.1 秒；支持时间线缩放。
4. 移动预览位置或按空格播放所选片段的场景示意。可以导入 30 MB 以内、浏览器支持的音频作为本机参考，显示真实解码波形；不上传文件。波形与秒刻度对齐，短音频不会拉长铺满轨道。
5. “保存到 Cue 14”只保存演示内的编辑内容。底部 GO 才会从已保存 Cue 取得独立模拟运行快照并推进待执行项。Cue 列表也能分别选择编辑对象和待执行对象。

快捷键：空格预览，⌘/Ctrl S 保存，⌘/Ctrl Z 撤销，⇧⌘/Ctrl Z 重做。输入框中不截获这些编辑动作。刷新页面恢复初始演示数据，音乐参考也需重新导入。

## 实现边界

- React／TypeScript／Vite 前端，内存演示适配器 `src/demo-session.ts`；未连接 Rust、Tauri、真实设备或云端。它是验证交互的可替换模拟层，不定义正式工程格式、调度或合成语义。
- 时间线展示一个 Cue 内部的 20 秒示例；尚无整场节目时间码编排、跨 Cue 连续执行、跟踪、效果引擎或任意关键帧曲线。当前包络只表达淡入／淡出。
- 舞台采用生成插图，按选中片段反馈亮度与色相；不是全部轨道的合成结果，不是光学预演或现场监看。音乐仅用于参考试听，不承担演出音频输出；监听时延尚未验收。
- 支持桌面和窄屏基础重排。窄屏时间线保留横向滚动、页面纵向滚动；手机端完整编排交互与触摸手势尚未验收。
- 所有数据只存内存。工程保存、崩溃恢复、撤销事务、命令权限和远程控制仍需正式契约。

## 代码与资产

`App.tsx` 编排界面状态；`Timeline.tsx` 管理片段编辑手势；`StagePreview.tsx` 提供示意预览；`AudioReference.tsx` 负责参考音频；`ClipEnvelope.tsx` 绘制真实淡入／淡出数值的包络。接口与状态通过显式 props／动作传递。

视觉依据：[原始三方案](visual-directions.md)、[时间线修订提示词](timeline-revision-prompt.md)。选定设计参考保存在 `data/ui-design/2026-09-21/`，这四张设计基准 PNG 显式纳入版本管理；其他运行截图仍作为项目内验收产物忽略。

舞台资产 `apps/ui-prototype/public/assets/stage-blue.png` 为本轮 Image Gen 从选定舞台图重新生成的独立场景，保留八台蓝色逆光、桁架、台阶及音箱，去除 UI、文字、灯具编号和选择标记。未把整张界面图铺作应用背景。图标来自 [Phosphor React](https://github.com/phosphor-icons/react)（MIT）；未引入新的业务语言。舞台和控件布局并非剪映界面的复制；参考其公开的[桌面编辑流程](https://www.capcut.com/resource/how-to-use-capcut)与[关键帧说明](https://www.capcut.com/help/keyframes-in-capcut-pc)，将资源、预览、属性和时间线用于灯光任务。

## 复现

本机 Node 24.17.0／npm 11.17.0，依赖锁定在 app 的 `package-lock.json`。Node 的内置 TypeScript 去类型能力用于测试，不另加测试运行器。

```sh
cd /Users/sunqi/projects/stagemaster
mkdir -p tmp
export TMPDIR="$PWD/tmp"
export npm_config_cache="$PWD/tmp/npm-cache"
cd apps/ui-prototype
npm ci
npm run check
npm test
npm run build
npm run dev -- --port 4173 --strictPort
```

本轮实际检查、10 项测试及生产构建通过。构建保留模板的静态资源服务封装，不代表云端已经部署；不依赖该封装进行本机预览。Rust 未改动，未重复运行 Rust 测试。

后续先由灯光师完成“选灯 → 调整 → 预览 → 保存 → 播放 → 修改”并反馈，再确定最小可用编辑命令与 Rust 状态接口。框架和领域语义的正式接入应另立工单，避免把原型的简化示例固化为内核规则。
