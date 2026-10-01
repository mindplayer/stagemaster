import type {
  EditCommand,
  ProjectView,
  SceneView,
} from "../../application-host";
import { uniqueName } from "../../editor-tools.ts";
import { sceneCopyCommands } from "./scene-copy-tools.ts";

/** Commands use the current project after the shared draft guard has completed. */
export function sceneActions({
  read,
  run,
  edit,
  select,
  clearQuery,
  notice,
}: {
  read(): ProjectView;
  run(action: () => Promise<void>): Promise<boolean>;
  edit(command: EditCommand): Promise<void>;
  select(scene: SceneView): void;
  clearQuery(): void;
  notice(message: string): void;
}) {
  async function copy(ids: readonly string[]) {
    let created: string[] | null = null;
    const requested = [...ids];
    const ok = await run(async () => {
      const project = read();
      const original = new Set(project.scenes.map((scene) => scene.id));
      const commands = sceneCopyCommands(project.scenes, requested);
      await edit({ op: "batch", commands });
      const copies = read().scenes.filter((scene) => !original.has(scene.id));
      if (!copies.length) throw new Error("未能取得新场景，请检查工程状态");
      created = copies.map((scene) => scene.id);
      select(copies[0]);
      clearQuery();
      notice(`已复制 ${copies.length} 个场景，可撤销恢复`);
    });
    return ok ? created : null;
  }
  return {
    copy,
    choose(scene: SceneView) {
      void run(async () => {
        const current = read().scenes.find((s) => s.id === scene.id);
        if (!current) throw new Error("场景已不存在，请重新选择");
        select(current);
      });
    },
    add() {
      void run(async () => {
        await edit({
          op: "addScene",
          name: uniqueName(
            "场景",
            read().scenes.map((s) => s.name),
          ),
        });
        select(read().scenes.at(-1)!);
        clearQuery();
        notice("已创建场景");
      });
    },
  };
}
