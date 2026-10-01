import { useState } from "react";
import type { ComponentProps } from "react";
import { PositionAxisPanel } from "./PositionAxisPanel";
import { PositionReferencePanel } from "./PositionReferencePanel";
export type { PositionHandle } from "./PositionAxisPanel";
export function PositionPanel(props: ComponentProps<typeof PositionAxisPanel>) {
  const [reference, setReference] = useState(false);
  const single = props.fixtures.length === 1 ? props.fixtures[0] : null;
  return (
    <>
      <div className="position-tabs" aria-label="位置工具">
        <button
          type="button"
          aria-pressed={!reference}
          disabled={props.busy}
          onClick={async () => {
            if (await props.beforeChange()) setReference(false);
          }}
        >
          轴与指向
        </button>
        <button
          type="button"
          aria-pressed={reference}
          disabled={props.busy || !single}
          title={
            single
              ? "记录世界目标与当前场景轴设定，检查模型偏差"
              : "请单独选择一台灯具"
          }
          onClick={async () => {
            if (await props.beforeChange()) setReference(true);
          }}
        >
          参考点检查
        </button>
      </div>
      {reference && single ? (
        <PositionReferencePanel {...props} fixture={single} />
      ) : (
        <PositionAxisPanel {...props} />
      )}
    </>
  );
}
