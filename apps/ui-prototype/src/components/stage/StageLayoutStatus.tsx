export function StageLayoutStatus({
  kind,
  count,
  problem,
  computing = false,
}: {
  kind: "排列" | "挂接";
  count: number;
  problem: string;
  computing?: boolean;
}) {
  return (
    <div
      className="stage-arrangement-status"
      role="status"
      data-error={!!problem}
    >
      {problem
        ? `${kind}输入有误，平面显示已应用位置`
        : computing
          ? `正在计算${kind}位置，平面显示已应用位置`
          : `${kind}草稿 · ${count} 台 · 尚未应用`}
    </div>
  );
}
