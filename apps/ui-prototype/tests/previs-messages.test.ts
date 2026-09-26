import { test } from "node:test";
import assert from "node:assert/strict";
import { readPrevisMessage } from "../src/previs-messages.ts";
const proposal = {
  kind: "placement", requestId: "a".repeat(32), generation: 7, version: "9007199254740993",
  placement: { fixtureId: "fixture-1", spaceId: null, positionMeters: { x: "-0.123456", y: "2", z: "3" }, rotationDegreesXYZ: { x: "25", y: "0", z: "0" } },
};
test("renderer proposals preserve exact revision and signed decimal coordinates", () => {
  assert.deepEqual(readPrevisMessage(JSON.stringify(proposal)), proposal);
  assert.deepEqual(readPrevisMessage('{"kind":"selection","fixtureId":""}'), { kind: "selection", fixtureId: "" });
});
test("renderer proposals reject malformed, oversized and lossy payloads", () => {
  for (const patch of [{ generation: 2 ** 32 }, { generation: 2.5 }, { version: "18446744073709551616" }, { version: "1e3" }, { requestId: "" }, { placement: {} }])
    assert.equal(readPrevisMessage(JSON.stringify({ ...proposal, ...patch })), null);
  for (const x of ["NaN", "Infinity", "0.1234567", 2, "1e10"])
    assert.equal(readPrevisMessage(JSON.stringify({ ...proposal, placement: { ...proposal.placement, positionMeters: { x, y: "0", z: "0" } } })), null);
  assert.equal(readPrevisMessage(" ".repeat(8193)), null);
  assert.equal(readPrevisMessage("{"), null);
  assert.equal(readPrevisMessage('{"kind":"execute"}'), null);
});
test('剖视状态兼容旧渲染器，拒绝错误字段类型', () => {
  const state = { kind: 'state', status: '场景预演', selection: '', workLight: '工作照明：开', move: false };
  assert.deepEqual(readPrevisMessage(JSON.stringify(state)), { ...state, cutaway: false });
  assert.deepEqual(readPrevisMessage(JSON.stringify({ ...state, cutaway: true })), { ...state, cutaway: true });
  assert.equal(readPrevisMessage(JSON.stringify({ ...state, cutaway: 'false' })), null);
});
