import { useState } from "react";
import type {
  AudioCommand,
  AudioPosition,
  AudioTimeline,
} from "../../audio-types";
import {
  audioLoopExitCommand,
  audioLoopRuntime,
} from "../../audio-performance-tools";
import "./audio-performance.css";

export function AudioPerformanceControls({
  track,
  position,
  disabled,
  command,
}: {
  track: AudioTimeline;
  position: AudioPosition;
  disabled: boolean;
  command(value: AudioCommand): Promise<void>;
}) {
  const [sending, setSending] = useState(false);
  const current = audioLoopRuntime(track, position);
  if (!position.performance) return null;
  const action = audioLoopExitCommand(track, position);
  const state = position.performance;
  return (
    <div className="audio-performance" aria-label="演出循环控制">
      <output>
        {current ? (
          <>
            <strong>{current.region.name}</strong>
            {state.pass && <>第 {state.pass} 遍</>}
            <small>
              {current.region.plays.kind === "count"
                ? `共 ${current.region.plays.count} 遍`
                : "持续循环"}
            </small>
            {current.exiting && <small>本圈结束后继续</small>}
          </>
        ) : state.ended ? (
          "音乐已结束"
        ) : (
          "按演出编排播放"
        )}
      </output>
      {current && (
        <button
          disabled={disabled || sending || !action}
          onClick={async () => {
            if (!action) return;
            setSending(true);
            try {
              await command(action);
            } finally {
              setSending(false);
            }
          }}
        >
          {current.exiting ? "取消继续" : "本圈结束后继续"}
        </button>
      )}
      {state.controlProblem && (
        <span role="status">{state.controlProblem}</span>
      )}
    </div>
  );
}
