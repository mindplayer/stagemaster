import { LockSimpleIcon, LockSimpleOpenIcon } from "@phosphor-icons/react";
import type { StageSelection, StageView } from "../../stage-types";
import { isStageLocked } from "../../stage-locks";
import "./stage-locks.css";
export function StageLockControls({
  stage,
  targets,
  busy,
  onLock,
}: {
  stage: StageView;
  targets: StageSelection[];
  busy: boolean;
  onLock(locked: boolean): void;
}) {
  const count = targets.filter((t) => isStageLocked(stage, t)).length;
  const all = count === targets.length;
  return (
    <div className="stage-lock-controls" aria-label="场地编辑保护">
      <div>
        <span role="status">
          {count ? <LockSimpleIcon /> : <LockSimpleOpenIcon />}
          {targets.length === 1
            ? count
              ? "已锁定"
              : "可编辑"
            : `已锁定 ${count} / ${targets.length} 台`}
        </span>
        <div className="stage-lock-buttons">
          <button type="button" disabled={busy} onClick={() => onLock(!all)}>
            {targets.length > 1
              ? all
                ? "全部解锁"
                : "全部锁定"
              : all
                ? "解锁"
                : "锁定"}
          </button>
          {!!count && !all && (
            <button type="button" disabled={busy} onClick={() => onLock(false)}>
              {targets.length > 1 ? "全部解锁" : "解锁"}
            </button>
          )}
        </div>
      </div>
      {!!count && (
        <small>
          {targets[0]?.kind === "placement"
            ? "灯位固定，灯光编排仍可调整"
            : "几何与属性固定，仍可选择查看"}
        </small>
      )}
    </div>
  );
}
