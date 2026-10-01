import { useEffect, useRef, useState } from "react";
import { useResourcePins } from "./resource-pin-store";
import type { PinKind } from "./pinned-resources";
import "./pinned-resources.css";
export function PinnedResources({
  projectId,
  kind,
  items,
  currentId,
  busy,
  onSelect,
}: {
  projectId: string;
  kind: PinKind;
  items: { id: string; name: string }[];
  currentId?: string;
  busy: boolean;
  onSelect(id: string): void;
}) {
  const pins = useResourcePins(projectId, kind);
  const [error, setError] = useState("");
  const bar = useRef<HTMLDivElement>(null),
    pinButton = useRef<HTMLButtonElement>(null);
  useEffect(() => setError(""), [currentId, projectId]);
  const label = kind === "groups" ? "灯组" : "预设";
  const current = items.find((item) => item.id === currentId);
  const pinned = !!current && pins.ids.includes(current.id);
  const visible = pins.ids.flatMap((id) => {
    const item = items.find((i) => i.id === id);
    return item ? [item] : [];
  });
  function toggle(id: string, restoreFocus = false) {
    try {
      const index = visible.findIndex((item) => item.id === id);
      pins.toggle(
        id,
        items.map((i) => i.id),
      );
      setError("");
      if (restoreFocus)
        requestAnimationFrame(() => {
          const choices = bar.current?.querySelectorAll<HTMLButtonElement>(
            ".pinned-resource button:first-child",
          );
          const next = choices?.[Math.min(index, choices.length - 1)];
          const fallback = pinButton.current?.disabled
            ? pinButton.current?.parentElement?.querySelector<HTMLButtonElement>(
                ".resource-picker-trigger",
              )
            : pinButton.current;
          (next ?? fallback)?.focus();
        });
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }
  return (
    <>
      <button
        type="button"
        ref={pinButton}
        className="pin-current-resource"
        disabled={busy || !current}
        aria-pressed={pinned}
        title="固定在本机此工程的快捷栏，不改变工程内容"
        onClick={() => current && toggle(current.id)}
      >
        {pinned ? `取消固定${label}` : `固定${label}`}
      </button>
      {!!visible.length && (
        <div
          ref={bar}
          className="pinned-resources"
          role="group"
          aria-label={`常用${label}`}
        >
          <span>常用{label}</span>
          {visible.map((item) => (
            <span className="pinned-resource" key={item.id}>
              <button
                type="button"
                disabled={busy}
                aria-label={`选择常用${label}：${item.name}`}
                aria-pressed={item.id === currentId}
                title={item.name}
                onClick={() => onSelect(item.id)}
              >
                {item.name}
              </button>
              <button
                type="button"
                className="unpin-resource"
                disabled={busy}
                aria-label={`取消固定${label}：${item.name}`}
                title={`取消固定${item.name}`}
                onClick={() => toggle(item.id, true)}
              >
                ×
              </button>
            </span>
          ))}
        </div>
      )}
      {(error || pins.problem) && (
        <small className="pin-resource-error" role="alert">
          {error || pins.problem}
        </small>
      )}
    </>
  );
}
