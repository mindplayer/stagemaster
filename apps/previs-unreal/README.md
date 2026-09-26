# 舞台大师三维适配器

此模块消费 Rust 生成的空间和灯光状态，在舞台大师“舞台 → 三维预演”内部显示。工程、播放时钟和真实输出均不归 UE 管理。接口见 [预演契约](../../docs/module-api/previsualization.md)，接入决策见 [ADR-019](../../docs/development/decisions/PRODUCT-ADR-019-embedded-previsualization.md)。

当前开发环境已验证 macOS / Apple Silicon、UE 5.8.3、Xcode 26.1.1 和 Metal 工具链 17B54。UE 开发组件在后台离屏运行；客户独立安装包尚未完成。官方灯具资源随已安装引擎加载，不将 Epic 的资源文件复制进 Git。详见[资源清单](../../docs/previsualization-library.md)。

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
node --test tools/previs/signalling.test.mjs
env TMPDIR="$PWD/tmp" \
  "UE-LocalDataCachePath=$PWD/data/previs-derived-cache" \
  "UE-ZenDataPath=$PWD/data/previs-zen" \
  '/Users/Shared/Epic Games/UE_5.8/Engine/Binaries/Mac/UnrealEditor-Cmd' \
  "$PWD/apps/previs-unreal/StageMasterPreview.uproject" \
  -unattended -NullRHI -NoSound -NoSplash -NoP4 -NoTraceServer \
  '-ExecCmds=Automation RunTests StageMaster.Previs; Automation Quit' \
  '-TestExit=Automation Test Queue Empty' \
  -ReportExportPath="$PWD/tmp/previs/protocol-tests" \
  -abslog="$PWD/logs/previs-unreal-tests.log"
```

测试报告 `tmp/previs/protocol-tests/index.json` 的 3 项 `StageMaster.Previs` 必须全部为 `Success`；引擎进程正常退出本身不代表测试通过。无图形测试不能替代原生画面验收。

原生验收顺序：打开实际工程，开启三维预演，切换灯光来源，检查房间／舞台／灯位、光束、透视／俯视／全场、拾取／聚焦、工作照明、平面往返以及关闭／重新开启。故障测试只能终止本应用拥有的渲染子进程，并确认桌面工程仍可操作。当前三维灯位修改入口尚未开放，等待草稿保护和历史同步接通。
