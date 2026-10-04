// Menu/command preflight only; Rust remains the output authority.
import type { FunctionDefinition } from "./fixture-function-types";
import { attributeBase, isEmitterFunction } from "./fixture-emitter-keys.ts";
export const functionSafetyMessage =
  "声控、自走、自动轮盘、复位及未知控制宏已屏蔽，不能用于演出";
export function functionSelectionAllowed(
  attribute: string,
  f: Pick<FunctionDefinition, "key" | "mode">,
) {
  if (attribute === "fixture-program") return f.key === "external";
  if (
    f.key
      .split(/[._-]/)
      .some((part) =>
        [
          "sound",
          "soundcontrol",
          "soundactivated",
          "auto",
          "automatic",
          "random",
          "reset",
          "macro",
          "program",
        ].includes(part),
      )
  )
    return false;
  const slot = f.mode === "slot",
    base = attributeBase(attribute);
  if (isEmitterFunction(attribute)) {
    return base === "shutter"
      ? (["open", "closed"].includes(f.key) && slot) ||
          (f.key === "strobe" && !slot)
      : ((f.key === "open" ||
          (f.key.startsWith("slot-") && f.key.length > 5)) &&
          slot) ||
          (base === "gobo-wheel" &&
            f.key.startsWith("shake-") &&
            f.key.length > 6 &&
            !slot);
  }
  switch (attribute) {
    case "color-wheel":
      return slot;
    case "gobo-wheel":
      return slot || f.key === "shake" || f.key.startsWith("shake-");
    case "shutter":
      return (
        (["open", "closed"].includes(f.key) && slot) ||
        (!slot && (f.key === "strobe" || f.key.startsWith("strobe-")))
      );
    case "prism":
      return slot && ["off", "on"].includes(f.key);
    default:
      return false;
  }
}
