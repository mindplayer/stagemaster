import type { ProjectRequest } from "./application-host";
import type { ManualRecordingContext } from "./manual-capture-types";

export function manualRecordingActions(
  run: (work: () => Promise<void>) => Promise<boolean>,
  request: (command: ProjectRequest) => Promise<unknown>,
  setNotice: (message: string) => void,
): Pick<ManualRecordingContext, "onRecord" | "onMerge"> {
  async function commit(command: ProjectRequest, message: string) {
    let failure = "工程操作未完成，请重试";
    const ok = await run(async () => {
      try {
        await request(command);
        setNotice(message);
      } catch (reason) {
        failure = reason instanceof Error ? reason.message : String(reason);
        throw reason;
      }
    });
    if (!ok) throw new Error(failure);
  }
  return {
    onRecord: (generation, token, name) =>
      commit(
        { kind: "recordManualScene", generation, token, name },
        `已录入场景“${name}”，可撤销恢复；手动层保持`,
      ),
    onMerge: (generation, token, changed) =>
      commit(
        { kind: "mergeManualScene", generation, token },
        changed
          ? "已合并到原场景，可撤销恢复；后台运行版本保持"
          : "记录值相同，场景未修改；后台运行版本保持",
      ),
  };
}
