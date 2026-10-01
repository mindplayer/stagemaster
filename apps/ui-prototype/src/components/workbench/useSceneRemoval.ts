import { useRef, useState } from "react";
import type { EditCommand, ProjectView } from "../../application-host";
import { sceneRemovalReview, sceneRemovalCommands } from "./scene-removal";
export function useSceneRemoval({
  read,
  run,
  edit,
  onRemoved,
}: {
  read(): ProjectView | null;
  run(action: () => Promise<void>, flushFirst?: boolean): Promise<boolean>;
  edit(command: EditCommand): Promise<void>;
  onRemoved(ids: readonly string[]): void;
}) {
  const [intent, setIntent] = useState<{
    projectId: string;
    ids: string[];
  } | null>(null);
  const [problem, setProblem] = useState("");
  const origin = useRef<HTMLElement | null>(null);
  function close() {
    setIntent(null);
    setProblem("");
    requestAnimationFrame(() => {
      if (origin.current?.isConnected)
        origin.current.focus({ preventScroll: true });
    });
  }
  return {
    intent,
    problem,
    close,
    dismiss() {
      setIntent(null);
      setProblem("");
    },
    request(ids: readonly string[]) {
      const projectId = read()?.id;
      const requested = [...ids];
      origin.current =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      void run(async () => {
        const project = read();
        if (!project || project.id !== projectId)
          throw new Error("工程已切换，请重新选择");
        sceneRemovalReview(project, requested);
        setProblem("");
        setIntent({ projectId: project.id, ids: requested });
      });
    },
    async remove() {
      if (!intent) return;
      setProblem("");
      const ok = await run(async () => {
        const project = read();
        if (!project || project.id !== intent.projectId)
          throw new Error("工程已切换，请取消后重新选择");
        await edit({
          op: "batch",
          commands: sceneRemovalCommands(project, intent.ids),
        });
        onRemoved(intent.ids);
      }, false);
      if (ok) close();
      else
        setProblem(
          "删除未完成，场景保持不变；请检查工程提示或取消后重新检查引用。",
        );
    },
  };
}
