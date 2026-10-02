# HOST-002：实际工程与独立执行进程

状态：实施中；基线 `29e91e2`；main 工作区；当前 Astra 单写者，保留用户 `output/`。上一轮 HOST-001 已提交并验证，属于进展。本轮接续框架主线，不返回 AUDIO-020 局部界面。

依据 [ADR-099](../decisions/PRODUCT-ADR-099-local-execution-process.md)，新增独立程序及本机受保护入口，贯通保存工程到独立执行宿主。作用范围为 `apps/execution-host/`、工作区注册／锁文件、对应契约与状态；复用现有核心，不修改真实设备或已有窗口。

源码按准备、私有运行目录、协议值、会话／幂等、命令处理、状态投影、HTTP 与进程入口拆分。验收覆盖 ADR 全部门槛，重点使用真正的服务进程与独立客户端进程，不能拿同进程线程测试冒充崩溃隔离。

PLAN-004 补充：遵循 [ADR-100](../decisions/PRODUCT-ADR-100-composable-device-family.md) 的设备家族分工，本项只验证有操作系统的执行宿主和本机控制入口，不把 HTTP／Tokio 设为所有节点的共同依赖。开始时仅有空 Cargo 清单、无 Rust 源码；提前注册已撤回，实际实现时再加入可编译模块。复用工作区现有 Axum／Tokio／Serde 版本，不引入另一套 HTTP 服务栈。
