# 工程格式设计期检查

本工具检查 [0.1.0-draft.1 格式](../../docs/project-format/README.md) 和设计样例。它没有设备通信、播放、自动下载或授权功能，不作为产品读取器、Rust 领域校验或发布编译器的替代。

在本目录使用 Node.js 24+：

~~~sh
TMPDIR=/Users/sunqi/projects/stagemaster/tmp npm_config_cache=/Users/sunqi/projects/stagemaster/tmp/npm-cache npm ci --ignore-scripts --no-audit --no-fund
npm run check
npm test
~~~

运行 node check.mjs 后追加文件路径可以检查额外文档；跨文件检查必须同时传入对应工程和现场绑定。无参数检查仓库内五份示例。仅格式／部分引用检查，不会验证素材字节或网络目录；没有执行程序及签名授权的占位清单也可作为设计样例通过。

schemas/*.schema.json 是结构源；修改它们时同步规范和示例。check.mjs 的附加检查用来发现设计矛盾，覆盖清单见格式文档。后续 Rust 读取器应复用这些样例和非法变体作为验收资料，避免长期保留两套独立业务实现。

依赖固定版本与完整性摘要记录在 package-lock.json；Ajv 8.20.0（MIT）负责 JSON Schema 2020-12，jsonc-parser 3.3.1（MIT）负责严格解析前的标记／语法树检查。解析器虽支持 JSONC，本工具明确拒绝注释与尾逗号。校验失败退出码为 1；示例通过也不等于设备可播放。
