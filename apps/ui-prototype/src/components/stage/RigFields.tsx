import type { RigShape, StageSpace } from "../../stage-types";
export function RigFields({
  value,
  spaces,
  onChange,
}: {
  value: RigShape;
  spaces: StageSpace[];
  onChange(value: RigShape): void;
}) {
  const change = (patch: Partial<RigShape>) => onChange({ ...value, ...patch });
  const field = (
    label: string,
    value: string,
    change: (s: string) => void,
    min = -100000,
    max = 100000,
  ) => (
    <label>
      {label}
      <input
        aria-label={label}
        type="number"
        step="any"
        required
        min={min}
        max={max}
        value={value}
        onChange={(e) => change(e.target.value)}
      />
    </label>
  );
  return (
    <div className="rig-fields">
      <div className="stage-pair">
        <label>
          支撑体类型
          <select
            aria-label="支撑体类型"
            value={value.rigKind}
            onChange={(e) =>
              change({ rigKind: e.target.value as RigShape["rigKind"] })
            }
          >
            <option value="truss">直线桁架</option>
            <option value="pipe">灯杆</option>
          </select>
        </label>
        <label>
          所属空间
          <select
            aria-label="支撑体所属空间"
            value={value.spaceId ?? ""}
            onChange={(e) => change({ spaceId: e.target.value || null })}
          >
            <option value="">未归属</option>
            {spaces.map((s) => (
              <option value={s.id} key={s.id}>
                {s.name}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="stage-pair">
        {field(
          "长度（米）",
          value.lengthMeters,
          (v) => change({ lengthMeters: v }),
          0.1,
          1000,
        )}
        {field(
          "水平角（度）",
          value.yawDegrees,
          (v) => change({ yawDegrees: v }),
          -3600,
          3600,
        )}
      </div>
      <div className="stage-pair">
        {field(
          "截面宽（米）",
          value.widthMeters,
          (v) => change({ widthMeters: v }),
          0.02,
          10,
        )}
        {field(
          "截面高（米）",
          value.heightMeters,
          (v) => change({ heightMeters: v }),
          0.02,
          10,
        )}
      </div>
      <div className="stage-pair">
        {field("中心 X（米）", value.positionMeters.x, (v) =>
          change({ positionMeters: { ...value.positionMeters, x: v } }),
        )}
        {field("中心 Y（米）", value.positionMeters.y, (v) =>
          change({ positionMeters: { ...value.positionMeters, y: v } }),
        )}
      </div>
      {field("中心标高（米）", value.positionMeters.z, (v) =>
        change({ positionMeters: { ...value.positionMeters, z: v } }),
      )}
    </div>
  );
}
