import { useEffect, useState } from "react";
import {
  HouseLineIcon,
  CubeIcon,
  LightbulbIcon,
  CaretRightIcon,
} from "@phosphor-icons/react";
import type { ProjectView } from "../../application-host";
import type { StageSelection } from "../../stage-types";
export function StageOutliner({
  project,
  selection,
  selectedIds,
  query,
  busy,
  onSelect,
}: {
  project: ProjectView;
  selection: StageSelection | null;
  selectedIds: string[];
  query: string;
  busy: boolean;
  onSelect(target: StageSelection, additive?: boolean): void;
}) {
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const parentId =
    selection?.kind === "construction"
      ? project.stage.constructions.find((c) => c.id === selection.id)?.shape
          .spaceId
      : selection?.kind === "placement"
        ? project.stage.placements.find((p) => p.fixtureId === selection.id)
            ?.spaceId
        : null;
  useEffect(() => {
    if (parentId)
      setCollapsed((old) => {
        if (!old.has(parentId)) return old;
        const next = new Set(old);
        next.delete(parentId);
        return next;
      });
  }, [parentId, selection?.id]);
  const search = query.trim().toLocaleLowerCase();
  const members = [
    ...project.stage.constructions.map((c) => ({
      target: { kind: "construction" as const, id: c.id },
      name: c.name,
      space: c.shape.spaceId,
      detail: c.shape.kind === "enclosure" ? "墙体与地板" : "舞台",
    })),
    ...project.stage.placements.map((p) => ({
      target: { kind: "placement" as const, id: p.fixtureId },
      name: project.fixtures.find((f) => f.id === p.fixtureId)?.name ?? "灯具",
      space: p.spaceId,
      detail: `高度 ${p.positionMeters.z} 米`,
    })),
  ];
  const matches = (name: string) => name.toLocaleLowerCase().includes(search);
  const row = (
    target: StageSelection,
    name: string,
    detail: string,
    nested = false,
  ) => (
    <button
      key={`${target.kind}:${target.id}`}
      className={`stage-object ${nested ? "nested" : ""}`}
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
  const rooms = project.stage.spaces.filter(
    (s) =>
      matches(s.name) ||
      members.some((m) => m.space === s.id && matches(m.name)),
  );
  const loose = members.filter((m) => m.space === null && matches(m.name));
  return (
    <div className="stage-objects" aria-label="场地对象">
      {rooms.map((s) => {
        const children = members.filter(
          (m) => m.space === s.id && (matches(s.name) || matches(m.name)),
        );
        const expanded = !!search || !collapsed.has(s.id);
        return (
          <section className="stage-object-group" key={s.id}>
            <div className="stage-room-row">
              <button
                className="stage-disclosure"
                aria-label={`${expanded ? "收起" : "展开"}${s.name}`}
                aria-expanded={expanded}
                onClick={() =>
                  setCollapsed((old) => {
                    const next = new Set(old);
                    if (next.has(s.id)) next.delete(s.id);
                    else next.add(s.id);
                    return next;
                  })
                }
              >
                <CaretRightIcon />
              </button>
              {row(
                { kind: "space", id: s.id },
                s.name,
                s.clearHeightMeters === null
                  ? "开放空间"
                  : `净高 ${s.clearHeightMeters} 米`,
              )}
            </div>
            {expanded &&
              children.map((m) => row(m.target, m.name, m.detail, true))}
          </section>
        );
      })}
      {loose.length > 0 && (
        <section className="stage-object-group">
          <h3>未归属空间</h3>
          {loose.map((m) => row(m.target, m.name, m.detail))}
        </section>
      )}
      {!rooms.length && !loose.length && (
        <p className="wb-dim">{search ? "没有匹配的对象" : "尚未创建场地"}</p>
      )}
    </div>
  );
}
