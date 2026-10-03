# PRODUCT-ADR-147：三维灯位框选

2026-10-04，已接受；基线 8e8e4a4，main，当前会话单写者。实施见 STAGE-008。

参考 MA 官方 [Select Fixtures](https://help2.malighting.com/grandMA3/2.4/HTML/operate_select_fixtures.html) 的三维区域选灯和 Autodesk [Rectangular Selection Region](https://help.autodesk.com/cloudhelp/2026/ENU/3DSMax-Basics/files/GUID-944C8B75-EBE0-4078-9435-519602DFF30D.htm) 的矩形拖动反馈。复用 UE 的投影、射线与 Slate 绘制和既有 selectionGroup；不另建工程组或业务 API。本项目按灯位安装点的屏幕投影判定包含，不声称实现几何体相交／窗口完全包围或自由套索。

查看模式左键按下暂存源场地代次与修饰键；超过 3 像素显示矩形。释放才发布选择，点击仍按真实命中深度和既有修饰键切换单灯。框选方式下拉框明确替换／加选／减选，缺省替换；Shift 追加、Command／Ctrl 减去优先于下拉设置，单击仍保持既有语义；追加保留原灯序，新增按场地原灯序。默认只选未遮挡灯位，显式穿透设置可选镜头前且矩形内的被遮挡灯位；镜头后与视窗外排除。最大 1024 台，移动仍遵循 256 与锁定限制。

取消／失焦／出界／视口变化／镜头或工具变化／场地代次更新／主机选择变化取消未完成框选，原选择不变。框选不写工程、不动灯、不产生撤销条目；发布仍经现有异步选择、草稿提交和来源／连接作用域门。三维运动提案与灯光执行均不变。

周期 state 增加可选 marqueeMode（replace/add/remove，缺省 replace）和布尔 marqueeSupported、selectionThrough，新 UI 仅在能力存在时显示穿透开关并提示拖框；旧状态缺省 false，交互版本 3 保持。单字段动作 selectionThrough、marqueeReplace／marqueeAdd／marqueeRemove 只切换查看设置。裸 UE 与内嵌视窗共用手势实现，设置只驻留视窗。混合构件三维选择仍后续，不借本增量改变其身份协议。
