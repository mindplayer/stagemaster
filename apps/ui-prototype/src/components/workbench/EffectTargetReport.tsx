import type { EffectTargetIssue } from "../../effect-targets";

export function EffectTargetReport({
  issues,
}: {
  issues: EffectTargetIssue[];
}) {
  if (!issues.length) return null;
  return (
    <div className="effect-target-report" role="status">
      <strong>{issues.length} 项目标问题</strong>
      <ul>
        {issues.slice(0, 5).map((issue, i) => (
          <li key={`${issue.id}:${i}`}>
            {issue.name}：{issue.reason}
          </li>
        ))}
      </ul>
      {issues.length > 5 && (
        <small>另有 {issues.length - 5} 项，请调整目标灯具。</small>
      )}
    </div>
  );
}
