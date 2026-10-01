import { StageFixtureObjects } from "./StageFixtureObjects";
import type { LabelBox } from "../../plan-label-layout";
import type { PlanLabelMode } from "../../fixture-plan-display";
import { seatingLayout } from "../../seating-tools";
import { SeatingPlanObject } from "./SeatingPlanObject";
import type { ProjectView } from "../../application-host";
import type { StageObject, StageSelection, StageView } from "../../stage-types";
import { objectOutline } from "../../stage-tools";
import { rigOutline } from "../../rigging-tools";
import { displayMeters } from "./stage-display";
export function StagePlanObjects({
  project,
  stage,
  drawn,
  isSelected,
  selectedIds,
  unit,
  labels,
  viewport,
  onHiddenLabels,
}: {
  project: ProjectView;
  stage: StageView;
  drawn(object: StageObject): StageObject;
  isSelected(kind: StageSelection["kind"], id: string): boolean;
  selectedIds: string[];
  unit: number;
  labels: PlanLabelMode;
  viewport?: LabelBox;
  onHiddenLabels?(count: number): void;
}) {
  const outline = (object: StageObject) =>
    objectOutline(drawn(object))!
      .map((p) => `${p[0]},${-Number(p[1])}`)
      .join(" ");
  return (
    <>
      {stage.spaces.map((space) => (
        <g
          key={space.id}
          data-kind="space"
          data-id={space.id}
          className={isSelected("space", space.id) ? "is-selected" : ""}
        >
          <polygon
            className="stage-room"
            points={outline({ kind: "space", value: space })}
            strokeWidth={unit * 0.14}
          />
          <text
            className="stage-room-label"
            x={
              Math.min(
                ...objectOutline(drawn({ kind: "space", value: space }))!.map(
                  (p) => Number(p[0]),
                ),
              ) + unit
            }
            y={
              -Math.max(
                ...objectOutline(drawn({ kind: "space", value: space }))!.map(
                  (p) => Number(p[1]),
                ),
              ) +
              unit * 2
            }
            fontSize={unit * 1.15}
          >
            {space.name}
          </text>
        </g>
      ))}
      {stage.constructions
        .filter((c) => c.shape.kind === "platform")
        .map((c) => (
          <g
            key={c.id}
            data-kind="construction"
            data-id={c.id}
            className={isSelected("construction", c.id) ? "is-selected" : ""}
          >
            <polygon
              className="stage-platform"
              points={outline({ kind: "construction", value: c })}
              strokeWidth={unit * 0.15}
            />
            <title>{c.name}</title>
          </g>
        ))}
      {stage.constructions
        .filter((c) => c.shape.kind === "rig")
        .map((c) => {
          const item = drawn({ kind: "construction", value: c });
          if (item.kind !== "construction" || item.value.shape.kind !== "rig")
            return null;
          const rig = item.value.shape;
          return (
            <g
              key={c.id}
              data-kind="construction"
              data-id={c.id}
              className={`stage-rig ${isSelected("construction", c.id) ? "is-selected" : ""}`}
            >
              <polygon
                points={rigOutline(rig)
                  .map((p) => `${p[0]},${-p[1]}`)
                  .join(" ")}
                strokeWidth={unit * 0.18}
              />
              {labels === "name" && (
                <text
                  x={Number(rig.positionMeters.x)}
                  y={
                    -Number(rig.positionMeters.y) -
                    Number(rig.widthMeters) / 2 -
                    unit * 1.4
                  }
                  fontSize={unit}
                  textAnchor="middle"
                >
                  {c.name}
                </text>
              )}
              <title>
                {c.name} · {displayMeters(rig.lengthMeters)} 米 · 标高{" "}
                {displayMeters(rig.positionMeters.z)} 米
              </title>
            </g>
          );
        })}
      {stage.constructions
        .filter((c) => c.shape.kind === "seating")
        .map((c) => {
          const item = drawn({ kind: "construction", value: c });
          if (
            c.shape.kind !== "seating" ||
            item.kind !== "construction" ||
            item.value.shape.kind !== "seating"
          )
            return null;
          return (
            <SeatingPlanObject
              key={c.id}
              id={c.id}
              name={c.name}
              shape={
                seatingLayout(item.value.shape) ? item.value.shape : c.shape
              }
              selected={isSelected("construction", c.id)}
              labels={labels === "name"}
              unit={unit}
            />
          );
        })}
      <StageFixtureObjects
        project={project}
        stage={stage}
        drawn={drawn}
        selectedIds={selectedIds}
        selected={(id) => isSelected("placement", id)}
        unit={unit}
        labels={labels}
        viewport={viewport}
        onHiddenLabels={onHiddenLabels}
      />
    </>
  );
}
