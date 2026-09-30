import { useRef, type KeyboardEvent } from "react";
import { panelBounds, panelSize } from "./layout-preferences";

export function PanelDivider({
  panel,
  value,
  onChange,
}: {
  panel: keyof typeof panelBounds;
  value: number;
  onChange(value: number): void;
}) {
  const drag = useRef<{ start: number; value: number; size: number } | null>(
    null,
  );
  const horizontal = panel === "editor";
  const sign = panel === "library" ? 1 : -1;
  const [min, max] = panelBounds[panel];
  function cancel() {
    if (drag.current) onChange(drag.current.value);
    drag.current = null;
  }
  function key(event: KeyboardEvent) {
    if (event.key === "Escape") {
      cancel();
      event.preventDefault();
      return;
    }
    const delta = ["ArrowLeft", "ArrowUp"].includes(event.key)
      ? -1
      : ["ArrowRight", "ArrowDown"].includes(event.key)
        ? 1
        : 0;
    if (!delta && event.key !== "Home" && event.key !== "End") return;
    event.preventDefault();
    onChange(
      event.key === "Home"
        ? min
        : event.key === "End"
          ? max
          : panelSize(panel, value + delta * sign * (event.shiftKey ? 40 : 10)),
    );
  }
  return (
    <div
      className={`panel-divider divider-${panel}`}
      role="separator"
      tabIndex={0}
      aria-label={
        horizontal
          ? "调整编排区高度"
          : panel === "library"
            ? "调整资源区宽度"
            : "调整属性区宽度"
      }
      aria-orientation={horizontal ? "horizontal" : "vertical"}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={Math.round(value)}
      onKeyDown={key}
      onBlur={cancel}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        event.preventDefault();
        event.currentTarget.focus();
        const region = event.currentTarget.parentElement
          ?.querySelector(`.region-${panel}`)
          ?.getBoundingClientRect();
        drag.current = {
          start: horizontal ? event.clientY : event.clientX,
          value,
          size: region ? (horizontal ? region.height : region.width) : value,
        };
        event.currentTarget.setPointerCapture(event.pointerId);
      }}
      onPointerMove={(event) => {
        if (drag.current)
          onChange(
            panelSize(
              panel,
              drag.current.size +
                sign *
                  ((horizontal ? event.clientY : event.clientX) -
                    drag.current.start),
            ),
          );
      }}
      onPointerUp={() => {
        drag.current = null;
      }}
      onPointerCancel={cancel}
      onLostPointerCapture={cancel}
    />
  );
}
