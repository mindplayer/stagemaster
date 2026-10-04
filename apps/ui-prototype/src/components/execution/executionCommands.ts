import type {
  ExecutionAction,
  ExecutionBatchRequest,
  ExecutionRequest,
  ExecutionStatus,
  ExecutionView,
  ExecutionOutputAction,
} from "../../execution-types";
import type { ExecutionMediaAction } from "../../execution-media-types";
import { mediaRequestIdentity } from "../../media-seek-receipt.ts";

// Assembly boundary only: each operation retains the same authority/version/receipt path.
export function executionCommands(
  runtime: ExecutionView | null | undefined,
  disabled: boolean,
  isLiveBusy: () => boolean,
  request: (r: ExecutionRequest) => Promise<ExecutionStatus | undefined>,
) {
  const state = runtime?.observation.snapshot?.state;
  return {
    async source(source: string, action: ExecutionAction) {
      if (!runtime || !state || disabled || isLiveBusy()) return;
      return request({
        kind: "apply",
        hostId: runtime.hostId,
        revision: state.revision,
        source,
        action,
      });
    },
    async batch(value: ExecutionBatchRequest) {
      if (
        !runtime ||
        !state ||
        disabled ||
        isLiveBusy() ||
        value.hostId !== runtime.hostId ||
        value.revision !== state.revision
      )
        return;
      return request(value);
    },
    async output(action: ExecutionOutputAction) {
      if (!runtime || !state || disabled || isLiveBusy()) return;
      return request({
        kind: "output",
        hostId: runtime.hostId,
        revision: state.revision,
        action,
      });
    },
    async media(action: ExecutionMediaAction) {
      const group = state?.media?.find(
        (m) => m.id === runtime?.catalog.audio?.group,
      );
      if (!runtime || !state || !group || disabled || isLiveBusy()) return null;
      const value = await request({
        kind: "media",
        hostId: runtime.hostId,
        revision: state.revision,
        group: group.id,
        generation: group.generation,
        action,
      });
      return mediaRequestIdentity(value?.runtime);
    },
  };
}
