import { useState } from "react";
import type { FixtureView, ProjectView } from "../../application-host";
export function PatchMap({
  project,
  selected,
  busy,
  onSelect,
}: {
  project: ProjectView;
  selected: string[];
  busy: boolean;
  onSelect(id: string): void;
}) {
  const [domain, setDomain] = useState(project.domains[0]?.id ?? ""),
    [line, setLine] = useState("1");
  const activeDomain = project.domains.some((d) => d.id === domain)
    ? domain
    : (project.domains[0]?.id ?? "");
  const validLine =
    /^\d+$/.test(line) && Number(line) >= 1 && Number(line) <= 65535;
  const fixtures = project.fixtures.filter(
    (f) => f.domainId === activeDomain && f.universe === Number(line),
  );
  const slots: Array<FixtureView | undefined> = Array.from(
    { length: 512 },
    (_, i) =>
      fixtures.find(
        (f) =>
          f.address !== null &&
          f.address <= i + 1 &&
          i + 1 < f.address + f.footprint,
      ),
  );
  const free = slots.filter((f) => !f).length;
  let largest = 0,
    run = 0;
  for (const f of slots) {
    run = f ? 0 : run + 1;
    largest = Math.max(largest, run);
  }
  return (
    <details className="patch-map">
      <summary>
        地址占用 · 线路 {validLine ? line : "—"}{" "}
        {validLine && `· ${512 - free}/512 通道`}
      </summary>
      <div className="patch-map-toolbar">
        <label>
          输出域
          <select
            aria-label="占用图输出域"
            value={activeDomain}
            onChange={(e) => setDomain(e.target.value)}
          >
            {project.domains.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          线路
          <input
            aria-label="占用图线路"
            type="number"
            min={1}
            max={65535}
            value={line}
            onChange={(e) => setLine(e.target.value)}
          />
        </label>
        <span>
          {validLine
            ? `空余 ${free} · 最大连续 ${largest}`
            : "线路应为 1–65535 的整数"}
        </span>
      </div>
      <div
        className="patch-slots"
        aria-label="512 通道占用"
        hidden={!validLine}
      >
        {slots.map((f, i) =>
          f ? (
            <button
              key={i}
              disabled={busy}
              className={selected.includes(f.id) ? "selected" : "occupied"}
              title={`${i + 1} · ${f.name} · ${f.address}–${f.address! + f.footprint - 1}`}
              aria-label={`通道 ${i + 1}，${f.name}`}
              onClick={() => onSelect(f.id)}
            >
              {i + 1}
            </button>
          ) : (
            <span key={i} title={`通道 ${i + 1} 空余`}>
              {i + 1}
            </span>
          ),
        )}
      </div>
    </details>
  );
}
