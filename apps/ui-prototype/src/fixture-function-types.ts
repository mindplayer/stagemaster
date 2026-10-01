export interface FunctionDefinition {
  key: string;
  name: string;
  mode: "slot" | "range";
  dmxFrom: number;
  dmxTo: number;
  dmxDefault: number;
}
export interface FunctionSelection {
  functionKey: string;
  position: number;
}
export interface FunctionAttribute {
  functions: FunctionDefinition[];
  default: FunctionSelection;
  fine: boolean;
}
export const functionLabels: Record<string, string> = {
  "color-wheel": "色盘",
  "gobo-wheel": "图案盘",
  shutter: "快门与频闪",
  prism: "棱镜",
};
export function initialFunction(f: FunctionDefinition): FunctionSelection {
  return {
    functionKey: f.key,
    position:
      f.mode === "slot"
        ? 0
        : Math.round(
            ((f.dmxDefault - f.dmxFrom) * 65535) / (f.dmxTo - f.dmxFrom),
          ),
  };
}
export function sameFunctions(
  a?: FunctionDefinition[],
  b?: FunctionDefinition[],
) {
  if (!a || !b) return a === b;
  return (
    a.length === b.length &&
    a.every((f, i) => {
      const g = b[i];
      return (
        f.key === g.key &&
        f.name === g.name &&
        f.mode === g.mode &&
        f.dmxFrom === g.dmxFrom &&
        f.dmxTo === g.dmxTo &&
        f.dmxDefault === g.dmxDefault
      );
    })
  );
}
