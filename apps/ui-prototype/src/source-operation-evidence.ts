import type { ExecutionView } from "./execution-types.ts";
import {
  httpRows,
  type MediaHttpEvidence,
} from "./media-operation-evidence.ts";

export type SourceCommand =
  | { kind: "start"; step: string }
  | { kind: "pause" | "resume" | "next" | "stop" };
export interface SourceOperationEvidence {
  target: null | {
    hostId: string;
    revision: string;
    source: string;
    action: SourceCommand;
  };
  serial: string | null;
  attempted: boolean;
  submission: MediaHttpEvidence;
  receiptRead: MediaHttpEvidence | null;
  receipt: null | {
    serial: string;
    complete: boolean;
    outcome: "applied" | "rejected" | "unknown" | "other" | null;
    code: string | null;
    sourceState: null | {
      revision: string;
      status: "Idle" | "Running" | "Paused" | "Finished";
      step: string | null;
    };
  };
  notSubmittedReason: "invalidTarget" | "sendPreflight" | null;
}
const actions = {
  start: "执行所选步骤",
  pause: "暂停节目",
  resume: "继续节目",
  next: "执行下一步",
  stop: "停止节目",
};
const states = {
  Idle: "待执行",
  Running: "运行中",
  Paused: "已暂停",
  Finished: "已结束",
};
/** Original receipt projection only. No current-state inference and no control calls. */
export function sourceEvidenceRows(
  view: ExecutionView,
  source: string,
): (readonly [string, string])[] {
  const value = view.sourceOperation;
  if (
    !value ||
    (value.target &&
      (value.target.hostId !== view.hostId || value.target.source !== source))
  )
    return [];
  const rows: (readonly [string, string])[] = [
    [
      "发送尝试",
      value.attempted
        ? "已开始发送尝试；不证明后台收到或接纳"
        : "未进入发送尝试",
    ],
  ];
  if (value.notSubmittedReason)
    rows.push([
      "本地检查",
      value.notSubmittedReason === "invalidTarget"
        ? "诊断目标未通过检查；无效输入不留原文，原操作规则不变"
        : "发送预检未通过，未提交本次操作",
    ]);
  if (value.target) {
    const { hostId, revision, action } = value.target;
    rows.push(
      ["动作", actions[action.kind]],
      ["后台身份", hostId],
      ["节目身份", source],
      ["预期修订", revision],
    );
    if (action.kind === "start") rows.push(["起始步骤身份", action.step]);
  }
  if (value.serial) rows.push(["原网络序号", value.serial]);
  rows.push(...httpRows("提交", value.submission));
  if (value.receiptRead)
    rows.push(...httpRows("原序号查询", value.receiptRead));
  const receipt = value.receipt;
  if (!receipt || receipt.serial !== value.serial) {
    rows.push(["原回执", "尚未取得匹配回执，结果未知"]);
    return rows;
  }
  rows.push(
    ["回执序号", receipt.serial],
    ["回执完成", receipt.complete ? "已完成" : "待核对"],
  );
  if (receipt.complete && receipt.outcome)
    rows.push([
      "回执结果",
      {
        applied: "后台已应用；不等于实灯输出完成",
        rejected: "已拒绝",
        unknown: "无法确认",
        other: "其他结果，不能推断执行成功",
      }[receipt.outcome],
    ]);
  if (receipt.code) rows.push(["回执分类", receipt.code]);
  if (
    receipt.complete &&
    receipt.outcome === "applied" &&
    value.target &&
    receipt.sourceState
  ) {
    rows.push(
      ["当次回执节目状态", states[receipt.sourceState.status]],
      ["当次回执修订", receipt.sourceState.revision],
    );
    if (receipt.sourceState.step)
      rows.push(["当次回执步骤身份", receipt.sourceState.step]);
  }
  return rows;
}
