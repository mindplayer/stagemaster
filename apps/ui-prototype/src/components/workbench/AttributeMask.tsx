export function AttributeMask({
  available,
  selected,
  onChange,
}: {
  available: { key: string; label: string }[];
  selected: string[];
  onChange(ids: string[]): void;
}) {
  return (
    <div className="wb-attribute-mask" role="group" aria-label="属性范围">
      {available.map((a) => (
        <label key={a.key}>
          <input
            type="checkbox"
            checked={selected.includes(a.key)}
            onChange={(e) =>
              onChange(
                e.target.checked
                  ? [...selected, a.key]
                  : selected.filter((k) => k !== a.key),
              )
            }
          />
          {a.label}
        </label>
      ))}
      <button
        type="button"
        onClick={() => onChange(available.map((a) => a.key))}
      >
        全部属性
      </button>
      {available.some((a) => ["red", "green", "blue"].includes(a.key)) && (
        <button
          type="button"
          onClick={() =>
            onChange(
              available
                .filter((a) => ["red", "green", "blue"].includes(a.key))
                .map((a) => a.key),
            )
          }
        >
          仅颜色
        </button>
      )}
    </div>
  );
}
