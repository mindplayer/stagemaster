export function FixtureLabelStatus({
  count,
  readOnly = false,
}: {
  count: number;
  readOnly?: boolean;
}) {
  return count > 0 ? (
    <p className="fixture-label-status">
      已省略 {count} 个重叠或超出边界的标签 ·
      {readOnly ? "可放大查看" : "可放大、搜索或选择对象查看"}
    </p>
  ) : null;
}
