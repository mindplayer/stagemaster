import type { availableParameterCategories } from "../../parameter-categories";
import "./parameter-categories.css";

export function ParameterCategories({
  categories,
  value,
  busy,
  onChange,
}: {
  categories: ReturnType<typeof availableParameterCategories>;
  value: string;
  busy: boolean;
  onChange(value: string): void;
}) {
  if (categories.length < 2) return null;
  return (
    <nav className="parameter-categories" aria-label="灯光属性分类">
      {[
        ...categories,
        {
          id: "all",
          label: "全部",
          count: categories.reduce((sum, c) => sum + c.count, 0),
        },
      ].map((c) => (
        <button
          key={c.id}
          type="button"
          aria-pressed={value === c.id}
          disabled={busy}
          title={`${c.label} · ${c.count} 项共同属性`}
          onClick={() => onChange(c.id)}
        >
          {c.label}
          <span aria-hidden="true">{c.count}</span>
        </button>
      ))}
    </nav>
  );
}
