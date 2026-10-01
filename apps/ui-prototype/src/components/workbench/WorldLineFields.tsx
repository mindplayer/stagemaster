import type { WorldLinePath } from "../../effect-types";

export function WorldLineFields({
  path,
  onChange,
}: {
  path: WorldLinePath;
  onChange(path: WorldLinePath): void;
}) {
  return (
    <>
      <p className="position-effect-note">
        世界坐标两点之间往返。灯间展开为 0 时，各灯追踪同一个目标点。
      </p>
      {(["fromMeters", "toMeters"] as const).map((point) => {
        const label = point === "fromMeters" ? "起点" : "终点";
        return (
          <section
            className="position-effect-axis"
            key={point}
            aria-label={`轨迹${label}`}
          >
            <strong>{label} · 米</strong>
            <div className="effect-fields">
              {(["x", "y", "z"] as const).map((axis, index) => (
                <label key={axis}>
                  {["左右 X", "前后 Y", "高度 Z"][index]}
                  <input
                    type="number"
                    required
                    min={-100000}
                    max={100000}
                    step={0.000001}
                    aria-label={`轨迹${label} ${axis.toUpperCase()}`}
                    value={path[point][axis]}
                    onChange={(e) =>
                      onChange({
                        ...path,
                        [point]: { ...path[point], [axis]: e.target.value },
                      })
                    }
                  />
                </label>
              ))}
            </div>
          </section>
        );
      })}
      <button
        type="button"
        onClick={() =>
          onChange({
            ...path,
            fromMeters: { ...path.toMeters },
            toMeters: { ...path.fromMeters },
          })
        }
      >
        交换起终点
      </button>
      <div className="effect-fields">
        <label>
          指向解分支
          <select
            aria-label="轨迹指向解分支"
            value={path.branch}
            onChange={(e) =>
              onChange({
                ...path,
                branch: e.target.value as WorldLinePath["branch"],
              })
            }
          >
            <option value="auto">自动选择</option>
            <option value="front">前向解</option>
            <option value="back">后向解</option>
          </select>
        </label>
        <label>
          允许误差 · 米
          <input
            type="number"
            required
            min={0.001}
            max={1}
            step={0.001}
            aria-label="轨迹允许误差"
            value={path.maxErrorMeters}
            onChange={(e) =>
              onChange({ ...path, maxErrorMeters: e.target.value })
            }
          />
        </label>
      </div>
      <p className="position-effect-note">
        误差包含轨迹近似和通道精度。不能替代实灯校准。
      </p>
    </>
  );
}
