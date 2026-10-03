import type { ProjectView } from "../../application-host";
import type { RigLayout } from "../../stage-types";
import type { RiggingCommand } from "../../rigging-preview-types";
import { canonical } from "../../stage-tools.ts";

export interface RiggingSession {
  source: string;
  ids: string[];
  rig: string;
  layout: boolean;
  draft: RigLayout;
  dirty: boolean;
}
function sourceKey(project: ProjectView) {
  return JSON.stringify([
    project.id,
    project.stage,
    project.fixtures.map((f) => f.id),
  ]);
}
export function beginRigging(
  project: ProjectView,
  ids: string[],
  rig: string,
): RiggingSession {
  return {
    source: sourceKey(project),
    ids: [...ids],
    rig,
    dirty: false,
    layout:
      !ids.length ||
      ids.some(
        (id) => !project.stage.placements.some((p) => p.fixtureId === id),
      ),
    draft: {
      startMarginMeters: "0.3",
      endMarginMeters: "0.3",
      dropMeters: "0.1",
    },
  };
}
export class RiggingInputError extends Error {
  readonly field: string;
  constructor(message: string, field: string) {
    super(message);
    this.field = field;
  }
}
export function riggingCommand(
  project: ProjectView,
  session: RiggingSession,
): RiggingCommand {
  if (session.source !== sourceKey(project))
    throw new Error("场地或灯具已变化，请取消本次挂接后重新选择");
  const rig = project.stage.constructions.find((r) => r.id === session.rig);
  if (!rig || rig.shape.kind !== "rig")
    throw new RiggingInputError("请选择目标支撑体", "rig");
  if (!session.ids.length) throw new Error("请先选择要挂接的灯具");
  if (
    session.ids.length > 256 ||
    new Set(session.ids).size !== session.ids.length
  )
    throw new Error("一次挂接需要 1–256 台不重复的灯具");
  if (session.ids.some((id) => !project.fixtures.some((f) => f.id === id)))
    throw new Error("所选灯具已不存在，请重新选择");
  let layout = null;
  if (session.layout) {
    for (const [key, label] of Object.entries({
      startMarginMeters: "首端余量",
      endMarginMeters: "末端余量",
      dropMeters: "下挂距离",
    })) {
      const value = session.draft[key as keyof RigLayout];
      if (
        !value.trim() ||
        !Number.isFinite(Number(value)) ||
        Number(value) < 0 ||
        Number(value) > 1000
      )
        throw new RiggingInputError(`${label}应为 0–1000 米之间的数字`, key);
    }
    const sum =
      Number(session.draft.startMarginMeters) +
      Number(session.draft.endMarginMeters);
    const length = Number(rig.shape.lengthMeters);
    if (sum > length || (session.ids.length > 1 && sum >= length))
      throw new RiggingInputError(
        "两端余量过大，没有足够长度排列灯具",
        "endMarginMeters",
      );
    layout = {
      startMarginMeters: canonical(session.draft.startMarginMeters),
      endMarginMeters: canonical(session.draft.endMarginMeters),
      dropMeters: canonical(session.draft.dropMeters),
    };
  }
  return {
    op: "attachFixtures",
    constructionId: session.rig,
    fixtureIds: [...session.ids],
    layout,
  };
}
