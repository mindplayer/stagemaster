import { isStageLocked } from "../../stage-locks";
import { forwardRef, useState, type ComponentPropsWithoutRef } from "react";
import type { ProjectView } from "../../application-host";
import { SharedPrevis, type SharedPrevisHandle } from "../stage/SharedPrevis";
import { FixturePlan } from "../scene-plan/FixturePlan";
import { DockPane } from "./DockPane";
import { StageViewTabs } from "./StageViewTabs";
import type { WorkbenchPage } from "./WorkbenchNavigation";
type View = "plan" | "three";
type PreviewProps = Omit<
  ComponentPropsWithoutRef<typeof SharedPrevis>,
  "viewControls" | "contextKey" | "onReveal" | "limitedFixtures"
>;
export const WorkbenchViewport = forwardRef<
  SharedPrevisHandle,
  {
    project: ProjectView;
    page: WorkbenchPage;
    stageView: View;
    onStageView(value: View): void;
    preview: PreviewProps;
    query: string;
    onlySelected: boolean;
    selected: string[];
    onQuery(value: string): void;
    onFilter(value: boolean): void;
    onSelect(ids: string[]): Promise<boolean>;
  }
>(function WorkbenchViewport(
  {
    project,
    page,
    stageView,
    onStageView,
    preview,
    query,
    onlySelected,
    selected,
    onQuery,
    onFilter,
    onSelect,
  },
  ref,
) {
  const [sceneView, setSceneView] = useState<View>("plan");
  const view = page === "stage" ? stageView : sceneView;
  const controls =
    page === "scenes" || page === "stage" ? (
      <StageViewTabs
        value={view}
        planLabel={page === "scenes" ? "平面选灯" : "平面布置"}
        busy={preview.busy}
        onChange={(value) => {
          if (page === "stage") onStageView(value);
          else void preview.run(async () => setSceneView(value));
        }}
      />
    ) : undefined;
  return (
    <>
      <DockPane
        region="viewport"
        keepConnected
        visible={
          !["fixtures", "profiles", "settings"].includes(page) &&
          (!["scenes", "stage"].includes(page) || view === "three")
        }
      >
        <SharedPrevis
          {...preview}
          placementLocked={isStageLocked(project.stage, {
            kind: "placement",
            id: preview.selectedId,
          })}
          limitedFixtures={project.fixtures.filter(
            (f) =>
              f.attributes.some((a) => a.function) &&
              project.stage.placements.some((p) => p.fixtureId === f.id),
          )}
          ref={ref}
          fixed
          viewControls={controls}
          contextKey={`${page}:${view}`}
          onReveal={() => {
            if (page === "stage") onStageView("three");
            else setSceneView("three");
          }}
        />
      </DockPane>
      <DockPane
        region="viewport"
        visible={page === "scenes" && sceneView === "plan"}
      >
        <FixturePlan
          project={project}
          selected={selected}
          query={query}
          onlySelected={onlySelected}
          busy={preview.busy}
          visible={page === "scenes" && sceneView === "plan"}
          onQuery={onQuery}
          onFilter={onFilter}
          onSelect={onSelect}
          viewControls={controls}
        />
      </DockPane>
    </>
  );
});
