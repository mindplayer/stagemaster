import "./previs-transform.css";
import { useEffect, useState } from "react";
import type { PrevisTool } from "../../previs-types";

export function PrevisMoveTools({
  moving,
  tool,
  disabled,
  contextKey,
  onAction,
  onExact,
}: {
  moving: boolean;
  tool: PrevisTool;
  disabled: boolean;
  contextKey: string;
  onAction(action: string): void;
  onExact(yawDegrees: string, spacingScale: string): void;
}) {
  const [value, setValue] = useState("0");
  const [error, setError] = useState("");
  useEffect(() => {
    setValue(tool === "scale" ? "1" : "0");
    setError("");
  }, [contextKey, tool, moving]);
  if (!moving) return null;
  const exact = tool === "rotate" || tool === "scale";
  return (
    <>
      <span role="group" aria-label="三维布置工具">
        {(
          [
            ["horizontal", "水平", "moveHorizontal"],
            ["vertical", "升降", "moveVertical"],
            ["rotate", "整组旋转", "rotate"],
            ["scale", "间距缩放", "scale"],
          ] as const
        ).map(([id, label, action]) => (
          <button
            key={id}
            disabled={disabled}
            aria-pressed={tool === id}
            onClick={() => onAction(action)}
          >
            {label}
          </button>
        ))}
      </span>
      {exact && (
        <form
          className="previs-exact"
          aria-label="精确整组变换"
          onSubmit={(event) => {
            event.preventDefault();
            const n = Number(value),
              min = tool === "scale" ? 0.01 : -360,
              max = tool === "scale" ? 100 : 360;
            if (
              !/^-?\d+(\.\d{1,6})?$/.test(value) ||
              !Number.isFinite(n) ||
              n < min ||
              n > max
            ) {
              setError(
                tool === "scale"
                  ? "比例需为 0.01～100，最多六位小数"
                  : "角度需为 −360～360，最多六位小数",
              );
              return;
            }
            if (disabled) return;
            onExact(
              tool === "rotate" ? value : "0",
              tool === "scale" ? value : "1",
            );
            setError("");
          }}
        >
          <label>
            {tool === "rotate" ? "旋转角度（度）" : "间距比例"}
            <input
              type="text"
              aria-invalid={!!error}
              inputMode="decimal"
              value={value}
              disabled={disabled}
              onChange={(e) => {
                setValue(e.target.value);
                setError("");
              }}
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  e.stopPropagation();
                  setValue(tool === "scale" ? "1" : "0");
                  setError("");
                  onAction("cancel");
                }
              }}
            />
          </label>
          <button type="submit" disabled={disabled}>
            应用
          </button>
          {error && <span role="alert">{error}</span>}
        </form>
      )}
    </>
  );
}
