import type { ProjectView, FixtureView } from "../src/application-host";
import { attributeName } from "../src/library-tools";
import { functionLabels } from "../src/fixture-function-types";

export function withFullScopeFixtures(base: ProjectView): ProjectView {
  const fixtures = [
    {
      id: "wheel",
      name: "色盘摇头灯",
      keys: [
        "dimmer",
        "shutter",
        "color-wheel",
        "gobo-wheel",
        "prism",
        "zoom",
        "focus",
        "iris",
        "pan",
        "tilt",
      ],
    },
    { id: "rgb", name: "染色灯", keys: ["dimmer", "red", "green", "blue"] },
  ].map(({ id, name, keys }, index): FixtureView => ({
    ...base.fixtures[0],
    id,
    name,
    address: index * 20 + 1,
    footprint: keys.length,
    attributes: keys.map((key) => ({
      key,
      label: attributeName(key),
      defaultValue: 0,
      ...(key in functionLabels
        ? {
            function: {
              fine: false,
              default: { functionKey: "slot", position: 0 },
              functions: [
                {
                  key: "slot",
                  name: "固定档位",
                  mode: "slot",
                  dmxFrom: 0,
                  dmxTo: 255,
                  dmxDefault: 0,
                },
              ],
            },
          }
        : {}),
    })),
  }));
  const values = fixtures.flatMap((f) =>
    f.attributes.map((a) => ({
      fixtureId: f.id,
      attribute: a.key,
      value: a.function ? 0 : 32768,
      mode: "value" as const,
      presetId: null,
      presetName: null,
      ...(a.function
        ? { functionValue: { functionKey: "slot", position: 0 } }
        : {}),
    })),
  );
  return {
    ...base,
    fixtures,
    scenes: [{ ...base.scenes[0], values }],
    presets: [{ ...base.presets[0], name: "完整属性", values }],
  };
}
