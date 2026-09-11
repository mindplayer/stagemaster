# 官方目录覆盖与研究状态

核查日期：2026-09-10。

已登记 1,701 个手册主题，其中 720 个正文已获取，195 个被模块报告直接引用。另有 3,842 个 Titan 19.2 API 主题入口和 8 个补充官方来源。

**目录覆盖不是正文逐页精读率，也不是实机测试率。** `retrieved` 只代表成功获取正文；`cited_in_module` 只代表被模块报告引用，并不表示该页全部内容或所有 API 都已验证。未直接引用的专题可能只用于目录定位、关联阅读或后续查阅，统一保留 `not_individually_cited`。

本轮全模块整理见 [22 个模块入口](README.md)；功能条目见 [统一索引](feature-matrix.md)；证据冲突及真实设备验证缺口见 [核查记录](evidence-issues.md)。

## 状态字段

| 字段 | 含义 |
| --- | --- |
| indexed | 官方目录存在该主题，已登记标题和链接 |
| retrieved | 正文已获取到临时阅读缓存，不等于已总结全文 |
| cited_in | 引用该页的模块文件；以模块具体说法为结论范围 |
| module_mapping | 该主题类别对应的研究模块；不是该页每项细节已被覆盖的声明 |
| coverage_role | 功能主题、命令 API、硬件、教程、导航或历史版本 |
| displayed_version | 页面正文中显示的版本；可能早于目录集合版本 |

## 手册目录逐类对应

命令／API、硬件多语言说明和历史版本采用独立参考目录，不逐个当作新功能重复计数。

### grandMA3

| 官方目录 | 研究模块 | 类型 | 索引 | 已获取 | 被模块引用 |
| --- | --- | --- | ---: | ---: | ---: |
| [grandMA3 User Manual](https://help.malighting.com/grandMA3/2.5/HTML/help.html) | [M22](M22-glossary-design.md) | navigation_reference | 1 | 1 | 0 |
| [New in the Manual](https://help.malighting.com/grandMA3/2.5/HTML/news.html) | [M21](M21-version-baseline.md) | navigation_reference | 1 | 1 | 0 |
| [About the Help](https://help.malighting.com/grandMA3/2.5/HTML/about_the_manual.html) | [M20](M20-maintenance-diagnostics.md) | navigation_reference | 7 | 0 | 0 |
| [Device Overview](https://help.malighting.com/grandMA3/2.5/HTML/device_overview.html) | [M01](M01-products-capacity.md)、[M20](M20-maintenance-diagnostics.md) | hardware_reference | 116 | 6 | 1 |
| [System Overview](https://help.malighting.com/grandMA3/2.5/HTML/system.html) | [M01](M01-products-capacity.md)、[M16](M16-dmx-network.md)、[M18](M18-sessions-backup.md) | module_topics | 7 | 7 | 4 |
| [First Steps](https://help.malighting.com/grandMA3/2.5/HTML/first_steps.html) | [M02](M02-workspace-users.md)、[M03](M03-show-files.md)、[M04](M04-fixtures-patch.md)、[M06](M06-programmer-attributes.md)、[M08](M08-cues-tracking.md)、[M20](M20-maintenance-diagnostics.md) | module_topics | 20 | 20 | 0 |
| [grandMA3 onPC](https://help.malighting.com/grandMA3/2.5/HTML/onpc.html) | [M01](M01-products-capacity.md)、[M20](M20-maintenance-diagnostics.md)、[M21](M21-version-baseline.md) | module_topics | 10 | 10 | 1 |
| [Show File Handling](https://help.malighting.com/grandMA3/2.5/HTML/show_file_management.html) | [M03](M03-show-files.md) | module_topics | 7 | 7 | 3 |
| [Workspace](https://help.malighting.com/grandMA3/2.5/HTML/workspace.html) | [M02](M02-workspace-users.md) | module_topics | 31 | 31 | 1 |
| [Command Syntax and Keywords](https://help.malighting.com/grandMA3/2.5/HTML/command_syntax_keywords.html) | [M15](M15-macros-automation.md) | command_api_reference | 451 | 1 | 1 |
| [Windows, Views, and Menus](https://help.malighting.com/grandMA3/2.5/HTML/wvm.html) | [M02](M02-workspace-users.md) | module_topics | 16 | 16 | 1 |
| [Networking](https://help.malighting.com/grandMA3/2.5/HTML/network.html) | [M16](M16-dmx-network.md)、[M17](M17-remotes-api.md)、[M18](M18-sessions-backup.md) | module_topics | 17 | 17 | 4 |
| [DMX In and Out](https://help.malighting.com/grandMA3/2.5/HTML/dmx.html) | [M16](M16-dmx-network.md) | module_topics | 7 | 7 | 4 |
| [Single User and Multi User Systems](https://help.malighting.com/grandMA3/2.5/HTML/user.html) | [M02](M02-workspace-users.md)、[M18](M18-sessions-backup.md) | module_topics | 6 | 6 | 3 |
| [Patch and Fixture Setup](https://help.malighting.com/grandMA3/2.5/HTML/patch.html) | [M04](M04-fixtures-patch.md)、[M14](M14-preview-3d.md) | module_topics | 21 | 21 | 10 |
| [Operate Fixtures](https://help.malighting.com/grandMA3/2.5/HTML/operate_fixtures.html) | [M05](M05-selection-groups-layout.md)、[M06](M06-programmer-attributes.md)、[M11](M11-recipes-reuse.md) | module_topics | 25 | 25 | 10 |
| [Label Objects](https://help.malighting.com/grandMA3/2.5/HTML/wvm_pool_label.html) | [M02](M02-workspace-users.md) | module_topics | 1 | 1 | 1 |
| [Notes](https://help.malighting.com/grandMA3/2.5/HTML/notes.html) | [M02](M02-workspace-users.md)、[M19](M19-live-show.md) | module_topics | 3 | 3 | 1 |
| [Scribbles](https://help.malighting.com/grandMA3/2.5/HTML/scribbles.html) | [M02](M02-workspace-users.md) | module_topics | 4 | 4 | 1 |
| [Appearances](https://help.malighting.com/grandMA3/2.5/HTML/appear.html) | [M02](M02-workspace-users.md) | module_topics | 4 | 4 | 1 |
| [Images](https://help.malighting.com/grandMA3/2.5/HTML/images.html) | [M02](M02-workspace-users.md)、[M12](M12-pixel-media.md) | module_topics | 1 | 1 | 0 |
| [Supported File Formats](https://help.malighting.com/grandMA3/2.5/HTML/file_formats.html) | [M03](M03-show-files.md)、[M12](M12-pixel-media.md) | module_topics | 1 | 1 | 1 |
| [Screenshots](https://help.malighting.com/grandMA3/2.5/HTML/screenshot.html) | [M02](M02-workspace-users.md)、[M20](M20-maintenance-diagnostics.md) | module_topics | 1 | 1 | 1 |
| [Meshes](https://help.malighting.com/grandMA3/2.5/HTML/meshes.html) | [M14](M14-preview-3d.md) | module_topics | 1 | 1 | 0 |
| [Videos](https://help.malighting.com/grandMA3/2.5/HTML/videos.html) | [M12](M12-pixel-media.md) | module_topics | 1 | 1 | 1 |
| [Gobos](https://help.malighting.com/grandMA3/2.5/HTML/gobos.html) | [M04](M04-fixtures-patch.md)、[M06](M06-programmer-attributes.md) | module_topics | 1 | 1 | 0 |
| [Symbols](https://help.malighting.com/grandMA3/2.5/HTML/symbols.html) | [M02](M02-workspace-users.md)、[M05](M05-selection-groups-layout.md) | module_topics | 3 | 3 | 1 |
| [Groups](https://help.malighting.com/grandMA3/2.5/HTML/group.html) | [M05](M05-selection-groups-layout.md)、[M09](M09-playback-mixing.md) | module_topics | 5 | 5 | 1 |
| [Presets](https://help.malighting.com/grandMA3/2.5/HTML/presets.html) | [M07](M07-presets-palettes.md) | module_topics | 6 | 6 | 4 |
| [Worlds and Filters](https://help.malighting.com/grandMA3/2.5/HTML/worldfilter.html) | [M06](M06-programmer-attributes.md)、[M18](M18-sessions-backup.md) | module_topics | 5 | 5 | 1 |
| [MAtricks and Shuffle](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html) | [M05](M05-selection-groups-layout.md)、[M10](M10-effects-phasers.md)、[M11](M11-recipes-reuse.md) | module_topics | 7 | 7 | 1 |
| [Cues and Sequences](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence.html) | [M08](M08-cues-tracking.md)、[M09](M09-playback-mixing.md) | module_topics | 19 | 19 | 14 |
| [Executors](https://help.malighting.com/grandMA3/2.5/HTML/executor.html) | [M09](M09-playback-mixing.md)、[M19](M19-live-show.md) | module_topics | 5 | 5 | 3 |
| [Masters](https://help.malighting.com/grandMA3/2.5/HTML/masters.html) | [M09](M09-playback-mixing.md)、[M19](M19-live-show.md) | module_topics | 8 | 8 | 3 |
| [Recipes](https://help.malighting.com/grandMA3/2.5/HTML/recipes.html) | [M11](M11-recipes-reuse.md) | module_topics | 4 | 4 | 2 |
| [Phasers](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html) | [M10](M10-effects-phasers.md) | module_topics | 6 | 6 | 1 |
| [Shapes](https://help.malighting.com/grandMA3/2.5/HTML/shapes.html) | [M10](M10-effects-phasers.md)、[M11](M11-recipes-reuse.md) | module_topics | 1 | 1 | 1 |
| [Generator - Random](https://help.malighting.com/grandMA3/2.5/HTML/generator.html) | [M10](M10-effects-phasers.md) | module_topics | 1 | 1 | 1 |
| [Bitmap](https://help.malighting.com/grandMA3/2.5/HTML/bitmap.html) | [M12](M12-pixel-media.md) | module_topics | 1 | 1 | 1 |
| [XYZ](https://help.malighting.com/grandMA3/2.5/HTML/xyz.html) | [M14](M14-preview-3d.md)、[M17](M17-remotes-api.md) | module_topics | 3 | 3 | 1 |
| [Tags](https://help.malighting.com/grandMA3/2.5/HTML/tags.html) | [M02](M02-workspace-users.md)、[M19](M19-live-show.md) | module_topics | 1 | 1 | 1 |
| [Macros](https://help.malighting.com/grandMA3/2.5/HTML/macros.html) | [M15](M15-macros-automation.md) | module_topics | 10 | 10 | 2 |
| [Agenda](https://help.malighting.com/grandMA3/2.5/HTML/agenda.html) | [M15](M15-macros-automation.md) | module_topics | 5 | 5 | 1 |
| [Timers](https://help.malighting.com/grandMA3/2.5/HTML/timers.html) | [M13](M13-timecode-audio.md)、[M15](M15-macros-automation.md) | module_topics | 3 | 3 | 1 |
| [Preview](https://help.malighting.com/grandMA3/2.5/HTML/preview.html) | [M14](M14-preview-3d.md) | module_topics | 1 | 1 | 1 |
| [Timecode Show](https://help.malighting.com/grandMA3/2.5/HTML/timecode.html) | [M13](M13-timecode-audio.md) | module_topics | 11 | 11 | 6 |
| [Layouts](https://help.malighting.com/grandMA3/2.5/HTML/layouts.html) | [M02](M02-workspace-users.md)、[M05](M05-selection-groups-layout.md) | module_topics | 8 | 8 | 1 |
| [Plugins](https://help.malighting.com/grandMA3/2.5/HTML/plugins.html) | [M15](M15-macros-automation.md) | command_api_reference | 160 | 160 | 3 |
| [Quickeys](https://help.malighting.com/grandMA3/2.5/HTML/quickeys.html) | [M02](M02-workspace-users.md)、[M15](M15-macros-automation.md) | module_topics | 4 | 4 | 1 |
| [Data Pools](https://help.malighting.com/grandMA3/2.5/HTML/datapool.html) | [M03](M03-show-files.md)、[M18](M18-sessions-backup.md) | module_topics | 1 | 1 | 1 |
| [System](https://help.malighting.com/grandMA3/2.5/HTML/system_information.html) | [M20](M20-maintenance-diagnostics.md) | module_topics | 10 | 10 | 2 |
| [Remote In and Out](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs.html) | [M17](M17-remotes-api.md) | module_topics | 16 | 16 | 6 |
| [Sound](https://help.malighting.com/grandMA3/2.5/HTML/sound.html) | [M13](M13-timecode-audio.md) | module_topics | 3 | 3 | 3 |
| [RDM (Remote Device Management)](https://help.malighting.com/grandMA3/2.5/HTML/rdm.html) | [M04](M04-fixtures-patch.md)、[M16](M16-dmx-network.md) | module_topics | 1 | 1 | 1 |
| [Local Settings](https://help.malighting.com/grandMA3/2.5/HTML/local_settings.html) | [M02](M02-workspace-users.md)、[M20](M20-maintenance-diagnostics.md) | module_topics | 1 | 1 | 1 |
| [Update the Software](https://help.malighting.com/grandMA3/2.5/HTML/update.html) | [M20](M20-maintenance-diagnostics.md)、[M21](M21-version-baseline.md) | module_topics | 9 | 9 | 3 |
| [Fixture Types](https://help.malighting.com/grandMA3/2.5/HTML/fixture_types.html) | [M04](M04-fixtures-patch.md) | module_topics | 13 | 13 | 1 |
| [File Management](https://help.malighting.com/grandMA3/2.5/HTML/file_management.html) | [M03](M03-show-files.md)、[M20](M20-maintenance-diagnostics.md) | module_topics | 3 | 3 | 0 |
| [Show Creator](https://help.malighting.com/grandMA3/2.5/HTML/show-creator.html) | [M03](M03-show-files.md)、[M07](M07-presets-palettes.md)、[M11](M11-recipes-reuse.md) | module_topics | 7 | 7 | 3 |
| [Control other MA Devices](https://help.malighting.com/grandMA3/2.5/HTML/control_other_ma_devices.html) | [M16](M16-dmx-network.md)、[M17](M17-remotes-api.md)、[M20](M20-maintenance-diagnostics.md) | module_topics | 4 | 4 | 0 |
| [Troubleshooting](https://help.malighting.com/grandMA3/2.5/HTML/troubleshooting.html) | [M20](M20-maintenance-diagnostics.md) | module_topics | 5 | 5 | 2 |
| [Glossary](https://help.malighting.com/grandMA3/2.5/HTML/glossary.html) | [M22](M22-glossary-design.md) | navigation_reference | 1 | 1 | 0 |
| [grandMA3 List of Trademarks](https://help.malighting.com/grandMA3/2.5/HTML/key_grandma3_listoftrademarks.html) | [M22](M22-glossary-design.md) | navigation_reference | 1 | 1 | 0 |
| [grandMA3 Quick Start Guide](https://help.malighting.com/grandMA3/2.5/HTML/qsg.html) | [M02](M02-workspace-users.md)、[M04](M04-fixtures-patch.md)、[M06](M06-programmer-attributes.md)、[M08](M08-cues-tracking.md)、[M10](M10-effects-phasers.md)、[M11](M11-recipes-reuse.md)、[M16](M16-dmx-network.md) | tutorial_reference | 19 | 19 | 0 |
| [Quick Manuals grandMA3 Devices](https://help.malighting.com/grandMA3/2.5/HTML/quick-manuals-grandma3-devices.html) | [M01](M01-products-capacity.md)、[M20](M20-maintenance-diagnostics.md) | hardware_multilingual_reference | 414 | 0 | 0 |
| [grandMA3 Release Notes](https://help.malighting.com/grandMA3/2.5/HTML/release_notes.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 2.5](https://help.malighting.com/grandMA3/2.5/HTML/release-notes-2-5.html) | [M21](M21-version-baseline.md) | current_release | 8 | 8 | 6 |
| [Release Notes 2.4](https://help.malighting.com/grandMA3/2.5/HTML/release-notes-2-4.html) | [M21](M21-version-baseline.md) | historical_release | 7 | 7 | 0 |
| [Release Notes 2.3](https://help.malighting.com/grandMA3/2.5/HTML/key_releasenotes_2-3.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 2.2](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v2_2.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 2.1](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v2_1.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 2.0](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v2_0.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.9](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_9.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.8](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_8.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.7](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_7.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.6](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_6.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.5](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_5.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.4](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_4.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.3](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_3.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.2](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_2.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.1](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_1.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |
| [Release Notes 1.0](https://help.malighting.com/grandMA3/2.5/HTML/key_rn_v1_0.html) | [M21](M21-version-baseline.md) | historical_release | 1 | 1 | 0 |

### Avolites Titan

| 官方目录 | 研究模块 | 类型 | 索引 | 已获取 | 被模块引用 |
| --- | --- | --- | ---: | ---: | ---: |
| [Introduction](https://manual.avolites.com/docs/) | [M01](M01-products-capacity.md)、[M22](M22-glossary-design.md) | module_topics | 1 | 1 | 1 |
| [about-the-consoles](https://manual.avolites.com/docs/about-the-consoles) | [M01](M01-products-capacity.md)、[M20](M20-maintenance-diagnostics.md) | module_topics | 13 | 13 | 4 |
| [alpha-index](https://manual.avolites.com/docs/alpha-index) | [M22](M22-glossary-design.md) | module_topics | 1 | 1 | 0 |
| [capture-visualiser](https://manual.avolites.com/docs/capture-visualiser) | [M14](M14-preview-3d.md) | module_topics | 5 | 5 | 3 |
| [chases](https://manual.avolites.com/docs/chases) | [M08](M08-cues-tracking.md)、[M09](M09-playback-mixing.md) | module_topics | 7 | 7 | 1 |
| [controlling-fixtures](https://manual.avolites.com/docs/controlling-fixtures) | [M05](M05-selection-groups-layout.md)、[M06](M06-programmer-attributes.md) | module_topics | 6 | 6 | 4 |
| [cue-lists](https://manual.avolites.com/docs/cue-lists) | [M08](M08-cues-tracking.md)、[M09](M09-playback-mixing.md) | module_topics | 8 | 8 | 6 |
| [cues](https://manual.avolites.com/docs/cues) | [M08](M08-cues-tracking.md)、[M09](M09-playback-mixing.md) | module_topics | 7 | 7 | 2 |
| [effects](https://manual.avolites.com/docs/effects) | [M10](M10-effects-phasers.md)、[M11](M11-recipes-reuse.md)、[M12](M12-pixel-media.md) | module_topics | 7 | 7 | 4 |
| [fixture-personalities](https://manual.avolites.com/docs/fixture-personalities) | [M04](M04-fixtures-patch.md) | module_topics | 1 | 1 | 1 |
| [glossary](https://manual.avolites.com/docs/glossary) | [M22](M22-glossary-design.md) | module_topics | 1 | 1 | 0 |
| [networking](https://manual.avolites.com/docs/networking) | [M12](M12-pixel-media.md)、[M16](M16-dmx-network.md)、[M18](M18-sessions-backup.md) | module_topics | 6 | 6 | 2 |
| [palettes](https://manual.avolites.com/docs/palettes) | [M07](M07-presets-palettes.md) | module_topics | 6 | 6 | 4 |
| [patching](https://manual.avolites.com/docs/patching) | [M04](M04-fixtures-patch.md)、[M11](M11-recipes-reuse.md) | module_topics | 5 | 5 | 3 |
| [quick-start](https://manual.avolites.com/docs/quick-start) | [M02](M02-workspace-users.md)、[M04](M04-fixtures-patch.md)、[M06](M06-programmer-attributes.md)、[M08](M08-cues-tracking.md)、[M10](M10-effects-phasers.md) | module_topics | 7 | 7 | 0 |
| [remote-control](https://manual.avolites.com/docs/remote-control) | [M17](M17-remotes-api.md) | module_topics | 4 | 4 | 3 |
| [running-the-show](https://manual.avolites.com/docs/running-the-show) | [M09](M09-playback-mixing.md)、[M13](M13-timecode-audio.md)、[M17](M17-remotes-api.md)、[M19](M19-live-show.md) | module_topics | 6 | 6 | 5 |
| [synergy](https://manual.avolites.com/docs/synergy) | [M12](M12-pixel-media.md) | module_topics | 3 | 3 | 2 |
| [system-settings](https://manual.avolites.com/docs/system-settings) | [M02](M02-workspace-users.md)、[M03](M03-show-files.md)、[M16](M16-dmx-network.md)、[M20](M20-maintenance-diagnostics.md) | module_topics | 12 | 12 | 7 |
| [timelines](https://manual.avolites.com/docs/timelines) | [M13](M13-timecode-audio.md) | module_topics | 4 | 4 | 3 |
| [titan-basics](https://manual.avolites.com/docs/titan-basics) | [M02](M02-workspace-users.md)、[M03](M03-show-files.md)、[M15](M15-macros-automation.md)、[M20](M20-maintenance-diagnostics.md) | module_topics | 10 | 10 | 7 |
| [titan-net](https://manual.avolites.com/docs/titan-net) | [M01](M01-products-capacity.md)、[M18](M18-sessions-backup.md) | module_topics | 3 | 3 | 1 |
| [titan-reference](https://manual.avolites.com/docs/titan-reference) | [M15](M15-macros-automation.md) | module_topics | 2 | 2 | 1 |

## Titan API 参考

[19.2 API 索引](titan-api-index.json)保留 3,842 个唯一主题链接，覆盖方法、属性和类型等参考项目；已剔除无目标的目录分组链接。它不等于这么多个独立功能。API 总体模式和选定播放方法见 M15／M17。

## 补充官方来源

| 来源 | 用于模块 |
| --- | --- |
| [官方页面](https://api.avolites.com/19.2) | [M15](M15-macros-automation.md)、[M17](M17-remotes-api.md) |
| [官方页面](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.0.pdf) | [M21](M21-version-baseline.md) |
| [官方页面](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.1.pdf) | [M21](M21-version-baseline.md) |
| [官方页面](https://web3.avolites.com/Portals/0/Downloads/ReleaseNotes/TitanSuite/TitanReleaseNotesV19.2.pdf) | [M12](M12-pixel-media.md)、[M21](M21-version-baseline.md) |
| [官方页面](https://www.avolites.com/support/D9) | [M21](M21-version-baseline.md) |
| [官方页面](https://www.avolites.com/support/all-titan-pc-suite-downloads) | [M21](M21-version-baseline.md) |
| [官方页面](https://www.avolites.com/support/release-notes) | [M20](M20-maintenance-diagnostics.md) |
| [官方页面](https://www.malighting.com/downloads/products/grandMA3) | [M21](M21-version-baseline.md) |

## 维护方式

手工修改模块报告，再运行 `python3 docs/console-research/tools/build_catalogue.py` 重建总表与统计。脚本会拒绝重复编号、无证据条目、未映射的来源分类和不符合结构的功能行。

`source_index.py` 负责索引和按范围获取手册；正文只保存在系统临时目录。索引中的缓存键便于当前会话核查，不保证临时文件长期存在。版本更新时应明确切换研究基线并复核变化，不把下载成功当作研究完成。

两个 Python 文件仅是使用标准库的资料维护工具，不属于 StageMaster 产品业务代码，也不改变 Rust＋TypeScript 技术选型。
