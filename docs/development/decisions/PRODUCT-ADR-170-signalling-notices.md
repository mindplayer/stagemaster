# PRODUCT-ADR-170：告知材料、来源核对与发行批准分开

状态：2026-10-05有限决定已实施并实际验证，结果 `e48d8c0`。关联[PREVIS-011](../tasks/PREVIS-011-signalling-notices.md)、[PREVIS-005](../tasks/PREVIS-005-signalling-component.md)、[PREVIS-010](../tasks/PREVIS-010-macos-distribution-preflight.md)。只是现行组件的私有构建交付，不改产品持久格式、模块依赖、运行权限或商业政策。

## 实际依据与成熟机制

现行锁124包，三项无独立许可文件的历史事实保持。cookie-signature@1.0.7原发布Readme.md内有完整告知，不等于所有README或MIT字段都足够。Epic common@0.1.0／signalling@0.2.0的npm版本元数据给出不同gitHead；各固定提交的package.json名称／版本吻合、上游LICENSE.md确实存在，须记录散列和来源，不消费浮动master。

原始来源：[common发布提交](https://raw.githubusercontent.com/EpicGames/PixelStreamingInfrastructure/15f96c6fb5bb0cacd3aee333bc618f92ef418612/Common/package.json)、[对应许可](https://raw.githubusercontent.com/EpicGames/PixelStreamingInfrastructure/15f96c6fb5bb0cacd3aee333bc618f92ef418612/LICENSE.md)、[signalling发布提交](https://raw.githubusercontent.com/EpicGames/PixelStreamingInfrastructure/6872b8a8f57cb83b3f084bf14a66c6b4e4b0d9c3/Signalling/package.json)、[对应许可](https://raw.githubusercontent.com/EpicGames/PixelStreamingInfrastructure/6872b8a8f57cb83b3f084bf14a66c6b4e4b0d9c3/LICENSE.md)。采用成熟的随包保留原告知／版本绑定／哈希核对机制，复用原锁和已有文件工具，不建设完整法律审批平台或重新实现依赖解析。

## 有限决定

1. 每项组件依赖保留原独立许可并定位到包路径／哈希；只为准确绑定的三个已核对缺独立文件项补足可查阅材料。cookie消费原README原文，Epic原样补充固定提交文本。所有来源说明与原文本分开，不改上游版权行。
2. 固定来源索引绑定名称、版本、锁完整性、文件哈希和原始来源。构建离线，未知／变更／畸形或无文本保持缺项，不能以字符串MIT或文件名默认取得合格状态；已有目标／越界／链接和超预算拒绝，不改依赖树。
3. 原PREVIS-005许可索引及needsLicenseReview语义保持；另交告知材料索引与阅读入口。新结果只说明材料存在／与核对来源一致，商业发行批准始终为false，仍需全产品许可及最终发行审查。
4. 仅原组装接入小模块，按职责分文件；原七项连接／隔离／EOF退出用例不改，实际新实例和移位重复验证。旧证据不写成当时已有这些材料，新包仍非客户资格，不重签官方Node或处理调试资格。

最终客户文件权限、Developer ID身份、Shipping／公证、UE与素材许可仍开放。Xcode系统临时例外和物理测量条件缺失只影响相应部分；本轮软件工作不扩大权限、操作真实设备或自动召回Astra，完整goal active。

## 实际有限结论

195 Node（28新增）、相关语法／格式通过，初始24项和自审2项真实失败保留后修复，未弱化原断言。原锁离线新组件与中文空格移位各原7项实际连接／EOF退出通过；只新增告知全文和索引两文件，其他2273文件等于原组件。Node原完整告知与124项依赖材料均有散列；cookie原README全文、两项Epic固定npm gitHead对应的原清单／许可字节实际匹配，旧三项待审标记保持。

同名其他版本不会借固定文本解除缺项，空白／畸形／超预算／来源篡改和覆盖明确拒绝。证据 `data/PREVIS-011/verification.json`及工单；材料缺项0仅为已核对组件的技术交付结论，**商业／全产品许可批准仍false**。实际Node证书／runtime／时间戳与get-task-allow=true均保持，本决定没有处置其调试资格或签名，也不解除PREVIS-010阻止报告、客户权限／Shipping／原生环境和其他产品出口。
