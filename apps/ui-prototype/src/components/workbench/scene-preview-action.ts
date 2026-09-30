import type { ApplicationHost } from "../../application-host";

/** Compose existing host operations, keeping its generation/epoch/serial guards. */
export async function startScenePreview(
  host: Pick<ApplicationHost, "request" | "preview">,
  sceneId: string,
  current: () => boolean,
) {
  function guard() {
    if (!current()) throw new Error("编辑目标已更换，请重新预演当前场景");
  }
  guard();
  const project = await host.request({ kind: "snapshot" });
  guard();
  const loaded = await host.preview({
    kind: "loadScene",
    generation: project.generation,
    sceneId,
  });
  guard();
  if (loaded.loaded?.sceneId !== sceneId)
    throw new Error("场景载入未完成，请重试");
  return host.preview({
    kind: "control",
    epoch: loaded.epoch,
    serial: loaded.controlSerial + 1,
    command: { kind: "execute", stepId: sceneId },
  });
}
