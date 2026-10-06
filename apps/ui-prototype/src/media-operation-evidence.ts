import type { ExecutionMediaAction } from "./execution-media-types";
import type { ExecutionView } from "./execution-types";

export interface MediaHttpEvidence {
  status: number | null;
  bodyComplete: boolean;
  code: string | null;
  problem:
    | "encoding"
    | "connection"
    | "incomplete"
    | "bodyLimit"
    | "httpRefused"
    | "invalidJson"
    | "receiptMismatch"
    | null;
}
export interface MediaOperationEvidence {
  target: null | {
    hostId: string;
    revision: string;
    group: string;
    generation: string;
    action: ExecutionMediaAction;
  };
  serial: string | null;
  attempted: boolean;
  submission: MediaHttpEvidence;
  receiptRead: MediaHttpEvidence | null;
  receipt: null | {
    serial: string;
    complete: boolean;
    outcome: "accepted" | "rejected" | "unknown" | "other" | null;
    code: string | null;
    mediaRequest: string | null;
    generation: string | null;
  };
  notSubmittedReason: "invalidTarget" | "sendPreflight" | null;
}

const actions = {
  play: "播放音乐",
  pause: "暂停音乐",
  stop: "停止音乐",
  seek: "定位音乐",
  recover: "重新准备音乐",
  exitLoop: "本遍结束退出／取消",
};
const problems = {
  encoding: "请求编码未完成",
  connection: "连接结果未知",
  incomplete: "响应正文不完整",
  bodyLimit: "响应超过原容量限制",
  httpRefused: "HTTP请求未成功，不能推断已经接纳",
  invalidJson: "响应无法解析为原回执",
  receiptMismatch: "返回回执与原请求不匹配",
};
type Row = readonly [string, string];

export function httpRows(label: string, value: MediaHttpEvidence): Row[] {
  const rows: Row[] = [
    [
      `${label}HTTP`,
      value.status === null ? "未取得状态" : String(value.status),
    ],
    [`${label}正文`, value.bodyComplete ? "读取完整" : "尚未读取完整"],
  ];
  if (value.code) rows.push([`${label}失败分类`, value.code]);
  if (value.problem) rows.push([`${label}问题`, problems[value.problem]]);
  return rows;
}

/** Read-only projection of original client evidence. Never calls the control port. */
export function mediaEvidenceRows(view: ExecutionView, group: string): Row[] {
  const evidence = view.mediaOperation;
  if (
    !evidence ||
    (evidence.target &&
      (evidence.target.hostId !== view.hostId ||
        evidence.target.group !== group))
  )
    return [];
  const rows: Row[] = [
    [
      "发送尝试",
      evidence.attempted
        ? "已开始发送尝试；不证明后台收到或接纳"
        : "未进入发送尝试",
    ],
  ];
  if (evidence.notSubmittedReason)
    rows.push([
      "未提交原因",
      evidence.notSubmittedReason === "invalidTarget"
        ? "客户端目标检查未通过；无效输入不留原文"
        : "发送预检未通过，请核对原错误说明",
    ]);
  if (evidence.serial) rows.push(["原网络序号", evidence.serial]);
  if (evidence.target) {
    const { action, hostId, revision, generation } = evidence.target;
    rows.push(
      ["动作", actions[action.kind]],
      ["后台身份", hostId],
      ["预期修订", revision],
      ["音乐组", group],
      ["播放代次", generation],
    );
    if (action.kind === "exitLoop")
      rows.push(
        ["原音源实例", action.instance],
        ["原区段索引", String(action.region)],
        ["原遍次", action.pass],
        ["圈末动作", action.requested ? "申请退出" : "取消退出"],
      );
    if (action.kind === "seek" || action.kind === "recover")
      rows.push(["目标毫秒", String(action.positionMs)]);
    if (action.kind === "seek")
      rows.push(["定位后状态", action.playing ? "继续播放" : "保持暂停"]);
  }
  rows.push(...httpRows("提交", evidence.submission));
  if (evidence.receiptRead)
    rows.push(...httpRows("原序号查询", evidence.receiptRead));
  const receipt = evidence.receipt;
  if (!receipt) {
    rows.push(["原回执", "尚未取得匹配回执，结果未知"]);
    return rows;
  }
  rows.push(
    ["回执序号", receipt.serial],
    ["回执完成", receipt.complete ? "已完成" : "待核对"],
  );
  if (receipt.outcome)
    rows.push([
      "回执结果",
      {
        accepted: "已接纳；仍需音乐完成确认",
        rejected: "已拒绝",
        unknown: "无法确认",
        other: "其他结果，不能推断音乐成功",
      }[receipt.outcome],
    ]);
  if (receipt.code) rows.push(["回执分类", receipt.code]);
  if (
    receipt.complete &&
    receipt.outcome === "accepted" &&
    receipt.serial === evidence.serial &&
    receipt.mediaRequest
  ) {
    rows.push(["原媒体请求", receipt.mediaRequest]);
    const current = view.observation.snapshot?.state.media?.find(
      (m) => m.id === group,
    )?.control;
    rows.push([
      "媒体实际确认",
      current?.request === receipt.mediaRequest
        ? {
            pending: "等待应用",
            applied: "同一请求已应用",
            failed: "同一请求失败",
            timedOut: "同一请求超时",
          }[current.status]
        : "当前观察不对应原媒体请求，不能确认",
    ]);
  }
  return rows;
}
