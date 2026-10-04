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

export function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
export async function until(predicate: () => boolean) {
  for (let i = 0; i < 2000; i++) {
    if (predicate()) return;
    await Promise.resolve();
  }
  assert.fail("gesture did not settle");
}

// Observable transport fixture only. Actual backend and atomic semantics need native/Rust checks.
export function brightnessRig() {
  let status = levelFixture(),
    available = true,
    busy = false,
    now = 0;
  let active = 0,
    maxActive = 0,
    serial = 0,
    view = idleLevelGesture;
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
        if (request.kind === "apply" || request.kind === "output") {
          if (gate) await gate.promise;
          const state = status.runtime!.observation.snapshot!.state;
          assert.equal(request.revision, state.revision);
          if (request.kind === "output") {
            if (request.action.kind === "level")
              state.output!.percent = request.action.percent;
            else state.output!.blackout = request.action.enabled;
          } else {
            const source = state.sources.find((s) => s.id === request.source)!;
            if (request.action.kind === "level")
              source.level = request.action.value;
            else if (request.action.kind === "patch") {
              for (const change of request.action.changes) {
                assert.equal(change.value.kind, "normalized");
                if (change.value.kind !== "normalized") continue;
                const index = source.held!.findIndex(
                  (t) =>
                    t.fixtureId === change.fixtureId &&
                    t.attribute === change.attribute,
                );
                if (index < 0) {
                  source.held!.push({
                    fixtureId: change.fixtureId,
                    attribute: change.attribute,
                  });
                  source.heldValues!.push(change.value.value);
                } else source.heldValues![index] = change.value.value;
              }
            } else assert.fail("unexpected control action");
          }
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
    get mutations() {
      return commands.filter(
        (c) => c.request.kind === "apply" || c.request.kind === "output",
      );
    },
    get values() {
      return commands.flatMap(({ request }) =>
        request.kind === "apply" && request.action.kind === "patch"
          ? request.action.changes.flatMap((c) =>
              c.value.kind === "normalized" ? [c.value.value] : [],
            )
          : [],
      );
    },
  };
}
