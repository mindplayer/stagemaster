import { readPrevisMessage } from "./previs-messages.ts";
import { sameFixtureSelection } from "./previs-selection.ts";
import { fixtureTargets, sameTargets } from "./previs-objects.ts";
import type { PrevisInteractions } from "./previs-types.ts";
import type { PrevisInteractionScope } from "./previs-interaction-scope.ts";

/** One handler per viewer connection; no project mutation outside the host callbacks. */
export function createPrevisResponder({
  active,
  callbacks,
  state,
  send,
  scope,
  selectionScope,
  connection,
  now = () => performance.now(),
}: {
  active(): boolean;
  callbacks(): PrevisInteractions;
  state(
    value: Extract<
      NonNullable<ReturnType<typeof readPrevisMessage>>,
      { kind: "state" }
    >,
  ): void;
  send(value: object): void;
  scope: PrevisInteractionScope;
  selectionScope: PrevisInteractionScope;
  connection(): number;
  now?(): number;
}) {
  const proposals = new Set<string>();
  let selectionEpoch = 0;
  function restoreSelection() {
    const host = callbacks();
    send(
      host.selectedTargets
        ? { action: "selectTargets", targets: host.selectedTargets }
        : { action: "selectGroup", fixtureIds: host.selectedIds },
    );
  }
  return (response: string) => {
    if (!active()) return;
    const value = readPrevisMessage(response);
    if (!value) return;
    if (value.kind === "state") {
      state(value);
      return;
    }
    if (
      value.kind === "selection" ||
      value.kind === "selectionGroup" ||
      value.kind === "selectionTargets"
    ) {
      const selected = ++selectionEpoch,
        epoch = connection();
      const valid = selectionScope.capture();
      const isActive = () =>
        active() &&
        valid() &&
        selected === selectionEpoch &&
        epoch === connection();
      const host = callbacks();
      const ids =
        value.kind === "selectionGroup"
          ? value.fixtureIds
          : value.kind === "selection" && value.fixtureId
            ? [value.fixtureId]
            : [];
      const pending =
        value.kind === "selectionTargets"
          ? (host.onSelectTargets?.(value.targets, isActive) ??
            Promise.resolve(false))
          : host.onSelect(ids, isActive);
      const restore = () => {
        if (isActive()) restoreSelection();
      };
      void pending.then((ok) => {
        if (!ok) restore();
      }, restore);
      return;
    }
    const reply = (accepted: boolean) =>
      send({ action: "placementResult", requestId: value.requestId, accepted });
    if (value.kind === "placement") {
      reply(false);
      return;
    }
    if (proposals.has(value.requestId)) return;
    proposals.add(value.requestId);
    if (proposals.size > 32) proposals.delete(proposals.values().next().value!);
    const epoch = connection(),
      selected = selectionEpoch,
      at = now(),
      permitted = scope.capture(),
      host = callbacks();
    const targets = host.selectedTargets ?? fixtureTargets(host.selectedIds);
    const matching =
      value.kind === "objectTranslation"
        ? sameTargets(targets, value.targets)
        : targets.every((t) => t.kind === "placement") &&
          sameFixtureSelection(host.selectedIds, value.fixtureIds);
    if (!permitted() || !matching) {
      reply(false);
      return;
    }
    const valid = () =>
      active() &&
      permitted() &&
      selected === selectionEpoch &&
      connection() === epoch &&
      now() - at < 2500;
    const pending =
      value.kind === "objectTranslation"
        ? (host.onObjectTranslation?.(value, valid) ?? Promise.resolve(false))
        : value.kind === "translation"
          ? host.onTranslation(value, valid)
          : host.onTransform(value, valid);
    const complete = (accepted: boolean) => {
      if (active() && connection() === epoch) reply(accepted);
    };
    void pending.then(complete, () => complete(false));
  };
}
