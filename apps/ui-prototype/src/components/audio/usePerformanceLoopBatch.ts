import { useEffect, useRef, useState } from "react";
import type { AudioEdit, AudioTimeline } from "../../audio-types";
import type { AudioLoopRegion } from "../../audio-performance-types";
import type { ProjectView } from "../../application-host";
import type { OrderedSelection } from "../selection/ordered-selection";
import {
  guardLoopGroup,
  loopGroupCommand,
  type LoopGroupOperation,
} from "./performance-loop-group";

/** Exact group input is independent from the persisted edit; failures keep the input. */
export function usePerformanceLoopBatch(props: {
  track: AudioTimeline | null;
  identity: string;
  visible: boolean;
  active: boolean;
  selection: OrderedSelection<AudioLoopRegion>;
  execute(
    command: AudioEdit | ((track: AudioTimeline) => AudioEdit),
  ): Promise<ProjectView | null>;
}) {
  const [destination, setDestination] = useState<string | null>(null);
  const [problem, setProblem] = useState("");
  const [removing, setRemoving] = useState(false);
  const [working, setWorking] = useState(false);
  const inFlight = useRef(false),
    epoch = useRef(0);
  const signature = `${props.identity}:${props.active}:${props.selection.ids.join("|")}`;
  const context = `${signature}:${props.visible}`;
  const old = useRef(context);
  if (old.current !== context) {
    old.current = context;
    epoch.current++;
  }
  useEffect(() => {
    setDestination(null);
    setProblem("");
    setRemoving(false);
  }, [signature]);
  useEffect(() => {
    if (!props.visible) setRemoving(false);
  }, [props.visible]);
  useEffect(
    () => () => {
      epoch.current++;
    },
    [],
  );
  const cancel = () => {
    setDestination(null);
    setProblem("");
    setRemoving(false);
  };
  async function run(kind: LoopGroupOperation, target = destination ?? "") {
    if (inFlight.current || !props.track || !props.visible || !props.active)
      return;
    const ticket = epoch.current;
    inFlight.current = true;
    setWorking(true);
    setProblem("");
    try {
      const command = loopGroupCommand(
        props.track,
        props.selection.ids,
        kind,
        target,
      );
      const originals = (props.track.loopRegions ?? []).filter((r) =>
        props.selection.ids.includes(r.id),
      );
      const result = await props.execute((track) => {
        guardLoopGroup(track, originals);
        return command;
      });
      if (ticket !== epoch.current) return;
      if (!result) {
        setProblem("区段组操作未完成；输入已保留，请查看工程错误提示");
        return;
      }
      cancel();
      if (kind === "copy") {
        const oldIds = new Set(props.track.loopRegions?.map((r) => r.id));
        props.selection.replace(
          (result.audio?.loopRegions ?? [])
            .filter((r) => !oldIds.has(r.id))
            .map((r) => r.id),
        );
      } else if (kind === "remove") props.selection.replace([]);
    } catch (error) {
      if (ticket === epoch.current)
        setProblem(error instanceof Error ? error.message : String(error));
    } finally {
      inFlight.current = false;
      setWorking(false);
    }
  }
  return {
    destination,
    setDestination,
    problem,
    removing,
    setRemoving,
    working,
    pending: destination !== null || removing,
    cancel,
    run,
  };
}
