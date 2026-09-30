import { useEffect, useRef, useState } from "react";
import type { ApplicationHost, SceneView } from "../../application-host";
import type {
  PreviewCommand,
  PreviewSnapshot,
  SequenceView,
} from "../../sequence-types";
import { startScenePreview } from "./scene-preview-action";

export interface PreviewControllerOptions {
  host: ApplicationHost;
  sequence?: SequenceView;
  scene?: SceneView;
  beforeAction(): Promise<boolean>;
  visible: boolean;
  onView3d?(): void;
}

/** Shared host player commands and polling, with no independent playback clock. */
export function usePreviewController({
  host,
  sequence,
  scene,
  beforeAction,
  visible,
  onView3d,
}: PreviewControllerOptions) {
  const [snapshot, setSnapshot] = useState<PreviewSnapshot>({
    epoch: 0,
    controlSerial: 0,
    loaded: null,
  });
  const [error, setError] = useState("");
  const errorSource = useRef<"poll" | "command" | null>(null);
  const [working, setWorking] = useState(false);
  const current = useRef(snapshot);
  const controlBusy = useRef(false);
  const epochRequest = useRef(0);
  const alive = useRef(true);
  const target = useRef(scene?.id);
  target.current = visible ? scene?.id : undefined;
  const publish = (value: PreviewSnapshot) => {
    if (alive.current) {
      // Another workspace may successfully replace a failed preview. Keep command
      // errors during ordinary polls, but never attach them to a new player epoch.
      if (
        errorSource.current === "poll" ||
        value.epoch !== current.current.epoch
      ) {
        errorSource.current = null;
        setError("");
      }
      current.current = value;
      setSnapshot(value);
    }
  };
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);
  useEffect(() => {
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      const version = epochRequest.current;
      if (!controlBusy.current) {
        try {
          const next = await host.preview({ kind: "snapshot" });
          if (!disposed && version === epochRequest.current) publish(next);
        } catch (reason) {
          if (!disposed && version === epochRequest.current) {
            errorSource.current = "poll";
            setError(String(reason));
          }
        }
      }
      if (!disposed) timer = setTimeout(poll, visible ? 80 : 500);
    };
    void poll();
    return () => {
      disposed = true;
      clearTimeout(timer);
    };
  }, [host, visible]);
  async function act(command: PreviewCommand | "load" | "startScene") {
    if (controlBusy.current) return;
    controlBusy.current = true;
    epochRequest.current++;
    setWorking(true);
    errorSource.current = null;
    setError("");
    try {
      // Stop/pause must remain available even when an unrelated editor draft is invalid.
      if (
        (typeof command === "string" ||
          !["stop", "pause"].includes(command.kind)) &&
        !(await beforeAction())
      )
        return;
      // Flush can change the project generation, so obtain the host's authoritative snapshot.
      if (command === "startScene" && scene) {
        const result = await startScenePreview(
          host,
          scene.id,
          () => alive.current && target.current === scene.id,
        );
        publish(result);
        if (alive.current && target.current === scene.id) onView3d?.();
      } else if (command === "load") {
        if (!sequence && !scene) return;
        const project = await host.request({ kind: "snapshot" });
        publish(
          await host.preview(
            scene
              ? {
                  kind: "loadScene",
                  generation: project.generation,
                  sceneId: scene.id,
                }
              : {
                  kind: "load",
                  generation: project.generation,
                  sequenceId: sequence!.id,
                },
          ),
        );
      } else if (typeof command !== "string") {
        // Several views share one player. Read the current serial, never restart a local counter.
        const latest = await host.preview({ kind: "snapshot" });
        if (latest.epoch !== current.current.epoch) {
          publish(latest);
          throw new Error("预览内容已更换，请确认后重试");
        }
        publish(
          await host.preview({
            kind: "control",
            epoch: current.current.epoch,
            serial: latest.controlSerial + 1,
            command,
          }),
        );
      }
    } catch (reason) {
      if (alive.current) {
        errorSource.current = "command";
        setError(reason instanceof Error ? reason.message : String(reason));
      }
    } finally {
      controlBusy.current = false;
      if (alive.current) setWorking(false);
    }
  }
  return { snapshot, error, working, act };
}
export type PreviewController = ReturnType<typeof usePreviewController>;
