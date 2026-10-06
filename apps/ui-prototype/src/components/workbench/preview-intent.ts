import type { ApplicationHost } from "../../application-host.ts";
import type { PreviewCommand, PreviewSnapshot } from "../../sequence-types.ts";
import { startScenePreview } from "./scene-preview-action.ts";

export type PreviewIntentCommand = PreviewCommand | "load" | "startScene";
export interface PreviewIntent {
  host: Pick<ApplicationHost, "request" | "preview">;
  sceneId?: string;
  sequenceId?: string;
  command: PreviewIntentCommand;
  expectedEpoch: number;
  draftEffect: boolean;
  beforeAction(): Promise<boolean>;
  current(): boolean;
  replaced(snapshot: PreviewSnapshot): void;
}

/** Only compose the existing host player; leaving a binding cancels UNSENT intent, not a sent command. */
export async function performPreviewIntent({
  host,
  sceneId,
  sequenceId,
  command,
  expectedEpoch,
  draftEffect,
  beforeAction,
  current,
  replaced,
}: PreviewIntent): Promise<PreviewSnapshot | undefined> {
  if (!current()) return;
  const needsDraft =
    typeof command === "string" ||
    (!["stop", "pause", "setRate"].includes(command.kind) &&
      !(command.kind === "resume" && draftEffect));
  if (needsDraft && !(await beforeAction())) return;
  if (!current()) return;
  let result: PreviewSnapshot;
  if (command === "startScene") {
    if (!sceneId) return;
    result = await startScenePreview(host, sceneId, current);
  } else if (command === "load") {
    if (!sceneId && !sequenceId) return;
    const project = await host.request({ kind: "snapshot" });
    if (!current()) return;
    result = await host.preview(
      sceneId
        ? { kind: "loadScene", generation: project.generation, sceneId }
        : {
            kind: "load",
            generation: project.generation,
            sequenceId: sequenceId!,
          },
    );
  } else {
    const latest = await host.preview({ kind: "snapshot" });
    if (!current()) return;
    if (latest.epoch !== expectedEpoch) {
      replaced(latest);
      throw new Error("预览内容已更换，请确认后重试");
    }
    result = await host.preview({
      kind: "control",
      epoch: latest.epoch,
      serial: latest.controlSerial + 1,
      command,
    });
  }
  return current() ? result : undefined;
}
