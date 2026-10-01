import { test } from "node:test";
import assert from "node:assert/strict";
import { createWorldLine, readWorldLine } from "../src/world-line-tools.ts";
import {
  reuseEffect,
  supportsEffect,
  effectCommands,
} from "../src/effect-tools.ts";
import { toKeyframes } from "../src/keyframe-tools.ts";
import type { FixtureView } from "../src/application-host.ts";

test("world intent keeps two axes together, copies independently and never adds illumination", () => {
  const fixture = {
    id: "one",
    name: "灯",
    positioning: { kind: "intersectingOrthogonal" },
    attributes: ["pan", "tilt", "dimmer"].map((key) => ({ key })),
  } as FixtureView;
  const source = createWorldLine("source", [fixture.id]);
  assert(supportsEffect([fixture], "worldLine"));
  assert(
    !supportsEffect([{ ...fixture, positioning: undefined }], "worldLine"),
  );
  assert(!supportsEffect([], "worldLine"));
  const copy = reuseEffect(source, "copy");
  copy.targetPath!.fromMeters.x = "-3";
  assert.equal(source.targetPath!.fromMeters.x, "-1");
  assert.equal(copy.enabled, false);
  assert.equal(effectCommands("scene", source, [fixture], false).length, 1);
  assert.throws(() => toKeyframes(source), /运动属性/);
});

test("world coordinates normalize precision and reject empty paths or invalid accuracy", () => {
  const path = createWorldLine("source", ["one"]).targetPath!;
  const result = readWorldLine({
    ...path,
    fromMeters: { ...path.fromMeters, x: "-1.250000" },
    maxErrorMeters: "0.1000",
  });
  assert.equal(result.fromMeters.x, "-1.25");
  assert.equal(result.maxErrorMeters, "0.1");
  assert.throws(
    () => readWorldLine({ ...path, toMeters: { ...path.fromMeters } }),
    /不能重合/,
  );
  for (const invalid of ["0", "1.1", "Infinity", ""])
    assert.throws(() => readWorldLine({ ...path, maxErrorMeters: invalid }));
  assert.throws(
    () =>
      readWorldLine({
        ...path,
        fromMeters: { ...path.fromMeters, x: "100001" },
      }),
    /100000/,
  );
  assert.equal(path.maxErrorMeters, "0.1");
});
