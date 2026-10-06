# PRODUCT-ADR-181：桌面预演的共同进程本地用户环境

状态：2026-10-06当前会话审定，先记录再实现；关联[PREVIS-014](../tasks/PREVIS-014-renderer-runtime-environment.md)。沿用[ADR-165](PRODUCT-ADR-165-packaged-renderer-file-access.md)的Foundation与UE目录分开、最小文件资格、失败不改变执行权威原则。

编辑器-game与独立Game沿用同一Rust Renderer，但共同路径没有Foundation用户环境，只有component设置CFFIXED_USER_HOME。DYLD清理又只包含继承项及两个显式固定项；Command里的其他显式加载路径会残留。原UserDir不等于Foundation路径，不能靠字符串或原component资格推断editor也已本地化。

共同configure_runtime在原全部祖先预检／目录准备成功后，设置RuntimePaths.user/platform-user，并清除父环境和该Command中全部DYLD_*键；不改变HOME、PATH、其他业务环境或读取凭据。保持原UE UserDir／本地缓存／DDC图，component原CFFIXED_USER_HOME相同，没有新的客户目录／权限决定或迁移。Editor原UE_SKIP_UBT_SDK_SETUP仍只用于预编译-game运行，component继续删除，构建SDK验证不受影响。

方法复用原component本地Foundation行为和原Loader去污染；以实际Command红绿和既有系统Foundation子进程验证边界，不引入依赖、代理／通用环境平台或全局偏好改写。App Sandbox与签名仍由既有组装方案限定，设置用户目录不代表新增权限；NSTemporaryDirectory等系统管理例外继续单列，不能声称第三方零系统临时写入。

本项只收敛文件环境归属，不自动终止桌面崩溃报告进程、不证明UE内部问题修复、GPU／客户发行／真实输出或当前完整组合通过。实际权限与重签授权尚未获得的出口仍暂停，其他普通软件实施自主继续。
