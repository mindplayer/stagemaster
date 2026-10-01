pub(super) const FORMAT: &str = "StageMaster 配灯表/1";
pub(super) const HEADERS: [&str; 28] = [
    "资料格式",
    "工程名称",
    "工程标识",
    "来源保存修订",
    "工程快照 SHA256",
    "灯具标识",
    "灯具名称",
    "厂家",
    "型号",
    "模式",
    "档案修订",
    "输出域",
    "输出域标识",
    "线路",
    "起始地址",
    "结束地址",
    "通道数",
    "配适状态",
    "布置状态",
    "所属空间",
    "支撑体",
    "世界 X（米）",
    "世界 Y（米）",
    "世界 Z（米）",
    "安装旋转 X（度）",
    "安装旋转 Y（度）",
    "安装旋转 Z（度）",
    "灯组",
];
pub(super) fn recognizes(bytes: &[u8]) -> bool {
    crate::report_csv::recognizes(bytes, &HEADERS, FORMAT)
}
