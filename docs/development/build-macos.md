# Apple 芯片 Mac：下载依赖、开发与 UE 整包构建

更新：2026-10-07。面向接手源码的同事，所有命令从仓库根目录执行。

仓库：`https://github.com/mindplayer/stagemaster`，私有。需要 GitHub 账号已获得仓库访问权限。本仓库提交业务源码、测试、配置、依赖清单／锁文件、许可告知和文档；不提交第三方库、UE 引擎、构建缓存、音乐素材、用户工程及编译后的应用。依赖在每位开发者的电脑上自行下载。

本流程生成含 UE 的**本机内部开发包**，三维画面在舞台大师窗口内显示。UE 渲染进程与主程序分开运行，但运行所需组件装在同一个 `.app` 内。当前采用本地 ad-hoc 签名，运行目录绑定构建机器的项目路径，尚不是可随意复制给客户的签名／公证安装包。更换机器或移动项目目录后重新构建和组装；不要把本机产物复制到 `/Applications` 后当成正式发行版。

## 1. 工具链

| 工具 | 当前源码采用的版本与要求 |
| --- | --- |
| 机器 | Apple 芯片 Mac，原生 arm64 终端，不使用 Rosetta |
| Xcode | 完整 Xcode 26.1.1（17B100）；只有 Command Line Tools 不足以构建 UE |
| Metal | Xcode 的 Metal Toolchain，本机已使用 17B54；`metal`／`metallib` 均需可发现 |
| Unreal Engine | Epic Games Launcher 安装 UE 5.8.3；默认 `/Users/Shared/Epic Games/UE_5.8` |
| Node.js | **24.17.0 arm64**，信令打包工具检查这一版本；使用官方完整发行包或 NVM 安装 |
| Rust | 1.97.1，workspace 最低 1.97；包括 rustfmt／Clippy |
| Git | 可用 HTTPS 或 SSH 认证；不得把访问令牌写进仓库或远程 URL |

以上是本项目采用的工具版本，不是要求始终升级到各产品最新版。电脑系统需满足 Xcode 和 UE 对应版本要求；整包记录的最低运行系统 14.0 不表示这些构建工具可以安装到所有旧系统。Xcode／UE 由同事通过各自官方账户下载并接受许可；无需 PostgreSQL、Docker、ESP32 工具链即可开发桌面版。

安装 Xcode 后首次打开，完成组件安装并选择其命令行工具。Metal 缺失时通过 Xcode 组件界面安装，随后检查：

```sh
uname -m
xcodebuild -version
xcrun --no-cache --find metal
xcrun --no-cache --find metallib
node --version
node -p process.arch
rustc --version
cargo --version
```

Node 必须有完整发行目录：`bin/node` 的上级目录中包含 `LICENSE` 和 `lib/node_modules/npm/bin/npm-cli.js`。NVM 用户可运行 `nvm install 24.17.0 && nvm use 24.17.0`。单独复制 Node 可执行文件、部分 Homebrew 布局不能满足信令打包要求。Rust 可用 `rustup toolchain install 1.97.1 --profile minimal --component rustfmt --component clippy` 安装；后面的环境变量选择该版本。

## 2. 克隆与首次下载依赖

```sh
git clone https://github.com/mindplayer/stagemaster.git
cd stagemaster
mkdir -p tmp/cargo-home tmp/npm-cache logs
export RUSTUP_TOOLCHAIN=1.97.1
export CARGO_HOME="$PWD/tmp/cargo-home"
export TMPDIR="$PWD/tmp"
export npm_config_cache="$PWD/tmp/npm-cache"
export NODE_DISABLE_COMPILE_CACHE=1
export STAGEMASTER_UE_ROOT="/Users/Shared/Epic Games/UE_5.8"

cargo fetch --locked
npm --prefix apps/ui-prototype ci
npm --prefix tools/previs ci --ignore-scripts
npm --prefix tools/project-format ci
```

`cargo fetch` 与 `npm ci` 需要联网，并依据提交的锁文件下载依赖。不要用不带锁的更新操作代替它们。后续桌面 release 编译和信令打包使用离线依赖缓存，因此必须先完成这里的下载。更换终端后重新设置这些环境变量；缓存仍保留在项目 `tmp/`。引擎在别的位置时修改 `STAGEMASTER_UE_ROOT`。

## 3. 日常界面和桌面开发

仅开发网页界面：

```sh
npm --prefix apps/ui-prototype run dev -- --host 127.0.0.1
```

网页方式用于界面开发，不能替代 Tauri 原生文件、设备和音频服务。完整桌面开发入口：

```sh
node tools/desktop/run.mjs dev
```

仅构建调试桌面 `.app`：

```sh
node tools/desktop/run.mjs build
```

产物在 `target/debug/bundle/macos/舞台大师.app`（使用默认项目 target 时）。这一入口没有把 UE 与 Node 组件装入应用；调试三维需本机 UE 开发组件。要得到含 UE 的整包，使用下面第 4–6 步。

## 4. 构建 UE 独立渲染组件

首次从源码克隆需先生成 Editor 目标，供后续烘焙使用。编译前停止正在运行的预演，不并发修改或构建同一 UE 工程。

```sh
mkdir -p tmp/ue-build/platform-user tmp/ue-build/dotnet tmp/nuget-packages
STAGEMASTER_METAL_BIN="$(dirname "$(xcrun --no-cache --find metallib)")"
env TMPDIR="$PWD/tmp" \
  CFFIXED_USER_HOME="$PWD/tmp/ue-build/platform-user" \
  DOTNET_CLI_HOME="$PWD/tmp/ue-build/dotnet" \
  NUGET_PACKAGES="$PWD/tmp/nuget-packages" \
  PATH="$STAGEMASTER_METAL_BIN:$PATH" \
  "$STAGEMASTER_UE_ROOT/Engine/Build/BatchFiles/Mac/Build.sh" \
  StageMasterPreviewEditor Mac Development \
  "-Project=$PWD/apps/previs-unreal/StageMasterPreview.uproject" \
  -WaitMutex "-Log=$PWD/logs/previs-unreal-build.log"
```

Xcode 会使用少量由 macOS 管理的系统临时文件；这是构建工具的行为。执行上面的构建以及下面显式开关表示接受这一工具链行为。项目控制的缓存、日志与包仍保存在本项目内。

```sh
STAGEMASTER_ALLOW_PLATFORM_TEMP=1 \
  node tools/previs/package-renderer.mjs --build-only
```

本命令实际执行编译、烘焙、归档以及静态依赖检查，成功记录为 `component-built`。不启动 Game 做资源验收，不代表 GPU／画面已验收。无参数模式仍要求真实成功的资源报告；原始 UE 沙盒不允许写入指定报告目录时会失败，不能把“进程退出 0”算成通过。内部整包组装会为自己的副本准备报告模板和限定运行目录。

成功后终端打印独立组件主程序路径，在当前终端设置：

```sh
export STAGEMASTER_GAME="这里粘贴本次输出的 StageMasterPreview 主程序绝对路径"
```

路径形如 `项目/data/PREVIS-004/previs-package-本次实例/Mac/StageMasterPreview.app/Contents/MacOS/StageMasterPreview`。使用本步原始归档，不使用已由 `qualify-renderer-files.mjs` 重签的验证副本。构建记录在同一实例归档的 `build-record.json`；日志在 `logs/PREVIS-004/`。

## 5. 打包 Node 与信令组件

```sh
node tools/previs/package-signalling.mjs
```

命令从本机完整 Node 发行包和项目 npm 缓存生成独立信令目录，自带 Node、锁定依赖和许可告知；执行原目录及移位目录的相关小型检查。成功状态必须是 `signalling-component-verified`。完成后将终端打印的 `previs` 目录设置为：

```sh
export STAGEMASTER_SIGNALLING="这里粘贴本次输出的 previs 目录绝对路径"
```

路径形如 `项目/data/PREVIS-005/previs-signalling-本次实例/previs`。它不是源码 `tools/previs/`，也不是 `node_modules/`。

## 6. 构建桌面、组装 UE 并生成启动入口

先保存并提交本次源码修改；构建和组装期间不要切换分支或继续修改源码。以下代码复用现有构建／组装模块，创建唯一实例，使用本次构建返回的准确路径，不自动寻找“最新文件”。它会对新整包副本进行本地 ad-hoc 签名，保持原 UE 组件不变。

不要在组装前启动纯桌面中间包，否则同名实例运行目录已存在时组装会拒绝复用。

```sh
node --input-type=module <<'JS'
import { existsSync, mkdirSync, mkdtempSync, realpathSync, writeFileSync } from 'node:fs';
import { basename, join } from 'node:path';
import { buildInternalRelease } from './tools/desktop/release-build.mjs';
import { assembleDevelopmentDesktop } from './tools/previs/assemble-development-desktop.mjs';

const root = realpathSync('.');
const game = process.env.STAGEMASTER_GAME;
const signalling = process.env.STAGEMASTER_SIGNALLING;
if (!game || !signalling || !existsSync(game) || !existsSync(signalling))
  throw new Error('请先设置本次成功构建的 Game 主程序和信令目录');
mkdirSync(join(root, 'tmp'), { recursive: true });
const instance = basename(mkdtempSync(join(root, 'tmp/desktop-release-')));
const desktop = await buildInternalRelease(root, instance);
const assembled = await assembleDevelopmentDesktop(
  [desktop.bundle, game, signalling],
  join(desktop.plan.archive, 'build-record.json'),
);
const quote = value => "'" + value.replaceAll("'", "'\\''") + "'";
const environment = assembled.plan.launchEnvironment;
const launcher = join(assembled.plan.archive, '启动舞台大师.command');
writeFileSync(launcher, [
  '#!/bin/sh',
  ...Object.entries(environment).map(([key, value]) => `export ${key}=${quote(value)}`),
  `exec ${quote(join(assembled.plan.bundle, 'Contents/MacOS/stagemaster-desktop'))}`,
  '',
].join('\n'), { mode: 0o755, flag: 'wx' });
console.log(`启动入口：${launcher}`);
JS
```

成功时生成：

- `data/PREVIS-007/desktop-release-本次实例/舞台大师 内部验收.app`：最终整包。
- 同目录 `启动舞台大师.command`：双击启动，或在终端执行其完整路径；内部实例需要其中的环境变量。
- 同目录 `assembly-record.json`：组件来源、文件校验、签名检查及组装状态。
- `data/DESKTOP-005/desktop-release-本次实例/build-record.json`：桌面源码提交、指纹和构建记录。

运行整包时不需要外部 UE 编辑器或外部 Node；首次打开三维可能需要准备着色器缓存。这个命令只负责构建和组装，不自动打开工程、连接硬件或播放。预演画面、声音及真实灯具输出分别验证，构建成功不替代这些结论。运行所需目录与项目绑定，暂时保留项目位置。

UE／信令源码和依赖锁未变时可以复用此前成功的第 4、5 步组件，只重新执行第 6 步。组件源码、引擎版本或锁文件变化后重建对应组件。不要在 Git 中加入这些产物，也不要为了同事交接提交 `node_modules` 或引擎文件。

## 7. 常见问题与相关检查

| 现象 | 处理 |
| --- | --- |
| 克隆提示找不到仓库 | 私有仓库未授权，或终端登录的 GitHub 账号不是已获授权账号 |
| `cargo --offline` 缺包 | 检查项目 `CARGO_HOME`；联网执行同目录 `cargo fetch --locked`，保留锁文件 |
| 信令 `npm ci --offline` 缺缓存 | 以项目 `npm_config_cache` 再执行第 2 步的信令 `npm ci` |
| Node 缺许可文件／npm | 换成完整官方发行目录或 NVM 的 24.17.0 arm64，不只复制二进制 |
| Metal／Xcode SDK 找不到 | 检查完整 Xcode、首次运行组件和选中的工具链；不跳过 SDK 检查 |
| `runtime-assets/index.json` 不存在 | 默认运行验收未取得报告；构建交接用明确的 `--build-only`，不要修改失败记录冒充通过 |
| 组装提示来源变化／目录已存在 | 源码变更后重建桌面并使用新实例；不删除检查条件或复用已启动实例 |
| 应用缺内部实例环境 | 使用本次 `启动舞台大师.command`，不要直接启动中间包 |

修改相关代码后按模块检查，不要求每次打包重跑全量。例如：

```sh
npm --prefix apps/ui-prototype run check
node --test tools/previs/renderer-package-check.test.mjs tools/previs/packaging-plan.test.mjs
cargo fmt --all -- --check
```

Rust 逻辑变更执行对应 crate 测试；候选版收敛再按 [开发规则](../../AGENTS.md) 集中回归。当前云端、正式签名／公证及跨机器发行未在这份构建交接中补完。ESP32 固件是独立工具链，不在上述命令中自动构建或烧录，入口见 [板卡与固件说明](../hardware/waveshare-esp32-s3-rs485-can.md)。

代码入口：[模块地图](sol-handoff/code-map.md)；当前任务：[STATE 顶部](STATE.md)；三维细节：[UE 模块说明](../../apps/previs-unreal/README.md)。历史文档里的本机绝对路径和实例只是当时记录，不是新机器必须具备的输入。
