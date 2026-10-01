import { SeatingLayoutPreview } from "./SeatingLayoutPreview";
import type { SeatingShape, StageSpace } from "../../stage-types";
import { seatingLayout } from "../../seating-tools";
import { displayMeters } from "./stage-display";
import "./seating.css";
export function SeatingFields({
  value: s,
  spaces,
  onChange,
}: {
  value: SeatingShape;
  spaces: StageSpace[];
  onChange(value: SeatingShape): void;
}) {
  const change = (patch: Partial<SeatingShape>) => onChange({ ...s, ...patch });
  const layout = seatingLayout(s);
  const number = (
    label: string,
    value: string | number,
    update: (v: string) => void,
    min: number,
    max: number,
    step = "any",
  ) => (
    <label>
      {label}
      <input
        aria-label={label}
        type="number"
        required
        step={step}
        min={min}
        max={max}
        value={value}
        onChange={(e) => update(e.target.value)}
      />
    </label>
  );
  return (
    <div className="seating-fields">
      <div className="seating-summary" aria-live="polite">
        {layout ? (
          <>
            <strong>{s.rows * s.columns} 个座位</strong>
            <span>
              {displayMeters(layout.width)} × {displayMeters(layout.depth)} 米
            </span>
          </>
        ) : (
          <span>请检查座数、间距、通道和半径；座椅不能重叠</span>
        )}
      </div>
      <SeatingLayoutPreview shape={s} />
      <div className="seating-arrangement">
        <span>排列方式</span>
        <div role="group" aria-label="座区排列方式">
          <button
            type="button"
            aria-pressed={!s.arc}
            onClick={() => change({ arc: null })}
          >
            直排
          </button>
          <button
            type="button"
            aria-pressed={!!s.arc}
            onClick={() => {
              if (!s.arc) change({ arc: { radiusMeters: "5" } });
            }}
          >
            弧形
          </button>
        </div>
      </div>
      {s.arc && (
        <>
          {number(
            "前排半径（米）",
            s.arc.radiusMeters,
            (v) => change({ arc: { radiusMeters: v } }),
            1,
            10000,
          )}
          {layout?.worldFocus && (
            <small>
              共同焦点 X {displayMeters(layout.worldFocus[0])} / Y{" "}
              {displayMeters(layout.worldFocus[1])} 米
            </small>
          )}
        </>
      )}
      <div className="stage-pair">
        {number(
          "排数",
          s.rows || "",
          (v) => change({ rows: Number(v) }),
          1,
          Math.min(64, Math.floor(512 / (s.columns || 1))),
          "1",
        )}
        {number(
          "每排座位",
          s.columns || "",
          (v) => change({ columns: Number(v) }),
          1,
          Math.min(64, Math.floor(512 / (s.rows || 1))),
          "1",
        )}
      </div>
      <div className="stage-pair">
        {number(
          "椅宽（米）",
          s.seatWidthMeters,
          (v) => change({ seatWidthMeters: v }),
          0.3,
          1.2,
        )}
        {number(
          "椅深（米）",
          s.seatDepthMeters,
          (v) => change({ seatDepthMeters: v }),
          0.3,
          1.2,
        )}
      </div>
      <div className="stage-pair">
        {number(
          s.arc ? "前排弧长中心距（米）" : "座椅中心距（米）",
          s.columnSpacingMeters,
          (v) => change({ columnSpacingMeters: v }),
          Number(s.seatWidthMeters) || 0.3,
          5,
        )}
        {number(
          s.arc ? "径向排中心距（米）" : "排中心距（米）",
          s.rowSpacingMeters,
          (v) => change({ rowSpacingMeters: v }),
          Number(s.seatDepthMeters) || 0.3,
          10,
        )}
      </div>
      <label className="seating-aisle-toggle">
        <input
          type="checkbox"
          checked={!!s.aisle}
          aria-label="设置纵向通道"
          onChange={(e) =>
            change({
              aisle: e.target.checked
                ? {
                    afterColumn: Math.max(1, Math.floor(s.columns / 2)),
                    widthMeters: String(
                      Math.max(
                        1.2,
                        Number(s.columnSpacingMeters) -
                          Number(s.seatWidthMeters),
                      ),
                    ),
                  }
                : null,
            })
          }
        />
        纵向通道
      </label>
      {s.aisle && (
        <div className="stage-pair">
          {number(
            "通道位于第几列后",
            s.aisle.afterColumn || "",
            (v) => change({ aisle: { ...s.aisle!, afterColumn: Number(v) } }),
            1,
            Math.max(0, s.columns - 1),
            "1",
          )}
          {number(
            "通道净宽（米）",
            s.aisle.widthMeters,
            (v) => change({ aisle: { ...s.aisle!, widthMeters: v } }),
            Math.max(
              0.3,
              Number(s.columnSpacingMeters) - Number(s.seatWidthMeters),
            ),
            10,
          )}
        </div>
      )}
      <div className="stage-pair">
        {number(
          "座区中心 X（米）",
          s.positionMeters.x,
          (v) => change({ positionMeters: { ...s.positionMeters, x: v } }),
          -100000,
          100000,
        )}
        {number(
          "座区中心 Y（米）",
          s.positionMeters.y,
          (v) => change({ positionMeters: { ...s.positionMeters, y: v } }),
          -100000,
          100000,
        )}
      </div>
      <div className="stage-pair">
        {number(
          "座区地面标高（米）",
          s.positionMeters.z,
          (v) => change({ positionMeters: { ...s.positionMeters, z: v } }),
          -100000,
          99999.15,
        )}
        {number(
          "座区朝向（度）",
          s.yawDegrees,
          (v) => change({ yawDegrees: v }),
          -3600,
          3600,
        )}
      </div>
      <small>0° 面向平面图上方，正角度逆时针旋转。</small>
      <label>
        所属空间
        <select
          aria-label="座区所属空间"
          value={s.spaceId ?? ""}
          onChange={(e) => change({ spaceId: e.target.value || null })}
        >
          <option value="">未归属</option>
          {spaces.map((space) => (
            <option key={space.id} value={space.id}>
              {space.name}
            </option>
          ))}
        </select>
      </label>
    </div>
  );
}
