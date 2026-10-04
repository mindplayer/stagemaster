import type { ManualFixture } from "./execution-manual.ts";
import type { ExecutionSourceState } from "./execution-source-progress.ts";
type Attribute = ManualFixture["attributes"][number];

export function manualReading(attribute: Attribute, value: number) {
  if (!Number.isInteger(value) || value < 0 || value > 65535) return null;
  const table = attribute.function;
  if (!table)
    return {
      label: `${Number(((value * 100) / 65535).toFixed(2))}%`,
      raw: value,
    };
  if (!table.fine && value % 257 !== 0) return null;
  const native = table.fine ? value : value / 257;
  const f = table.functions.find(
    (f) => native >= f.dmxFrom && native <= f.dmxTo,
  );
  if (!f || (f.mode === "slot" && native !== f.dmxDefault)) return null;
  const label =
    f.mode === "slot"
      ? f.name
      : `${f.name} · ${Number((((native - f.dmxFrom) * 100) / (f.dmxTo - f.dmxFrom)).toFixed(2))}%`;
  return { label, raw: value, native, appearance: f.appearance };
}
export function manualRows(
  fixtures: ManualFixture[],
  state?: ExecutionSourceState,
) {
  if (
    !state?.held ||
    !state.heldValues ||
    state.held.length !== state.heldValues.length
  )
    return null;
  const rows = state.held.map((target, index) => {
    const fixture = fixtures.find((f) => f.id === target.fixtureId);
    const attribute = fixture?.attributes.find(
      (a) => a.key === target.attribute,
    );
    const reading =
      attribute && manualReading(attribute, state.heldValues![index]);
    return fixture && attribute && reading
      ? { target, fixture, attribute, reading }
      : null;
  });
  // Never display a partially valid observation as if it were complete.
  return rows.every((row) => row !== null) ? rows : null;
}
export function selectedManualReading(
  rows: NonNullable<ReturnType<typeof manualRows>>,
  selected: string[],
  attribute: string,
) {
  const held = rows.filter(
    (r) =>
      selected.includes(r.target.fixtureId) && r.target.attribute === attribute,
  );
  const distinct = new Set(held.map((r) => r.reading.raw));
  return {
    held: held.length,
    unheld: selected.length - held.length,
    label: !held.length
      ? "未持有"
      : distinct.size === 1
        ? held[0].reading.label
        : "多个不同值",
    raw: distinct.size === 1 ? held[0].reading.raw : undefined,
  };
}
