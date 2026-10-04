import { useEffect, useRef, useState } from "react";
import type { AudioEdit, AudioTimeline } from "../../audio-types";
import type { ProjectView } from "../../application-host";
import type { AudioLoopGroupAction } from "../../audio-performance-types";
import { newPerformanceLoop, type PerformanceLoopDraft } from "./performance-loop-draft";

/** Drafts use the existing workbench transaction. Delayed actions cannot cross workspaces. */
export function usePerformanceLoopActions(props: {
  track: AudioTimeline | null;
  identity: string;
  selected: string;
  visible: boolean;
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
  const context = `${props.identity}:${props.visible}:${props.selected}`;
  const previous = useRef(context);
  current.current = props;
  if (previous.current !== context) {
    previous.current = context;
    epoch.current++;
  }
  const [acting, setActing] = useState(false);
  const [removing, setRemoving] = useState<{ id: string; name: string } | null>(null);
  useEffect(() => { setRemoving(null); }, [context]);
  useEffect(() => () => { epoch.current++; }, []);
  const region = props.track?.loopRegions?.find((r) => r.id === props.selected);
  async function run(work: (active: () => boolean) => Promise<void>) {
    if (inFlight.current || !current.current.visible) return;
    const ticket = epoch.current;
    inFlight.current = true;
    setActing(true);
    try {
      if (!(await current.current.beforeChange()) || ticket !== epoch.current) return;
      await work(() => ticket === epoch.current);
    } catch (error) {
      if (ticket === epoch.current) current.current.onProblem(error instanceof Error ? error.message : String(error));
    } finally {
      inFlight.current = false;
      setActing(false);
    }
  }
  async function group(action: AudioLoopGroupAction) {
    const id = region?.id;
    if (!id) return;
    await run(async (active) => {
      const result = await current.current.edit({ kind: "loopRegions", command: { kind: "edit", ids: [id], action } });
      if (!result && active()) current.current.onProblem("区段操作未完成，请查看工程错误提示");
    });
  }
  return {
    region, acting, removing,
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
      if (region) await run(async () => { setRemoving({ id: region.id, name: region.name }); });
    },
    cancelRemove: () => setRemoving(null),
    async remove() {
      if (!removing) return;
      await run(async (active) => {
        const result = await current.current.edit({ kind: "loopRegions", command: { kind: "edit", ids: [removing.id], action: { kind: "remove" } } });
        if (result && active()) { setRemoving(null); current.current.onSelect(""); }
      });
    },
  };
}
