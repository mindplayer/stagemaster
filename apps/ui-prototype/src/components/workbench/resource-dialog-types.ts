import type { ResourceKind } from "../../library-types";

export type ResourceDialog =
  | { kind: "group"; id?: string }
  | { kind: "preset"; id?: string }
  | { kind: "copyValues" }
  | {
      kind: "rename" | "duplicate" | "remove";
      id: string;
      resource: ResourceKind;
      name: string;
    };
