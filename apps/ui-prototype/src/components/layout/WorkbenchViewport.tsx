import { isStageLocked } from "../../stage-locks";
import {
  forwardRef,
  useContext,
  useState,
  type ComponentPropsWithoutRef,
} from "react";
import type { ProjectView } from "../../application-host";
import { SharedPrevis, type SharedPrevisHandle } from "../stage/SharedPrevis";
import { FixturePlan } from "../scene-plan/FixturePlan";
import { StageOverview } from "../stage/StageOverview";
import { DockPane, ViewportRevealContext } from "./DockPane";
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
  const revealViewport = useContext(ViewportRevealContext);
  const [sceneView, setSceneView] = useState<View>("plan");
  const [playbackView, setPlaybackView] = useState<View>("plan");
  const view =
    page === "stage" ? stageView : page === "scenes" ? sceneView : playbackView;
  const overviewPage = page === "sequences" || page === "audio";
  const viewportVisible = !["fixtures", "profiles", "settings"].includes(page);
  const { transport, ...previs } = preview;
  const controls = viewportVisible ? (
    <StageViewTabs
      value={view}
      planLabel={
        page === "scenes"
          ? "平面选灯"
          : page === "stage"
            ? "平面布置"
            : "平面场地"
      }
      busy={preview.busy}
      onChange={(value) => {
        if (page === "stage") onStageView(value);
        else
          void preview.run(async () => {
            if (page === "scenes") setSceneView(value);
            else setPlaybackView(value);
          });
      }}
    />
  ) : undefined;
  return (
    <>
      <DockPane
        region="viewport"
        keepConnected
        visible={viewportVisible && view === "three"}
      >
        <SharedPrevis
          {...previs}
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
            revealViewport();
            if (page === "stage") onStageView("three");
            else if (page === "scenes") setSceneView("three");
            else setPlaybackView("three");
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
      <DockPane region="viewport" visible={overviewPage && view === "plan"}>
        <StageOverview
          project={project}
          visible={overviewPage && view === "plan"}
          viewControls={controls}
        />
      </DockPane>
      <DockPane
        region="viewport"
        className="viewport-transport-pane"
        visible={viewportVisible}
      >
        {transport}
      </DockPane>
    </>
  );
});
