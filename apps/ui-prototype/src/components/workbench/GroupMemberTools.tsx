import type { FixtureView } from "../../application-host";
import { arrangeFixtureIds, type FixtureOrder } from "../../fixture-order";

export function GroupMemberTools({
  ids,
  fixtures,
  selected,
  initial,
  onChange,
}: {
  ids: string[];
  fixtures: FixtureView[];
  selected: string[];
  initial: string[];
  onChange(ids: string[]): void;
}) {
  return (
    <div className="wb-resource-tools" role="group" aria-label="灯组整体整理">
      {(
        [
          ["reverse", "反转顺序"],
          ["oddFirst", "奇数位在前"],
          ["name", "按名称排序"],
          ["patch", "按配适排序"],
        ] as [FixtureOrder, string][]
      ).map(([order, label]) => (
        <button
          key={order}
          type="button"
          disabled={ids.length < 2}
          onClick={() => onChange(arrangeFixtureIds(ids, fixtures, order))}
          title={
            order === "oddFirst"
              ? "按当前完整灯序先排第 1、3、5…位，再排第 2、4、6…位；保留所有成员"
              : order === "patch"
                ? "按输出域、线路、地址排序，未配适灯具保留原顺序置后"
                : undefined
          }
        >
          {label}
        </button>
      ))}
      <button
        type="button"
        disabled={!selected.length}
        onClick={() => onChange([...selected])}
      >
        用当前选灯替换
      </button>
      <button type="button" onClick={() => onChange([...initial])}>
        恢复初始成员
      </button>
    </div>
  );
}
