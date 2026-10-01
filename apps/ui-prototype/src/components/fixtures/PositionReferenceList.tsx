import type { PositionReferenceView } from "../../position-reference";
export function PositionReferenceList({
  record,
  disabled,
  onRemove,
  onClear,
}: {
  record?: PositionReferenceView;
  disabled: boolean;
  onRemove(id: string): void;
  onClear(): void;
}) {
  if (!record) return <p className="wb-dim">尚未记录参考点</p>;
  return (
    <section className="position-reference-list" aria-label="已记录参考点">
      <div className="wb-section-title">
        <h3>参考点 · {record.points.length}/16</h3>
        <button type="button" disabled={disabled} onClick={onClear}>
          清空参考点
        </button>
      </div>
      {!record.compatible && (
        <p className="wb-library-error">
          灯具档案已改变。保留旧记录供核对，清除后才能按新模式记录。
        </p>
      )}
      {record.points.map(({ point, check }) => (
        <article key={point.id}>
          <div className="wb-section-title">
            <strong>{point.name}</strong>
            <button
              type="button"
              disabled={disabled}
              aria-label={`删除参考点 ${point.name}`}
              onClick={() => onRemove(point.id)}
            >
              删除
            </button>
          </div>
          <p>
            目标：{point.targetMeters.x} / {point.targetMeters.y} /{" "}
            {point.targetMeters.z} 米
          </p>
          <p>
            轴设定：水平 {((point.panValue * 100) / 65535).toFixed(3)}% · 垂直{" "}
            {((point.tiltValue * 100) / 65535).toFixed(3)}%
          </p>
          {check.status === "checked" ? (
            <p>
              模型偏差{" "}
              {check.missMeters < 0.001
                ? "小于 0.001"
                : check.missMeters.toFixed(3)}{" "}
              米 · 夹角 {check.angleDegrees.toFixed(2)}°
              {check.distanceAlongMeters < 0 && " · 目标在光束背后"}
            </p>
          ) : (
            <p className="wb-dim">{check.reason}</p>
          )}
        </article>
      ))}
    </section>
  );
}
