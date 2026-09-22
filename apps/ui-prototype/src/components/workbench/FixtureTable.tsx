import { LightbulbIcon } from "@phosphor-icons/react";
import type { FixtureView } from "../../application-host";
export function FixtureTable({
  fixtures,
  selected,
  busy,
  onSelect,
}: {
  fixtures: FixtureView[];
  selected: string;
  busy: boolean;
  onSelect: (fixture: FixtureView) => void;
}) {
  return fixtures.length ? (
    <div className="wb-fixture-table">
      <div className="wb-table-head">
        <span>灯具</span>
        <span>模式</span>
        <span>线路</span>
        <span>地址</span>
      </div>
      {fixtures.map((fixture, i) => (
        <button
          className="wb-fixture-row"
          aria-pressed={selected === fixture.id}
          key={fixture.id}
          disabled={busy}
          onClick={() => onSelect(fixture)}
        >
          <span>
            <i>{String(i + 1).padStart(2, "0")}</i>
            <strong>{fixture.name}</strong>
          </span>
          <span>{fixture.profileName}</span>
          <span>{fixture.universe ?? "—"}</span>
          <span>
            {fixture.address
              ? `${fixture.address}–${fixture.address + fixture.footprint - 1}`
              : "未配适"}
          </span>
        </button>
      ))}
    </div>
  ) : (
    <div className="wb-empty">
      <LightbulbIcon size={32} />
      <p>尚未添加灯具</p>
    </div>
  );
}
