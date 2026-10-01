import { useState } from "react";
import type { FixtureView } from "../../application-host";
import type { FixturePlacement } from "../../stage-types";
import { arrangeFixtureIds, type FixtureOrder } from "../../fixture-order";
import {
  arrangeSpatialFixtureIds,
  fixtureAxisPositions,
  type SpatialOrderAxis,
  type SpatialOrderDirection,
} from "../../spatial-fixture-order";
import "./fixture-order.css";

export function FixtureOrderTools({
  ids,
  fixtures,
  placements,
  onChange,
}: {
  ids: string[];
  fixtures: FixtureView[];
  placements: FixturePlacement[];
  onChange(ids: string[]): void;
}) {
  const [axis, setAxis] = useState<SpatialOrderAxis>("x");
  const [direction, setDirection] =
    useState<SpatialOrderDirection>("ascending");
  const positions = fixtureAxisPositions(placements, axis);
  const placed = ids.filter((id) => positions.has(id)).length;
  function change(next: string[]) {
    if (next.some((id, index) => id !== ids[index])) onChange(next);
  }
  return (
    <div className="fixture-order-tools" role="group" aria-label="整组灯序">
      <div className="fixture-order-buttons">
        {(
          [
            ["reverse", "反转顺序"],
            ["oddFirst", "奇数位在前"],
            ["name", "按名称排序"],
            ["patch", "按配适排序"],
          ] as [FixtureOrder, string][]
        ).map(([order, label]) => (
          <button
            type="button"
            key={order}
            disabled={ids.length < 2}
            title={
              order === "oddFirst"
                ? "按当前完整灯序先排第 1、3、5…位，再排第 2、4、6…位；保留所有灯具"
                : order === "patch"
                  ? "按控制域、输出路、地址排序，未配适灯具保留原序置后"
                  : undefined
            }
            onClick={() => change(arrangeFixtureIds(ids, fixtures, order))}
          >
            {label}
          </button>
        ))}
      </div>
      <details className="fixture-spatial-order">
        <summary>按空间位置排序</summary>
        <div
          className="fixture-order-fields"
          data-editor-navigation="true"
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              event.stopPropagation();
            }
          }}
        >
          <label>
            坐标轴
            <select
              aria-label="排序坐标轴"
              value={axis}
              onChange={(event) =>
                setAxis(event.target.value as SpatialOrderAxis)
              }
            >
              <option value="x">X · 平面左右</option>
              <option value="y">Y · 平面上下</option>
              <option value="z">Z · 高度</option>
            </select>
          </label>
          <label>
            方向
            <select
              aria-label="空间排序方向"
              value={direction}
              onChange={(event) =>
                setDirection(event.target.value as SpatialOrderDirection)
              }
            >
              <option value="ascending">
                {axis === "x"
                  ? "从左到右"
                  : axis === "y"
                    ? "从下到上"
                    : "从低到高"}
              </option>
              <option value="descending">
                {axis === "x"
                  ? "从右到左"
                  : axis === "y"
                    ? "从上到下"
                    : "从高到低"}
              </option>
            </select>
          </label>
        </div>
        <p className="wb-dim">
          使用世界坐标，同坐标保留原顺序；{ids.length - placed}{" "}
          台未布置或坐标无效，保留原顺序置后。
        </p>
        <button
          type="button"
          disabled={ids.length < 2 || placed === 0}
          onClick={() =>
            change(arrangeSpatialFixtureIds(ids, placements, axis, direction))
          }
        >
          采用空间灯序
        </button>
      </details>
    </div>
  );
}
