import type { ExecutionView } from "../../execution-types";
import {
  mediaControlNotice,
  type MediaControlNotice,
} from "../../media-control-notice";

const messages: Record<MediaControlNotice, string> = {
  pending: "音乐操作已接纳，正在准备与确认。",
  applied: "最近音乐操作已完成",
  failed: "音乐操作未完成，请核对状态后重试。",
  timedOut: "音乐操作超时，请核对状态后重试。",
  rejected: "本次控制请求被拒绝；上次音乐回执不代表本次成功，请核对拒绝说明。",
  unknown: "本次控制结果无法确认，请核对原回执和当前状态。",
  unconfirmed: "尚未确认本次音乐操作完成，请核对原回执和当前状态。",
  superseded: "本次音乐操作已被后续操作替代，请核对当前状态。",
  notSubmitted:
    "本次音乐请求未进入发送尝试；上次音乐完成不代表本次成功，请核对原错误说明。",
};

export function MediaControlStatus({
  runtime,
  group,
}: {
  runtime: ExecutionView;
  group: string;
}) {
  const notice = mediaControlNotice(runtime, group);
  if (!notice) return null;
  return (
    <p role={["pending", "applied"].includes(notice) ? "status" : "alert"}>
      {messages[notice]}
    </p>
  );
}
