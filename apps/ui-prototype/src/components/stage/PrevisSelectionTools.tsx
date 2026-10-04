import type { MarqueeMode } from "../../previs-types";
import "./previs-selection.css";

/** Selection is view state; these controls never issue project edits. */
export function PrevisSelectionTools({
  supported,
  objectsSupported = false,
  allObjects = false,
  disabled,
  through,
  mode,
  onAction,
}: {
  supported: boolean;
  objectsSupported?: boolean;
  allObjects?: boolean;
  disabled: boolean;
  through: boolean;
  mode: MarqueeMode;
  onAction(action: string): void;
}) {
  if (!supported) return null;
  return (
    <>
      {objectsSupported && (
        <select
          className="previs-selection-mode"
          aria-label="三维选择范围"
          disabled={disabled}
          value={allObjects ? "all" : "fixtures"}
          onChange={(e) =>
            onAction(
              e.target.value === "all"
                ? "selectAllObjects"
                : "selectFixturesOnly",
            )
          }
        >
          <option value="fixtures">仅灯具</option>
          <option value="all">全部对象</option>
        </select>
      )}
      <select
        className="previs-selection-mode"
        aria-label="三维框选方式"
        disabled={disabled}
        value={mode}
        onChange={(e) =>
          onAction(
            {
              replace: "marqueeReplace",
              add: "marqueeAdd",
              remove: "marqueeRemove",
            }[e.target.value as MarqueeMode],
          )
        }
      >
        <option value="replace">框选：替换选择</option>
        <option value="add">框选：加选</option>
        <option value="remove">框选：减选</option>
      </select>
      <button
        disabled={disabled}
        aria-pressed={through}
        title="框选时包含被遮挡的对象中心或灯位"
        onClick={() => onAction("selectionThrough")}
      >
        穿透框选
      </button>
    </>
  );
}
