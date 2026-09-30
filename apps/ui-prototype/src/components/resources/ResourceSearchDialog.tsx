import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { searchResources, type ResourceOption } from "./resource-search";
const PAGE_SIZE = 50;
export function ResourceSearchDialog({
  label,
  value,
  options,
  onClose,
  onSelect,
}: {
  label: string;
  value?: string;
  options: ResourceOption[];
  onClose(): void;
  onSelect(id: string): void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const results = useRef<HTMLDivElement>(null);
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(0);
  const matches = useMemo(
    () => searchResources(options, query),
    [options, query],
  );
  const lastPage = Math.max(0, Math.ceil(matches.length / PAGE_SIZE) - 1);
  const currentPage = Math.min(page, lastPage);
  const shown = matches.slice(
    currentPage * PAGE_SIZE,
    (currentPage + 1) * PAGE_SIZE,
  );
  useEffect(() => {
    const previous = document.activeElement;
    const element = dialog.current!;
    element.showModal();
    input.current?.focus();
    return () => {
      element.close();
      if (previous instanceof HTMLElement && previous.isConnected)
        previous.focus();
    };
  }, []);
  function focusResult(index: number) {
    const buttons =
      results.current?.querySelectorAll<HTMLButtonElement>("button");
    if (!buttons?.length) return;
    const button = buttons[Math.max(0, Math.min(index, buttons.length - 1))];
    button.focus();
    button.scrollIntoView({ block: "nearest" });
  }
  return createPortal(
    <dialog
      ref={dialog}
      className="resource-search-dialog"
      aria-label={label}
      data-navigation-dialog="true"
      onCancel={(e) => {
        e.preventDefault();
        onClose();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          e.stopPropagation();
          onClose();
        }
      }}
      onClick={(e) => {
        if (e.target !== e.currentTarget) return;
        const r = e.currentTarget.getBoundingClientRect();
        if (
          e.clientX < r.left ||
          e.clientX > r.right ||
          e.clientY < r.top ||
          e.clientY > r.bottom
        )
          onClose();
      }}
    >
      <header>
        <h2>{label}</h2>
        <button type="button" onClick={onClose}>
          取消
        </button>
      </header>
      <div className="resource-search-input">
        <input
          ref={input}
          type="search"
          aria-label={`搜索${label}`}
          placeholder="搜索名称或属性"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setPage(0);
          }}
          onKeyDown={(e) => {
            if (e.nativeEvent.isComposing) return;
            if (e.key === "ArrowDown" || e.key === "ArrowUp") {
              e.preventDefault();
              focusResult(e.key === "ArrowDown" ? 0 : shown.length - 1);
            } else if (e.key === "Enter" && shown.length) {
              e.preventDefault();
              onSelect(shown[0].id);
            }
          }}
        />
      </div>
      <div
        className="resource-search-results"
        ref={results}
        aria-label={`${label}结果`}
      >
        {shown.map((option, index) => (
          <button
            type="button"
            key={option.id}
            tabIndex={index === 0 ? 0 : -1}
            aria-pressed={option.id === value}
            onClick={() => onSelect(option.id)}
            onKeyDown={(e) => {
              const next =
                e.key === "ArrowDown"
                  ? index + 1
                  : e.key === "ArrowUp"
                    ? index - 1
                    : e.key === "Home"
                      ? 0
                      : e.key === "End"
                        ? shown.length - 1
                        : null;
              if (next !== null) {
                e.preventDefault();
                focusResult(next);
              }
            }}
          >
            <span>
              <strong>{option.label}</strong>
              {option.detail && <small>{option.detail}</small>}
            </span>
            {option.id === value && <em>当前</em>}
          </button>
        ))}
        {!shown.length && <p role="status">没有匹配结果</p>}
      </div>
      <footer>
        <span role="status">
          {matches.length} 项{query && ` / 共 ${options.length} 项`}
        </span>
        {lastPage > 0 && (
          <nav aria-label="资源分页">
            <button
              type="button"
              disabled={!currentPage}
              onClick={() => setPage(currentPage - 1)}
            >
              上一页
            </button>
            <span>
              {currentPage + 1} / {lastPage + 1}
            </span>
            <button
              type="button"
              disabled={currentPage === lastPage}
              onClick={() => setPage(currentPage + 1)}
            >
              下一页
            </button>
          </nav>
        )}
      </footer>
    </dialog>,
    document.querySelector(".workbench") ?? document.body,
  );
}
