export function LayoutActions({
  full,
  lower,
  focused,
  busy,
  library,
  inspector,
  onFocus,
  onToggle,
  onRestore,
}: {
  full: boolean;
  lower: boolean;
  focused: boolean;
  busy: boolean;
  library: boolean;
  inspector: boolean;
  onFocus(): void;
  onToggle(side: "showLibrary" | "showInspector"): void;
  onRestore(): void;
}) {
  return (
    <div className="editor-layout-actions" aria-label="布局">
      <button
        aria-pressed={focused}
        disabled={full || !lower || busy}
        onClick={onFocus}
        title={
          focused
            ? "恢复舞台画布与原编排区高度"
            : "展开编排区，保留资源、属性与音乐控制"
        }
      >
        {focused ? "显示舞台" : "专注编排"}
      </button>
      <button
        aria-label={library ? "收起资源区" : "展开资源区"}
        aria-pressed={library}
        disabled={full || busy}
        onClick={() => onToggle("showLibrary")}
      >
        资源
      </button>
      <button
        aria-label={inspector ? "收起属性区" : "展开属性区"}
        aria-pressed={inspector}
        disabled={full || busy}
        onClick={() => onToggle("showInspector")}
      >
        属性
      </button>
      <button disabled={busy} onClick={onRestore}>
        恢复布局
      </button>
    </div>
  );
}
