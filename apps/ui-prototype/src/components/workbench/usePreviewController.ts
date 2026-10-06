import { useEffect, useMemo, useRef, useState } from "react";
import type { ApplicationHost, SceneView } from "../../application-host";
import type {
  PreviewCommand,
  PreviewSnapshot,
  SequenceView,
} from "../../sequence-types";
import { performPreviewIntent } from "./preview-intent";

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
  const context = useMemo(
    () => ({ host, sceneId: scene?.id, sequenceId: sequence?.id, visible }),
    [host, scene?.id, sequence?.id, visible],
  );
  const selected = useRef(context);
  selected.current = context;
  const errorContext = useRef(context);
  const errorHost = useRef(host);
  const [working, setWorking] = useState(false);
  const current = useRef(snapshot);
  const snapshotHost = useRef(host);
  const controlBusy = useRef(false);
  const epochRequest = useRef(0);
  const alive = useRef(true);
  const publish = (value: PreviewSnapshot) => {
    if (alive.current && selected.current.host === host) {
      // Another workspace may successfully replace a failed preview. Keep command
      // errors during ordinary polls, but never attach them to a new player epoch.
      if (
        errorSource.current === "poll" ||
        value.epoch !== current.current.epoch
      ) {
        errorSource.current = null;
        setError("");
      }
      snapshotHost.current = host;
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
            errorHost.current = host;
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
    const valid = () =>
      alive.current && selected.current === context && context.visible;
    if (controlBusy.current || !valid()) return;
    controlBusy.current = true;
    epochRequest.current++;
    setWorking(true);
    errorSource.current = null;
    setError("");
    try {
      const result = await performPreviewIntent({
        host,
        sceneId: context.sceneId,
        sequenceId: context.sequenceId,
        command,
        expectedEpoch:
          snapshotHost.current === host ? current.current.epoch : 0,
        draftEffect: !!current.current.loaded?.draftEffectId,
        beforeAction,
        current: valid,
        replaced: publish,
      });
      if (result && valid()) {
        publish(result);
        if (command === "startScene") onView3d?.();
      }
    } catch (reason) {
      if (valid()) {
        errorSource.current = "command";
        errorContext.current = context;
        errorHost.current = host;
        setError(reason instanceof Error ? reason.message : String(reason));
      }
    } finally {
      controlBusy.current = false;
      if (alive.current) setWorking(false);
    }
  }
  return {
    snapshot:
      snapshotHost.current === host
        ? snapshot
        : { epoch: 0, controlSerial: 0, loaded: null },
    error:
      errorHost.current === host &&
      (errorSource.current !== "command" || errorContext.current === context)
        ? error
        : "",
    working,
    act,
  };
}
export type PreviewController = ReturnType<typeof usePreviewController>;
