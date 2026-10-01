import type { ProjectView } from "../../application-host";
import type { useStageObjects } from "./useStageObjects";
import { SeatingCreateDialog } from "./SeatingCreateDialog";
import { RigCreateDialog } from "./RigCreateDialog";
import { StageCreateDialog } from "./StageCreateDialog";
import { DeleteDialog } from "../workbench/DeleteDialog";
export function StageObjectDialogs({
  project,
  busy,
  error,
  actions,
}: {
  project: ProjectView;
  busy: boolean;
  error: string;
  actions: ReturnType<typeof useStageObjects>;
}) {
  const { rigCreation, seatingCreation, creation, deleteTarget } = actions;
  return (
    <>
      {seatingCreation && (
        <SeatingCreateDialog
          project={project}
          initial={seatingCreation.shape}
          name={seatingCreation.name}
          busy={busy}
          error={error}
          onCancel={() => actions.setSeatingCreation(null)}
          onApply={async (command) => !!(await actions.createObject(command))}
        />
      )}
      {rigCreation && (
        <RigCreateDialog
          project={project}
          initial={rigCreation.shape}
          name={rigCreation.name}
          busy={busy}
          error={error}
          onCancel={() => actions.setRigCreation(null)}
          onApply={async (command) => !!(await actions.createObject(command))}
        />
      )}
      {creation && (
        <StageCreateDialog
          initial={creation}
          busy={busy}
          error={error}
          onCreate={(command) => void actions.createObject(command)}
          onCancel={() => actions.setCreation(null)}
        />
      )}
      {deleteTarget && (
        <DeleteDialog
          name={
            deleteTarget.kind === "placement"
              ? "此灯位"
              : deleteTarget.value.name
          }
          busy={busy}
          error={error}
          description={
            deleteTarget.kind === "space"
              ? "同时移除该空间的围护。灯具和舞台保留原位置并解除空间归属，灯光编排保留。"
              : deleteTarget.kind === "placement"
                ? "灯具配适与编排保留，只移除安装位置。"
                : deleteTarget.kind === "construction" &&
                    deleteTarget.value.shape.kind === "rig"
                  ? "保留全部灯具和灯位，解除挂接后删除支撑体；一次撤销可恢复。"
                  : "此操作可撤销恢复。"
          }
          onCancel={() => actions.setDeleteTarget(null)}
          onDelete={() => void actions.remove()}
        />
      )}
    </>
  );
}
