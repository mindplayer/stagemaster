# 舞台大师三维适配器

此模块消费 Rust 生成的空间和灯光状态，在舞台大师“舞台 → 三维预演”或“编排 → 显示三维”内部显示。工程、播放时钟和真实输出均不归 UE 管理。接口见 [预演契约](../../docs/module-api/previsualization.md)，接入决策见 [ADR-019](../../docs/development/decisions/PRODUCT-ADR-019-embedded-previsualization.md)。

协议 2 接入 Rust 量化后的两轴姿态，独立显示底座／水平支架／垂直灯头。支持场景静态指向和列表轴角渐变；通用网格不代表真实灯具尺寸，未实现图案盘、物理光度或自动校准。详见 [POSITION-001](../../docs/development/tasks/POSITION-001-moving-head-workflow.md)。

当前开发环境已验证 macOS / Apple Silicon、UE 5.8.3、Xcode 26.1.1 和 Metal 工具链 17B54。UE 开发组件在后台离屏运行；客户独立安装包尚未完成。官方灯具资源随已安装引擎加载，不将 Epic 的资源文件复制进 Git。详见[资源清单](../../docs/previsualization-library.md)。

## 独立组件打包进展（PREVIS-004）

工具 `tools/previs/package-renderer.mjs` 使用官方 UAT 生成 Mac ARM64 Development Game 包；实例暂存／日志／归档分别在 `tmp/previs-package-*`、`logs/PREVIS-004/`、`data/PREVIS-004/`。文件 Pak 不用 ZenStore，实际 SDK 和 Metal 编译保留；本机已有 Metal 工具链只读发现后交给所属进程，未安装或升级系统组件。进程级 CoreFoundation 缓存路径不改 HOME。新工具与原信令测试可运行：

~~~sh
TMPDIR="$PWD/tmp" node --test tools/previs/packaging-plan.test.mjs \
  tools/previs/packaging-process.test.mjs tools/previs/packaging-tools.test.mjs \
  tools/previs/signalling.test.mjs tools/previs/signalling-discovery.test.mjs
~~~

**此入口尚未达到完整出口**：实际独立 Game 已烘焙并正常启动，必需资源／H264 检查运行成功，但包的 App Sandbox 拒绝指定项目报告路径写入，且缺编辑器 HTML 报告模板，不能登记自动化报告通过。工具会拒绝空／缺／失败报告。缺省拒绝新的 Xcode 构建，因为已观察到其系统临时脚本不服从 TMPDIR；只有用户允许这一 OS 临时例外后，调用方才能显式传 `STAGEMASTER_ALLOW_PLATFORM_TEMP=1` 执行该工具。该变量不是自动授权。

Node／信令运行依赖、Tauri 整包、子进程目录权限、实际 GPU／内嵌画面与无编辑器客户环境还未验收；不关闭 Sandbox 迁就测试，不把内部 Development 包当作 Shipping／签名公证完成。当前进度及失败日志见 [PREVIS-004](../../docs/development/tasks/PREVIS-004-packaged-renderer.md)。

从仓库根目录安装锁定的信令依赖、构建开发适配器：

```sh
npm_config_cache="$PWD/tmp/npm-cache" npm --prefix tools/previs ci
TMPDIR="$PWD/tmp" '/Users/Shared/Epic Games/UE_5.8/Engine/Build/BatchFiles/Mac/Build.sh' \
  StageMasterPreviewEditor Mac Development \
  -Project="$PWD/apps/previs-unreal/StageMasterPreview.uproject" \
  -WaitMutex -Log="$PWD/logs/previs-unreal-build.log"
npm --prefix apps/ui-prototype run desktop:build
```

重编译前先在舞台大师停止预演。非默认引擎安装位置需设置 `STAGEMASTER_UE_ROOT` 给桌面进程；构建命令使用相应的引擎路径。开发桌面包记录构建时 Node 路径，正式分发需要随包提供运行时及信令依赖，不能依赖客户本机 Node。

协议边界、坐标绕序和官方资源加载测试：

```sh
node --test tools/previs/signalling.test.mjs tools/previs/signalling-discovery.test.mjs
env TMPDIR="$PWD/tmp" \
  "UE_LocalDataCachePath=$PWD/data/previs-derived-cache" \
  UE_SKIP_UBT_SDK_SETUP=1 \
  '/Users/Shared/Epic Games/UE_5.8/Engine/Binaries/Mac/UnrealEditor-Cmd' \
  "$PWD/apps/previs-unreal/StageMasterPreview.uproject" \
  -unattended -NullRHI -NoSound -NoSplash -NoP4 -NoTraceServer \
  "-UserDir=$PWD/data/previs-user" \
  "-LocalDataCachePath=$PWD/data/previs-derived-cache" \
  '-DDC=(ProjectPak,InstalledProjectPak,EnginePak=InstalledEnginePak,Local)' \
  '-ExecCmds=Automation RunTests StageMaster.Previs; Automation Quit' \
  '-TestExit=Automation Test Queue Empty' \
  -ReportExportPath="$PWD/tmp/previs/protocol-tests" \
  -abslog="$PWD/logs/previs-unreal-tests.log"
```

测试报告 `tmp/previs/protocol-tests/index.json` 中全部 `StageMaster.Previs` 测试必须为 `Success`；引擎进程正常退出本身不代表测试通过。无图形测试不能替代原生画面验收。

Apple／Unix 的 UE 环境读取会把旧变量名中的连字符转换为下划线，故进程必须设置 `UE_LocalDataCachePath`，不能只设置 `UE-LocalDataCachePath`。桌面启动沿用引擎只读缓存包与项目内文件缓存，并明确设置 UserDir／TMPDIR；不启动全局 Zen 服务或修改用户全局设置。已编译适配器的 `-game` 运行／自动化测试使用无人值守模式并跳过后台跨平台 SDK 导出，避免 Turnkey 弹窗及 UBT 写用户级日志；独立构建命令仍执行工具链检查，不因此声称客户打包或未知平台就绪。

首次启动可能在项目内编译着色器。本地单渲染器的官方前端只查询一次目录；收到空目录的观看端，在已认证渲染器就绪后收到一次官方渲染列表通知。不再同时启用目录轮询，避免旧定时查询再次订阅而中断当前视频；实际连接失败仍保留原有限重试预算，SDP／ICE／订阅与认证仍由既有官方库处理。通知不重建观看连接，也不干扰已播放的订阅。视窗区分“等待渲染”“连接画面”和真实播放手势受阻；只有后者显示“播放画面”。关闭预演仍按原生命周期回收本应用拥有的进程，不控制节目或音乐。冷缓存原生验收见 [AUDIO-020](../../docs/development/tasks/AUDIO-020-performance-loop-sections.md#sol-接续增量迟到渲染连接)。

原生验收顺序：打开实际工程，开启三维预演，切换灯光来源，检查房间／舞台／灯位、光束、透视／俯视／全场、拾取／聚焦、工作照明、平面往返以及关闭／重新开启。选择“移动灯位”后可在当前高度水平拖动；一次松手形成一次撤销，拖出视窗取消，属性中的精确坐标与工程同步。修改仍由 Rust 校验和保存；无效属性草稿不会被三维选择或拖动覆盖。安装旋转和空间归属在属性中编辑。

“跟随列表预览”读取已有列表执行器，执行／暂停／继续／停止均在“列表与预览”操作。已用真实 60 秒渐变验证内嵌画面联动；没有真实设备输出。故障测试只能终止本应用拥有的渲染子进程，并确认桌面工程仍可操作。原生验收记录及尚未测量的光学／性能边界见 [PREVIS-001](../../docs/development/tasks/PREVIS-001-real-stage-preview.md)。
