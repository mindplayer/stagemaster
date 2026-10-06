import type { ExecutionView } from "../../execution-types";
import { sourceEvidenceRows } from "../../source-operation-evidence";
import "./media-request-details.css";

export function SourceRequestDetails({
  runtime,
  source,
}: {
  runtime: ExecutionView;
  source: string;
}) {
  const rows = sourceEvidenceRows(runtime, source);
  if (!rows.length) return null;
  return (
    <details className="media-request-details">
      <summary>最近节目控制详情</summary>
      <p>
        只保留最后一条进入客户端的明确节目控制，维护不覆盖。当次回执不是当前状态或实灯反馈；未确认时不要重复发送。
      </p>
      <dl>
        {rows.map(([label, value]) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </details>
  );
}
