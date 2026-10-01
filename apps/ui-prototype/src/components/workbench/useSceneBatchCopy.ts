import { useRef, useState } from "react";
import type { SceneView } from "../../application-host";
import { MAX_SCENE_COPIES } from "./scene-copy-tools";
export function useSceneBatchCopy(
  scenes: SceneView[],
  busy: boolean,
  beforeChange: () => Promise<boolean>,
  onCopy: (ids: string[]) => Promise<string[] | null>,
) {
  const [active, setActive] = useState(false);
  const [ids, setIds] = useState<string[]>([]);
  const [working, setWorking] = useState(false);
  const [problem, setProblem] = useState("");
  const anchor = useRef("");
  const operating = useRef(false);
  const selected = scenes
    .filter((scene) => ids.includes(scene.id))
    .map((scene) => scene.id);
  const blocked = busy || working;
  function replace(next: string[]) {
    if (blocked) return;
    if (!next.length) anchor.current = "";
    setIds([...new Set(next)]);
    setProblem("");
  }
  return {
    active,
    selected,
    blocked,
    problem,
    async toggleMode() {
      if (blocked || operating.current) return;
      operating.current = true;
      try {
        if (await beforeChange()) {
          setActive(!active);
          setProblem("");
        }
      } finally {
        operating.current = false;
      }
    },
    replace,
    toggle(id: string, visible: SceneView[], range: boolean) {
      if (blocked) return;
      const from = visible.findIndex((s) => s.id === anchor.current),
        to = visible.findIndex((s) => s.id === id);
      if (range && from >= 0 && to >= 0)
        replace([
          ...selected,
          ...visible
            .slice(Math.min(from, to), Math.max(from, to) + 1)
            .map((s) => s.id),
        ]);
      else
        replace(
          selected.includes(id)
            ? selected.filter((value) => value !== id)
            : [...selected, id],
        );
      anchor.current = id;
    },
    async copy() {
      if (blocked || operating.current) return;
      if (!selected.length || selected.length > MAX_SCENE_COPIES) {
        setProblem(`请选择 1–${MAX_SCENE_COPIES} 个场景`);
        return;
      }
      operating.current = true;
      setWorking(true);
      setProblem("");
      try {
        const result = await onCopy([...selected]);
        if (result) {
          setIds(result);
          anchor.current = "";
        } else setProblem("复制未完成，请先修正工程提示；已保留本次选择。");
      } catch (error) {
        setProblem(error instanceof Error ? error.message : String(error));
      } finally {
        operating.current = false;
        setWorking(false);
      }
    },
  };
}
