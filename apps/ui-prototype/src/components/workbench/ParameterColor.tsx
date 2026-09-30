export function ParameterColor({
  color,
  mixed,
  draft,
  onChange,
  onCancel,
}: {
  color: string;
  mixed: boolean;
  draft: string | null;
  onChange(value: string): void;
  onCancel(): void;
}) {
  const swatch = draft ?? color;
  return (
    <div className="wb-color">
      <div className="wb-section-title">
        <label htmlFor="param-color">颜色</label>
        <span>{mixed && draft === null ? "混合色" : "RGB"}</span>
      </div>
      <div className="wb-color-inputs">
        <input
          id="param-color"
          type="color"
          aria-label="颜色"
          value={/^#[a-f\d]{6}$/i.test(swatch) ? swatch : "#000000"}
          onChange={(e) => onChange(e.target.value)}
        />
        <input
          aria-label="颜色十六进制"
          placeholder={mixed ? "混合色" : "#RRGGBB"}
          pattern="#[0-9a-fA-F]{6}"
          required={draft !== null}
          spellCheck={false}
          value={draft ?? (mixed ? "" : color.toUpperCase())}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              e.preventDefault();
              e.stopPropagation();
              onCancel();
            }
          }}
        />
      </div>
    </div>
  );
}
