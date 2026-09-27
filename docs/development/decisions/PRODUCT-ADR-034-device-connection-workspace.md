# PRODUCT-ADR-034：主机设备连接与原生蓝牙边界

日期：2026-09-28；状态：接受，DEVICE-001 实施中。

## 决定

沿用 ADR-014 的 20 字节诊断协议。`stagemaster-device-link` 提供无 I/O 的主机回执／诊断解析，与设备 Session 共用协议；独立主机连接模块拥有发现、连接生命周期、单调时钟、超时和心跳。Tauri 只负责命令适配，React 只持有查看／搜索选择状态，切页或关闭设备面板不销毁会话。连接不属于工程，不进入工程文件、撤销或播放时钟。

使用 `btleplug` 原生适配 CoreBluetooth／WinRT／BlueZ，而不是自行实现系统 BLE、把 Python 探针打包进产品或由网页直接操作 GATT。版本锁定，当前验收 macOS；库支持其他平台不等于本产品已完成其他平台权限、打包和后台运行验收。网页宿主明确报告连接不可用，未来远程网关仍走应用接口。

仅发现诊断服务，限制候选数量、操作时间和单连接；每次扫描的标识只用作当前连接句柄，不持久绑定随机地址。状态带递增修订；命令带操作代号拒绝旧窗口的迟到取消／断开／连接。取消进入资源清理阶段，底层连接／扫描完成清理后才允许下一操作，避免迟到系统回调接管新任务。未验证握手和诊断前不报告已连接；只认当前请求对应的应用回执，过期不得通过读取或错误消息续期。

本轮不实现跨连接身份认证、控制租约、节目传输、Flash、许可或 DMX。已有设备运行／安装模块不依赖此诊断 UI，后续能力协商及正式设备身份另立契约；名称／地址／随机会话不是认证。

## 参考与复用

- [btleplug 官方](https://github.com/deviceplug/btleplug)：复用跨平台异步原生 BLE 适配；平台状态／支持差异由适配层收敛，不泄漏至工程核心。
- [官方读写／通知示例](https://github.com/deviceplug/btleplug/blob/master/examples/subscribe_notify_characteristic.rs)：复用带响应写入与服务特征发现；现有协议允许读取单格回执，本产品必须关联请求，不能把 ATT 成功当业务成功。
- [Apple 蓝牙用途声明](https://developer.apple.com/documentation/bundleresources/information-property-list/nsbluetoothalwaysusagedescription)：应用包提供中文用途，权限由系统处理；不伪造已获权限状态。

## 验证记录

`btleplug 0.13.2` 锁定。285 项工作区测试、74 项 UI 测试、严格 Clippy、类型与真实桌面构建通过；独立原生 QA 窗口已验证空工程入口、查询保持、Esc 焦点恢复及工程草稿不受影响。

实际发现 macOS 首次授权可能等待超过初始化超时。适配层已保留初始化 Future，避免重试泄漏多个中央管理器；准备与扫描阶段分开。核对库源码后，重连通过 `retrieve_peripherals` 重新取得已被 CoreBluetooth 释放的外设；真实重连待权限确认后验证。原生工具拒绝访问系统权限确认窗口，已请用户亲自授权；未以其他工具绕过。具体证据与余项在工单，不把库支持范围当实测结果。
