import { useEffect, useMemo, useRef, useState } from "react";
import {
  HouseLineIcon,
  CubeIcon,
  LightbulbIcon,
  CaretRightIcon,
  EyeIcon,
  EyeSlashIcon,
} from "@phosphor-icons/react";
import type { ProjectView } from "../../application-host";
import type { StageSelection } from "../../stage-types";
import {
  outlineMembers,
  memberCategories,
  categoryKey,
} from "./stage-outliner-model";
import { displayMeters, type PlanVisibility } from "./stage-display";
import "./stage-organization.css";
export function StageOutliner({
  project,
  selection,
  selectedIds,
  query,
  busy,
  visibility,
  onVisibility,
  onSelect,
}: {
  project: ProjectView;
  selection: StageSelection | null;
  selectedIds: string[];
  query: string;
  busy: boolean;
  visibility: PlanVisibility;
  onVisibility(value: PlanVisibility): void;
  onSelect(target: StageSelection, additive?: boolean): void;
}) {
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const root = useRef<HTMLDivElement>(null);
  const members = useMemo(() => outlineMembers(project), [project]);
  const parent = members.find(
    (m) => m.target.id === selection?.id && m.target.kind === selection.kind,
  );
  useEffect(() => {
    if (!selection) return;
    const keys = parent
      ? [parent.space ?? "loose", categoryKey(parent.space, parent.category)]
      : [selection.id];
    setExpanded((old) => ({
      ...old,
      ...Object.fromEntries(keys.map((key) => [key, true])),
    }));
    const frame = requestAnimationFrame(() =>
      root.current
        ?.querySelector<HTMLElement>(
          `[data-selection="${selection.kind}:${selection.id}"]`,
        )
        ?.scrollIntoView({ block: "nearest" }),
    );
    return () => cancelAnimationFrame(frame);
  }, [selection?.id, selection?.kind, parent?.space, parent?.category]);
  const search = query.trim().toLocaleLowerCase();
  const matches = (name: string) => name.toLocaleLowerCase().includes(search);
  const toggle = (key: string, current: boolean) =>
    setExpanded((old) => ({ ...old, [key]: !current }));
  function row(
    target: StageSelection,
    name: string,
    detail: string,
    nested = false,
    hidden = false,
  ) {
    return (
      <button
        key={`${target.kind}:${target.id}`}
        data-selection={`${target.kind}:${target.id}`}
        className={`stage-object ${nested ? "nested" : ""} ${hidden ? "plan-hidden" : ""}`}
        title={`${name} · ${detail}${hidden ? " · 选择后在平面图中显示" : ""}`}
        aria-pressed={
          target.kind === "placement"
            ? selectedIds.includes(target.id)
            : selection?.id === target.id && selection.kind === target.kind
        }
        disabled={busy}
        onClick={(e) => onSelect(target, e.shiftKey || e.metaKey || e.ctrlKey)}
      >
        {target.kind === "space" ? (
          <HouseLineIcon />
        ) : target.kind === "placement" ? (
          <LightbulbIcon />
        ) : (
          <CubeIcon />
        )}
        <span>
          <strong>{name}</strong>
          <small>{detail}</small>
        </span>
      </button>
    );
  }
  const rooms = [
    ...project.stage.spaces.map((s) => ({
      id: s.id as string | null,
      name: s.name,
      space: s,
    })),
    { id: null, name: "未归属空间", space: null },
  ];
  return (
    <div ref={root} className="stage-objects" aria-label="场地对象">
      {rooms.map((room) => {
        const all = members.filter((m) => m.space === room.id);
        const children = all.filter(
          (m) => matches(room.name) || matches(`${m.name} ${m.detail}`),
        );
        if (!children.length && (!room.space || !matches(room.name)))
          return null;
        const key = room.id ?? "loose",
          open = !!search || (expanded[key] ?? true),
          hidden =
            room.id !== null && visibility.hiddenSpaces.includes(room.id);
        return (
          <section className="stage-object-group" key={key}>
            <div className="stage-room-row">
              <button
                className="stage-disclosure"
                aria-label={`${open ? "收起" : "展开"}${room.name}`}
                aria-expanded={open}
                onClick={() => toggle(key, open)}
              >
                <CaretRightIcon />
              </button>
              {room.space ? (
                row(
                  { kind: "space", id: room.space.id },
                  room.name,
                  `${all.length} 个对象 · ${room.space.clearHeightMeters === null ? "开放空间" : `净高 ${displayMeters(room.space.clearHeightMeters)} 米`}`,
                  false,
                  hidden,
                )
              ) : (
                <strong>
                  {room.name} · {all.length}
                </strong>
              )}
              {room.id && (
                <button
                  className="stage-visibility"
                  disabled={busy}
                  aria-label={`平面中${hidden ? "显示" : "隐藏"}${room.name}`}
                  title={`${hidden ? "显示" : "隐藏"}此空间及所属对象，仅影响平面图`}
                  onClick={() =>
                    onVisibility({
                      ...visibility,
                      hiddenSpaces: hidden
                        ? visibility.hiddenSpaces.filter((id) => id !== room.id)
                        : [...visibility.hiddenSpaces, room.id!],
                    })
                  }
                >
                  {hidden ? <EyeSlashIcon /> : <EyeIcon />}
                </button>
              )}
            </div>
            {open &&
              memberCategories.map(([category, label]) => {
                const items = children.filter((m) => m.category === category);
                if (!items.length) return null;
                const groupKey = categoryKey(room.id, category);
                const show =
                  !!search ||
                  (expanded[groupKey] ??
                    (items.length <= 12 && members.length <= 100));
                return (
                  <div key={groupKey} className="stage-type-group">
                    <button
                      className="stage-type-heading"
                      aria-label={`${show ? "收起" : "展开"}${room.name}的${label}`}
                      aria-expanded={show}
                      onClick={() => toggle(groupKey, show)}
                    >
                      <CaretRightIcon />
                      <span>{label}</span>
                      <small>{items.length}</small>
                    </button>
                    {show &&
                      items.map((m) =>
                        row(
                          m.target,
                          m.name,
                          m.detail,
                          true,
                          hidden || visibility.hiddenLayers.includes(category),
                        ),
                      )}
                  </div>
                );
              })}
          </section>
        );
      })}
      {!project.stage.spaces.length && !members.length && (
        <p className="wb-dim">尚未创建场地</p>
      )}
      {!!search &&
        !rooms.some(
          (room) =>
            (!!room.space && matches(room.name)) ||
            members.some(
              (m) => m.space === room.id && matches(`${m.name} ${m.detail}`),
            ),
        ) && <p className="wb-dim">没有匹配的对象</p>}
    </div>
  );
}
