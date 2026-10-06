import assert from "node:assert/strict";
import test from "node:test";
import {
  sourceEvidenceRows,
  type SourceOperationEvidence,
} from "../src/source-operation-evidence.ts";
import { manualFixture } from "./execution-manual-fixture.ts";

const http = { status: 200, bodyComplete: true, code: null, problem: null };
function fixture() {
  const view = manualFixture();
  const source = view.catalog.sources[0].id;
  const evidence: SourceOperationEvidence = {
    target: {
      hostId: view.hostId,
      source,
      revision: "9007199254740994",
      action: { kind: "pause" },
    },
    serial: "9007199254740995",
    attempted: true,
    submission: { ...http },
    receiptRead: null,
    receipt: {
      serial: "9007199254740995",
      complete: true,
      outcome: "applied",
      code: null,
      sourceState: {
        revision: "9007199254740996",
        status: "Paused",
        step: null,
      },
    },
    notSubmittedReason: null,
  };
  view.sourceOperation = evidence;
  return { view, source, evidence };
}
test("original pause receipt stays independent of newer running observation and renewal", () => {
  const { view, source } = fixture();
  view.record = {
    serial: "999",
    status: "complete",
    outcome: { kind: "renewed", message: null },
  };
  const rows = Object.fromEntries(sourceEvidenceRows(view, source));
  assert.equal(rows["动作"], "暂停节目");
  assert.equal(rows["原网络序号"], "9007199254740995");
  assert.equal(rows["当次回执节目状态"], "已暂停");
  assert.match(rows["回执结果"], /不等于实灯/);
});
test("missing and replaced host/source evidence must not appear on another program", () => {
  const { view, source, evidence } = fixture();
  assert.deepEqual(sourceEvidenceRows(view, "another"), []);
  evidence.target!.hostId = "another";
  assert.deepEqual(sourceEvidenceRows(view, source), []);
  delete view.sourceOperation;
  assert.deepEqual(sourceEvidenceRows(view, source), []);
});
test("rejected unknown pending or mismatched receipt cannot borrow the old applied state", () => {
  for (const kind of ["rejected", "unknown", "other"] as const) {
    const { view, source, evidence } = fixture();
    evidence.receipt!.outcome = kind;
    assert.equal(
      Object.fromEntries(sourceEvidenceRows(view, source))["当次回执节目状态"],
      undefined,
    );
  }
  const { view, source, evidence } = fixture();
  evidence.receipt!.complete = false;
  assert.equal(
    Object.fromEntries(sourceEvidenceRows(view, source))["当次回执节目状态"],
    undefined,
  );
  assert.equal(
    Object.fromEntries(sourceEvidenceRows(view, source))["回执结果"],
    undefined,
  );
  evidence.receipt!.serial = "other";
  assert.match(
    Object.fromEntries(sourceEvidenceRows(view, source))["原回执"],
    /未知/,
  );
});
test("not submitted and POST refusal versus original receipt query remain distinct", () => {
  const { view, source, evidence } = fixture();
  evidence.target = null;
  evidence.attempted = false;
  evidence.serial = null;
  evidence.receipt = null;
  evidence.notSubmittedReason = "sendPreflight";
  assert.match(
    Object.fromEntries(sourceEvidenceRows(view, source))["发送尝试"],
    /未进入/,
  );
  evidence.attempted = true;
  evidence.serial = "2";
  evidence.submission = { ...http, status: 503, problem: "httpRefused" };
  evidence.receiptRead = {
    ...http,
    status: 409,
    code: "notRetained",
    problem: "httpRefused",
  };
  const rows = Object.fromEntries(sourceEvidenceRows(view, source));
  assert.equal(rows["提交HTTP"], "503");
  assert.equal(rows["原序号查询HTTP"], "409");
  assert.match(rows["原回执"], /未知/);
});
