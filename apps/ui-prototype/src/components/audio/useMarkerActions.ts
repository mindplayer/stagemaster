import { useEffect, useRef, useState } from "react";
import type { ApplicationHost, ProjectView } from "../../application-host";
import type { useAudio } from "./useAudio";

export function useMarkerActions({
  project,
  selected,
  host,
  generation,
  audio,
  beforeChange,
  onView3d,
}: {
  project: ProjectView;
  selected: string;
  host: ApplicationHost;
  generation(): number;
  audio: ReturnType<typeof useAudio>;
  beforeChange(): Promise<boolean>;
  onView3d?(): void;
}) {
  const [acting, setActing] = useState(false);
  const [problem, setProblem] = useState("");
  const mounted = useRef(true);
  const latest = useRef({ project, selected });
  latest.current = { project, selected };
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  async function preview() {
    if (acting || !selected || !project.audio) return;
    const target = {
      projectId: project.id,
      markerId: selected,
      asset: project.audio.asset.digest,
    };
    const current = () =>
      mounted.current &&
      latest.current.project.id === target.projectId &&
      latest.current.selected === target.markerId &&
      latest.current.project.audio?.asset.digest === target.asset;
    setActing(true);
    setProblem("");
    try {
      if (!(await beforeChange()) || !current()) return;
      const snapshot = await host.request({ kind: "snapshot" });
      if (
        !current() ||
        snapshot.generation !== generation() ||
        snapshot.project?.id !== target.projectId
      )
        return;
      const marker = snapshot.project.audio?.markers.find(
        (m) => m.id === target.markerId,
      );
      if (!marker) throw new Error("此卡点已被删除，请重新选择");
      if (await audio.previewAt(marker.timeMs)) {
        if (current()) onView3d?.();
      }
    } catch (reason) {
      if (current())
        setProblem(reason instanceof Error ? reason.message : String(reason));
    } finally {
      if (mounted.current) setActing(false);
    }
  }
  return { acting, problem, preview };
}
