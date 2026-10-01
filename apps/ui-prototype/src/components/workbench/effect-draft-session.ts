import type { ApplicationHost, EditOperation } from "../../application-host";
import type { SceneEffect } from "../../effect-types";

export interface EffectDraftValue {
  sceneId: string;
  effect: SceneEffect;
  illuminate: boolean;
}
export interface EffectDraftState {
  active: boolean;
  working: boolean;
  message: string;
}
export function effectDraftValue(commands: EditOperation[]): EffectDraftValue {
  const first = commands[0];
  if (first?.op !== "effect" || first.command.kind !== "put")
    throw new Error("效果草稿不可用，请重新打开效果");
  return {
    sceneId: first.command.sceneId,
    effect: first.command.effect,
    illuminate: commands.length > 1,
  };
}

/** A single in-flight update plus its newest replacement; no independent show clock. */
export class EffectDraftSession {
  private version = 0;
  private owner: {
    epoch: number;
    generation: number;
    serial: number;
    identity: string;
  } | null = null;
  private latest: EffectDraftValue | null = null;
  private flight: Promise<void> = Promise.resolve();
  private pumping = false;
  private revision = 0;
  private state: EffectDraftState = {
    active: false,
    working: false,
    message: "",
  };
  private host: Pick<ApplicationHost, "preview" | "request">;
  private notify: (state: EffectDraftState) => void;
  constructor(
    host: Pick<ApplicationHost, "preview" | "request">,
    notify: (state: EffectDraftState) => void,
  ) {
    this.host = host;
    this.notify = notify;
  }
  private publish(patch: Partial<EffectDraftState>) {
    this.state = { ...this.state, ...patch };
    this.notify(this.state);
  }
  async begin(value: EffectDraftValue) {
    if (this.state.working || this.owner) return false;
    const version = ++this.version;
    this.publish({ working: true, message: "" });
    let ok = false;
    const work = async () => {
      try {
        const project = await this.host.request({ kind: "snapshot" });
        const preview = await this.host.preview({ kind: "snapshot" });
        if (version !== this.version) return;
        const next = await this.host.preview({
          kind: "beginEffectDraft",
          generation: project.generation,
          epoch: preview.epoch,
          ...value,
        });
        if (version !== this.version) {
          await this.host.preview({
            kind: "endEffectDraft",
            epoch: next.epoch,
          });
          return;
        }
        this.owner = {
          epoch: next.epoch,
          generation: project.generation,
          serial: 0,
          identity: value.effect.id,
        };
        this.publish({ active: true, message: "即时预演中 · 修改尚未应用" });
        ok = true;
      } catch (e) {
        if (version === this.version) this.problem(e);
      } finally {
        if (version === this.version) this.publish({ working: false });
      }
    };
    this.flight = work();
    await this.flight;
    return ok;
  }
  update(value: EffectDraftValue) {
    if (!this.owner) return;
    this.revision++;
    this.latest = value;
    if (this.pumping) return;
    const version = this.version;
    this.pumping = true;
    this.flight = (async () => {
      try {
        while (this.latest && this.owner && version === this.version) {
          const next = this.latest;
          this.latest = null;
          const revision = this.revision;
          const owner = this.owner;
          try {
            const project = await this.host.request({ kind: "snapshot" });
            if (version !== this.version) return;
            const result = await this.host.preview({
              kind: "updateEffectDraft",
              generation: project.generation,
              epoch: owner.epoch,
              serial: ++owner.serial,
              effect: next.effect,
              illuminate: next.illuminate,
            });
            if (version !== this.version) return;
            if (
              result.epoch !== owner.epoch ||
              result.loaded?.draftEffectId !== owner.identity
            ) {
              this.lost();
              return;
            }
            if (revision === this.revision)
              this.publish({ message: "即时预演中 · 修改尚未应用" });
          } catch (e) {
            if (version === this.version && revision === this.revision)
              this.problem(e);
          }
        }
      } finally {
        this.pumping = false;
      }
    })();
  }
  problem(reason: unknown) {
    this.revision++;
    this.latest = null;
    this.publish({
      message: `${reason instanceof Error ? reason.message : String(reason)}${this.owner ? "；画面保留最近有效参数" : ""}`,
    });
  }
  async inspect() {
    const owner = this.owner,
      version = this.version,
      revision = this.revision;
    if (!owner || this.pumping || this.state.working) return;
    try {
      const next = await this.host.preview({ kind: "snapshot" });
      if (
        version === this.version &&
        (next.epoch !== owner.epoch ||
          next.loaded?.draftEffectId !== owner.identity)
      )
        this.lost();
    } catch (e) {
      if (
        version === this.version &&
        revision === this.revision &&
        !this.pumping
      )
        this.problem(e);
    }
  }
  private lost() {
    this.version++;
    this.owner = null;
    this.latest = null;
    this.publish({
      active: false,
      working: false,
      message: "即时预演已结束，当前使用新的播放或工程内容",
    });
  }
  async end() {
    if (!this.owner && !this.state.working && !this.pumping) return;
    const version = ++this.version;
    const owner = this.owner;
    this.owner = null;
    this.latest = null;
    this.publish({ active: false, working: true });
    await this.flight;
    try {
      if (owner)
        await this.host.preview({ kind: "endEffectDraft", epoch: owner.epoch });
      if (version === this.version)
        this.publish({ message: "即时预演已结束，已恢复原场景参数" });
    } catch (e) {
      if (version === this.version) this.problem(e);
    } finally {
      if (version === this.version) this.publish({ working: false });
    }
  }
}
