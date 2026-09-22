import { useEffect, useRef, useState } from "react";
export function SceneParameter({
  label,
  value,
  mode,
  preset,
  disabled,
  onCommit,
}: {
  label: string;
  value: number;
  mode: string;
  preset: string | null;
  disabled: boolean;
  onCommit: (
    value: number,
    mode: "literal" | "release" | "remove",
  ) => Promise<boolean>;
}) {
  const [draft, setDraft] = useState(value);
  const latest = useRef(value);
  useEffect(() => {
    setDraft(value);
    latest.current = value;
  }, [value]);
  const update = (n: number) => {
    latest.current = n;
    setDraft(n);
  };
  const commit = () => {
    if (latest.current !== value || mode !== "literal")
      void onCommit(latest.current, "literal").then((ok) => {
        if (!ok) update(value);
      });
  };
  return (
    <div className="wb-parameter">
      <div>
        <label>{label}</label>
        <output>
          {(mode === "release" || mode === "none") && draft === value
            ? "—"
            : `${Math.round((draft / 65535) * 1000) / 10}%`}
        </output>
      </div>
      <input
        aria-label={label}
        type="range"
        min={0}
        max={65535}
        step={1}
        value={draft}
        disabled={disabled}
        onChange={(event) => update(Number(event.target.value))}
        onPointerUp={commit}
        onKeyUp={(event) => {
          if (
            [
              "ArrowLeft",
              "ArrowRight",
              "ArrowUp",
              "ArrowDown",
              "Home",
              "End",
              "PageUp",
              "PageDown",
            ].includes(event.key)
          )
            commit();
        }}
        onBlur={() => {
          if (latest.current !== value) commit();
        }}
        onPointerCancel={() => update(value)}
      />
      <div className="wb-parameter-mode">
        <span>
          {preset
            ? `预设 · ${preset}`
            : mode === "release"
              ? "已释放"
              : mode === "none"
                ? "未记录"
                : "已记录"}
        </span>
        <button
          disabled={disabled || mode === "release"}
          onClick={() => onCommit(0, "release")}
        >
          释放
        </button>
        <button
          disabled={disabled || mode === "none"}
          onClick={() => onCommit(0, "remove")}
        >
          清除
        </button>
      </div>
    </div>
  );
}
