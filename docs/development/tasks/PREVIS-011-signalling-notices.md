# PREVIS-011：信令组件的可追溯第三方告知

状态：**有限告知材料增量完成，自审通过**，2026-10-05。基线 main `c8ea5eb04d2b5236cdad6a68bd0c061cea40841a`，计划／[ADR-170](../decisions/PRODUCT-ADR-170-signalling-notices.md)先提交 `d6401d499a230f6f143f53b47523980289a4bee1`，实现结果 `e48d8c077a74ee314d373b34b733c205a1ed42b3`；主工作区单写者。接续 H5 的真实许可材料缺口，不改变客户权限／签名／Shipping决定，不因其缺条件停掉独立软件工作；用户output/及旧包／证据保持。

## 一个问题与可用增量

PREVIS-005保留全部npm发布内容与124项许可元数据，但三项没有独立LICENSE文件，现有索引无法直接定位供客户查阅的完整告知。实际核对cookie-signature@1.0.7完整许可已随Readme.md发布；两项Epic包则未带独立文本，应以各自npm发布gitHead对应的上游许可补充，不能凭MIT字段或最新master推断已交付告知。

- 只改现有信令组件组装接线，新增独立告知收集／验证及测试，必要的上游原样许可和固定来源索引留tools/previs/；不新增播放器、签名器、依赖解析器或替代组装平台。
- 复用lockedPackages／packageLicenses／fileHash／严格JSON。全部原独立许可原样保留、散列定位；cookie只使用准确版本／锁完整性和文本哈希绑定的原README，不泛化为“文件名像README就许可合格”。两项Epic许可来自各自固定gitHead，记录npm完整性／上游URL／哈希；运行组装必须离线消费仓库资料，不动态抓取最新版。
- 告知清单与可阅读材料随组件licenses/交付，明确原包文本／已核对嵌入文本／固定上游补充／仍缺项；不修改node_modules、不回写原needsLicenseReview，不将“材料已准备”当法律或商业发行批准。版本／完整性／原文本／来源哈希不符、空／非法UTF-8、链接越界、超预算、重复／路径穿越及已有目标拒绝且定位；不静默猜许可或覆盖。
- 只新增私有构建产物格式，公共工程／协议／核心语义／控制权／时钟／依赖锁不变。无Rust／UI／UE／固件或现行客户包改变；不重签Node、不启用Game、不修改沙盒、不操作私钥／公证／设备。

## 验收与失败行为

先对旧“独立LICENSE列表足够”的行为补红灯，原断言不弱化；覆盖嵌入文本、两个固定上游版本、元数据不等于文本、缺项保持、篡改／换版本／完整性、文本与路径保护、确定性输出、取消式已有目录拒绝／写入失败不登记成功。当前全部previs Node保护与相关语法／格式、文档引用／严格JSON／diff实际执行。

实际用原Node／现行锁重新在唯一项目内目录离线组装，原目录与中文空格移位各执行原7项真实回环／EOF生命周期断言，核对新增告知材料／索引与原许可逐字节相同、两目录清单等价、所属进程／端口退出。旧源依赖树、原Game／Node、007／009资格包、受保护工程／默认最近目录／历史证据保持；原Node调试资格仍未处理。不重跑无源码变更的Rust／UI／UE或冒记听音／GPU／物理通过。

基线、实际上游获取与失败、测试、组装清单／自审结论保存data/PREVIS-011/／logs/previs-011-*，临时目录显式项目tmp/。共用工具原命名生成data/PREVIS-005/下全新实例，由011记录准确引用，旧实例不改名或回写。完成后更新STATE／工单并提交；完整H1–H5 goal保持active，不扩H6。

下一步仍为客户权限／签名／Shipping及实际无编辑器客户验收；本任务只准备Node与信令许可材料，不覆盖Rust／UI／UE／素材的全部发行清单，不选择法律政策或接受新商业条款。

## 实际交付与自审

现有package-signalling入口仅两行接线；新增收集／文件保护／测试夹具与两份测试，分别178／129／75／166／123行，全部职责内聚且小于300行。固定来源索引和原Epic许可／说明留[告知材料目录](../../../tools/previs/notices/README.md)；固定文本不机械格式化，运行构建不联网找新版本。

- 初始旧“独立文件目录足够”的接缝实际26项：2通过／24断言失败；准确红灯源码与测试哈希在补丁前记录red-result.json，没有把绿灯后哈希叫红灯快照。原26项实现后通过；自审新增同名嵌套不同版本及纯空白两项失败，修复后最终全部 **195 Node／195通过，28新增，0跳过**。原输出链接测试的缺清单前提已换成真实有效组件／链接licenses，不用缺清单偶然拒绝证明链接保护。
- 六JS实际语法和七项相关Prettier（含固定来源JSON）通过；源码与暂存差异检查通过。文本512KiB、合并8MiB、单包16份、256锁包预算；固定缓冲多读一字节检测增长、空白／非法UTF-8／NUL拒绝。补充只匹配准确名称／版本／许可／锁完整性，嵌套其他版本保持缺项；原文散列不符、浮动上游、越界／链接、旧目标覆盖和合并预算拒绝，失败不返回成功。未知材料继续missing，不提升到商业批准。
- 原Node与原锁实际离线新建 `data/PREVIS-005/previs-signalling-sVNMaV/previs/`，安装124包；包内Node原目录与中文空格移位副本各 **原7项真实连接／发现／EOF退出与两端口回收**通过。新组件2275文件：仅增加`licenses/THIRD-PARTY-NOTICES.txt`／`licenses/notices.json`，其余2273文件与PREVIS-005原最终实例逐项字节等价；两目录完整清单与告知相同。未将本轮告知追写到旧包，原组装005命名空间由011的assembly-ref.json说明。
- 全文 **314482字节**，含Node原完整告知和124项依赖文本；124项来源散列逐项核对，材料缺项0，但旧三项needsLicenseReview仍true。cookie是原1490字节README完整保留，不生成假的独立LICENSE；两项Epic的npm版本完整性与各固定gitHead吻合，上游package.json与实际安装文件原字节一致、两个LICENSE.md均1051字节且等于仓库固定副本。材料数量／MIT字段不等于全产品合法发行结论。
- 新包Node与原运行时散列相同，实际只读严格签名及独立Developer ID校验退出0，Team HX7739G8FX／runtime／安全时间戳保持；原get-task-allow=true仍存在，未重签或删除资格。源node_modules2267文件、原Game／Node及007／009资格包清单、受保护工程／最近目录和009五Rust源码保持；所属新Node进程全部结束，用户output/未动，无UE／音频／设备／权限／证书操作。

证据 `data/PREVIS-011/verification.json`、`assembly-ref.json`、`source-observations.json`、原始registry／固定上游清单／文本、`red-result.json`与`delivery-checks.json`；日志 `logs/previs-011-notices-red.log`／`notices-green.log`／`review-red.log`／`all-node-final.log`／`format-final.log`／`package.log`／`verification-final.log`（均previs-011前缀）及真实实例 `logs/PREVIS-005/previs-signalling-sVNMaV/`。第一次探索URL将非scope包版本分隔符编码，实际HTTP405但未保存独立原始记录，不能冒作已保存；后继正确版本原始获取已保存。首汇总导入猜测的解析器名退出1日志保留，改用真实export后完成，未更改产品源码来迁就证据脚本。

无Rust／UI／UE／固件源码改变，不重复其编译／原生GPU或听音；本轮真实回环不当作灯光物理通过。仅收敛信令告知交付，customerPackageVerified和commercialReleaseApproved保持false。下一步最终客户目录／权限、Node调试资格处置／签名与Shipping；系统临时例外、完整产品许可／客户GPU／公证／外部代表任务及H1／H3／H4真实门槛保持，完整goal active。
