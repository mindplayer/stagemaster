import type { ProjectView } from "../../application-host";
import type { StageSelection } from "../../stage-types";
import type { OutlineMember } from "./stage-outliner-model";
export const stageTargetKey = (target: StageSelection) =>
  `${target.kind}:${target.id}`;
export function outlineRooms(
  project: ProjectView,
  members: OutlineMember[],
  query: string,
) {
  const search = query.trim().toLocaleLowerCase();
  const matches = (value: string) => value.toLocaleLowerCase().includes(search);
  return [
    ...project.stage.spaces.map((space) => ({
      id: space.id as string | null,
      name: space.name,
      space,
    })),
    { id: null, name: "未归属空间", space: null },
  ].map((room) => {
    const all = members.filter((m) => m.space === room.id);
    const children = all.filter(
      (m) => matches(room.name) || matches(`${m.name} ${m.detail}`),
    );
    return {
      ...room,
      all,
      children,
      visible: !!children.length || (!!room.space && matches(room.name)),
    };
  });
}
export function outlineSelectionStatus(
  project: ProjectView,
  members: OutlineMember[],
  query: string,
  selection: StageSelection | null,
  selectedIds: string[],
) {
  const all = new Set([
    ...project.stage.spaces.map((s) =>
      stageTargetKey({ kind: "space", id: s.id }),
    ),
    ...members.map((m) => stageTargetKey(m.target)),
  ]);
  const shown = new Set(
    outlineRooms(project, members, query)
      .filter((r) => r.visible)
      .flatMap((r) => [
        ...(r.space ? [stageTargetKey({ kind: "space", id: r.space.id })] : []),
        ...r.children.map((m) => stageTargetKey(m.target)),
      ]),
  );
  const chosen = new Set(
    selection?.kind === "placement"
      ? selectedIds
          .map((id) => stageTargetKey({ kind: "placement", id }))
          .filter((key) => all.has(key))
      : selection && all.has(stageTargetKey(selection))
        ? [stageTargetKey(selection)]
        : [],
  );
  return {
    count: chosen.size,
    hidden: [...chosen].filter((key) => !shown.has(key)).length,
    active: selection && all.has(stageTargetKey(selection)) ? selection : null,
  };
}
