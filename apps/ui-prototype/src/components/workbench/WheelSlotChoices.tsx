import type { FunctionDefinition } from "../../fixture-function-types";
import { WheelSwatch } from "../fixtures/WheelSwatch";
export function WheelSlotChoices({
  functions,
  selected,
  onSelect,
}: {
  functions: FunctionDefinition[];
  selected?: string;
  onSelect(f: FunctionDefinition): void;
}) {
  return (
    <div className="wheel-slot-choices" role="group" aria-label="色盘固定档位">
      {functions
        .filter((f) => f.mode === "slot")
        .map((f) => (
          <button
            type="button"
            key={f.key}
            aria-pressed={selected === f.key}
            title={`${f.name} · ${f.dmxFrom}–${f.dmxTo} · 代表值 ${f.dmxDefault}`}
            onClick={() => onSelect(f)}
          >
            <WheelSwatch value={f.appearance} />
            <span>{f.name}</span>
          </button>
        ))}
    </div>
  );
}
