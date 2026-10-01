# PRODUCT-ADR-079：可携带的灯具模式

状态：接受；2026-10-01；当前 Astra 自行评估；FIXTURE-004，基线 `875f107`。

M04-14 与 F02／工程交接已记录缺口。[MA3 灯型导出](https://help.malighting.com/grandMA3/2.5/HTML/ft_export.html)区分本机格式与 GDTF，[导入](https://help.malighting.com/grandMA3/2.5/HTML/ft_import.html)先进入模式库再配灯；[Titan](https://manual.avolites.com/docs/fixture-personalities/)将工程内定义与外部灯库更新隔离。本轮吸收独立携带、先检查再使用和不自动替换机制；复用既有 ProfileDefinition、严格 serde、Rust 编辑验证和原子存储，不自研 GDTF／OFL 解析器或宣称格式互通。完整个人库目录／云端同步后续。

模式文件是现有作者命令定义的版本化封装，JSON UTF-8，文件名 `.smfixture.json`：`{format:"stagemaster-fixture-profile",formatVersion:1,source:{profileId,revision},definition:ProfileDefinition}`。source 仅为来源记录（UUID），不是导入身份或真实性认证；没有路径、工程／灯具／配适／节目。定义与嵌套字段严格拒绝未知项／未支持属性，载荷上限 512 KiB、递归深度沿 serde 默认。读取后在独立临时 Document 内执行原 saveProfile／完整验证，无活动工程副作用。新版本必须显式识别，不能忽略能力。导出只针对可无损还原的现有受支持模式；重建与原档案去除 id／revision 后不同则拒绝，不截断未知元信息。

核心公开 `Document.profile_file(profile_id) -> ProfileFile` 与 `ProfileFile.decode/encode/definition/source`；文件携带自己完整定义，不引用外部资源。project-store 提供独立 ProfileFileStore 的有界读入及 .smfixture.json 目的保存，沿用基线／锁／临时提交；拒绝链接、非普通文件及覆盖工程／非模式文件。该小文件适配不扩大现有工程大小预算。

桌面新增 `profile_file_import(generation)` 返回 `{generation,fileName,source,definition}` 或 null；`profile_file_export(generation,profileId)` 返回 `{generation,profileId,revision,path:null|string,warning:null|string}`。单模式文件操作门与既有工程操作门序列化；Session 只在校验 generation／捕获源快照时短暂持锁，选择器和编解码在锁外。取消不改工程；读入不直接发编辑命令，不启动预演或设备。浏览器宿主明确未支持本地文件能力。

界面位于工程灯库：导入前收集原草稿；读入成功进入“导入模式”草稿，显示来源文件／来源修订，完整复用元数据／通道／功能编辑。用户保存时调用原 saveProfile(id=null)，生成新模式与新修订、一次历史；同名允许且提示已有同名，绝不猜测匹配替换灯具。取消不产生模式；未检查的导入草稿须显式保存到工程或取消，顶栏保存／导航拦截并定位草稿保存按钮；已有普通编辑草稿继续沿统一收集机制。异步回执校验挂载、页面和版本上下文，过时丢弃并提示重试。导出需先完成或取消当前模式草稿；返回来源模式／修订与路径，切换选择不冒充当前模式结果。

验收：不同粗细顺序／默认值／功能区间／双轴完整往返、未知／未来／超限／错误区间拒绝、来源与新身份分离、无源变更；保存冲突／链接／其他文件拒绝；原生取消、无效原草稿、导入检查编辑／取消／保存、同名提示、撤销／重做／重开、导入后既有配适和预演版本不改。所有操作使用项目内验收副本，不发送 DMX 或刷机。
