export function StageViewTabs({
  value,
  busy,
  onChange,
}: {
  value: "plan" | "three";
  busy: boolean;
  onChange(value: "plan" | "three"): void;
}) {
  return (
    <div className="stage-view-switch" aria-label="舞台视图">
      <button
        aria-pressed={value === "plan"}
        disabled={busy}
        onClick={() => onChange("plan")}
      >
        平面布置
      </button>
      <button
        aria-pressed={value === "three"}
        disabled={busy}
        onClick={() => onChange("three")}
      >
        三维舞台
      </button>
    </div>
  );
}
