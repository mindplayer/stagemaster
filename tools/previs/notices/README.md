# 信令组件的固定告知来源

这些材料只供现行Node／npm信令组件随包告知使用，不是全产品许可或商业发行批准。构建离线读取sources.json；不改变node_modules，不通过网络追随最新分支。

- Epic common@0.1.0：[npm版本元数据](https://registry.npmjs.org/%40epicgames-ps%2Flib-pixelstreamingcommon-ue5.8/0.1.0)的gitHead为15f96c6fb5bb0cacd3aee333bc618f92ef418612；[固定提交清单](https://raw.githubusercontent.com/EpicGames/PixelStreamingInfrastructure/15f96c6fb5bb0cacd3aee333bc618f92ef418612/Common/package.json)与实际安装清单逐字节相同，SHA-256为bdbf5e4731723e20659334714cde1b1a46086920109dccf7a4e6d0cac2fd6cbc。
- Epic signalling@0.2.0：[npm版本元数据](https://registry.npmjs.org/%40epicgames-ps%2Flib-pixelstreamingsignalling-ue5.8/0.2.0)的gitHead为6872b8a8f57cb83b3f084bf14a66c6b4e4b0d9c3；[固定提交清单](https://raw.githubusercontent.com/EpicGames/PixelStreamingInfrastructure/6872b8a8f57cb83b3f084bf14a66c6b4e4b0d9c3/Signalling/package.json)与实际安装清单逐字节相同，SHA-256为27e4ebb2c1ed6b2c42b0723480a05f7883576cc6c5e16f135a9e21e0bf9ad71f。
- 两项固定提交LICENSE.md原字节相同，1051字节、SHA-256为64cc3c15c799a5a75c92931a65ae253429b642563384797aa3ca9f2d3f1de26d；本目录epic-LICENSE.md保持原文和换行，不格式化或改版权行。每项补充告知分别保留对应提交URL。
- cookie-signature@1.0.7使用其原发布Readme.md全文，1490字节、SHA-256为1621ed10d0b2f865eb8608e0474a356cf7a9737a384b6593b61b30a9f6e50366；许可嵌在原文件中，不补造LICENSE或自动识别任意README。锁完整性与上述两项Epic一样固定在sources.json。

来源实际获取、npm版本完整性／清单对照与原文证据见项目data/PREVIS-011/；可复现固定URL及散列留本目录和Git。旧索引needsLicenseReview保持，新增索引只标材料是否已准备及其来源；Node调试资格、UE／素材和其他产品依赖的许可、正式权限／签名／公证及客户环境验收仍需单独完成。
