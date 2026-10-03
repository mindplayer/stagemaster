import type { ComponentProps } from "react";
import { StageInspector } from "./StageInspector";
import { StageMultiInspector } from "./StageMultiInspector";

/** Shared selection context; draft-specific inspectors remain owned by their sessions. */
export function StageSelectionInspector({
  ids,
  onArrange,
  onClear,
  ...props
}: ComponentProps<typeof StageInspector> & {
  ids: string[];
  onArrange(): void;
  onClear(): void;
}) {
  return props.object?.kind === "placement" && ids.length > 1 ? (
    <StageMultiInspector
      project={props.project}
      ids={ids}
      busy={props.busy}
      error={props.error}
      onArrange={onArrange}
      onHang={props.onHang}
      onDetach={() => props.onDetach(ids)}
      onClear={onClear}
      onLock={props.onLock}
    />
  ) : (
    <StageInspector {...props} />
  );
}
