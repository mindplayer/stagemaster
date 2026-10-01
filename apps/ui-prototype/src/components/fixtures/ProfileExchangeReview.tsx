import type {
  ProfileExchangeReview as Review,
  ColorSlotChange,
} from "../../profile-exchange-review";
import { WheelSwatch } from "./WheelSwatch";
import "./profile-exchange-review.css";

function Slot({ value }: { value: ColorSlotChange["before"] }) {
  return (
    <span className="profile-slot-version">
      <span>
        <WheelSwatch value={value.appearance} /> {value.name}
      </span>
      <span>
        区间 {value.dmxFrom}–{value.dmxTo} · 代表值 {value.dmxDefault}
      </span>
    </span>
  );
}
export function ProfileExchangeReview({
  review,
  targetName,
  accepted,
  onAccept,
}: {
  review: Review;
  targetName: string;
  accepted: boolean;
  onAccept(value: boolean): void;
}) {
  return (
    <section className="profile-exchange-review" aria-label="色盘版本差异">
      <h4>色盘差异 · 目标：{targetName}</h4>
      <div
        className="profile-exchange-changes"
        tabIndex={0}
        aria-label="滚动查看色盘差异"
      >
        {review.groups.map((group) => (
          <div key={group.sourceId}>
            <strong>原模式：{group.sourceName}</strong>
            <p>
              {group.fixtures.map((f) => f.name).join("、")}（
              {group.fixtures.length} 台）
            </p>
            {group.changes.length ? (
              <table>
                <thead>
                  <tr>
                    <th scope="col">原档位</th>
                    <th scope="col">新档位</th>
                  </tr>
                </thead>
                <tbody>
                  {group.changes.map((change) => (
                    <tr key={change.before.key}>
                      <td>
                        <Slot value={change.before} />
                      </td>
                      <td>
                        <Slot value={change.after} />
                        {change.remap && <b>通道值改变</b>}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <p className="wb-dim">色盘档位没有变化</p>
            )}
          </div>
        ))}
      </div>
      {review.needsRemap && (
        <label className="patch-check">
          <input
            type="checkbox"
            name="colorRemap"
            checked={accepted}
            onChange={(e) => onAccept(e.target.checked)}
          />
          采用新版本的固定色盘通道值
        </label>
      )}
      {review.needsRemap && (
        <p className="wb-dim">
          已有场景和预设保留档位对应关系，所选灯具改用新代表值。请先核对实灯色盘。
        </p>
      )}
    </section>
  );
}
