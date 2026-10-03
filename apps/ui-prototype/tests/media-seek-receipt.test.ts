import test from "node:test";
import assert from "node:assert/strict";
import {
  mediaRequestIdentity,
  mediaSeekReceipt,
} from "../src/media-seek-receipt.ts";
import type { ExecutionView } from "../src/execution-types.ts";
import type { ExecutionMediaState } from "../src/execution-media-types.ts";

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
