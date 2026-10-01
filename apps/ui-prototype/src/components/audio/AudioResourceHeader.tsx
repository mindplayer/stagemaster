import type { ApplicationHost } from "../../application-host";
import type { AudioTimeline } from "../../audio-types";
export function AudioResourceHeader({
  track,
  blocked,
  host,
  choose,
  locate,
  remove,
  importMusic,
  problem,
  preparing,
  cancelPrepare,
}: {
  track: AudioTimeline | null;
  blocked: boolean;
  host: ApplicationHost;
  problem: string | null;
  preparing: boolean;
  choose(id: string): void;
  locate(): void;
  remove(): void;
  importMusic(): void;
  cancelPrepare(): void;
}) {
  return (
    <>
      <header className="audio-header">
        <div>
          <h2>音乐与卡点</h2>
          <span>{track?.asset.fileName ?? "用音乐安排灯光节奏"}</span>
        </div>
        <div className="wb-actions">
          {track ? (
            <>
              <button disabled={blocked} onClick={() => choose("")}>
                裁切范围
              </button>
              <button disabled={blocked} onClick={() => locate()}>
                重新定位音乐
              </button>
              <button disabled={blocked} onClick={() => remove()}>
                移除音乐
              </button>
            </>
          ) : (
            <button
              className="primary"
              disabled={blocked || host.kind !== "desktop"}
              onClick={importMusic}
            >
              导入音乐
            </button>
          )}
        </div>
      </header>
      {problem && (
        <div role="alert" className="audio-error">
          {problem}
        </div>
      )}
      {preparing && (
        <div role="status" className="audio-progress">
          正在准备音乐波形…
          <button onClick={() => cancelPrepare()}>取消准备</button>
        </div>
      )}
    </>
  );
}
