import { useEffect, useRef, useState } from "react";
import type { AudioEdit, AudioTimeline } from "../../audio-types";
import type { ProjectView } from "../../application-host";
import type { AudioLoopGroupAction } from "../../audio-performance-types";
import {
  newPerformanceLoop,
  type PerformanceLoopDraft,
} from "./performance-loop-draft";
import { loopMotionCommand, type LoopMotion } from "./performance-loop-motion";
import type { AudioLoopRegion } from "../../audio-performance-types";

/** Drafts use the existing workbench transaction. Delayed actions cannot cross workspaces. */
export function usePerformanceLoopActions(props: {
  track: AudioTimeline | null;
  identity: string;
  selected: string;
  visible: boolean;
  contextKey?: string;
  position: number;
  beforeChange(): Promise<boolean>;
  edit(command: AudioEdit): Promise<ProjectView | null>;
  onDraft(value: PerformanceLoopDraft): void;
  onSelect(id: string): void;
  onProblem(value: string): void;
}) {
  const current = useRef(props);
  const inFlight = useRef(false);
  const epoch = useRef(0);
  const context = `${props.identity}:${props.visible}:${props.selected}:${props.contextKey ?? ""}`;
  const previous = useRef(context);
  current.current = props;
  if (previous.current !== context) {
    previous.current = context;
    epoch.current++;
  }
  const [acting, setActing] = useState(false);
  const [removing, setRemoving] = useState<{ id: string; name: string } | null>(
    null,
  );
  useEffect(() => {
    setRemoving(null);
  }, [context]);
  useEffect(
    () => () => {
      epoch.current++;
    },
    [],
  );
  const region = props.track?.loopRegions?.find((r) => r.id === props.selected);
  async function run<T>(
    work: (active: () => boolean) => Promise<T>,
  ): Promise<T | null> {
    if (inFlight.current || !current.current.visible) return null;
    const ticket = epoch.current;
    inFlight.current = true;
    setActing(true);
    try {
      if (!(await current.current.beforeChange()) || ticket !== epoch.current)
        return null;
      current.current.onProblem("");
      const result = await work(() => ticket === epoch.current);
      return ticket === epoch.current ? result : null;
    } catch (error) {
      if (ticket === epoch.current)
        current.current.onProblem(
          error instanceof Error ? error.message : String(error),
        );
      return null;
    } finally {
      inFlight.current = false;
      setActing(false);
    }
  }
  async function group(action: AudioLoopGroupAction) {
    const id = region?.id;
    if (!id) return;
    await run(async (active) => {
      const result = await current.current.edit({
        kind: "loopRegions",
        command: { kind: "edit", ids: [id], action },
      });
      if (!result && active())
        current.current.onProblem("区段操作未完成，请查看工程错误提示");
    });
  }
  return {
    region,
    acting,
    removing,
    async mode(changeMode: () => void) {
      await run(async () => {
        changeMode();
      });
    },
    execute(command: AudioEdit | ((track: AudioTimeline) => AudioEdit)) {
      return run(async () => {
        const track = current.current.track;
        if (!track) return null;
        return current.current.edit(
          typeof command === "function" ? command(track) : command,
        );
      });
    },
    async motion(
      original: AudioLoopRegion,
      next: AudioLoopRegion,
      mode: LoopMotion,
    ) {
      await run(async () => {
        const track = current.current.track;
        if (track)
          await current.current.edit(
            loopMotionCommand(track, original, next, mode),
          );
      });
    },
    async add() {
      await run(async () => {
        const p = current.current;
        if (!p.track) return;
        const draft = newPerformanceLoop(p.track, p.position);
        p.onSelect("new-performance-loop");
        p.onDraft(draft);
      });
    },
    lock: () => group({ kind: "locked", locked: !region?.locked }),
    enabled: () => group({ kind: "enabled", enabled: !region?.enabled }),
    async requestRemove() {
      if (region)
        await run(async () => {
          setRemoving({ id: region.id, name: region.name });
        });
    },
    cancelRemove: () => setRemoving(null),
    async remove() {
      if (!removing) return;
      await run(async (active) => {
        const result = await current.current.edit({
          kind: "loopRegions",
          command: {
            kind: "edit",
            ids: [removing.id],
            action: { kind: "remove" },
          },
        });
        if (result && active()) {
          setRemoving(null);
          current.current.onSelect("");
        }
      });
    },
  };
}
