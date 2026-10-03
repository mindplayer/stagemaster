import test from "node:test";
import assert from "node:assert/strict";
import {
  acceptRunReply,
  elapsedLabel,
  ownsRun,
  shouldRenew,
  runLabel,
} from "../src/device-runtime-tools.ts";
import type {
  DeviceRunReply,
  DeviceRunState,
  DeviceRunView,
} from "../src/device-runtime-types.ts";
const state: DeviceRunState = {
  mode: "operation",
  package: "x",
  selected: null,
  loaded: null,
  status: null,
  instance: null,
  step: null,
  elapsedMs: "0",
  owner: { lease: "9007199254740993", expiresMs: "60000" },
};
const reply: DeviceRunReply = {
  id: "9007199254740993",
  boot: "a",
  revision: "9007199254740993",
  observedMs: "1",
  programCount: 1,
  stepCount: 0,
  body: { kind: "state", state, error: null },
};
const view: DeviceRunView = {
  epoch: 2,
  connectionEpoch: 2,
  peer: {
    device: "d",
    boot: "a",
    session: "s",
    control: true,
    installation: true,
  },
  pending: false,
  lastResponse: reply,
  reply,
};
test("late replies, wrong boots, old connection history cannot replace current state", () => {
  const newer = { ...reply, id: "9007199254740994" };
  assert.equal(acceptRunReply(newer, view, 2), newer);
  assert.equal(acceptRunReply(newer, { ...view, peer: null }, 2), newer);
  assert.equal(
    acceptRunReply(newer, { ...view, connectionEpoch: 1 }, 2),
    newer,
  );
  assert.equal(
    acceptRunReply(newer, { ...view, reply: { ...reply, boot: "other" } }, 2),
    newer,
  );
  assert.equal(acceptRunReply(reply, { ...view, reply: newer }, 2), newer);
});
test("renew only explicitly acquired matching lease, throttled in device time", () => {
  assert.equal(ownsRun(state, null), false);
  assert.equal(ownsRun(state, "9007199254740992"), false);
  assert.equal(shouldRenew(state, "30000", state.owner!.lease, "0"), true);
  assert.equal(shouldRenew(state, "31000", state.owner!.lease, "30000"), false);
  assert.equal(shouldRenew(state, "10000", state.owner!.lease, "0"), false);
});
test("empty or maintenance states cannot claim loaded playback or physical output", () => {
  assert.equal(runLabel(state), "尚未载入节目");
  assert.equal(runLabel({ ...state, mode: "maintenance" }), "安装维护中");
  assert.equal(elapsedLabel("125999"), "2:05");
});
