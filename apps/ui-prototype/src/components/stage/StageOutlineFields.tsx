import { TrashIcon, PlusIcon } from "@phosphor-icons/react";
import { decimal, objectOutline } from "../../stage-tools";
import type { StageObject } from "../../stage-types";
export function StageOutlineFields({
  object,
  onChange,
}: {
  object: StageObject;
  onChange(value: StageObject): void;
}) {
  const outline = objectOutline(object);
  const update = (fn: (copy: StageObject) => void) => {
    const copy = structuredClone(object);
    fn(copy);
    onChange(copy);
  };
  return (
    <>
      {outline && (
        <>
          <details className="stage-outline">
            <summary>高级轮廓 · {outline.length} 个顶点</summary>
            <h3>
              顶点坐标 <span>米</span>
            </h3>
            <div className="stage-point-head">
              <span>顶点</span>
              <span>X</span>
              <span>Y</span>
              <span />
            </div>
            {outline.map((point, index) => (
              <div className="stage-point" key={index}>
                <span>{index + 1}</span>
                {([0, 1] as const).map((axis) => (
                  <input
                    key={axis}
                    aria-label={`顶点 ${index + 1} ${axis === 0 ? "X" : "Y"}`}
                    type="number"
                    required
                    step="any"
                    min={-100000}
                    max={100000}
                    value={point[axis]}
                    onChange={(e) =>
                      update((c) => {
                        const p = objectOutline(c);
                        if (p) p[index]![axis] = e.target.value;
                      })
                    }
                  />
                ))}
                <button
                  type="button"
                  aria-label={`删除顶点 ${index + 1}`}
                  disabled={outline.length <= 3}
                  onClick={() =>
                    update((c) => {
                      objectOutline(c)?.splice(index, 1);
                    })
                  }
                >
                  <TrashIcon />
                </button>
              </div>
            ))}
            <button
              type="button"
              disabled={outline.length >= 128}
              onClick={() =>
                update((c) => {
                  const p = objectOutline(c);
                  if (p && p.length >= 3) {
                    const a = p.at(-1)!,
                      b = p[0]!;
                    p.push([
                      decimal((Number(a[0]) + Number(b[0])) / 2),
                      decimal((Number(a[1]) + Number(b[1])) / 2),
                    ]);
                  }
                })
              }
            >
              <PlusIcon />
              添加轮廓顶点
            </button>
          </details>
        </>
      )}
    </>
  );
}
