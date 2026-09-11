# 工程文件、备份、导入导出

节目文件不仅保存 Cue，还涉及灯具档案、用户数据、布局、媒体和输出配置。文件版本、内容引用和本机设置需要分别处理，才能在换机、巡演及未来云端分发时得到可预测结果。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M03-01 | 保存与另存 | Save／Save As、Quick Save、内部／外部存储 | Save／Save As、Quick Save、版本标签 | [MA Save](https://help.malighting.com/grandMA3/2.5/HTML/sfh_save.html)、[Titan Save](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-02 | 自动保存和历史 | 自动保存及 Backup 文件 | 自动保存、手动／快速／自动版本浏览 | [MA Backup](https://help.malighting.com/grandMA3/2.5/HTML/sfh_backup.html)、[Titan Save](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-03 | 分项加载 | Show Data、Local Settings、Output Stations、DMX Protocols 等可分开选择 | 可保留现有 DMX 设置或载入文件中的设置 | [MA Load](https://help.malighting.com/grandMA3/2.5/HTML/sfh_load.html)、[Titan Save](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-04 | 部分节目导入 | PSR 匹配灯具、选择对象、处理引用和目标池位置 | Import Show 建立灯具映射，从 Show Library 选内容 | [MA PSR](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[Titan Save](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-05 | 灯具映射 | PSR 依据 ID／GUID／名称等匹配并显示冲突 | 源灯可映射到当前节目中的目标灯具 | 同上 |
| M03-06 | 模板与基础工程 | 模板、演示节目、Show Creator 和池对象导入导出 | 开机标准 Show、节目片段复用 | [MA Backup](https://help.malighting.com/grandMA3/2.5/HTML/sfh_backup.html)、[MA Import/Export](https://help.malighting.com/grandMA3/2.5/HTML/import-export.html)、[Titan Save](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-07 | 灯型随节目 | Show／Fixture Types 及导入导出共同管理 | 已配适 Personality 内嵌于 Show，库更新不自动改变它 | [MA Fixture Types](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html)、[Titan Personalities](https://manual.avolites.com/docs/fixture-personalities) |
| M03-08 | 舞台交换 | MVR 导入／导出及网络交换 | Capture 舞台可独立导入／导出 | [MA MVR](https://help.malighting.com/grandMA3/2.5/HTML/patch_mvr.html)、[Titan Capture](https://manual.avolites.com/docs/capture-visualiser/capture-show-files) |
| M03-09 | 报表 | 可用对象导出／命令相关功能；本次未确认等同 Titan 报表向导 | Create Reports 生成节目和配适资料 | [Titan Reports](https://manual.avolites.com/docs/titan-basics/creating-reports) |
| M03-10 | 异常恢复 | 备份与软件／设备恢复流程分别管理 | Recover Show 尝试读取临时数据；另可载入旧版本 | [MA Backup](https://help.malighting.com/grandMA3/2.5/HTML/sfh_backup.html)、[Titan Save](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-11 | 版本兼容 | PSR 要求源 Show 保存于相同软件版本；其他迁移看发布说明 | 支持旧 Show 进入新软件；不支持新 Show 可靠回读旧软件 | [MA PSR](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)、[Titan Save](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |
| M03-12 | 对象依赖 | 预设／Recipe／跨池引用需连同依赖检查 | Palette／Playback／灯具映射需检查引用结果 | [MA Data Pools](https://help.malighting.com/grandMA3/2.5/HTML/datapool.html)、[Titan Library](https://manual.avolites.com/docs/titan-basics/show-library) |

## 合并和恢复不是普通复制

MA PSR 有不可 Oops、需 Standalone 或 Idle Master、清除编程器及部分引用不保留等限制；有关 Phaser Recipe／Shape 的限制见版本与待验证清单。它不是任意时刻可无感执行的数据库 Merge。[MA Partial Show Read](https://help.malighting.com/grandMA3/2.5/HTML/sc_psr.html)。

Titan 同名保存仍形成新版本，便于按标签回退；但载入旧版软件不是回退文件的通用办法。工程回退、软件回退和恢复设备系统应是三种不同操作。[Titan Loading and Saving Shows](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows)。

## 工作流程与 StageMaster 建议

自拟流程：将包含跨素材引用、图片、音乐和输出设置的节目保存到另一台机器，缺少一个媒体文件时检查提示；只导入一个 Cue，确认其灯具映射和依赖是否完整；再恢复到之前的项目版本。

以下属于设计建议。工程格式要有 schema 版本、稳定 ID、依赖清单与校验。保存采用完整快照／事务性提交，避免失败后覆盖唯一好版本。编辑工程、发布包、本机配置和用户偏好分开管理。

Fastify + PostgreSQL 负责账号、项目版本、权限和分发记录；对象存储保存大文件。上传或同步完成不等于现场激活；运行设备保留已验证的当前版本，先下载校验，再在明确切换点启用新包。
