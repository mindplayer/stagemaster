import test from "node:test";
import assert from "node:assert/strict";
import { mediaEvidenceRows } from "../src/media-operation-evidence.ts";
import { mediaControlNotice } from "../src/media-control-notice.ts";
import { evidenceView, http } from "./media-evidence-fixture.ts";

test("a refusal or incomplete receipt never borrows an old Applied from an attached state", () => {
  for (const [outcome, complete] of [
    ["rejected", true],
    ["unknown", true],
    ["other", true],
    ["accepted", false],
  ] as const) {
    const view = evidenceView();
    Object.assign(view.mediaOperation!.receipt!, {
      outcome,
      complete,
      mediaRequest: "7",
    });
    const fields = Object.fromEntries(mediaEvidenceRows(view, "music"));
    assert.equal(fields["原媒体请求"], undefined, `${outcome}/${complete}`);
    assert.equal(fields["媒体实际确认"], undefined, `${outcome}/${complete}`);
  }
});

test("an unmatched compact receipt serial cannot confirm a media request", () => {
  const view = evidenceView();
  Object.assign(view.mediaOperation!.receipt!, {
    outcome: "accepted",
    serial: "9007199254740994",
    mediaRequest: "7",
  });
  const fields = Object.fromEntries(mediaEvidenceRows(view, "music"));
  assert.equal(fields["原媒体请求"], undefined);
  assert.equal(fields["媒体实际确认"], undefined);
});

test("later non-media operations cannot relabel the last rejected or unknown music request as successful", () => {
  for (const outcome of ["rejected", "unknown"] as const) {
    const view = evidenceView();
    view.mediaOperation!.receipt!.outcome = outcome;
    view.record = {
      serial: "9007199254740994",
      status: "complete",
      outcome: { kind: "released", message: null },
    };
    assert.equal(mediaControlNotice(view, "music"), outcome);
  }
});

test("original target and large serial survive unrelated maintenance receipts without a new confirmation", () => {
  const view = evidenceView();
  const rows = mediaEvidenceRows(view, "music");
  const fields = Object.fromEntries(rows);
  assert.equal(fields["原网络序号"], "9007199254740993");
  assert.equal(fields["原音源实例"], "2");
  assert.equal(fields["原区段索引"], "0");
  assert.equal(fields["原遍次"], "2");
  assert.equal(fields["回执结果"], "已拒绝");
  assert.equal(fields["回执分类"], "loopTargetChanged");
  assert.equal(fields["媒体实际确认"], undefined);
  const original = JSON.stringify(view.mediaOperation);
  view.record = {
    serial: "9007199254740994",
    status: "complete",
    outcome: { kind: "renewed", message: null },
  };
  view.observation.snapshot!.cycles = "200";
  assert.deepEqual(mediaEvidenceRows(view, "music"), rows);
  assert.equal(JSON.stringify(view.mediaOperation), original);
  assert.equal(mediaControlNotice(view, "music"), "rejected");
});

test("POST and original serial query are distinct and neither admits an unknown command", () => {
  const view = evidenceView();
  const evidence = view.mediaOperation!;
  evidence.submission = { ...http(503), problem: "httpRefused" };
  evidence.receiptRead = {
    ...http(409),
    code: "notRetained",
    problem: "httpRefused",
  };
  evidence.receipt = null;
  view.pending = true;
  const fields = Object.fromEntries(mediaEvidenceRows(view, "music"));
  assert.equal(fields["提交HTTP"], "503");
  assert.equal(fields["原序号查询HTTP"], "409");
  assert.equal(fields["原序号查询失败分类"], "notRetained");
  assert.equal(fields["原回执"], "尚未取得匹配回执，结果未知");
  assert.match(fields["发送尝试"], /不证明后台收到或接纳/);
  assert.equal(mediaControlNotice(view, "music"), null);
});

test("a damaged original response remains visible after a matching receipt is obtained", () => {
  const view = evidenceView();
  view.mediaOperation!.submission.problem = "invalidJson";
  view.mediaOperation!.receipt!.outcome = "accepted";
  view.mediaOperation!.receipt!.mediaRequest = "7";
  const fields = Object.fromEntries(mediaEvidenceRows(view, "music"));
  assert.equal(fields["提交问题"], "响应无法解析为原回执");
  assert.equal(fields["原序号查询HTTP"], "200");
  assert.equal(fields["回执结果"], "已接纳；仍需音乐完成确认");
  assert.equal(fields["媒体实际确认"], "同一请求已应用");
});

test("only an exact observed music request exposes pending applied failed or timed out", () => {
  const view = evidenceView();
  view.mediaOperation!.receipt!.outcome = "accepted";
  view.mediaOperation!.receipt!.mediaRequest = "7";
  for (const [status, label] of [
    ["pending", "等待应用"],
    ["applied", "同一请求已应用"],
    ["failed", "同一请求失败"],
    ["timedOut", "同一请求超时"],
  ] as const) {
    view.observation.snapshot!.state.media![0].control!.status = status;
    assert.equal(
      Object.fromEntries(mediaEvidenceRows(view, "music"))["媒体实际确认"],
      label,
    );
  }
  for (const request of ["6", "8", "9007199254740993"]) {
    view.observation.snapshot!.state.media![0].control!.request = request;
    assert.match(
      Object.fromEntries(mediaEvidenceRows(view, "music"))["媒体实际确认"],
      /不能确认/,
    );
  }
});

test("invalid local inputs are not displayed or confused with previous Applied", () => {
  const view = evidenceView();
  view.mediaOperation = {
    target: null,
    serial: null,
    attempted: false,
    submission: http(null),
    receiptRead: null,
    receipt: null,
    notSubmittedReason: "invalidTarget",
  };
  const fields = Object.fromEntries(mediaEvidenceRows(view, "music"));
  assert.equal(fields["发送尝试"], "未进入发送尝试");
  assert.equal(fields["原网络序号"], undefined);
  assert.equal(fields["原音源实例"], undefined);
  assert.match(fields["未提交原因"], /无效输入不留原文/);
  assert.equal(mediaControlNotice(view, "music"), "notSubmitted");
});

test("a replaced host group or absent evidence produces no details placeholder", () => {
  const view = evidenceView();
  assert.deepEqual(mediaEvidenceRows(view, "other"), []);
  view.hostId = "replacement";
  assert.deepEqual(mediaEvidenceRows(view, "music"), []);
  delete view.mediaOperation;
  assert.deepEqual(mediaEvidenceRows(view, "music"), []);
});

test("unknown and other outcomes cannot imply acceptance or successful music application", () => {
  for (const outcome of ["unknown", "other"] as const) {
    const view = evidenceView();
    view.mediaOperation!.receipt!.outcome = outcome;
    view.mediaOperation!.receipt!.mediaRequest = null;
    const fields = Object.fromEntries(mediaEvidenceRows(view, "music"));
    assert.match(
      fields["回执结果"],
      outcome === "unknown" ? /无法确认/ : /不能推断音乐成功/,
    );
    assert.equal(fields["媒体实际确认"], undefined);
  }
});
