import type { FixtureSymbolInfo } from "../../fixture-plan-display";
import "./fixture-symbol.css";
export function FixturePlanSymbol({
  symbol,
  unit = 1,
  selected = false,
}: {
  symbol: FixtureSymbolInfo;
  unit?: number;
  selected?: boolean;
}) {
  return (
    <g
      className={`fixture-symbol ${selected ? "symbol-selected" : ""}`}
      transform={`scale(${unit})`}
    >
      {selected && (
        <rect
          className="fixture-selection-frame"
          x={-1.12}
          y={-1.12}
          width={2.24}
          height={2.24}
          rx={0.2}
          strokeWidth={0.14}
        />
      )}
      {symbol.moving ? (
        <>
          <rect
            className="fixture-body"
            x={-0.62}
            y={-0.63}
            width={1.24}
            height={1.26}
            rx={0.28}
            strokeWidth={0.13}
          />
          <path
            className="fixture-yoke"
            d="M -.93 -.25 V .76 H .93 V -.25 M -.45 .96 H .45"
            strokeWidth={0.13}
          />
        </>
      ) : (
        <circle className="fixture-body" r={0.78} strokeWidth={0.14} />
      )}
      {symbol.rgb ? (
        <g className="fixture-mix">
          <circle cx={0} cy={-0.23} r={0.14} />
          <circle cx={-0.23} cy={0.17} r={0.14} />
          <circle cx={0.23} cy={0.17} r={0.14} />
        </g>
      ) : symbol.wheel ? (
        <g className="fixture-wheel">
          <circle r={0.35} strokeWidth={0.1} />
          <path
            d="M 0 0 V -.35 M 0 0 L .3 .175 M 0 0 L -.3 .175"
            strokeWidth={0.09}
          />
        </g>
      ) : (
        <path
          className="fixture-mark"
          d="M -.3 0 H .3 M 0 -.3 V .3"
          strokeWidth={0.12}
        />
      )}
      {symbol.rgb && symbol.wheel && (
        <circle
          className="fixture-wheel-badge"
          cx={0.65}
          cy={-0.65}
          r={0.23}
          strokeWidth={0.1}
        />
      )}
    </g>
  );
}
