# 舞台大师三维适配器

首次从私有仓库接手请按 [Apple 芯片 Mac 完整构建说明](../../docs/development/build-macos.md) 下载依赖并组装桌面。`package-renderer.mjs --build-only` 是明确的构建入口，只记录 `component-built`；不传参数仍执行并严格检查资源报告。以下工单段落包含历史组件验收边界，不能代替最新整包构建说明。

此模块消费 Rust 生成的空间和灯光状态，在舞台大师“舞台 → 三维预演”或“编排 → 显示三维”内部显示。工程、播放时钟和真实输出均不归 UE 管理。接口见 [预演契约](../../docs/module-api/previsualization.md)，接入决策见 [ADR-019](../../docs/development/decisions/PRODUCT-ADR-019-embedded-previsualization.md)。

协议 2 接入 Rust 量化后的两轴姿态，独立显示底座／水平支架／垂直灯头。支持场景静态指向和列表轴角渐变；通用网格不代表真实灯具尺寸，未实现图案盘、物理光度或自动校准。详见 [POSITION-001](../../docs/development/tasks/POSITION-001-moving-head-workflow.md)。

当前开发环境已验证 macOS / Apple Silicon、UE 5.8.3、Xcode 26.1.1 和 Metal 工具链 17B54。UE 开发组件在后台离屏运行；客户独立安装包尚未完成。官方灯具资源随已安装引擎加载，不将 Epic 的资源文件复制进 Git。详见[资源清单](../../docs/previsualization-library.md)。

## 独立组件打包进展（PREVIS-004）

工具 `tools/previs/package-renderer.mjs` 使用官方 UAT 生成 Mac ARM64 Development Game 包；实例暂存／日志／归档分别在 `tmp/previs-package-*`、`logs/PREVIS-004/`、`data/PREVIS-004/`。文件 Pak 不用 ZenStore，实际 SDK 和 Metal 编译保留；本机已有 Metal 工具链只读发现后交给所属进程，未安装或升级系统组件。进程级 CoreFoundation 缓存路径不改 HOME。新工具与原信令测试可运行：

~~~sh
TMPDIR="$PWD/tmp" node --test tools/previs/packaging-plan.test.mjs \
  tools/previs/packaging-process.test.mjs tools/previs/packaging-tools.test.mjs \
  tools/previs/mac-load-commands.test.mjs tools/previs/mac-bundle-inspection.test.mjs \
  tools/previs/signalling.test.mjs tools/previs/signalling-discovery.test.mjs
~~~

**此入口尚未达到完整出口**：实际独立 Game 已烘焙并正常启动，必需资源／H264 检查运行成功，但包的 App Sandbox 拒绝指定项目报告路径写入，且缺编辑器 HTML 报告模板，不能登记自动化报告通过。工具会拒绝空／缺／失败报告。缺省拒绝新的 Xcode 构建，因为已观察到其系统临时脚本不服从 TMPDIR；只有用户允许这一 OS 临时例外后，调用方才能显式传 `STAGEMASTER_ALLOW_PLATFORM_TEMP=1` 执行该工具。该变量不是自动授权。

Tauri 整包、子进程目录权限、实际 GPU／内嵌画面与无编辑器客户环境还未验收；不关闭 Sandbox 迁就测试，不把内部 Development 包当作 Shipping／签名公证完成。当前进度及失败日志见 [PREVIS-004](../../docs/development/tasks/PREVIS-004-packaged-renderer.md)。Node／信令组件的有限独立资格见下节，不能代替完整预演包通过。

`tools/previs/inspect-renderer.mjs` 是不需要 Xcode 构建的只读检查入口，唯一参数为项目内 `StageMasterPreview.app/Contents/MacOS/StageMasterPreview` 文件路径；标准输出是实际 `lipo`／`otool` 原文、ARM64 静态链接闭包与最高最低系统要求的 JSON，错误时非零退出。正式打包工具在 UAT 后、Game 启动前执行同一资格；包外 UE 库不得补齐缺项，动态加载环境覆盖不继承。当前原包 6 镜像／66 依赖的包内解析、中文空格移位副本与故意缺库拒绝已实际核对；仍不是运行期 `dlopen`、GPU、沙盒报告或完整客户安装验收。实际最低 macOS 为 14.0，桌面配置 12.0 的整包兼容性差异尚未决定，不在此私改最低版本。

### Development 限定目录报告

`tools/previs/qualify-renderer-files.mjs` 接受一个项目内已生成 Game 主程序路径，例如：

~~~sh
TMPDIR="$PWD/tmp" node tools/previs/qualify-renderer-files.mjs \
  "$PWD/data/PREVIS-004/previs-package-dGdEOa/Mac/StageMasterPreview.app/Contents/MacOS/StageMasterPreview"
~~~

该工具不调用 Xcode／UAT。它保留原包，只在新 `data/PREVIS-004/previs-file-access-*/` 副本增加官方报告模板及五个实例目录的受限文件资格，App Sandbox保持启用；本地 ad-hoc 签名后校验，原嵌套库和 Pak不改。真实 Game的 NullRHI／NoSound检查要求 JSON／HTML，另测范围外拒绝与恢复。日志／暂存在项目 logs／tmp，资格不是整个项目根，链接父目录在写入前拒绝。

开发副本正向／恢复 JSON和非空 HTML已实际通过，负向哨兵保持且系统拒绝与所属 Game PID关联；原四份默认打包失败仍保留。工具仅登记 `development-file-verified`、`customerPackageVerified:false`，不把本机绝对路径资格副本装入正式 Tauri，不代表客户签名／公证、可移动目录、实际 GPU或正式整包通过。证据／范围见 [PREVIS-004](../../docs/development/tasks/PREVIS-004-packaged-renderer.md#限定文件资格交付)和 [ADR-165](../../docs/development/decisions/PRODUCT-ADR-165-packaged-renderer-file-access.md)。

### 自包含信令组件（PREVIS-005）

从仓库根运行以下独立软件工具，不需要 Xcode，也不启动 UE 或修改正式桌面包：

~~~sh
TMPDIR="$PWD/tmp" node --test tools/previs/*.test.mjs
TMPDIR="$PWD/tmp" node tools/previs/package-signalling.mjs
~~~

工具仅支持本机已验证的 Node 24.17.0／Mac ARM64；按现行锁在全新 `data/PREVIS-005/previs-signalling-*/previs/` 目录做 offline、无安装脚本的 npm ci，不依赖客户 Node，不覆盖源 node_modules。依赖缓存缺失时明确失败，单独处理缓存后再使用原离线入口。日志／暂存在项目 logs／tmp，最终组件含 Node、现行信令、锁定依赖、Node 完整 LICENSE 和 npm 许可清单。所属测试结束移入 tmp，不把测试文件留在最终组件。

实际原目录／中文空格移位各用自己的 Node 通过 7 项认证／发现／EOF 和真实端口测试，2,273 文件字节等价；Node ARM64、4 系统库及最低 macOS 13.5 已只读确认。两份 Epic npm 包和 cookie-signature 缺独立许可文本，清单标待发行审查。此处只登记 `signalling-component-verified`，不接入 Tauri 资源、不修改沙盒或签名；仍需独立 Game JSON通过、组合系统要求、许可、客户整包和实际 GPU 验收。详情与失败证据见 [PREVIS-005](../../docs/development/tasks/PREVIS-005-signalling-component.md)。

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
