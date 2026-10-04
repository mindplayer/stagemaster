import type { ProfileDraft } from "./fixture-tools";
import {
  newFunction,
  nextChannel,
  type FunctionDraft,
} from "./fixture-function-draft.ts";
import { programKey } from "./fixture-program-rules.ts";
export {
  programKey,
  programLabel,
  programKind,
} from "./fixture-program-rules.ts";
export function newProgramFunction(
  kind: "external" | "auto" | "sound",
): FunctionDraft {
  const f = newFunction();
  return {
    ...f,
    key: kind === "external" ? kind : `${kind}.${crypto.randomUUID()}`,
    name: kind === "external" ? "外部通道控制" : "",
    dmxFrom: "",
    dmxTo: "",
    dmxDefault: "",
  };
}
export function addProgramChannel(draft: ProfileDraft): ProfileDraft {
  if (draft.channels.some((c) => c.attribute === programKey)) return draft;
  const coarse = nextChannel(draft);
  return {
    ...draft,
    footprint: String(Math.max(Number(draft.footprint) || 0, coarse)),
    channels: [
      ...draft.channels,
      {
        attribute: programKey,
        coarse: String(coarse),
        fine: "",
        bits: "8",
        percent: "",
        functions: [newProgramFunction("external")],
        defaultFunction: { functionKey: "external", position: "0" },
      },
    ],
  };
}
