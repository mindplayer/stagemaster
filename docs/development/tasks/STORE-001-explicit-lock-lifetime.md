# STORE-001：文件锁生命周期与并行子进程

状态：已完成；基线 `abc5f7a`，主工作区当前会话。FIXTURE-003B 全量回归再次触发此前 DEVICE-002B 记录的目录锁偶发占用，原失败日志保留于 `logs/FIXTURE-003B/workspace-1.log`。仅检查／修复文件适配层，不动 Flash、蓝牙、真实输出或持久化格式。

假设：并行逐点退出测试启动子进程期间，重复文件描述符延长了原锁的生存期。现有适配层只靠关闭最后一个文件描述符解锁；关闭本地句柄不必然立即释放锁。[Apple flock 文档](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html)描述 fork／dup 的共享锁，[Rust File 文档](https://doc.rust-lang.org/std/fs/struct.File.html#method.lock)说明显式解锁与句柄关闭行为。先以重复描述符构造确定性负例，再用拥有锁的 RAII 对象明确释放，保持真正竞争者拒绝和独立读源租约。

不通过减少测试线程、增加重试或放松独占性来规避。具体偶发失败是否全由该竞争引起需结合复现与回归证据，不仅凭一次通过下结论。

确定性负例已在修改前失败（`storage-duplicate-before.log`），与原“对象已 drop 但目录仍被占用”症状一致。新增私有 Lease 在 Drop 显式 unlock；独占写入锁最后释放，临时文件先清理再释放槽租约。没有向调用方暴露可复制句柄。另测两个独立共享读源、重复读源描述符、重复暂存描述符，验证只有持有对象结束才解除它的锁且不能释放另一个独立读源。存储整组 14 项通过（1 个受父测试调用的中断工作器仍按原设计忽略），随后工作区回归的本模块 14 项也通过，严格工作区 Clippy 通过；该回归后来停在本轮新增灯具测试的调用参数错误，未作为全量通过。具体偶发 fork 时刻尚无系统追踪证据，因此不宣称已证明所有历史失败都属于这一个原因。

审查结论：私有锁封装不改变存储格式、互斥准入或读源保留契约；退出仍由系统释放资源，正常析构主动终止已结束的租约。测试对私有 `_writer`／`_lease` 复制为刻意的竞争复现，仅该测试模块说明并允许 underscore-binding lint。结果为本次 `fix(storage): release owned leases before incidental descriptors close` 提交。
