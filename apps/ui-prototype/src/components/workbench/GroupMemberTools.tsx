import type { FixtureView } from "../../application-host";
import type { FixturePlacement } from "../../stage-types";
import { FixtureOrderTools } from "../fixtures/FixtureOrderTools";

export function GroupMemberTools({
  ids,
  fixtures,
  placements,
  selected,
  initial,
  onChange,
}: {
  ids: string[];
  fixtures: FixtureView[];
  placements: FixturePlacement[];
  selected: string[];
  initial: string[];
  onChange(ids: string[]): void;
}) {
  return (
    <div className="wb-resource-tools" role="group" aria-label="灯组整体整理">
      <FixtureOrderTools {...{ ids, fixtures, placements, onChange }} />
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
