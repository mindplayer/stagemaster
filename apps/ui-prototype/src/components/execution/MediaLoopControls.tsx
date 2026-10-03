import type {
  ExecutionAudioState,
  ExecutionMediaAction,
} from "../../execution-media-types";

export function MediaLoopControls({
  audio,
  disabled,
  onAction,
}: {
  audio: ExecutionAudioState;
  disabled: boolean;
  onAction(action: ExecutionMediaAction): unknown;
}) {
  const loop = audio.loopState;
  if (!loop || !audio.instance) return null;
  const instance = audio.instance;
  const requested = loop.pendingExit ?? loop.exitRequested;
  return (
    <section aria-label="后台演出循环">
      <p>{loop.name} · 第 {loop.pass} 遍</p>
      <button
        disabled={disabled || !["playing", "paused"].includes(audio.status)}
        onClick={() => void onAction({
          kind: "exitLoop",
          instance,
          region: loop.region,
          pass: loop.pass,
          requested: !requested,
        })}
      >
        {requested ? "取消本遍退出" : "本遍结束后退出循环"}
      </button>
      {(loop.pendingExit !== null || loop.exitRequested) && (
        <p role="status">
          {loop.pendingExit !== null
            ? "正在确认循环操作"
            : "本遍结束后继续后续编排"}
        </p>
      )}
    </section>
  );
}
