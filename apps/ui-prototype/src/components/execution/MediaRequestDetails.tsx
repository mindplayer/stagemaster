import type { ExecutionView } from "../../execution-types";
import { mediaEvidenceRows } from "../../media-operation-evidence";
import "./media-request-details.css";

export function MediaRequestDetails({
  runtime,
  group,
}: {
  runtime: ExecutionView;
  group: string;
}) {
  const rows = mediaEvidenceRows(runtime, group);
  if (!rows.length) return null;
  return (
    <details className="media-request-details">
      <summary>最近音乐请求详情</summary>
      <p>
        只保留最后一条进入客户端的显式音乐操作；维护不覆盖。详情不是播放或实灯反馈。
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
