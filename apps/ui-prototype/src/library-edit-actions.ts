import type { EditCommand, ProjectView, Snapshot } from "./application-host.ts";
import type { LibraryEdit } from "./library-types.ts";

/** Resource callback extracted from the workbench; queue and host remain authoritative. */
export function libraryEditActions(
  run: (work: () => Promise<void>) => Promise<boolean>,
  current: () => Snapshot,
  edit: (command: EditCommand) => Promise<void>,
  notice: (message: string) => void,
) {
  return async (command: LibraryEdit): Promise<ProjectView | null> => {
    let project: ProjectView | null = null;
    const ok = await run(async () => {
      // Capture inside the original queue, AFTER its draft flush, not when clicked.
      const before = current();
      if (!before.project) throw Error("请先打开工程再编辑资源");
      await edit({ op: "library", command });
      const after = current();
      if (after.project?.id !== before.project.id)
        throw Error("工程已更换，请核对资源操作结果；不重复发送");
      project = after.project;
      notice(
        after.generation === before.generation
          ? "资源未变化，未新增撤销记录"
          : "资源已更新，可撤销恢复",
      );
    });
    return ok ? project : null;
  };
}
