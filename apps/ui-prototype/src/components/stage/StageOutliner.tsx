import { isStageLocked } from "../../stage-locks";
import { useMemo } from "react";
import {
  outlineRooms,
  outlineSelectionStatus,
} from "./stage-outline-navigation";
import { useOutlineNavigation } from "./useOutlineNavigation";
import {
  HouseLineIcon,
  CubeIcon,
  LightbulbIcon,
  CaretRightIcon,
  EyeIcon,
  EyeSlashIcon,
  LockSimpleIcon,
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
  onClearQuery,
}: {
  project: ProjectView;
  selection: StageSelection | null;
  selectedIds: string[];
  query: string;
  busy: boolean;
  visibility: PlanVisibility;
  onVisibility(value: PlanVisibility): void;
  onClearQuery(): void;
  onSelect(target: StageSelection, additive?: boolean): void;
}) {
  const members = useMemo(() => outlineMembers(project), [project]);
  const status = outlineSelectionStatus(
    project,
    members,
    query,
    selection,
    selectedIds,
  );
  const parent = members.find(
    (m) => m.target.id === selection?.id && m.target.kind === selection.kind,
  );
  const navigation = useOutlineNavigation(
    status.active,
    parent
      ? [parent.space ?? "loose", categoryKey(parent.space, parent.category)]
      : selection
        ? [selection.id]
        : [],
    busy,
    onClearQuery,
  );
  const { expanded, toggle, root } = navigation;
  const search = query.trim().toLocaleLowerCase();
  function row(
    target: StageSelection,
    name: string,
    detail: string,
    nested = false,
    hidden = false,
  ) {
    const locked = isStageLocked(project.stage, target);
    return (
      <button
        key={`${target.kind}:${target.id}`}
        data-selection={`${target.kind}:${target.id}`}
        className={`stage-object ${nested ? "nested" : ""} ${hidden ? "plan-hidden" : ""}`}
        title={`${name}${locked ? " · 已锁定" : ""} · ${detail}${hidden ? " · 选择后在平面图中显示" : ""}`}
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
          <small>
            {detail}
            {locked ? " · 已锁定" : ""}
          </small>
        </span>
        {locked && (
          <LockSimpleIcon className="stage-object-lock" aria-hidden="true" />
        )}
      </button>
    );
  }
  const rooms = outlineRooms(project, members, query);
  return (
    <div className="stage-outline-shell">
      <div className="stage-outline-navigation">
        <small role="status">
          已选 {status.count} 个
          {status.hidden ? ` · ${status.hidden} 个在筛选外` : ""}
        </small>
        <div>
          <button disabled={busy || !status.active} onClick={navigation.locate}>
            定位所选对象
          </button>
          {!!query && (
            <button disabled={busy} onClick={onClearQuery}>
              清除场地筛选
            </button>
          )}
        </div>
      </div>
      <div
        ref={root}
        className="stage-objects"
        aria-label="场地对象"
        onKeyDown={navigation.onKeyDown}
      >
        {rooms.map((room) => {
          const { all, children } = room;
          if (!room.visible) return null;
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
                          ? visibility.hiddenSpaces.filter(
                              (id) => id !== room.id,
                            )
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
                            hidden ||
                              visibility.hiddenLayers.includes(category),
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
        {!!search && !rooms.some((room) => room.visible) && (
          <p className="wb-dim">没有匹配的对象</p>
        )}
      </div>
    </div>
  );
}
