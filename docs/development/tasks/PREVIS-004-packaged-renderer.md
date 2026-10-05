# PREVIS-004：独立预演组件的可复现打包

状态：ready，首个有限增量先纳入版本后实施。基线 main `0885847e3de629ac0ead1eb498b4e84cc30082b7`，主工作区单写者；只有用户未跟踪 `output/`，不触碰。属于 H5 客户无 UE 编辑器路径的准备，不关闭 H5 或整个持续 goal。

## 本增量范围

现有 `StageMasterPreview` Game Target 与桌面资源启动分支已存在，但无可复现 Build／Cook／Stage／Archive 流程。先只生成 **Mac ARM64 Development 独立渲染组件**，检查实际包内资源与非编辑器程序启动。复用原 UE 适配器、官方 UAT 与本机已安装引擎；不重写渲染、信令、播放器或时钟。

- 源码／工具限 `apps/previs-unreal/`、`tools/previs/`，以及相关文档；无 Rust／UI／持久格式、控制权或公开协议变化，暂不需要新 ADR。
- 构建缓存／暂存在项目 `tmp/` 与既有 UE 工程缓存，日志在 `logs/`，独立包与证据在 `data/`。每次生成独立目录，不覆盖已有合格包；失败保留日志且不登记为成功。
- 使用 Development 便于实际运行 UE 自动化；它不是 Shipping／签名／公证／公开发行承诺。SDK 检查保留，不用运行时跳过 SDK 的措施掩盖构建条件。
- 添加一个只读的必需视觉资源自动化检查，覆盖当前实际渲染所需资源，兼容编辑器与 Game 测试；原官方库广泛加载检查仍保留，不为了让小包通过而改弱旧验收。
- 不将 Epic 原始素材加入 Git，不执行 DMX、不连接真实设备，不安装或修改全局引擎配置。

## 验收与失败行为

1. 打包命令、平台／架构／输出路径与错误处理具可执行脚本测试；非支持平台、缺失引擎／工程、子进程失败明确拒绝，不留下假成功记录。
2. 实际执行官方 UAT，输出独立 Game `.app` 及烘焙内容，保存完整命令、版本、构建结果和产物清单。
3. 运行产物中的 Game 程序而非 UnrealEditor；在项目内 UserDir／日志、无图形模式执行必需资源检查。必须核对自动化报告中的实际成功项，不只看退出码；空报告、失败报告、缺资源不能判成功。
4. 实际运行现有 UE 全部相关编辑器自动化，原信令测试及差异／文档／JSON 检查。只读资源／打包变化不重复不相关 Rust／UI 全量；构建失败先定位，不更改原核心保护。
5. 没有 GPU／真实内嵌画面就明确未验收；无 UE 编辑器的桌面整包还需要 Node 与信令运行依赖、自包含动态库、Tauri 组装及正式桌面操作，作为后续内聚增量。

## 成熟机制依据

采用 Epic 的 [BuildCookRun 分阶段流程](https://dev.epicgames.com/documentation/unreal-engine/build-operations-cooking-packaging-deploying-and-running-projects-in-unreal-engine?lang=en-US) 与 [独立打包说明](https://dev.epicgames.com/documentation/unreal-engine/packaging-your-project?lang=en-US)：预先烘焙、暂存、归档，不要求客户使用 cook server。参数同时核对本机 UE 5.8 的 `ProjectParams.cs`、`CommandEnvironment.cs` 与 `RunUAT.sh`；日志／UserDir／文件缓存全部指定项目内路径。第三方许可证与客户交付清单后续一并核对，不因本地构建成功宣称具有最终商业分发条件。

## 交付记录

实施与实际验证后填入结果版本、命令／日志、产物规模、成功／失败事实、剩余门槛及下一入口。
