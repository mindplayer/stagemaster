export const numberInput = (
  name: string,
  label: string,
  raw: string,
  min: number,
  max: number,
  change: (v: string) => void,
) => (
  <label>
    {label}
    <input
      name={name}
      aria-label={label}
      type="number"
      min={min}
      max={max}
      step={name.endsWith("percent") ? "any" : 1}
      value={raw}
      required
      onChange={(e) => change(e.target.value)}
    />
  </label>
);
