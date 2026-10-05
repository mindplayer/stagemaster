import test from "node:test";
import assert from "node:assert/strict";
import {
  mediaRequestIdentity,
  mediaSeekReceipt,
} from "../src/media-seek-receipt.ts";
import type { ExecutionView } from "../src/execution-types.ts";
import type { ExecutionMediaState } from "../src/execution-media-types.ts";
import { mediaControlNotice } from "../src/media-control-notice.ts";

function media(
  request: string,
  status: "pending" | "applied" | "failed" | "timedOut",
): ExecutionMediaState {
  return {
    id: "music",
    generation: "5",
    status: "Stopped",
    positionMs: 0,
    control: { request, status, problem: null },
  };
}
function view(): ExecutionView {
  return {
    hostId: "host",
    sessionId: "controller",
    controlling: true,
    pending: false,
    catalog: {
      projectId: "show",
      layout: "layout",
      sources: [],
      physicalOutput: false,
    },
    record: {
      serial: "9007199254740993",
      status: "resolved",
      outcome: {
        kind: "accepted",
        message: null,
        state: { media: [media("7", "pending")] },
      },
    },
    observation: {
      phase: "running",
      fault: null,
      snapshot: {
        cycles: "10",
        missedPeriods: "0",
        state: {
          revision: "20",
          sources: [],
          owner: null,
          fault: false,
          media: [media("7", "pending")],
        },
      },
    },
  };
}

test("lost HTTP acknowledgement and source preparation retain the draft until the same seek completes", () => {
  const current = view();
  const identity = mediaRequestIdentity(current)!;
  const accepted = current.record!.outcome;
  current.pending = true;
  current.record!.outcome = null;
  assert.equal(mediaSeekReceipt(identity, current, "music"), "waiting");
  current.pending = false;
  current.record!.outcome = accepted;
  assert.equal(mediaSeekReceipt(identity, current, "music"), "waiting");
  current.observation.snapshot!.state.media = [media("6", "applied")];
  assert.equal(mediaSeekReceipt(identity, current, "music"), "waiting");
  current.observation.snapshot!.state.media = [media("7", "applied")];
  assert.equal(mediaSeekReceipt(identity, current, "music"), "applied");
});

test("previous HTTP snapshots and later commands never confirm a different seek", () => {
  const current = view();
  const identity = mediaRequestIdentity(current)!;
  current.record!.serial = "9007199254740992";
  assert.equal(mediaSeekReceipt(identity, current, "music"), "waiting");
  current.record!.serial = "9007199254740994";
  assert.equal(mediaSeekReceipt(identity, current, "music"), "superseded");
  current.record!.serial = identity.serial;
  current.observation.snapshot!.state.media = [media("8", "applied")];
  assert.equal(mediaSeekReceipt(identity, current, "music"), "superseded");
});

test("refusal timeout and missing evidence cannot clear input as successful", () => {
  for (const status of ["failed", "timedOut"] as const) {
    const current = view();
    current.observation.snapshot!.state.media = [media("7", status)];
    assert.equal(
      mediaSeekReceipt(mediaRequestIdentity(current)!, current, "music"),
      "failed",
    );
  }
  const current = view();
  const identity = mediaRequestIdentity(current)!;
  current.record!.outcome!.kind = "rejected";
  assert.equal(mediaSeekReceipt(identity, current, "music"), "failed");
  current.record!.outcome!.kind = "accepted";
  current.record!.outcome!.state = null;
  assert.equal(mediaSeekReceipt(identity, current, "music"), "failed");
});

test("a replaced host or controller cannot reuse the old operation identity", () => {
  const current = view();
  const identity = mediaRequestIdentity(current)!;
  current.sessionId = "replacement";
  assert.equal(mediaSeekReceipt(identity, current, "music"), "superseded");
  current.sessionId = identity.sessionId;
  current.hostId = "replacement";
  assert.equal(mediaSeekReceipt(identity, current, "music"), "superseded");
  assert.equal(mediaRequestIdentity(null), null);
  current.sessionId = null;
  assert.equal(mediaRequestIdentity(current), null);
});

test("new rejected and unknown controls cannot display an older music Applied as new success", () => {
  for (const kind of ["rejected", "unknown"] as const) {
    const current = view();
    current.record!.outcome!.kind = kind;
    current.observation.snapshot!.state.media = [media("7", "applied")];
    assert.equal(mediaControlNotice(current, "music"), kind);
  }
});

test("new accepted music request waits instead of borrowing an older completion", () => {
  const current = view();
  current.observation.snapshot!.state.media = [media("6", "applied")];
  assert.equal(mediaControlNotice(current, "music"), "pending");
  current.observation.snapshot!.state.media = [media("8", "applied")];
  assert.equal(mediaControlNotice(current, "music"), "superseded");
});

test("missing acknowledgement or accepted media evidence never proves completion", () => {
  const current = view();
  current.observation.snapshot!.state.media = [media("7", "applied")];
  current.record!.outcome = null;
  assert.equal(mediaControlNotice(current, "music"), "unconfirmed");
  current.record!.outcome = { kind: "accepted", message: null, state: null };
  assert.equal(mediaControlNotice(current, "music"), "unconfirmed");
  current.record!.outcome.state = { media: [media("7", "pending")] };
  current.sessionId = null;
  assert.equal(mediaControlNotice(current, "music"), "unconfirmed");
});

test("only the same music request can expose its actual pending applied or failure state", () => {
  const current = view();
  for (const status of ["pending", "applied", "failed", "timedOut"] as const) {
    current.observation.snapshot!.state.media = [media("7", status)];
    assert.equal(mediaControlNotice(current, "music"), status);
  }
  current.pending = true;
  assert.equal(mediaControlNotice(current, "music"), null);
});

test("readonly and non-media completed operations retain the actual historical music status", () => {
  const current = view();
  current.record = null;
  current.sessionId = null;
  current.observation.snapshot!.state.media = [media("7", "applied")];
  assert.equal(mediaControlNotice(current, "music"), "applied");
  assert.equal(mediaControlNotice(current, "other"), null);
  current.record = {
    serial: "1",
    status: "complete",
    outcome: { kind: "released", message: null },
  };
  assert.equal(mediaControlNotice(current, "music"), "applied");
});

test("a locally unsubmitted music attempt cannot display the previous Applied as its completion", () => {
  const current = view();
  current.observation.snapshot!.state.media = [media("7", "applied")];
  Object.assign(current, {
    mediaOperation: {
      target: null,
      serial: null,
      attempted: false,
      submission: {
        status: null,
        bodyComplete: false,
        code: null,
        problem: null,
      },
      receiptRead: null,
      receipt: null,
      notSubmittedReason: "invalidTarget",
    },
  });
  assert.equal(mediaControlNotice(current, "music"), "notSubmitted");
});
