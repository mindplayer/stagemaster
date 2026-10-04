import { sameTargets, type PrevisTarget } from "../../previs-objects.ts";
import type { RefObject } from "react";
import type { ProjectRequest, ProjectView } from "../../application-host";
import type { PrevisInteractions } from "../../previs-types";
import type { StageHandle } from "../stage/StageWorkspace";
import { sameFixtureSelection } from "../../previs-selection.ts";

/** Workspace policy and ordered host writes; renderer transport stays in the viewport. */
export function previsInteractions({
  page,
  stage,
  selectedIds,
  selectedTargets,
  project,
  run,
  request,
  select,
  notice,
}: {
  page: string;
  stage: RefObject<StageHandle | null>;
  selectedIds: string[];
  selectedTargets?: PrevisTarget[];
  project(): ProjectView | null | undefined;
  run(work: () => Promise<void>): Promise<boolean>;
  request(command: ProjectRequest): Promise<unknown>;
  select(ids: string[]): void;
  notice(message: string): void;
}): PrevisInteractions {
  return {
    selectedIds,
    selectedTargets,
    onSelectTargets: (targets, isActive) =>
      page === "stage"
        ? (stage.current?.selectObjects(targets, isActive) ??
          Promise.resolve(false))
        : Promise.resolve(false),
    onObjectTranslation: (proposal, isActive) =>
      run(async () => {
        if (
          page !== "stage" ||
          !isActive() ||
          !selectedTargets ||
          !sameTargets(selectedTargets, proposal.targets)
        )
          throw new Error("三维选择或编辑上下文已变化，场地未修改");
        await request({
          kind: "previsObjectTranslation",
          generation: proposal.generation,
          version: proposal.version,
          targets: proposal.targets,
          deltaMeters: proposal.deltaMeters,
        });
        notice(`${proposal.targets.length} 个场地对象已移动，可一次撤销恢复`);
      }),
    onSelect: (ids, isActive) => {
      if (page === "stage")
        return (
          stage.current?.selectFixtures(ids, isActive) ?? Promise.resolve(false)
        );
      if (page !== "scenes") return Promise.resolve(false);
      return run(async () => {
        if (!isActive()) throw new Error("三维选择上下文已变化，请重新选择");
        const fixtures = project()?.fixtures;
        if (ids.some((id) => !fixtures?.some((f) => f.id === id)))
          throw new Error("所选灯具已不存在");
        select(ids);
      });
    },
    onPrepareMove: () => run(async () => {}),
    onTransform: (proposal, isActive) =>
      run(async () => {
        if (
          page !== "stage" ||
          !isActive() ||
          !sameFixtureSelection(selectedIds, proposal.fixtureIds)
        )
          throw new Error("三维选择或编辑上下文已变化，灯位未修改");
        await request({
          kind: "previsTransform",
          generation: proposal.generation,
          version: proposal.version,
          fixtureIds: proposal.fixtureIds,
          yawDegrees: proposal.yawDegrees,
          spacingScale: proposal.spacingScale,
        });
        notice(
          `${proposal.fixtureIds.length} 台灯具已整组变换，可一次撤销恢复`,
        );
      }),
    onTranslation: (proposal, isActive) =>
      run(async () => {
        if (
          page !== "stage" ||
          !isActive() ||
          !sameFixtureSelection(selectedIds, proposal.fixtureIds)
        )
          throw new Error("三维选择或编辑上下文已变化，灯位未修改");
        await request({
          kind: "previsTranslation",
          generation: proposal.generation,
          version: proposal.version,
          fixtureIds: proposal.fixtureIds,
          deltaMeters: proposal.deltaMeters,
        });
        notice(`${proposal.fixtureIds.length} 台灯位已更新，可一次撤销恢复`);
      }),
  };
}
