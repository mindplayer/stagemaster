import { useMemo } from "react";
import type { FixtureView } from "../../application-host";
import { fixtureLegend } from "../../fixture-plan-display";
import { FixturePlanSymbol } from "./FixturePlanSymbol";
export function FixturePlanLegend({
  fixtures,
  ids,
}: {
  fixtures: FixtureView[];
  ids: string[];
}) {
  const entries = useMemo(
    () => fixtureLegend(fixtures, ids),
    [fixtures, ids.join("|")],
  );
  if (!entries.length) return null;
  return (
    <details className="fixture-plan-legend">
      <summary>灯位图例 · {entries.length} 类</summary>
      <div>
        {entries.map(({ symbol, count }) => (
          <span key={symbol.key} title="依据档案能力显示符号">
            <svg aria-hidden="true" viewBox="-1.3 -1.3 2.6 2.6">
              <FixturePlanSymbol symbol={symbol} />
            </svg>
            {symbol.label}
            <small>{count}</small>
          </span>
        ))}
      </div>
    </details>
  );
}
