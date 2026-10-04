import test from "node:test";
import assert from "node:assert/strict";
import {
  LevelGestureController,
  idleLevelGesture,
} from "../src/execution-level-gesture.ts";
import type {
  ExecutionRequest,
  ExecutionStatus,
} from "../src/execution-types.ts";
import { levelFixture } from "./execution-level-fixture.ts";

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
async function until(predicate: () => boolean) {
  for (let i = 0; i < 2000; i++) {
    if (predicate()) return;
    await Promise.resolve();
  }
  assert.fail("test operation did not settle");
}
function rig() {
  let status = levelFixture();
  let available = true,
    busy = false,
    now = 0,
    active = 0,
    maxActive = 0,
    serial = 0;
  let view = idleLevelGesture;
  let gate: ReturnType<typeof deferred> | null = null;
  let transform = (value: ExecutionStatus) => value;
  let afterSnapshot = () => {};
  const commands: { request: ExecutionRequest; at: number }[] = [];
  const controller = new LevelGestureController({
    context: () => ({ status, available, busy }),
    publish: (value) => {
      view = value;
    },
    now: () => now,
    wait: async (ms) => {
      now += ms;
    },
    request: async (request) => {
      active++;
      maxActive = Math.max(maxActive, active);
      commands.push({ request, at: now });
      try {
        if (request.kind === "apply") {
          if (gate) await gate.promise;
          const state = status.runtime!.observation.snapshot!.state;
          assert.equal(request.revision, state.revision);
          assert.equal(request.action.kind, "level");
          if (request.action.kind === "level")
            state.sources.find((s) => s.id === request.source)!.level =
              request.action.value;
          state.revision = String(BigInt(state.revision) + 1n);
          status.runtime!.record = {
            serial: String(++serial),
            status: "complete",
            outcome: { kind: "applied", message: null },
          };
          status = transform(status);
        } else afterSnapshot();
        return structuredClone(status);
      } finally {
        active--;
      }
    },
  });
  return {
    controller,
    commands,
    get view() {
      return view;
    },
    get maxActive() {
      return maxActive;
    },
    get status() {
      return status;
    },
    set available(value: boolean) {
      available = value;
    },
    set busy(value: boolean) {
      busy = value;
    },
    set gate(value: ReturnType<typeof deferred> | null) {
      gate = value;
    },
    set transform(value: typeof transform) {
      transform = value;
    },
    set afterSnapshot(value: typeof afterSnapshot) {
      afterSnapshot = value;
    },
    get values() {
      return commands.flatMap(({ request }) =>
        request.kind === "apply" && request.action.kind === "level"
          ? [request.action.value]
          : [],
      );
    },
  };
}

test("a delayed control keeps one latest position; release confirms exact final zero without stopping", async () => {
  const r = rig(),
    gate = deferred();
  r.gate = gate;
  assert.ok(r.controller.begin("source-0"));
  r.controller.change("source-0", 50000);
  await until(() => r.values.length === 1);
  for (let i = 49000; i > 0; i -= 1000) r.controller.change("source-0", i);
  r.controller.change("source-0", 0);
  r.controller.finish("source-0");
  assert.equal(r.view.target, 0);
  assert.deepEqual(r.values, [50000]);
  assert.equal(r.controller.begin("manual"), false);
  gate.resolve();
  await until(() => !r.controller.busy);
  assert.deepEqual(r.values, [50000, 0]);
  assert.equal(r.maxActive, 1);
  const mutations = r.commands.filter((c) => c.request.kind === "apply");
  assert.ok(mutations[1].at - mutations[0].at >= 50);
  assert.equal(
    r.status.runtime!.observation.snapshot!.state.sources[0].status,
    "Running",
  );
  assert.equal(r.view.problem, "");
});

test("cancel drops unsent values and blocks a new gesture until the old transport returns", async () => {
  const r = rig(),
    gate = deferred();
  r.gate = gate;
  r.controller.begin("source-0");
  r.controller.change("source-0", 40000);
  await until(() => r.values.length === 1);
  r.controller.change("source-0", 12000);
  r.controller.cancel("source-0");
  assert.equal(r.controller.begin("manual"), false);
  assert.equal(r.view.phase, "cancelled");
  gate.resolve();
  await until(() => !r.controller.busy);
  assert.deepEqual(r.values, [40000]);
  assert.equal(
    r.status.runtime!.observation.snapshot!.state.sources[0].level,
    40000,
  );
  assert.ok(r.controller.begin("manual"));
  r.controller.change("manual", 500);
  r.controller.finish("manual");
  await until(() => !r.controller.busy);
  assert.deepEqual(r.values, [40000, 500]);
  assert.equal(r.view.problem, "");
});

test("the same pending receipt must complete and its observed value must catch up", async () => {
  const r = rig();
  let reads = 0;
  r.transform = (s) => {
    s.runtime!.pending = true;
    s.runtime!.record!.status = "pending";
    s.runtime!.record!.outcome = null;
    s.runtime!.observation.snapshot!.state.sources[0].level = 65535;
    return s;
  };
  r.afterSnapshot = () => {
    if (!r.status.runtime!.record) return;
    reads++;
    if (reads === 2) {
      r.status.runtime!.pending = false;
      r.status.runtime!.record = {
        serial: "1",
        status: "complete",
        outcome: { kind: "applied", message: null },
      };
    }
    if (reads === 4)
      r.status.runtime!.observation.snapshot!.state.sources[0].level = 1234;
  };
  r.controller.begin("source-0");
  r.controller.change("source-0", 1234);
  r.controller.finish("source-0");
  await until(() => !r.controller.busy);
  assert.deepEqual(r.values, [1234]);
  assert.equal(reads, 4);
  assert.equal(r.view.problem, "");
});

test("failure unknown receipt and timeout never retry controls", async () => {
  for (const mode of [
    "rejected",
    "unknown",
    "pending",
    "missing",
    "replaced",
  ] as const) {
    const r = rig();
    r.transform = (s) => {
      if (mode === "missing") s.runtime!.record = null;
      else if (mode === "pending" || mode === "replaced") {
        s.runtime!.pending = true;
        s.runtime!.record!.status = "pending";
        s.runtime!.record!.outcome = null;
      } else s.runtime!.record!.outcome!.kind = mode;
      return s;
    };
    r.afterSnapshot = () => {
      if (mode === "replaced" && r.status.runtime!.record)
        r.status.runtime!.record.serial = "9";
    };
    r.controller.begin("source-0");
    r.controller.change("source-0", 2345);
    r.controller.finish("source-0");
    await until(() => !r.controller.busy);
    assert.equal(r.values.length, 1, mode);
    assert.ok(r.view.problem, mode);
  }
});

test("hidden disconnected replaced and read-only contexts cancel later positions", async () => {
  for (const kind of [
    "hidden",
    "host",
    "layout",
    "session",
    "owner",
    "control",
    "fault",
  ] as const) {
    const r = rig(),
      gate = deferred();
    r.gate = gate;
    r.controller.begin("source-0");
    r.controller.change("source-0", 4000);
    await until(() => r.values.length === 1);
    r.controller.change("source-0", 3000);
    const runtime = r.status.runtime!;
    if (kind === "hidden") r.available = false;
    if (kind === "host") runtime.hostId = "new";
    if (kind === "layout") runtime.catalog.layout = "new";
    if (kind === "session") runtime.sessionId = "new";
    if (kind === "owner")
      runtime.observation.snapshot!.state.owner!.sessionId = "new";
    if (kind === "control") runtime.controlling = false;
    if (kind === "fault") runtime.observation.fault = "failed";
    r.controller.validate();
    gate.resolve();
    await until(() => !r.controller.busy);
    assert.deepEqual(r.values, [4000], kind);
    assert.ok(r.view.problem, kind);
  }
});

test("no movement makes no mutation; invalid values and busy states cannot send", async () => {
  const r = rig();
  r.busy = true;
  assert.equal(r.controller.begin("source-0"), false);
  r.busy = false;
  r.status.runtime!.pending = true;
  assert.equal(r.controller.begin("source-0"), false);
  r.status.runtime!.pending = false;
  assert.equal(r.controller.begin("missing"), false);
  r.controller.begin("source-0");
  r.controller.finish("source-0");
  await until(() => !r.controller.busy);
  assert.deepEqual(r.values, []);
  for (const invalid of [-1, 65536, NaN, 12.5]) {
    r.controller.begin("source-0");
    r.controller.change("source-0", invalid);
    assert.equal(r.controller.busy, false);
    assert.ok(r.view.problem);
    assert.deepEqual(r.values, []);
  }
});

test("renewal pending before a gesture command is observed before choosing the next revision", async () => {
  const r = rig();
  let reads = 0;
  r.afterSnapshot = () => {
    reads++;
    r.status.runtime!.pending = reads < 3;
    if (reads === 3)
      r.status.runtime!.observation.snapshot!.state.revision =
        "9007199254740995";
  };
  r.controller.begin("manual");
  r.controller.change("manual", 0);
  r.controller.finish("manual");
  await until(() => !r.controller.busy);
  assert.equal(reads, 3);
  const request = r.commands.find((c) => c.request.kind === "apply")!.request;
  assert.equal(
    request.kind === "apply" && request.revision,
    "9007199254740995",
  );
  assert.deepEqual(r.values, [0]);
  assert.equal(r.view.problem, "");
});

test("manual zero preserves held values, and old serials cannot confirm a gesture", async () => {
  const r = rig();
  const manual = r.status.runtime!.observation.snapshot!.state.sources.find(
    (s) => s.id === "manual",
  )!;
  manual.held = [{ fixtureId: "fixture", attribute: "dimmer" }];
  manual.heldValues = [
    { fixtureId: "fixture", attribute: "dimmer", value: 24576 },
  ];
  r.controller.begin("manual");
  r.controller.change("manual", 0);
  r.controller.finish("manual");
  await until(() => !r.controller.busy);
  assert.equal(manual.level, 0);
  assert.equal(manual.held.length, 1);
  assert.equal(manual.heldValues[0].value, 24576);
  r.status.runtime!.record!.serial = "9007199254740993";
  r.controller.begin("manual");
  r.controller.change("manual", 3000);
  r.controller.finish("manual");
  await until(() => !r.controller.busy);
  assert.match(r.view.problem, /回执/);
});

test("the same fader can resume during final confirmation without losing the latest target", async () => {
  const r = rig(),
    gate = deferred();
  r.gate = gate;
  r.controller.begin("source-0");
  r.controller.change("source-0", 2000);
  r.controller.finish("source-0");
  await until(() => r.values.length === 1);
  assert.equal(r.view.phase, "settling");
  assert.ok(r.controller.begin("source-0"));
  r.controller.change("source-0", 5000);
  r.controller.finish("source-0");
  gate.resolve();
  await until(() => !r.controller.busy);
  assert.deepEqual(r.values, [2000, 5000]);
  assert.equal(r.view.problem, "");
});
